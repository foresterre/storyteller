use crate::EventHandler;
use crossbeam_channel::{Receiver, Sender};

/// A receiver, which receives messages and optionally acts upon them.
// Consider: maybe rename back to EventWriter, concerning intended use
pub trait EventListener {
    /// The type of message send from a reporter to some listener.
    type Event;

    /// The type of message to be send to the reporter, when a listener disconnects.
    type Disconnect;
}

/// A specialized version of an event listener which uses
pub trait ChannelEventListener: EventListener {
    /// Setups an instance of this type of listener.
    ///
    /// **Handler**
    ///
    /// The handler here refers to an event handler which accepts events
    /// compatible with this listener.
    ///
    /// Possibly, in a future version, this handler will move to the [`EventListener`].
    /// That will require us to make sure we can start a thread from the [`MpscEventListener::connect_with`]
    /// method, and use the listener from within the thread.
    // Consider: For the comment above, possibly an Arc will do
    //
    // Consider: change ChannelEventListener to a struct, then move fn(handler: H)
    // to EventListener;
    fn setup<H>(
        message_receiver: Receiver<Self::Event>,
        disconnect_sender: Sender<Self::Disconnect>,
        handler: H,
    ) -> Self
    where
        H: EventHandler<Event = Self::Event>;
}
