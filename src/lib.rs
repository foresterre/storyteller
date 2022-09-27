//! This library is intended to be used by tools, such as cli's, which have multiple user interface
//! options, through which they can communicate, while also having various separate commands (or
//! flows) which require each to carefully specify their own output formatting.
//!
//! The library consists of three primary building blocks, and a default implementation on top
//! of these building blocks.
//!
//! The three building blocks are:
//! * [`EventHandler`]: The event handler receives an event (our message type) as input,
//!     and decides what to do with this message. Examples include a json-lines handler which
//!     prints events to to stderr, a progress bar, a faked handler which collects events for
//!     which may be asserted on in software tests, or a handler which sends websocket messages
//!     for each event.
//!     
//! * [`EventReporter`]: Used to communicate messages to a user.
//! * [`EventListener`]: Receives the messages, send by a reporter and runs the `EventHandler`
//!     where appropriate.
//!
//! On top of these building blocks, a channel based implementation is provided which runs the `EventHandler`
//! in a separate thread.
//! To use this implementation, consult the docs for the [`ChannelReporter`],
//! and the [`ChannelEventListener`].
//!
//! [`EventHandler`]: crate::EventHandler
//! [`EventReporter`]: crate::EventReporter
//! [`EventListener`]: `crate::EventListener`
//! [`ChannelReporter`]: crate::ChannelReporter
//! [`ChannelEventListener`]: crate::ChannelEventListener

mod handler;
mod listener;
mod reporter;
#[cfg(test)]
mod tests;

#[cfg(feature = "channel_reporter")]
mod channel_reporter;

#[cfg(feature = "channel_reporter")]
pub use channel_reporter::{
    channel::event_channel, channel::EventReceiver, channel::EventSendError, channel::EventSender,
    listener::ChannelEventListener, listener::ChannelFinalizeHandler, reporter::ChannelReporter,
    reporter::EventReporterError,
};
pub use handler::EventHandler;
pub use listener::{EventListener, FinishProcessing};
pub use reporter::EventReporter;
