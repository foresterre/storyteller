use crate::{DisconnectSender, EventHandler, EventListener, EventReceiver};
use std::thread;

/// A listener which uses a channel to receive messages of type `Event`, and uses
/// a thread to run the event handler (in [`crate::ChannelEventListener::run_handler`].
///
/// The channels required to create an instance can be created by calling the [`crate::event_channel`]
/// and [`crate::disconnect_channel`] functions.
///
/// The [`crate::Reporter`] associated with this event listener is the [`crate::ChannelReporter`].
pub struct ChannelEventListener<Event> {
    message_receiver: EventReceiver<Event>,
    disconnect_sender: DisconnectSender,
}

impl<Event> ChannelEventListener<Event> {
    /// Create a new channel based event listener.
    ///
    /// The channels required to create an instance can be created by calling the [`crate::event_channel`]
    /// and [`crate::disconnect_channel`] functions.
    pub fn new(
        message_receiver: EventReceiver<Event>,
        disconnect_sender: DisconnectSender,
    ) -> Self {
        Self {
            message_receiver,
            disconnect_sender,
        }
    }

    /// If you use `ChannelEventListener` by wrapping it, instead of using it directly,
    /// for example if you want to write your own `EventListener` implementation,
    /// you will need this `&EventReceiver` to receive events.
    ///
    /// ### Example
    ///
    /// **NB:** This example should **not** be used on its own! It does not contain a fully working listener!
    /// See [`crate::EventListener`] on how to implement your own listener instead.
    ///
    /// ```no_run
    /// // NB: This example is incomplete!
    /// //     It does not contain a fully working listener!
    ///
    /// use storyteller::{ChannelEventListener, EventHandler, EventListener};
    ///
    /// struct MyEvent;
    ///
    /// struct WrappingListener {
    ///     listener: ChannelEventListener<MyEvent>,
    /// }
    ///
    /// impl EventListener for WrappingListener {
    ///     type Event = MyEvent;
    ///
    ///     fn run_handler<H>(self, handler: H) where H: EventHandler<Event=Self::Event> {     
    ///
    ///         let disconnect_sender = self.listener.disconnect_sender();
    ///         let message_receiver = self.listener.message_receiver(); // <---
    ///
    ///         loop {
    ///             if let Err(_) = message_receiver.recv() {
    ///                  disconnect_sender.acknowledge_disconnection().unwrap();
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn message_receiver(&self) -> &EventReceiver<Event> {
        &self.message_receiver
    }

    /// If you use `ChannelEventListener` by wrapping it, instead of using it directly,
    /// for example if you want to write your own `EventListener` implementation,
    /// you will need this `&DisconnectSender` to acknowledge when a reporter disconnects.
    ///
    /// ### Example
    ///
    /// **NB:** This example should **not*** be used on its own! It does not contain a fully working listener!
    /// See [`crate::EventListener`] on how to implement your own listener instead.
    ///
    /// ```no_run
    /// // NB: This example is incomplete!
    /// //     It does not contain a fully working listener!
    ///
    /// use storyteller::{ChannelEventListener, EventHandler, EventListener};
    ///
    /// struct MyEvent;
    ///
    /// struct WrappingListener {
    ///     listener: ChannelEventListener<MyEvent>,
    /// }
    ///
    /// impl EventListener for WrappingListener {
    ///     type Event = MyEvent;
    ///
    ///     fn run_handler<H>(self, handler: H) where H: EventHandler<Event=Self::Event> {     
    ///
    ///         let disconnect_sender = self.listener.disconnect_sender(); // <---
    ///         let message_receiver = self.listener.message_receiver();
    ///
    ///         loop {
    ///             if let Err(_) = message_receiver.recv() {
    ///                  disconnect_sender.acknowledge_disconnection().unwrap();
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn disconnect_sender(&self) -> &DisconnectSender {
        &self.disconnect_sender
    }
}

impl<Event> EventListener for ChannelEventListener<Event>
where
    Event: Send + 'static,
{
    type Event = Event;

    fn run_handler<H>(self, handler: H)
    where
        H: EventHandler<Event = Self::Event>,
    {
        thread::spawn(move || {
            let disconnect_sender = self.disconnect_sender();
            let message_receiver = self.message_receiver();

            loop {
                match message_receiver.recv() {
                    Ok(message) => handler.handle(message),
                    Err(_disconnect) => {
                        handler.finish();

                        let _ack = disconnect_sender.acknowledge_disconnection();

                        #[cfg(not(feature = "experimental_handle_disconnect_ack"))]
                        {
                            _ack.expect("Failed to send disconnect acknowledgement!");
                        }

                        break;
                    }
                }
            }
        });
    }
}
