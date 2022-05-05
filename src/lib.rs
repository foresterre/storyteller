#![deny(unused)]

mod handler;
mod listener;
mod reporter;
#[cfg(test)]
mod tests;

pub use handler::EventHandler;
pub use {listener::ChannelEventListener, listener::EventListener};
pub use {reporter::ChannelReporter, reporter::Reporter};

// Useful for matching up types
pub use crossbeam_channel;
