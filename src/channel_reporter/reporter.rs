use crate::{EventReporter, EventSender};
use std::error;
use std::fmt::{Debug, Display, Formatter};

/// Proof that a [`ChannelReporter`] has been disconnected.
///
/// Produced by [`ChannelReporter::disconnect`] and consumed by
/// [`ChannelHandlerGuard::join`]. Because the inner field is private
/// and this type has no public constructor, the only way to obtain a `DisconnectToken` value
/// is by calling [`ChannelReporter::disconnect`], which should guarantee correct ordering.
///
/// [`ChannelHandlerGuard::join`]: crate::ChannelHandlerGuard::join
pub struct DisconnectToken(());

/// A specialized type of reporter which uses a channel to transmit messages.
///
/// Use [`EventReporter::disconnect`] to disconnect the channel by dropping the `sender`.
/// This returns a [`DisconnectToken`] proof token which must be passed to
/// [`HandlerGuard::join`] to wait for the handler thread to drain and exit.
///
/// The channel sender (and channel receiver for the `listener`), required to create a
/// `ChannelReporter` instance can be created by calling the [`event_channel()`]
/// function.
///
/// The [`EventListener`] associated with this reporter is the [`ChannelEventListener`].
///
/// [`EventReporter::disconnect`]: crate::EventReporter::disconnect
/// [`HandlerGuard::join`]: crate::HandlerGuard::join
/// [`event_channel()`]: crate::event_channel()
/// [`EventListener`]: crate::EventListener
/// [`ChannelEventListener`]: crate::ChannelEventListener
pub struct ChannelReporter<Event> {
    event_sender: EventSender<Event>,
}

impl<Event> ChannelReporter<Event> {
    /// Setup a reporter which uses a channel.
    ///
    /// The channel required to create an instance can be created by calling the [`crate::event_channel`]
    /// function.
    pub fn new(event_sender: EventSender<Event>) -> Self {
        Self { event_sender }
    }
}

impl<Event> EventReporter for ChannelReporter<Event> {
    type Event = Event;
    type Err = EventReporterError<Event>;
    type DisconnectToken = DisconnectToken;

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.event_sender
            .send(event.into())
            .map_err(EventReporterError::SendError)
    }

    /// Disconnect the sender, returning a [`DisconnectToken`] token.
    ///
    /// Pass the token to [`HandlerGuard::join`] to wait for the handler thread to finish
    /// draining the queue. The token enforces that this call happens first.
    ///
    /// [`HandlerGuard::join`]: crate::HandlerGuard::join
    fn disconnect(self) -> Result<DisconnectToken, Self::Err> {
        self.event_sender.disconnect();
        Ok(DisconnectToken(()))
    }
}

pub enum EventReporterError<Event> {
    SendError(crate::EventSendError<Event>),
}

impl<Event> Debug for EventReporterError<Event> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SendError(_) => f.write_fmt(format_args!(
                "SendError(EventSendError({}))",
                std::any::type_name::<Event>()
            )),
        }
    }
}

impl<Event> Display for EventReporterError<Event>
where
    Event: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SendError(crate::EventSendError(ev)) => f.write_fmt(format_args!(
                "SendError(EventSendError({} = '{}'))",
                std::any::type_name::<Event>(),
                ev
            )),
        }
    }
}

impl<Event> error::Error for EventReporterError<Event> where Event: Display {}
