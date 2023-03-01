use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum MspDebugError {
    SpawnError(io::Error),
    StreamError(&'static str),
    ReadError(io::Error),
    WriteError(io::Error),
    UnexpectedSigil(char),
    UnexpectedShellMessage(String),
    CommsError(String),
}

impl fmt::Display for MspDebugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MspDebugError::SpawnError(_) => write!(f, "error spawning mspdebug"),
            MspDebugError::StreamError(stream) => {
                write!(f, "could not open mspdebug stream {}", stream)
            }
            MspDebugError::ReadError(_) => write!(f, "error reading mspdebug stdout"),
            MspDebugError::WriteError(_) => write!(f, "error writing mspdebug stdin"),
            MspDebugError::UnexpectedSigil(sigil) => {
                write!(
                    f,
                    "unexpected sigil, expected :, -, !, or \\, got {}",
                    sigil
                )
            }
            MspDebugError::UnexpectedShellMessage(msg) => {
                write!(f, "unexpected shell message, expected 'ready', 'busy', 'power-sample-us', or 'power-samples', got {}", msg)
            }
            MspDebugError::CommsError(msg) => {
                write!(f, "mspdebug could not communicate with the device {}", msg)
            }
        }
    }
}

impl error::Error for MspDebugError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            MspDebugError::SpawnError(io)
            | MspDebugError::ReadError(io)
            | MspDebugError::WriteError(io) => Some(io),
            MspDebugError::StreamError(_)
            | MspDebugError::UnexpectedSigil(_)
            | MspDebugError::UnexpectedShellMessage(_)
            | MspDebugError::CommsError(_) => None,
        }
    }
}
