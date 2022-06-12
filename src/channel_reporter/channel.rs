//! Channels which can be used by the `ChannelReporter` and `ChannelEventListener`.

use crate::Disconnect;
use std::fmt::Formatter;
use std::{any, fmt};

// --- Event channel variants

/// A channel over which events are sent, from the `ChannelReporter` to the `ChannelEventListener`.
pub fn event_channel<Event>() -> (EventSender<Event>, EventReceiver<Event>) {
    let (sender, receiver) = crossbeam_channel::unbounded::<Event>();

    (EventSender(sender), EventReceiver(receiver))
}

/// A sender, used by `ChannelReporter` and `ChannelEventListener`.
pub struct EventSender<T>(crossbeam_channel::Sender<T>);

impl<T> EventSender<T> {
    pub fn send(&self, message: T) -> Result<(), EventSendError<T>> {
        self.0.send(message).map_err(|err| EventSendError(err.0))
    }
}

/// A receiver, used by `ChannelReporter` and `ChannelEventListener`.
pub struct EventReceiver<T>(crossbeam_channel::Receiver<T>);

impl<T> EventReceiver<T> {
    pub fn recv(&self) -> Result<T, EventRecvError> {
        self.0.recv().map_err(|_| EventRecvError)
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

// --- Disconnect channel variants

/// A sender, used to communicate Disconnect's between the `ChannelReporter` and `ChannelEventListener`.
pub struct DisconnectSender(crossbeam_channel::Sender<Disconnect>);

impl DisconnectSender {
    pub fn acknowledge_disconnection(&self) -> Result<(), DisconnectSendError> {
        self.0.send(Disconnect).map_err(|_| DisconnectSendError)
    }
}

/// A receiver, used to communicate Disconnect's between the `ChannelReporter` and `ChannelEventListener`.
pub struct DisconnectReceiver(crossbeam_channel::Receiver<Disconnect>);

impl DisconnectReceiver {
    pub(crate) fn recv(&self) -> Result<Disconnect, DisconnectRecvError> {
        self.0.recv().map_err(|_| DisconnectRecvError)
    }
}

/// A channel used to by the `ChannelEventListener` to acknowledge the disconnection of the `ChannelReporter`.
///
/// Allows us to wait
pub fn disconnect_channel() -> (DisconnectSender, DisconnectReceiver) {
    let (sender, receiver) = crossbeam_channel::bounded::<Disconnect>(0);

    (DisconnectSender(sender), DisconnectReceiver(receiver))
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DisconnectSendError;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DisconnectRecvError;
