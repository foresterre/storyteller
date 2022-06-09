use crate::{Disconnect, DisconnectReceiver, EventSender, Reporter};

/// A specialized type of reporter which uses a channel to transmit messages.
///
/// Use [`Reporter::disconnect`] to keep the channels alive until a disconnect has been requested.
/// Otherwise, the reporter hang up after the regular program flow is finished, and events are sent,
/// which may be before all events have been handled in our separate thread.
///
/// The channels required to create an instance can be created by calling the [`crate::event_channel`]
/// and [`crate::disconnect_channel`] functions.
///
/// The [`crate::EventListener`] associated with this reporter is the [`crate::ChannelEventListener`].
pub struct ChannelReporter<Event> {
    message_sender: EventSender<Event>,
    disconnect_receiver: DisconnectReceiver,
}

impl<Event> ChannelReporter<Event> {
    /// Setup a reporter which uses two channels:
    /// 1. the `message_sender` channel sends events to listeners.
    /// 2. the `disconnect_receiver` which is practically a oneshot channel which receives one
    ///    message upon successful disconnection.
    ///
    /// The channels required to create an instance can be created by calling the [`crate::event_channel`]
    /// and [`crate::disconnect_channel`] functions.
    ///
    /// NB: Make sure you take care of the scope of the sender and receiver.
    pub fn new(
        message_sender: EventSender<Event>,
        disconnect_receiver: DisconnectReceiver,
    ) -> Self {
        Self {
            message_sender,
            disconnect_receiver,
        }
    }
}

impl<Event> Reporter for ChannelReporter<Event> {
    type Event = Event;
    type Err = ReporterError<Event>;

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.message_sender
            .send(event.into())
            .map_err(ReporterError::SendError)
    }

    /// Disconnect the sender, and wait for a response from the listener.
    ///
    /// Allows the program to wait for the listener to finish up queued events.
    fn disconnect(self) -> Result<Disconnect, Self::Err> {
        // close the channel
        //
        // `message_receiver.recv()` will receive an `Err(RecvError)`
        drop(self.message_sender);

        self.disconnect_receiver
            .recv()
            .map_err(ReporterError::DisconnectError)
    }
}

#[derive(Debug)]
pub enum ReporterError<Event> {
    SendError(crate::EventSendError<Event>),
    DisconnectError(crate::DisconnectRecvError),
}
