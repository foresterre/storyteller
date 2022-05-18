use crate::message::Disconnect;

pub(crate) mod channel_reporter;

/// A reporter (a type of transmitter) which sends events (the message to be transmitted) to
/// a listener (a type of receiver).
pub trait Reporter {
    /// The type of message send from a reporter to some listener.
    type Event;

    /// The type of error which will occur in case of a failure to report an event.
    type Err;

    /// Send an event to listeners.
    fn report_event(&self, event: Self::Event) -> Result<(), Self::Err>;

    /// Request to be disconnected.
    ///
    /// Rendezvous with the listener, allowing it to finish its queue of messages.
    /// The [`crate::Disconnect`] message will be send by the listener as a
    /// disconnection acknowledgement.
    fn disconnect(self) -> Result<Disconnect, Self::Err>;
}
