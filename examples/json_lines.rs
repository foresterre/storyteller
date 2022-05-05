use std::io::{Stderr, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};
use storyteller::crossbeam_channel::{Receiver, Sender};
use storyteller::{ChannelEventListener, ChannelReporter, EventHandler, EventListener, Reporter};

fn main() {
    let (sender, receiver) = crossbeam_channel::unbounded::<ExampleEvent>();
    let (disconnect_sender, disconnect_receiver) = crossbeam_channel::bounded::<Disconnect>(0);

    let handler = JsonHandler::default();
    let reporter = CargoMsrvReporter::setup(sender, disconnect_receiver);
    let _listener = CargoMsrvListener::setup(receiver, disconnect_sender, handler);

    #[allow(unused_must_use)]
    {
        reporter.report_event(ExampleEvent::text("[status]\t\tOne"));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::text("[status::before]\tTwo before reset"));
        reporter.report_event(ExampleEvent::event(MyEvent::Reset));
        reporter.report_event(ExampleEvent::text("[status::after]\t\tTwo after reset"));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::text("[status]\t\tThree"));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::event(MyEvent::Increment));
        reporter.report_event(ExampleEvent::text("[status]\t\tFour"));
    }
    let _ = reporter.disconnect();
}

// ------- Events + Disconnect

enum ExampleEvent {
    Event(MyEvent),
    Text(String),
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
    pub fn to_json(&self) -> String {
        match self {
            Self::Event(event) => event.to_json(),
            Self::Text(msg) => format!("{{ \"event\" : \"message\", \"value\" : \"{}\" }}", msg),
        }
    }
}

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

struct Disconnect;

// ------- Reporter

struct CargoMsrvReporter {
    message_sender: Sender<ExampleEvent>,
    disconnect_receiver: Receiver<Disconnect>,
}

impl Reporter for CargoMsrvReporter {
    type Event = ExampleEvent;
    type Disconnect = Disconnect;
    type Err = ();

    fn report_event(&self, event: Self::Event) -> Result<(), Self::Err> {
        self.message_sender.send(event).map_err(|_| ())
    }

    fn disconnect(self) -> Disconnect {
        // close the channel
        //
        // `message_receiver.recv()` will receive an `Err(RecvError)`
        drop(self.message_sender);

        self.disconnect_receiver.recv().unwrap()
    }
}

impl ChannelReporter for CargoMsrvReporter {
    fn setup(
        message_sender: Sender<Self::Event>,
        disconnect_receiver: Receiver<Self::Disconnect>,
    ) -> Self {
        Self {
            message_sender,
            disconnect_receiver,
        }
    }
}

// ------- Listener

struct CargoMsrvListener {
    #[allow(unused)]
    thread_handle: thread::JoinHandle<()>,
}

impl EventListener for CargoMsrvListener {
    type Event = ExampleEvent;
    type Disconnect = Disconnect;
}

impl ChannelEventListener for CargoMsrvListener {
    fn setup<H>(
        message_receiver: Receiver<Self::Event>,
        disconnect_sender: Sender<Self::Disconnect>,
        handler: H,
    ) -> Self
    where
        H: EventHandler<Event = Self::Event>,
    {
        let thread_handle = thread::spawn(move || {
            let disconnect_sender = disconnect_sender;

            loop {
                match message_receiver.recv() {
                    Ok(message) => handler.handle(message),
                    Err(_disconnect) => {
                        handler.finish();
                        disconnect_sender.send(Disconnect).unwrap();
                        break;
                    }
                }
            }
        });

        Self { thread_handle }
    }
}

// ----- A handler

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
        thread::sleep(Duration::from_secs(1));
        let message = event.to_json();

        let mut out = self.stream.lock().unwrap();
        let _ = write!(out, "{}\n", message);
        let _ = out.flush();
    }

    fn finish(&self) {
        let mut out = self.stream.lock().unwrap();

        let message = format!("{{ \"event\" : \"program-finished\", \"success\" : true }}");

        let _ = write!(out, "{}\n", message);
        let _ = out.flush();
    }
}
