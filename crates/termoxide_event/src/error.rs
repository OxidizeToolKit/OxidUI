//! # Errors — the single error type of the crate
//!
//! Every fallible operation of `termoxide_event` reports an [`Error`], and
//! [`Result`] is the matching alias, following the `std::io::Error` /
//! `std::io::Result` pattern. Having one flat error type means a caller can
//! propagate any failure of the crate with `?`, without unwrapping nested
//! results or converting between several error types.

use std::{fmt, io, sync::mpsc};

use crate::event::Event;

/// Error reported by the terminal-input reader.
#[derive(Debug)]
pub enum Error {
    /// The receiving end of the event channel was dropped, so translated
    /// events can no longer be delivered.
    Channel(mpsc::SendError<Event>),
    /// A `crossterm` terminal operation failed (polling, reading, or
    /// enabling/disabling raw mode).
    Terminal(io::Error),
}

/// A specialized [`Result`](std::result::Result) whose error defaults to the
/// crate's [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Channel(error) => {
                write!(f, "event channel receiver was dropped: {error}")
            },
            Error::Terminal(error) => {
                write!(f, "terminal operation failed: {error}")
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Channel(error) => Some(error),
            Error::Terminal(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_channel_error_describes_dropped_receiver() {
        let error = Error::Channel(mpsc::SendError(Event::ChannelReady));
        assert_eq!(
            format!("{error}"),
            "event channel receiver was dropped: sending on a closed channel"
        );
    }

    #[test]
    fn display_terminal_error_wraps_inner_message() {
        let inner = io::Error::other("boom");
        let error = Error::Terminal(inner);
        assert_eq!(format!("{error}"), "terminal operation failed: boom");
    }

    #[test]
    fn source_exposes_the_inner_error() {
        use std::error::Error as _;

        let channel = Error::Channel(mpsc::SendError(Event::ChannelReady));
        assert!(
            channel
                .source()
                .and_then(|s| s.downcast_ref::<mpsc::SendError<Event>>())
                .is_some(),
            "Channel source should be the inner SendError"
        );

        let terminal = Error::Terminal(io::Error::other("boom"));
        assert!(
            terminal.source().and_then(|s| s.downcast_ref::<io::Error>()).is_some(),
            "Terminal source should be the inner io::Error"
        );
    }
}
