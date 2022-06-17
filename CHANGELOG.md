# Changelog

## [Unreleased]

[Unreleased]: https://github.com/foresterre/storyteller/compare/v0.5.0...HEAD

## [0.5.0] - 2022-06-16

### Changed

* Remove Disconnect Channel in `ChannelReporter`
  * Removed all disconnect related types, such as: `Disconnect`, `DisconnectSender`, `DisconnectReceiver`, `disconnect_channel()`
  * Split process of disconnecting channel and waiting for unfinished events to be processed. The former can be done via `Reporter::disconnect()`, the latter via the new `FinishProcessing::finish_processing()`.  As a result, if  `FinishProcessing::finish_processing()` is not called after `Reporter::disconnect()`, events may go unprocessed.
    * Caution: if  `FinishProcessing::finish_processing()` is called before **`ChannelReporter::disconnect()`** (in case of the included `ChannelReporter` and `ChannelListener` implementations), the program will hang since the event handling thread will never be finish via the disconnect mechanism.
  * A `FinishProcessing` implementation is now returned by `EventListener::run_handler`

[0.5.0]: https://github.com/foresterre/bisector/compare/v0.4.0...v0.5.0

## [0.4.0] - 2022-06-09

### Changed

* Let the reporter take anything which can be converted into an Event via `impl Into<Reporter::Event>` instead of raw `Reporter::Event` instances.

[0.4.0]: https://github.com/foresterre/bisector/compare/v0.3.2...v0.4.0
