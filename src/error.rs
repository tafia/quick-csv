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
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Decode(ref msg) => write!(f, "CSV decode error: {}", msg),
            Error::Parse(ref err) => write!(f, "{}", err),
            Error::Io(ref err) => write!(f, "{}", err),
            Error::EOL => write!(f, "CSV expecting a column, found end of line"),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Decode(..) => "CSV decoding error",
            Error::Parse(..) => "CSV parse error",
            Error::Io(..) => "CSV IO error",
            Error::EOL => "CSV expecting a column, found end of line",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error { Error::Io(err) }
}

