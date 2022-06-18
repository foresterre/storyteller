// A sample implementation which collects the events it receives
extern crate core;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use storyteller::{
    event_channel, ChannelEventListener, ChannelReporter, EventHandler, EventListener,
    FinishProcessing, Reporter,
};

#[derive(Debug, Eq, PartialEq)]
struct MyEvent(usize);

// Caution: does only check whether `received` events match expected events
// Must also use `FinalizeHandler::finish_processing` to ensure panic's are caught.
struct CollectingHandler {
    expected_events: Vec<MyEvent>,
    nth: AtomicUsize,
}

impl CollectingHandler {
    fn new(expected: Vec<MyEvent>) -> Self {
        Self {
            expected_events: expected,
            nth: AtomicUsize::new(0),
        }
    }
}

impl EventHandler for CollectingHandler {
    type Event = MyEvent;

    fn handle(&self, event: Self::Event) {
        let nth = self.nth.load(Ordering::SeqCst);
        eprintln!("Test #{}", nth);

        let expected_event = self
            .expected_events
            .get(nth)
            .expect(&format!("No such expected value on index '{}'", nth));

        // compare
        assert_eq!(expected_event, &event);

        self.nth.fetch_add(1, Ordering::SeqCst);
    }

    fn finish(&self) {
        let received = self.nth.load(Ordering::SeqCst);
        let expected = self.expected_events.len();

        if received != expected {
            panic!(
                "Received '{}' events, but expected to receive '{}' events",
                received, expected
            );
        }
    }
}

#[test]
fn test() {
    let (event_sender, event_receiver) = event_channel::<MyEvent>();

    let reporter = ChannelReporter::new(event_sender);
    let listener = ChannelEventListener::new(event_receiver);

    let handler = CollectingHandler::new(vec![
        MyEvent(0),
        MyEvent(1),
        MyEvent(2),
        MyEvent(3),
        MyEvent(4),
    ]);

    let fin = listener.run_handler(Arc::new(handler));

    for i in 0..5 {
        reporter.report_event(MyEvent(i)).unwrap();
    }

    reporter.disconnect().unwrap();
    fin.finish_processing().unwrap();
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

    let handler = CollectingHandler::new(expected_events);

    let fin = listener.run_handler(Arc::new(handler));

    for i in 0..5 {
        reporter.report_event(MyEvent(i)).unwrap();
    }

    reporter.disconnect().unwrap();
    fin.finish_processing().unwrap();
}
