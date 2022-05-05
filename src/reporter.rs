use crossbeam_channel::{Receiver, Sender};

/// A reporter (a type of transmitter) which sends events (the message to be transmitted) to
/// a listener (a type of receiver).
pub trait Reporter {
    /// The type of message send from a reporter to some listener.
    type Event;

    /// The type of message to be received by the reporter, when a listener confirms the
    /// disconnect of the reporter.
    type Disconnect;

    /// The type of error which will occur in case of a failure to report an event.
    type Err;

    /// Send an event to listeners.
    fn report_event(&self, event: Self::Event) -> Result<(), Self::Err>;

    /// Request to be disconnected.
    ///
    /// Rendezvous with the listener, possibly allowing it to finish its queue of messages.
    /// The [`Self::Disconnect`] message is supposed to be send by the listener as a
    /// disconnection acknowledgement.
    #[must_use]
    fn disconnect(self) -> Self::Disconnect;
}

/// A specialized type of reporter which uses std::mpsc channels to transmit messages.
///
/// Use [`Reporter::disconnect`] to keep the channels alive until a disconnect has been requested.
/// Otherwise, the reporter will fire all events and hang up early.
pub trait ChannelReporter: Reporter {
    /// Setup a reporter which uses two channels:
    /// 1. the `message_sender` channel sends events to listeners.
    /// 2. the `disconnect_receiver` which is practically a oneshot channel which receives one
    ///    message upon successful disconnection.
    ///
    /// NB: Make sure you take care of the scope of the sender and receiver.
    ///    If the sender goes out of scope, it will send a disconnect message in the
    ///    shape of an [`crossbeam_channel::RecvError`] on `message_receiver.recv()`.
    ///    
    fn setup(
        message_sender: Sender<Self::Event>,
        disconnect_receiver: Receiver<Self::Disconnect>,
    ) -> Self;
}
