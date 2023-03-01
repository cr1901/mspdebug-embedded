use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{ChildStdin, ChildStdout};

use io::Write as _;

use super::{MspDebugCfg, MspDebugError};

enum OutputType<'a> {
    Normal(&'a str),
    Debug(&'a str),
    Error(ErrorSeverity<'a>),
    Shell(ShellType),
}

enum ErrorSeverity<'a> {
    Warning(&'a str),
    Error(&'a str),
}

enum ShellType {
    Ready,
    Busy,
    PowerSampleUs,
    PowerSamples,
}

pub struct MspDebugDriver {
    pub(crate) stdin: ChildStdin,
    pub(crate) stdout: io::BufReader<ChildStdout>,
    pub(crate) cfg: MspDebugCfg,
}

impl MspDebugDriver {
    fn get_line<'a>(&mut self, line: &'a mut String) -> Result<OutputType<'a>, MspDebugError> {
        loop {
            self.stdout
                .read_line(line)
                .map_err(MspDebugError::ReadError)?;

            match line.chars().nth(0) {
                Some(':') => return Ok(OutputType::Normal(&line[1..])),
                Some('-') => return Ok(OutputType::Debug(&line[1..])),
                Some('!') => {
                    return Ok(OutputType::Error(self.get_error_severity(&line[1..])));
                }
                Some('\\') => {
                    return Ok(OutputType::Shell(self.get_shell_type(&line[1..])?));
                }
                Some(un) => {
                    return Err(MspDebugError::UnexpectedSigil(un));
                }
                None => unreachable!(),
            }
        }
    }

    fn get_error_severity<'a>(&self, line: &'a str) -> ErrorSeverity<'a> {
        match line {
            line if line.starts_with("warning") => ErrorSeverity::Warning(line),
            line => ErrorSeverity::Error(line),
        }
    }

    fn get_shell_type(&self, line: &str) -> Result<ShellType, MspDebugError> {
        match line {
            line if line.eq("ready\n") || line.eq("ready\r\n") => Ok(ShellType::Ready),
            line if line.eq("busy\n") || line.eq("busy\r\n") => Ok(ShellType::Busy),
            line if line.eq("power-sample-us\n") || line.eq("power-sample-us\r\n") => {
                Ok(ShellType::PowerSampleUs)
            }
            line if line.eq("power-samples\n") || line.eq("power-samples\r\n") => {
                Ok(ShellType::PowerSamples)
            }
            line => Err(MspDebugError::UnexpectedShellMessage(line.to_owned())),
        }
    }

    pub fn wait_for_ready(&mut self) -> Result<(), MspDebugError> {
        let mut line = String::new();

        loop {
            match self.get_line(&mut line)? {
                OutputType::Shell(s) => match s {
                    ShellType::Ready => return Ok(()),
                    ShellType::Busy => {}
                    stype => unimplemented!(),
                },
                OutputType::Error(ErrorSeverity::Warning(_w)) => { /* todo!() */ }
                OutputType::Error(ErrorSeverity::Error(e)) => match e {
                    e if e.starts_with("fet: FET returned error code")
                        || e.starts_with("fet: command C_IDENT1 failed")
                        || e.starts_with("fet: FET returned NAK") => {}
                    e => return Err(MspDebugError::CommsError(e.into())),
                },
                _ => {}
            }

            line.clear();
        }
    }

    pub fn program<F>(&mut self, filename: F) -> Result<(), MspDebugError>
    where
        F: Into<PathBuf>,
    {
        let filename = filename.into();

        self.wait_for_ready()?;
        write!(self, ":prog {}", filename.display()).map_err(|e| MspDebugError::WriteError(e))?;

        Ok(())
    }
}

impl io::Read for MspDebugDriver {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdout.read(buf)
    }
}

impl io::Write for MspDebugDriver {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdin.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdin.flush()
    }
}
