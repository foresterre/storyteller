// A sample implementation which collects the events it receives
#![cfg(feature = "channel_reporter")]
extern crate core;

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use storyteller::{
    event_channel, ChannelEventListener, ChannelReporter, EventHandler, EventListener,
    FinishProcessing, Reporter,
};

// Caution: does only check whether `received` events match expected events
// Must also use `FinalizeHandler::finish_processing` to ensure panic's are caught.
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
        for handle in &self.handlers {
            handle.handle(event.clone())
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

    reporter.disconnect().unwrap();
    fin.finish_processing().unwrap();

    // NB: Order of these statements is important. The assertions must be placed after
    // finish_processing() to ensure all expected events have been processed
    let c1 = unsafe { handler.get_handler::<CountingHandler<usize>>(0) };
    assert_eq!(c1.count(), 5);

    let c2 = unsafe { handler.get_handler::<SummingHandler<usize>>(1) };
    assert_eq!(c2.count(), 10);
}
