/// A reporter (a type of transmitter) which sends events (the message to be transmitted) to
/// a listener (a type of receiver).
pub trait Reporter {
    /// The type of message send from a reporter to some listener.
    type Event;

    /// The type of error which will occur in case of a failure to report an event.
    type Err;

    /// Send an event to listeners.
    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err>;

    /// Disconnect the reporter from the [`EventListener`].
    ///
    /// [`EventListener`]: crate::EventListener
    fn disconnect(self) -> Result<(), Self::Err>;
}
