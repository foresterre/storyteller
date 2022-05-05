///
pub trait EventHandler: Send + 'static {
    /// The type of event to be handled.
    /// Usually the same type as send from a [`crate::Reporter`] to [`crate::Listener`].
    type Event;

    /// Act upon some received event.
    fn handle(&self, event: Self::Event);

    /// A final action which can be performed when no more events will be received, for example
    /// when the message channel will be disconnected.
    fn finish(&self);
}
