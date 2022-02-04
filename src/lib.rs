use std::convert::AsRef;
use std::error;
use std::fmt;
use std::io::{self, BufRead};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use strum_macros::AsRefStr;

#[derive(AsRefStr)]
pub enum Driver {
    #[strum(serialize = "rf2500")]
    Rf2500,
    #[strum(serialize = "sim")]
    Sim,
    #[strum(serialize = "tilib")]
    Tilib
}

pub struct MspDebugCfg {
    driver: Driver,
    quiet: bool
}

#[derive(Debug)]
pub enum MspDebugError {
    SpawnError(io::Error),
    StreamError(&'static str),
    ReadError(io::Error),
    UnexpectedShellMessage(&'static str, String)
}

impl fmt::Display for MspDebugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MspDebugError::SpawnError(_) => write!(f, "error spawning mspdebug"),
            MspDebugError::StreamError(stream) => write!(f, "could not open mspdebug stream {}", stream),
            MspDebugError::ReadError(_) => write!(f, "error reading mspdebug stdout"),
            MspDebugError::UnexpectedShellMessage(exp, act) => write!(f, "unexpected shell message, expected {}, got {}", exp, act),
        }
    }
}

impl error::Error for MspDebugError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            MspDebugError::SpawnError(io) | MspDebugError::ReadError(io) => Some(io),
            MspDebugError::StreamError(_) | MspDebugError::UnexpectedShellMessage(_, _) => None
        }
    }
}

impl MspDebugCfg {
    pub fn new() -> Self {
        MspDebugCfg {
            driver: Driver::Sim,
            quiet: true
        }
    }

    pub fn driver(self, driver: Driver) -> MspDebugCfg {
        MspDebugCfg {
            driver,
            ..self
        }
    }

    // Not part of public API for now. For testing.
    fn quiet(self, quiet: bool) -> MspDebugCfg {
        MspDebugCfg {
            quiet,
            ..self
        }
    }

    pub fn run(self) -> Result<MspDebugDriver, MspDebugError> {
        let mut cmd = Command::new("mspdebug");

        cmd.args(["--embedded", self.driver.as_ref()]);

        if self.quiet {
            cmd.arg("-q");
        }

        let mut child = cmd.stderr(Stdio::null())
                           .stdin(Stdio::piped())
                           .stdout(Stdio::piped())
                           .spawn()
                           .map_err(MspDebugError::SpawnError)?;

        let stdin = child.stdin.take().ok_or(MspDebugError::StreamError("stdin"))?;
        let stdout = io::BufReader::new(child.stdout.take().ok_or(MspDebugError::StreamError("stdout"))?);

        Ok(MspDebugDriver {
           stdin,
           stdout,
           cfg: self
       })
    }
}

pub struct MspDebugDriver {
    stdin: ChildStdin,
    stdout: io::BufReader<ChildStdout>,
    cfg: MspDebugCfg
}

impl MspDebugDriver {
    fn get_line(&mut self, line: String) -> Result<(), MspDebugError> {
        unimplemented!()
    }

    fn wait_for_ready(&mut self) -> Result<(), MspDebugError> {
        let mut line = String::new();

        loop {
            self.stdout.read_line(&mut line).map_err(MspDebugError::ReadError)?;

            if let Some('\\') = line.chars().nth(0) {
                break;
            }

            line.clear()
        }

        match line {
            line if line.eq("\\ready\n") || line.eq("\\ready\r\n") => Ok(()),
            line => Err(MspDebugError::UnexpectedShellMessage("\\ready\n", line))
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

#[cfg(test)]
mod tests {
    use super::MspDebugCfg;

    // Tests assume mspdebug is on the path.

    #[test]
    fn test_spawn() {
        let mspdebug = MspDebugCfg::new().run();

        assert!(mspdebug.is_ok(), "mspdebug did not spawn: {:?}", unsafe { mspdebug.unwrap_err_unchecked() });
    }

    #[test]
    fn test_ready() {
        let mut mspdebug = MspDebugCfg::new().run().unwrap();

        let cmd = mspdebug.wait_for_ready();
        assert!(cmd.is_ok(), "mspdebug did not receive ready: {:?}", unsafe { cmd.unwrap_err_unchecked() });
    }
}
