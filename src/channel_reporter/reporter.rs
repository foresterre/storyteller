use crate::{EventReporter, EventSender};
use std::error;
use std::fmt::{Debug, Display, Formatter};

/// A specialized type of reporter which uses a channel to transmit messages.
///
/// Use [`EventReporter::disconnect`] to disconnect the channel by dropping the `sender`.
/// If you want to finish up processing unprocessed events, you may do so by calling
/// the blocking [`FinishProcessing::finish_processing`].
///
/// The channel sender (and channel receiver for the `listener`), required to create a
/// `ChannelReporter` instance can be created by calling the [`event_channel()`]
/// function.
///
/// The [`EventListener`] associated with this reporter is the [`ChannelEventListener`].
///
/// [`EventReporter::disconnect`]: crate::EventReporter::disconnect
/// [`FinishProcessing::finish_processing`]: crate::FinishProcessing::finish_processing
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

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.event_sender
            .send(event.into())
            .map_err(EventReporterError::SendError)
    }

    /// Disconnect the sender.
    #[allow(clippy::unit_arg)]
    fn disconnect(self) -> Result<(), Self::Err> {
        Ok(self.event_sender.disconnect())
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
