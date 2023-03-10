use std::error;
use std::fmt;
use std::io;

use ctrlc;
use elf::parse::ParseError;

#[derive(Debug)]
pub enum Error {
    SpawnError(io::Error),
    ExpectedProcessGroup,
    ExpectedNoProcessGroup,
    BadInput(BadInputReason),
    StreamError(&'static str),
    ReadError(io::Error),
    WriteError(io::Error),
    UnexpectedSigil(char),
    UnexpectedShellMessage(String),
    CommsError(String),
    CtrlCError(ctrlc::Error),
    GdbError(io::Error),
    NoDevice,
    UnknownDevice(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SpawnError(_) => write!(f, "error spawning mspdebug"),
            Error::ExpectedProcessGroup => write!(
                f,
                "expected child mspdebug to be in a separate process group, but it wasn't"
            ),
            Error::ExpectedNoProcessGroup => write!(
                f,
                "expected child mspdebug to be in same process group as parent, found separate"
            ),
            Error::BadInput(_) => write!(
                f,
                "the file to program onto msp430 was not found or corrupt"
            ),
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
            Error::CtrlCError(e) => {
                write!(
                    f,
                    "mspdebug could not override the ctrl+C handler for gdb {}",
                    e
                )
            }
            Error::GdbError(e) => {
                write!(f, "child debugger exited unexpectedly {}", e)
            }
            Error::NoDevice => {
                write!(f, "device not known either by mspdebug or this crate")
            }
            Error::UnknownDevice(d) => {
                write!(f, "device known by mspdebug but not this crate, got {}", d)
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::SpawnError(io)
            | Error::ReadError(io)
            | Error::WriteError(io)
            | Error::GdbError(io) => Some(io),
            Error::CtrlCError(e) => Some(e),
            Error::BadInput(r) => Some(r),
            Error::ExpectedProcessGroup
            | Error::ExpectedNoProcessGroup
            | Error::StreamError(_)
            | Error::UnexpectedSigil(_)
            | Error::UnexpectedShellMessage(_)
            | Error::CommsError(_)
            | Error::NoDevice
            | Error::UnknownDevice(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum BadInputReason {
    IoError(io::Error),
    ElfParseError(ParseError),
}

impl fmt::Display for BadInputReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadInputReason::IoError(_) => write!(f, "error opening or reading input file"),
            BadInputReason::ElfParseError(_) => write!(f, "input file was not a valid ELF file"),
        }
    }
}

impl error::Error for BadInputReason {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            BadInputReason::IoError(io) => Some(io),
            BadInputReason::ElfParseError(elf) => Some(elf),
        }
    }
}
