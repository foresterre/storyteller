use crate::EventHandler;
use std::sync::Arc;

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
/// [`Reporter`]: crate::Reporter
/// [`EventHandler`]: crate::EventHandler
/// [`ChannelEventListener`]: crate::ChannelEventListener
/// [`ChannelReporter`]: crate::ChannelReporter
pub trait EventListener {
    /// The type of message send from a reporter to some listener.
    type Event;

    /// Can be used to stop running the event handler.
    type FinishProcessingHandle: FinishProcessing;

    fn run_handler<H>(&self, handler: Arc<H>) -> Self::FinishProcessingHandle
    where
        H: EventHandler<Event = Self::Event> + 'static;
}

/// Provides a way for to wait for [`EventListener`] instances to let their [`EventHandler`] instances
/// finish up on processing their received events.
///
/// ### Caution: Infinite looping
///
/// If the [`EventListener`] runs the handler in a loop,
/// then calling the [`FinishProcessing::finish_processing`]
/// method may cause an infinite loop if it does not contain a way to break out of
/// this loop. Usually, the [`Reporter::disconnect`] method should provide a way to break
/// out of the loop.
///
/// [`EventListener`]: crate::EventListener
/// [`EventHandler`]: crate::EventHandler
/// [`FinishProcessing::finish_processing`]: crate::FinishProcessing::finish_processing
/// [`Reporter::disconnect`]: crate::Reporter::disconnect
#[must_use]
pub trait FinishProcessing {
    type Err;

    /// Finish up processing of received events.
    ///
    /// See [`FinishProcessing`] for more.
    ///
    /// [`FinishProcessing`]: crate::FinishProcessing
    fn finish_processing(self) -> Result<(), Self::Err>;
}
