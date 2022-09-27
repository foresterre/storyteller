// A sample implementation which collects the events it receives
extern crate core;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use storyteller::{
    event_channel, ChannelEventListener, ChannelReporter, EventHandler, EventListener,
    EventReporter, FinishProcessing,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct MyEvent(usize);

// Caution: does only check whether `received` events match expected events
// Must also use `FinalizeHandler::finish_processing` to ensure panic's are caught.
struct RegisteringHandler {
    registered_events: Arc<Mutex<Vec<MyEvent>>>,
}

impl RegisteringHandler {
    fn new() -> Self {
        Self {
            registered_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn events(&self) -> Vec<MyEvent> {
        let guard = self.registered_events.lock().unwrap();
        guard.clone()
    }
}

impl EventHandler for RegisteringHandler {
    type Event = MyEvent;

    fn handle(&self, event: Self::Event) {
        let mut guard = self.registered_events.lock().unwrap();
        guard.push(event);

        dbg!(&guard);
    }
}

#[test]
fn test() {
    let (event_sender, event_receiver) = event_channel::<MyEvent>();

    let reporter = ChannelReporter::new(event_sender);
    let listener = ChannelEventListener::new(event_receiver);

    let handler = Arc::new(RegisteringHandler::new());
    let fin = listener.run_handler(handler.clone());

    for i in 0..5 {
        reporter.report_event(MyEvent(i)).unwrap();
    }

    reporter.disconnect().unwrap();
    fin.finish_processing().unwrap();

    // NB: Order is important, must be placed after finish_processing() to ensure all expected
    // events have been processed
    let expected = vec![MyEvent(0), MyEvent(1), MyEvent(2), MyEvent(3), MyEvent(4)];
    assert_eq!(handler.events(), expected);
}

#[yare::parameterized(
    to_few = { vec![ MyEvent(0), MyEvent(1), MyEvent(2), MyEvent(3), MyEvent(4), MyEvent(5)] },
    to_many = { vec![ MyEvent(0), MyEvent(1), MyEvent(2), MyEvent(3) ] },
    incorrect = { vec![ MyEvent(0), MyEvent(1), MyEvent(2), MyEvent(3), MyEvent(5), ] },
)]
#[should_panic]
fn expect_failure(expected_events: Vec<MyEvent>) {
    let (event_sender, event_receiver) = event_channel::<MyEvent>();

    let reporter = ChannelReporter::new(event_sender);
    let listener = ChannelEventListener::new(event_receiver);

    let handler = Arc::new(RegisteringHandler::new());

    let fin = listener.run_handler(handler.clone());

    for i in 0..5 {
        reporter.report_event(MyEvent(i)).unwrap();
    }

    reporter.disconnect().unwrap();
    fin.finish_processing().unwrap();

    // NB: Order is important, must be placed after finish_processing() to ensure all expected
    // events have been processed
    assert_eq!(handler.events(), expected_events);
}
