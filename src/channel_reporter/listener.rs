use crate::listener::FinishProcessing;
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
    type FinishProcessingHandle = ChannelFinalizeHandler;

    fn run_handler<H>(&self, handler: Arc<H>) -> Self::FinishProcessingHandle
    where
        H: EventHandler<Event = Self::Event> + 'static,
    {
        let event_receiver = self.event_receiver.clone();

        let handle = thread::spawn(move || {
            //
            'evl: loop {
                match event_receiver.recv() {
                    Ok(message) => handler.handle(message),
                    Err(_disconnect) => {
                        handler.finish();
                        break 'evl;
                    }
                }
            }
        });

        ChannelFinalizeHandler::new(handle)
    }
}

/// A [`FinishProcessing`] implementation for the [`ChannelEventListener`].
/// Used to wait for the [`EventHandler`] ran by the `listener` to finish processing
/// events.
///
/// ### Caution: Infinite looping
///
/// Calling [`FinishProcessing::finish_processing`] without first disconnecting
/// the sender channel of the reporter will cause the program to be stuck in an infinite
/// loop.
///
/// The reason for this is that disconnecting the channel causes the loop to process
/// a disconnect event, where we break out of the loop. If this disconnect does not
/// happen, the thread processing events will not be finished, and
/// [`FinishProcessing::finish_processing`] will block, since it waits for the thread
/// to be finished.
///
/// To disconnect the sender channel of the reporter, call [`disconnect`].
///
/// [`FinishProcessing`]: crate::FinishProcessing
/// [`EventHandler`]: crate::EventHandler
/// [`ChannelEventListener`]: crate::ChannelEventListener
/// [`disconnect`]: crate::EventReporter::disconnect
#[must_use]
pub struct ChannelFinalizeHandler {
    handle: JoinHandle<()>,
}

impl ChannelFinalizeHandler {
    fn new(handle: JoinHandle<()>) -> Self {
        Self { handle }
    }
}

impl FinishProcessing for ChannelFinalizeHandler {
    type Err = ();

    fn finish_processing(self) -> Result<(), Self::Err> {
        self.handle.join().map_err(|_| ())
    }
}
