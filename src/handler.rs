/// This can be anything, for example a progress bar, a fake reporter which collects events
/// for testing, a service which sends the events over HTTP, or maybe even a `MultiHandler` which
/// consists of a `Vec<Box<dyn EventHandler>>` and executes multiple handlers under the hood.
pub trait EventHandler: Send + 'static {
    /// The type of event to be handled.
    /// Usually the same type as send from a [`crate::Reporter`] to [`crate::Listener`].
    type Event;

    /// Act upon some received event.
    fn handle(&self, event: Self::Event);

    /// A final action which can be performed when no more events will be received, for example
    /// when the message channel will be disconnected.
    ///
    /// It is up to the [`crate::EventListener`] to call this method.
    fn finish(&self) {}
}
