//! Error types for the ordered-select prompt.

use thiserror::Error;

/// All errors that [`OrderedSelect::prompt`](crate::OrderedSelect::prompt) can
/// return.
#[derive(Debug, Error)]
pub enum Error {
    /// stdin is not connected to a terminal (piped, redirected, or closed).
    ///
    /// The caller should fall back to non-interactive behaviour.
    #[error("not a terminal (stdin is not a TTY)")]
    NotATerminal,

    /// The user cancelled with `Esc`, `q`, or `Ctrl+C`.
    #[error("operation cancelled by user")]
    Cancelled,

    /// An I/O error occurred while reading from or writing to the terminal.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// No options were provided.
    #[error("no options provided")]
    EmptyOptions,

    /// A default index was out of range.
    #[error("default index {index} is out of range (options len: {len})")]
    InvalidDefault {
        /// The offending index.
        index: usize,
        /// The number of options.
        len: usize,
    },
}
