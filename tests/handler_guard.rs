extern crate core;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use storyteller::{
    event_channel, ChannelEventListener, ChannelReporter, EventHandler, EventListener,
    EventReporter, HandlerGuard,
};

#[derive(Debug, Eq, PartialEq)]
struct MyEvent(usize);

struct NoopHandler;

impl EventHandler for NoopHandler {
    type Event = MyEvent;
    fn handle(&self, _: Self::Event) {}
}

struct TrackingHandler {
    handled: AtomicUsize,
    finished: AtomicBool,
}

impl TrackingHandler {
    fn new() -> Self {
        Self {
            handled: AtomicUsize::new(0),
            finished: AtomicBool::new(false),
        }
    }

    fn handled(&self) -> usize {
        self.handled.load(Ordering::SeqCst)
    }

    fn finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }
}

impl EventHandler for TrackingHandler {
    type Event = MyEvent;

    fn handle(&self, _: Self::Event) {
        self.handled.fetch_add(1, Ordering::SeqCst);
    }

    fn finish(&self) {
        self.finished.store(true, Ordering::SeqCst);
    }
}

#[test]
#[should_panic]
fn drop_without_join_panics() {
    let (sender, receiver) = event_channel::<MyEvent>();
    let reporter = ChannelReporter::new(sender);
    let listener = ChannelEventListener::new(receiver);
    let guard = listener.run_handler(Arc::new(NoopHandler));
    let token = reporter.disconnect().unwrap();
    drop(token);
    drop(guard);
}

#[test]
fn finish_called_with_no_events() {
    let (sender, receiver) = event_channel::<MyEvent>();
    let reporter = ChannelReporter::new(sender);
    let listener = ChannelEventListener::new(receiver);
    let handler = Arc::new(TrackingHandler::new());
    let guard = listener.run_handler(handler.clone());
    let token = reporter.disconnect().unwrap();
    guard.join(token).unwrap();
    assert_eq!(handler.handled(), 0);
    assert!(handler.finished());
}

#[test]
fn disconnect_and_join() {
    let (sender, receiver) = event_channel::<MyEvent>();
    let reporter = ChannelReporter::new(sender);
    let listener = ChannelEventListener::new(receiver);
    let handler = Arc::new(TrackingHandler::new());
    let guard = listener.run_handler(handler.clone());
    for i in 0..5 {
        reporter.report_event(MyEvent(i)).unwrap();
    }
    guard.disconnect_and_join(reporter).unwrap();
    assert_eq!(handler.handled(), 5);
    assert!(handler.finished());
}

#[test]
fn report_event_on_dropped_receiver() {
    let (sender, receiver) = event_channel::<MyEvent>();
    let reporter = ChannelReporter::new(sender);
    drop(receiver);
    assert!(reporter.report_event(MyEvent(0)).is_err());
}
