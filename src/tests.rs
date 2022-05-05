#![allow(unused_must_use)]

use crate::{ChannelEventListener, ChannelReporter, EventHandler, EventListener, Reporter};
use crossbeam_channel::{Receiver, Sender};
use serde::Serialize;
use std::io::{Stderr, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

// ------- Events + Disconnect

#[derive(Serialize)]
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

#[derive(Serialize)]
enum MyEvent {
    Increment,
    Reset,
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
        println!("sending event! (T=1)");
        self.message_sender.send(event).map_err(|_| ())
    }

    fn disconnect(self) -> Disconnect {
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
                let recv = message_receiver.recv();

                println!("received event! (T=2)");

                match recv {
                    Ok(message) => handler.handle(message),
                    Err(_disconnect) => {
                        handler.finish();
                        eprintln!("\n\nSender closed!");
                        disconnect_sender.send(Disconnect).unwrap();
                        break;
                    }
                }
            }
        });

        Self { thread_handle }
    }
}

// -----

struct IndicatifHandler {
    bar: indicatif::ProgressBar,
}

impl Default for IndicatifHandler {
    fn default() -> Self {
        let bar = indicatif::ProgressBar::new(10);
        bar.enable_steady_tick(250);

        Self { bar }
    }
}

impl EventHandler for IndicatifHandler {
    type Event = ExampleEvent;

    fn handle(&self, event: Self::Event) {
        /* some work, so we can show the bar progressing!  */
        thread::sleep(Duration::from_secs(1));
        /* some work */

        match event {
            ExampleEvent::Text(message) => {
                self.bar.println(message);
            }
            ExampleEvent::Event(event) => match event {
                MyEvent::Increment => {
                    self.bar.inc(1);
                }
                MyEvent::Reset => {
                    self.bar.reset();
                }
            },
        }
    }

    fn finish(&self) {
        self.bar.finish();
    }
}

// -----

struct JsonHandler {
    stdout: Arc<Mutex<Stderr>>,
}

impl Default for JsonHandler {
    fn default() -> Self {
        Self {
            stdout: Arc::new(Mutex::new(io::stderr())),
        }
    }
}

impl EventHandler for JsonHandler {
    type Event = ExampleEvent;

    fn handle(&self, event: Self::Event) {
        thread::sleep(Duration::from_secs(1));
        let message = serde_json::to_string(&event).unwrap_or_default();

        let mut out = self.stdout.lock().unwrap();
        write!(out, "{}\n", message);
        out.flush();
    }

    fn finish(&self) {}
}

#[test]
fn bar() {
    let (sender, receiver) = crossbeam_channel::unbounded::<ExampleEvent>();
    let (disconnect_sender, disconnect_receiver) = crossbeam_channel::bounded::<Disconnect>(0);

    let handler = IndicatifHandler::default();
    let reporter = CargoMsrvReporter::setup(sender, disconnect_receiver);
    CargoMsrvListener::setup(receiver, disconnect_sender, handler);

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

    // if we didn't call disconnect here, the program would exit before it would be allowed to handle
    // all messages.
    reporter.disconnect();
}

#[test]
fn json() {
    let (sender, receiver) = crossbeam_channel::unbounded::<ExampleEvent>();
    let (disconnect_sender, disconnect_receiver) = crossbeam_channel::bounded::<Disconnect>(0);

    let handler = JsonHandler::default();
    let reporter = CargoMsrvReporter::setup(sender, disconnect_receiver);
    CargoMsrvListener::setup(receiver, disconnect_sender, handler);

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

    reporter.disconnect();
}
