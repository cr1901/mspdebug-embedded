use std::io::{self, BufRead};
use std::process::{ChildStdin, ChildStdout};

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
            line if line.eq("warning") => ErrorSeverity::Warning(line),
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
                    stype => unimplemented!(),
                },
                _ => {}
            }

            line.clear();
        }
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
