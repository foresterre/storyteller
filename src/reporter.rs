/// A reporter (a type of transmitter) which sends events (the message to be transmitted) to
/// a listener (a type of receiver).
pub trait EventReporter {
    /// The type of message send from a reporter to some listener.
    type Event;

    /// The type of error which will occur in case of a failure to report an event.
    type Err;

    /// Guarantees that this reporter has been disconnected.
    ///
    /// Returned by [`disconnect`] and required by [`HandlerGuard::join`].
    /// Holding this value is a compile-time proof that [`disconnect`] has already been called,
    /// which allows [`HandlerGuard::join`] to enforce correct ordering and avoid a deadlock.
    ///
    /// [`disconnect`]: EventReporter::disconnect
    /// [`HandlerGuard::join`]: crate::HandlerGuard::join
    type DisconnectToken;

    /// Send an event to listeners.
    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err>;

    /// Disconnect the reporter from the [`EventListener`], returning a proof-of-disconnect
    /// token.
    ///
    /// Pass the returned [`Self::DisconnectToken`] token to [`HandlerGuard::join`] to wait for
    /// the handler to finish processing queued events.
    ///
    /// # Ordering
    ///
    /// `disconnect` mustr be called before calling [`HandlerGuard::join`], which is
    /// enforced by the `token` at compile time.
    ///
    /// [`EventListener`]: crate::EventListener
    /// [`HandlerGuard::join`]: crate::HandlerGuard::join
    fn disconnect(self) -> Result<Self::DisconnectToken, Self::Err>;
}
