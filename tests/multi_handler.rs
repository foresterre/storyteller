// A sample implementation which collects the events it receives
extern crate core;

use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use storyteller::{
    event_channel, ChannelEventListener, ChannelReporter, EventHandler, EventListener,
    EventReporter, HandlerGuard,
};

// Caution: does only check whether `received` events match expected events
// Must also use `ChannelHandlerGuard::join` to ensure panic's are caught.
struct MultiHandler<EventT: Clone> {
    handlers: Vec<Box<dyn EventHandler<Event = EventT>>>,
}

impl<EventT: Clone + Sync> MultiHandler<EventT> {
    fn new() -> Self {
        Self { handlers: vec![] }
    }

    fn add_handler(&mut self, handler: Box<dyn EventHandler<Event = EventT>>) {
        self.handlers.push(handler);
    }

    // SAFETY:
    // Here we downcast to get our handler type H back for unit testing, 🤞
    #[cfg(test)]
    unsafe fn get_handler<H>(&self, nth: usize) -> &H {
        &*(&*self.handlers[nth] as *const dyn EventHandler<Event = EventT> as *const H)
    }
}

impl<EventT: Clone + Sync> EventHandler for MultiHandler<EventT> {
    type Event = EventT;

    fn handle(&self, event: Self::Event) {
        for handler in &self.handlers {
            handler.handle(event.clone())
        }
    }

    fn finish(&self) {
        for handler in &self.handlers {
            handler.finish();
        }
    }
}

struct CountingHandler<EventT: Sync> {
    counter: AtomicUsize,
    phantom: PhantomData<EventT>,
}

impl<EventT: Sync> CountingHandler<EventT> {
    fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            phantom: PhantomData,
        }
    }

    fn count(&self) -> usize {
        self.counter.load(Ordering::SeqCst)
    }
}

impl<EventT: Send + Sync> EventHandler for CountingHandler<EventT> {
    type Event = EventT;

    fn handle(&self, _event: Self::Event) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

struct SummingHandler<EventT: Sync + Into<usize>> {
    counter: AtomicUsize,
    phantom: PhantomData<EventT>,
}

impl<EventT: Sync + Into<usize>> SummingHandler<EventT> {
    fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            phantom: PhantomData,
        }
    }

    fn count(&self) -> usize {
        self.counter.load(Ordering::SeqCst)
    }
}

impl<EventT: Send + Sync + Into<usize>> EventHandler for SummingHandler<EventT> {
    type Event = EventT;

    fn handle(&self, event: Self::Event) {
        let value = event.into();

        self.counter.fetch_add(value, Ordering::Relaxed);
    }
}

struct FinishFlagHandler {
    finished: Arc<AtomicBool>,
}

impl EventHandler for FinishFlagHandler {
    type Event = usize;
    fn handle(&self, _: Self::Event) {}
    fn finish(&self) {
        self.finished.store(true, Ordering::SeqCst);
    }
}

#[test]
fn finish_delegates_to_sub_handlers() {
    let (event_sender, event_receiver) = event_channel::<usize>();
    let reporter = ChannelReporter::new(event_sender);
    let listener = ChannelEventListener::new(event_receiver);
    let flag1 = Arc::new(AtomicBool::new(false));
    let flag2 = Arc::new(AtomicBool::new(false));
    let mut multi_handler = MultiHandler::<usize>::new();
    multi_handler.add_handler(Box::new(FinishFlagHandler {
        finished: flag1.clone(),
    }));
    multi_handler.add_handler(Box::new(FinishFlagHandler {
        finished: flag2.clone(),
    }));
    let fin = listener.run_handler(Arc::new(multi_handler));
    let token = reporter.disconnect().unwrap();
    fin.join(token).unwrap();
    assert!(flag1.load(Ordering::SeqCst));
    assert!(flag2.load(Ordering::SeqCst));
}

#[test]
fn test() {
    let (event_sender, event_receiver) = event_channel::<usize>();

    let reporter = ChannelReporter::new(event_sender);
    let listener = ChannelEventListener::new(event_receiver);

    let counter1 = CountingHandler::<usize>::new();
    let counter2 = SummingHandler::<usize>::new();

    let multi_handler = {
        let mut multi_handler = MultiHandler::<usize>::new();
        multi_handler.add_handler(Box::new(counter1));
        multi_handler.add_handler(Box::new(counter2));
        multi_handler
    };

    let handler = Arc::new(multi_handler);
    let fin = listener.run_handler(handler.clone());

    for i in 0usize..5 {
        reporter.report_event(i).unwrap();
    }

    let token = reporter.disconnect().unwrap();
    fin.join(token).unwrap();

    // NB: Order of these statements is important. The assertions must be placed after
    // join() to ensure all expected events have been processed
    let c1 = unsafe { handler.get_handler::<CountingHandler<usize>>(0) };
    assert_eq!(c1.count(), 5);

    let c2 = unsafe { handler.get_handler::<SummingHandler<usize>>(1) };
    assert_eq!(c2.count(), 10);
}
