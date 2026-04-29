use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use storyteller::{EventHandler, HandlerGuard};

use storyteller::{
    event_channel, ChannelEventListener, ChannelReporter, EventListener, EventReporter,
};

#[derive(serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event {
    DiceThrow { throw: u8 },
    YouWin,
    YouLose,
}

#[derive(Default)]
struct JsonHandler;

impl EventHandler for JsonHandler {
    type Event = Event;

    fn handle(&self, event: Self::Event) {
        let serialized_event = serde_json::to_string(&event).unwrap();

        println!("{}", serialized_event);
    }
}

// See the test function `bar` in src/tests.rs for an example where the handler is a progress bar.
fn main() {
    let (sender, receiver) = event_channel::<Event>();

    // Handlers are implemented by you. Here you find one which writes jsonlines messages to stderr.
    // This can be anything, for example a progress bar (see src/tests.rs for an example of this),
    // a fake reporter which collects events for testing or maybe even a "MultiHandler<'h>" which
    // consists of a Vec<&'h dyn EventHandler> and executes multiple handlers under the hood.
    #[allow(clippy::default_constructed_unit_structs)]
    let handler = JsonHandler::default();

    // This one is included with the library. It just needs to be hooked up with a channel.
    let reporter = ChannelReporter::new(sender);

    // This one is also included with the library. It also needs to be hooked up with a channel.
    let listener = ChannelEventListener::new(receiver);

    // Here we use the JsonHandler we defined above, in combination with the default `EventListener`
    // and  `ChannelEventListener` defined above.
    //
    // If we don't run the handler, we'll end up in an infinite loop, because our `reporter.disconnect()`
    // below will block until it receives a Disconnect message.
    let fin = listener.run_handler(Arc::new(handler));

    // Now onto this program, let's play a game of dice!
    for _ in 0..100 {
        let dice = roll_dice();
        reporter
            .report_event(Event::DiceThrow { throw: dice })
            .unwrap();

        if dice >= 3 {
            reporter.report_event(Event::YouWin).unwrap();
        } else {
            reporter.report_event(Event::YouLose).unwrap();
        }

        thread::sleep(Duration::from_millis(100))
    }

    // Disconnect the reporter... already sent events can still be processed by the handler thread.
    let token = reporter.disconnect().unwrap();

    // Join the handler guard to wait for the handler thread to drain and exit.
    fin.join(token).unwrap();
}

static SEED: AtomicU32 = AtomicU32::new(1);

fn roll_dice() -> u8 {
    let mut random = SEED.load(Ordering::SeqCst);
    random ^= random << 13;
    random ^= random >> 17;
    random ^= random << 5;
    SEED.store(random, Ordering::SeqCst);

    (random % 6 + 1) as u8
}
