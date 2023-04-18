# 🎙 Storyteller
_A library for working with user output_

## Table of contents

* 👋 [Introduction](#introduction)
* 🖼 [Visualized introduction](#visualized-introduction)
* 📄 [Examples](#examples)
  * [Real world usage](#real-world-example)
  * [Hello world example](#hello-world-example)
* ❓ [Origins](#origins)
* 💖 [Contributions & Feedback](#contributions)
* 🧾 [License](#license)

## Introduction

This library is intended to be used by tools, such as cli's, which have multiple user interface
options through which they can communicate, while also having various separate commands (or
flows) which require each to carefully specify their own output formatting.

The library consists of three primary building blocks, and a default implementation on top
of these building blocks. It helps you setup your program architecture 

The three building blocks are:
* `EventHandler`: The event handler which deals with the user output, for example:
	* A handler which formats events as json-lines, and prints them to stderr
	* A handler which updates a progress bar
	* A handler which collects events for software testing
	* A handler which sends websocket messages for each event
	* A handler which updates a user interface
    
* `EventReporter`: Called during your program logic. 
	Used to communicate user output to a user. The reporter is invoked with an Event during the programs logic, so you don't have to deal with formatting and display details in the middle of the program flow.

* `EventListener`: Receives events, send by a reporter and runs the `EventHandler`. Usually spins upa separate thread so it won't block.

On top of these building blocks, a channel based implementation is provided which runs the `EventHandler`
in a separate thread. To use this implementation, consult the docs for the `ChannelReporter`, and the
`ChannelEventListener`.

In addition to these provided elements, you have to:
* Define a type which can be used as Event
* Define one or more EventHandlers (i.e. `impl EventHandler<Event = YourEventType>`).

## Visualized introduction

Click [here](https://raw.githubusercontent.com/foresterre/storyteller/main/docs/sketches/introduction_dark.svg) for a larger version. 
![visualized introduction sketch](docs/sketches/introduction_dark.png)
([light svg](https://raw.githubusercontent.com/foresterre/storyteller/main/docs/sketches/introduction.svg), [dark svg](https://raw.githubusercontent.com/foresterre/storyteller/main/docs/sketches/introduction_dark.svg), [light png](https://raw.githubusercontent.com/foresterre/storyteller/main/docs/sketches/introduction.png), [dark png](https://raw.githubusercontent.com/foresterre/storyteller/main/docs/sketches/introduction_dark.png))

## Examples

### Real-world Example

Storyteller is used by [cargo-msrv](https://github.com/foresterre/cargo-msrv/tree/44444c55608edb749c3cbcd5b6983d7f8846b452/src/reporter) since `v0.16`. To preview how events are specified, you could click around in its [event](https://github.com/foresterre/cargo-msrv/tree/44444c55608edb749c3cbcd5b6983d7f8846b452/src/reporter/event) module. In the [handler](https://github.com/foresterre/cargo-msrv/tree/44444c55608edb749c3cbcd5b6983d7f8846b452/src/reporter/handler) module, several handlers can be found, such as one which writes JSON, one which prints pretty human readable output, one which prints minimal final result output used by shell commands, one which discards output and one which is used for integration testing.

### A taste of [storyteller](https://github.com/foresterre/storyteller)

```rust
use std::cell::RefCell;
use std::hash::Hasher;
use std::io::{Stderr, Write};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};
use storyteller::{EventHandler, FinishProcessing};

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

        eprintln!("{}", serialized_event);
    }
}

// See the test function `bar` in src/tests.rs for an example where the handler is a progress bar.
fn main() {
    let (sender, receiver) = event_channel::<Event>();

    // Handlers are implemented by you. Here you find one which writes jsonlines messages to stderr.
    // This can be anything, for example a progress bar (see src/tests.rs for an example of this),
    // a fake reporter which collects events for testing or maybe even a "MultiHandler<'h>" which
    // consists of a Vec<&'h dyn EventHandler> and executes multiple handlers under the hood.
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

    // Within the ChannelReporter, the sender is dropped, thereby disconnecting the channel
    // Already sent events can still be processed.
    let _ = reporter.disconnect();

    // To keep the processing of already sent events alive, we block the handler
    let _ = fin.finish_processing();
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

```

## Origins

This library is a refined implementation based on an earlier [experiment](https://github.com/foresterre/rust-experiment-air3/blob/main/src/main.rs).
It is intended to be used by, and was developed because of, [cargo-msrv](https://github.com/foresterre/cargo-msrv) 
which has outgrown its current user output implementation.

## Contributions

Contributions, feedback or other correspondence are more than welcome! Feel free to send a message or create an issue 😄.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


#### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
