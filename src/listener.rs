use crate::EventHandler;

/// A listener, which listens to events from a [`Reporter`],
/// and can act upon these events by using an [`EventHandler`].
///
/// You may use the included [`ChannelEventListener`] in combination with the
/// included [`ChannelReporter`], to create a reporter and listener which use a
/// crossbeam channel to communicate. This assumes the `channel_reporter` feature is enabled
/// (default).
///
/// The listener should not block (you can, for example, spawn a thread to which you can communicate
/// using a channel).
///
/// ### Example
///
/// ```
/// use storyteller::{EventHandler, EventListener};
///
/// struct MyEvent;
///
/// struct WrappingListener<L: EventListener<Event = MyEvent>> {
///     listener: L,
/// }
///
/// impl<L: EventListener<Event = MyEvent>> EventListener for WrappingListener<L> {
///     type Event = MyEvent;
///
///     fn run_handler<H>(self, handler: H) where H: EventHandler<Event=Self::Event> {
///         // For this example, we delegate the implementation to the wrapped listener...
///         self.listener.run_handler(handler);
///         
///         // Usually you would instead spawn a thread here, to prevent the listener from blocking
///         // your main flow.
///
///         // Within the thread you can then handle the messages, e.g. in a loop.
///         
///         // For example, if we would have wrapped a ChannelListener instead:
///         
///         // ```rust
///         //      std::thread::spawn(move || {
///         //             let disconnect_sender = self.listener.disconnect_sender();
///         //             let message_receiver = self.listener.message_receiver();
///         //
///         //             loop {
///         //                 match message_receiver.recv() {
///         //                     Ok(message) => handler.handle(message),
///         //                     Err(_disconnect) => {
///         //                         handler.finish();
///         //                         disconnect_sender.acknowledge_disconnection().unwrap();
///         //                         break;
///         //                     }
///         //                 }
///         //             }
///         //         });
///         // ```
///     }
/// }
/// ```
///
/// [`Reporter`]: crate::Reporter
/// [`EventHandler`]: crate::EventHandler
/// [`ChannelEventListener`]: crate::ChannelEventListener
/// [`ChannelReporter`]: crate::ChannelReporter
pub trait EventListener {
    /// The type of message send from a reporter to some listener.
    type Event;

    fn run_handler<H>(self, handler: H)
    where
        H: EventHandler<Event = Self::Event>;
}
