use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum MspDebugError {
    SpawnError(io::Error),
    StreamError(&'static str),
    ReadError(io::Error),
    UnexpectedShellMessage(&'static str, String),
}

impl fmt::Display for MspDebugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MspDebugError::SpawnError(_) => write!(f, "error spawning mspdebug"),
            MspDebugError::StreamError(stream) => {
                write!(f, "could not open mspdebug stream {}", stream)
            }
            MspDebugError::ReadError(_) => write!(f, "error reading mspdebug stdout"),
            MspDebugError::UnexpectedShellMessage(exp, act) => {
                write!(f, "unexpected shell message, expected {}, got {}", exp, act)
            }
        }
    }
}

impl error::Error for MspDebugError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            MspDebugError::SpawnError(io) | MspDebugError::ReadError(io) => Some(io),
            MspDebugError::StreamError(_) | MspDebugError::UnexpectedShellMessage(_, _) => None,
        }
    }
}
