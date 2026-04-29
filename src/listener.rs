use crate::EventHandler;
use std::sync::Arc;

/// A listener, which listens to events from a [`EventReporter`],
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
/// [`EventReporter`]: crate::EventReporter
/// [`EventHandler`]: crate::EventHandler
/// [`ChannelEventListener`]: crate::ChannelEventListener
/// [`ChannelReporter`]: crate::ChannelReporter
pub trait EventListener {
    /// The type of message send from a reporter to some listener.
    type Event;

    /// A guard that keeps the handler running and can be used to wait for it to finish.
    type Guard: HandlerGuard;

    fn run_handler<H>(&self, handler: Arc<H>) -> Self::Guard
    where
        H: EventHandler<Event = Self::Event> + 'static;
}

/// A guard over a running [`EventHandler`].
///
/// Returned by [`EventListener::run_handler`] and held until the caller is ready to wait
/// for the handler to finish processing all queued events.
///
/// ### Ordering: call [`EventReporter::disconnect`] first
///
/// If the [`EventListener`] runs the handler in a loop, calling [`HandlerGuard::join`]
/// without first disconnecting the reporter will cause an infinite loop (or deadlock),
/// because the loop only exits when the channel is disconnected.
///
/// The [`Self::Token`] associated type enforces correct ordering at compile time. The proof
/// value can only be obtained by calling [`EventReporter::disconnect`], so passing it here
/// guarantees the disconnect happened first.
///
/// For implementations that have no ordering requirement, you can use `type Token = ()`.
///
/// [`EventListener`]: crate::EventListener
/// [`EventHandler`]: crate::EventHandler
/// [`HandlerGuard::join`]: crate::HandlerGuard::join
/// [`EventReporter::disconnect`]: crate::EventReporter::disconnect
#[must_use]
pub trait HandlerGuard {
    type Err;

    /// A typed proof that the associated [`EventReporter`] has been disconnected.
    ///
    /// For implementations backed by a channel (where disconnect ordering matters), this is
    /// the [`EventReporter::DisconnectToken`] token returned by [`EventReporter::disconnect`].
    ///
    /// Implementations with no ordering requirement can just use `type Token = ()`.
    ///
    /// [`EventReporter`]: crate::EventReporter
    /// [`EventReporter::DisconnectToken`]: crate::EventReporter::DisconnectToken
    /// [`EventReporter::disconnect`]: crate::EventReporter::disconnect
    type Token;

    /// Wait for the handler to finish processing all queued events.
    ///
    /// `token` must be the [`Self::Token`] value produced by [`EventReporter::disconnect`].
    /// This guarantees the reporter was disconnected before this method is called.
    ///
    /// [`EventReporter::disconnect`]: crate::EventReporter::disconnect
    fn join(self, token: Self::Token) -> Result<(), Self::Err>;
}
