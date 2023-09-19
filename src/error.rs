//! Error management module
//! 
//! Provides all csv error conversion and description
//! Also provides `Result` as a alias of `Result<_, Error>

use std::fmt;
use std::io;

/// An error produced by an operation on CSV data.
#[derive(Debug)]
pub enum Error {
    /// An error reported by the type-based decoder.
    Decode(String),
    /// An error reported by the CSV parser.
    Parse(String),
    /// An error originating from reading or writing to the underlying buffer.
    Io(io::Error),
    /// An error originating from finding end of line instead of a column.
    EOL,
    /// Unescaped quote
    UnescapedQuote,
    /// Unexpected quote in a column which is non quoted column
    UnexpextedQuote,
    /// Column count mismatch
    ColumnMismatch(usize, usize),
}

/// Result type
pub type Result<T> = ::std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Decode(ref msg) => write!(f, "CSV decode error: {}", msg),
            Error::Parse(ref err) => write!(f, "{}", err),
            Error::Io(ref err) => write!(f, "{}", err),
            Error::EOL => write!(f, "Trying to access column but found End Of Line"),
            Error::UnescapedQuote => write!(f, "A CSV column has an unescaped quote"),
            Error::UnexpextedQuote => write!(f, "A CSV column has a quote but the entire column value is not quoted"),
            Error::ColumnMismatch(exp, cur) => write!(f, "Expectiong {} columns, found {}", exp, cur),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Decode(..) => "CSV decoding error",
            Error::Parse(..) => "CSV parse error",
            Error::Io(..) => "CSV IO error",
            Error::EOL => "Trying to access column but found End Of Line",
            Error::UnescapedQuote => "A CSV column has an unescaped quote",
            Error::UnexpextedQuote => "A CSV column has a quote but the entire column value is not quoted",
            Error::ColumnMismatch(..) => "Current column count mismatch with previous rows",
        }
    }

    fn cause(&self) -> Option<&dyn (::std::error::Error)> {
        match *self {
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error { Error::Io(err) }
}

