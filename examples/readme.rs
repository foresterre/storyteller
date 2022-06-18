use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{Stderr, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};
use storyteller::{
    event_channel, ChannelEventListener, ChannelReporter, EventHandler, EventListener,
    FinishProcessing, Reporter,
};

// --- In the main function, we'll instantiate a Reporter, a Listener, and an EventHandler.
//     For the reporter and listener, we'll use implementations included with the library.
//     The EventHandler must be defined by us, and can be found below.
//     We also need to define our event type, which can also be found below.

// See the test function `bar` in src/tests.rs for an example where the handler is a progress bar.
fn main() {
    let (sender, receiver) = event_channel::<ExampleEvent>();

    // Handlers are implemented by you. Here you find one which writes jsonlines messages to stderr.
    // This can be anything, for example a progress bar (see src/tests.rs for an example of this),
    // a fake reporter which collects events for testing or maybe even a "MultiHandler<'h>" which
    // consists of a Vec<&'h dyn EventHandler> and executes multiple handlers under the hood.
    //
    // Its implementation can be found below.
    let handler = JsonHandler::default();

    // This one is included with the library. It just needs to be hooked up with a channel.
    let reporter = ChannelReporter::new(sender);

    // This one is also included with the library. It also needs to be hooked up with a channel.
    // It's EventListener implementation spawns a thread in which event messages will be handled.
    // Events are send to this thread using channels, therefore the name ChannelEventListener ✨.
    let listener = ChannelEventListener::new(receiver);

    // Here we use the jsonlines handler we defined above, in combination with the default `EventListener`
    // implementation on the `ChannelEventListener` we used above.
    //
    //  As described above, it spawns a thread which handles updates, so it won't block.
    let event_handler = Arc::new(handler);
    let finalize_handler = listener.run_handler(event_handler);

    // Run your program's logic
    my_programming_logic(&reporter).unwrap();

    // First we disconnect the channel, so the thread which handles the events can be finished.
    reporter.disconnect().unwrap();
    // Next, we allow our event handler to finish processing its queue of unprocessed events.
    // This will block the main thread, until all unprocessed events are processed.
    finalize_handler.finish_processing().unwrap();
}

fn my_programming_logic(reporter: &ChannelReporter<ExampleEvent>) -> Result<(), Box<dyn Error>> {
    // These are the events we would call during the regular flow of our program, for example
    // if we use the library in a package manager, before, during or after downloading dependencies.
    // The use any-event-type-you-like nature allows you to go as crazy as you would like.
    reporter.report_event(ExampleEvent::text("One"))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::text("Two before reset"))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Reset))?;
    reporter.report_event(ExampleEvent::text("Two after reset"))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::text("Three"))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::event(MyEvent::Increment))?;
    reporter.report_event(ExampleEvent::text("Four"))?;

    Ok(())
}

// --- Here we define out Event Type.

// if we would have imported third-party libraries, we could have done: #[derive(serde::Serialize)]
#[derive(Debug)]
enum ExampleEvent {
    Event(MyEvent),
    Text(String),
}

impl Display for ExampleEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl ExampleEvent {
    pub fn event(event: MyEvent) -> Self {
        Self::Event(event)
    }

    pub fn text<T: AsRef<str>>(text: T) -> Self {
        Self::Text(text.as_ref().to_string())
    }
}

impl ExampleEvent {
    // Here we create some json by hand, so you can copy the example without importing other libraries, but you can also
    // replace all of this by, say `serde_json`, and derive a complete json output of your `Event` definition all at once (by design™ =)).
    pub fn to_json(&self) -> String {
        match self {
            Self::Event(event) => event.to_json(),
            Self::Text(msg) => format!("{{ \"event\" : \"message\", \"value\" : \"{}\" }}", msg),
        }
    }
}

#[derive(Debug)]
enum MyEvent {
    Increment,
    Reset,
}

impl MyEvent {
    pub fn to_json(&self) -> String {
        match self {
            Self::Increment => format!("{{ \"event\" : \"increment\" }}"),
            Self::Reset => format!("{{ \"event\" : \"reset\" }}"),
        }
    }
}

impl Display for MyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

// --- Here we define an Event Handler which deals with the user output.

struct JsonHandler {
    stream: Arc<Mutex<Stderr>>,
}

impl Default for JsonHandler {
    fn default() -> Self {
        Self {
            stream: Arc::new(Mutex::new(io::stderr())),
        }
    }
}

impl EventHandler for JsonHandler {
    type Event = ExampleEvent;

    fn handle(&self, event: Self::Event) {
        /* simulate some busy work, so we can more easily follow the user output */
        thread::sleep(Duration::from_secs(1));
        /* simulate some busy work */
        let message = event.to_json();

        let mut out = self.stream.lock().unwrap();
        let _ = writeln!(out, "{}", message);
        let _ = out.flush();
    }

    fn finish(&self) {
        let mut out = self.stream.lock().unwrap();

        let message = format!("{{ \"event\" : \"program-finished\", \"success\" : true }}");

        let _ = writeln!(out, "{}", message);
        let _ = out.flush();
    }
}
