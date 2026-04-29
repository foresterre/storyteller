use crate::channel_reporter::reporter::{ChannelReporter, DisconnectToken, EventReporterError};
use crate::listener::HandlerGuard;
use crate::reporter::EventReporter;
use crate::{EventHandler, EventListener, EventReceiver};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

/// A listener which uses a channel to receive messages of type `Event`, and uses
/// a thread to run the event handler (in [`ChannelEventListener::run_handler`]).
///
/// The channel based receiver required to create an instance can be created by calling the
/// [`event_channel()`] function.
///
/// The [`EventReporter`] associated with this event listener is the [`ChannelReporter`].
///
/// [`ChannelEventListener::run_handler`]: crate::ChannelEventListener::run_handler
/// [`event_channel()`]: crate::event_channel
/// [`EventReporter`]: crate::EventReporter
/// [`ChannelReporter`]: crate::ChannelReporter
pub struct ChannelEventListener<Event> {
    event_receiver: EventReceiver<Event>,
}

impl<Event> ChannelEventListener<Event> {
    /// Create a new channel based event listener.
    ///
    /// The channel based receiver required to create an instance can be created by calling the
    /// [`event_channel()`] function.
    ///
    /// [`event_channel()`]: crate::event_channel
    pub fn new(event_receiver: EventReceiver<Event>) -> Self {
        Self { event_receiver }
    }
}

impl<Event> EventListener for ChannelEventListener<Event>
where
    Event: Send + 'static,
{
    type Event = Event;
    type Guard = ChannelHandlerGuard;

    fn run_handler<H>(&self, handler: Arc<H>) -> Self::Guard
    where
        H: EventHandler<Event = Self::Event> + 'static,
    {
        let event_receiver = self.event_receiver.clone();

        let handle = thread::spawn(move || 'evl: loop {
            match event_receiver.recv() {
                Ok(message) => handler.handle(message),
                Err(_disconnect) => {
                    handler.finish();
                    break 'evl;
                }
            }
        });

        ChannelHandlerGuard::new(handle)
    }
}

/// A [`HandlerGuard`] for the [`ChannelEventListener`].
///
/// Holds the handler thread and allows waiting for it to finish via [`HandlerGuard::join`].
///
/// ### Ordering
///
/// [`HandlerGuard::join`] requires a [`DisconnectToken`] proof token produced by
/// [`ChannelReporter::disconnect`]. This enforces that at compile time that the reporter is
/// disconnected before joining the handler thread. Calling them in the wrong order is a
/// compile error iso a silent deadlock.
///
/// ### Drop behaviour
///
/// Dropping this guard without calling [`join`] is a programming error and will panic
/// (unless the thread is already unwinding).
///
/// [`HandlerGuard`]: crate::HandlerGuard
/// [`HandlerGuard::join`]: crate::HandlerGuard::join
/// [`ChannelEventListener`]: crate::ChannelEventListener
/// [`ChannelReporter::disconnect`]: crate::ChannelReporter::disconnect
/// [`join`]: HandlerGuard::join
#[must_use]
pub struct ChannelHandlerGuard {
    handle: Option<JoinHandle<()>>,
}

impl ChannelHandlerGuard {
    fn new(handle: JoinHandle<()>) -> Self {
        Self {
            handle: Some(handle),
        }
    }
}

impl ChannelHandlerGuard {
    /// Disconnect the reporter and join the handler guard in one step.
    ///
    /// This is a shorthand for:
    ///
    /// ```ignore
    /// let token = reporter.disconnect()?;
    /// guard.join(token)?;
    /// ```
    ///
    /// If the handler thread panicked, the panic is re-raised in the calling thread.
    pub fn disconnect_and_join<Event: Send>(
        self,
        reporter: ChannelReporter<Event>,
    ) -> Result<(), EventReporterError<Event>> {
        let token = reporter.disconnect()?;
        self.join(token).expect("handler thread panicked");
        Ok(())
    }
}

impl HandlerGuard for ChannelHandlerGuard {
    type Err = ();
    type Token = DisconnectToken;

    fn join(mut self, _token: DisconnectToken) -> Result<(), Self::Err> {
        self.handle.take().unwrap().join().map_err(|_| ())
    }
}

impl Drop for ChannelHandlerGuard {
    fn drop(&mut self) {
        if self.handle.is_some() && !thread::panicking() {
            panic!(
                "ChannelHandlerGuard dropped without calling join(). \
                 Call reporter.disconnect() then guard.join(token) before dropping"
            );
        }
    }
}
