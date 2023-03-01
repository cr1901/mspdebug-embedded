use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    SpawnError(io::Error),
    StreamError(&'static str),
    ReadError(io::Error),
    WriteError(io::Error),
    UnexpectedSigil(char),
    UnexpectedShellMessage(String),
    CommsError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SpawnError(_) => write!(f, "error spawning mspdebug"),
            Error::StreamError(stream) => {
                write!(f, "could not open mspdebug stream {}", stream)
            }
            Error::ReadError(_) => write!(f, "error reading mspdebug stdout"),
            Error::WriteError(_) => write!(f, "error writing mspdebug stdin"),
            Error::UnexpectedSigil(sigil) => {
                write!(
                    f,
                    "unexpected sigil, expected :, -, !, or \\, got {}",
                    sigil
                )
            }
            Error::UnexpectedShellMessage(msg) => {
                write!(f, "unexpected shell message, expected 'ready', 'busy', 'power-sample-us', or 'power-samples', got {}", msg)
            }
            Error::CommsError(msg) => {
                write!(f, "mspdebug could not communicate with the device {}", msg)
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::SpawnError(io)
            | Error::ReadError(io)
            | Error::WriteError(io) => Some(io),
            Error::StreamError(_)
            | Error::UnexpectedSigil(_)
            | Error::UnexpectedShellMessage(_)
            | Error::CommsError(_) => None,
        }
    }
}
