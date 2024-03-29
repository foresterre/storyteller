#![cfg(feature = "channel_reporter")]
#![allow(unused_must_use)]

// TODO: now we have a working proof of concept, and are starting to refine the library,
//  we also should start testing it properly! ^^
// TODO: test a MultiHandler like the one in rust-experiment-air3

use crate::{
    event_channel, ChannelEventListener, ChannelReporter, EventHandler, EventListener,
    EventReporter, FinishProcessing,
};
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

// -- a Handler

struct IndicatifHandler {
    bar: indicatif::ProgressBar,
}

impl Default for IndicatifHandler {
    fn default() -> Self {
        let bar = indicatif::ProgressBar::new(10);
        bar.enable_steady_tick(Duration::from_millis(250));

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
        thread::sleep(Duration::from_millis(100));
        let message = serde_json::to_string(&event).unwrap_or_default();

        let mut out = self.stdout.lock().unwrap();
        writeln!(out, "{}", message);
        out.flush();
    }

    fn finish(&self) {}
}

#[test]
fn bar() {
    let (sender, receiver) = event_channel::<ExampleEvent>();

    let handler = IndicatifHandler::default();
    let reporter = ChannelReporter::new(sender);
    let listener = ChannelEventListener::new(receiver);

    let finalize_handle = listener.run_handler(Arc::new(handler));

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

    reporter.disconnect().unwrap();
    finalize_handle.finish_processing().unwrap();
}

#[test]
fn json() {
    let (sender, receiver) = event_channel::<ExampleEvent>();

    let handler = JsonHandler::default();
    let reporter = ChannelReporter::new(sender);
    let listener = ChannelEventListener::new(receiver);

    let finalize_handle = listener.run_handler(Arc::new(handler));

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

    reporter.disconnect().unwrap();
    finalize_handle.finish_processing().unwrap();
}
