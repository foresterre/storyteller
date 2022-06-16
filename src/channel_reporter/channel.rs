//! Channels which can be used by the `ChannelReporter` and `ChannelEventListener`.

use std::fmt::Formatter;
use std::{any, fmt};

// --- Event channel variants

/// A channel over which events are sent, from the `ChannelReporter` to the `ChannelEventListener`.
pub fn event_channel<Event>() -> (EventSender<Event>, EventReceiver<Event>) {
    let (sender, receiver) = crossbeam_channel::unbounded::<Event>();

    (EventSender(sender), EventReceiver(receiver))
}

/// A sender, used by `ChannelReporter` and `ChannelEventListener`.
#[derive(Clone)]
pub struct EventSender<T>(crossbeam_channel::Sender<T>);

impl<T> EventSender<T> {
    pub fn send(&self, message: T) -> Result<(), EventSendError<T>> {
        self.0.send(message).map_err(|err| EventSendError(err.0))
    }

    /// When all senders are disconnected, the channel is disconnected
    pub fn disconnect(self) {
        drop(self.0)
    }
}

/// A receiver, used by `ChannelReporter` and `ChannelEventListener`.
pub struct EventReceiver<T>(crossbeam_channel::Receiver<T>);

impl<T> EventReceiver<T> {
    pub fn recv(&self) -> Result<T, EventRecvError> {
        self.0.recv().map_err(|_| EventRecvError)
    }
}

impl<T> Clone for EventReceiver<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct EventSendError<T>(pub T);

impl<T> fmt::Debug for EventSendError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("EventSendError({})", any::type_name::<T>()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EventRecvError;
