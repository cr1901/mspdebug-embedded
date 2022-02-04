use std::io::{self, BufRead};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use super::{MspDebugCfg, MspDebugError};

pub struct MspDebugDriver {
    pub(crate) stdin: ChildStdin,
    pub(crate) stdout: io::BufReader<ChildStdout>,
    pub(crate) cfg: MspDebugCfg,
}

impl MspDebugDriver {
    fn get_line(&mut self, line: String) -> Result<(), MspDebugError> {
        unimplemented!()
    }

    pub fn wait_for_ready(&mut self) -> Result<(), MspDebugError> {
        let mut line = String::new();

        loop {
            self.stdout
                .read_line(&mut line)
                .map_err(MspDebugError::ReadError)?;

            if let Some('\\') = line.chars().nth(0) {
                break;
            }

            line.clear();
        }

        match line {
            line if line.eq("\\ready\n") || line.eq("\\ready\r\n") => Ok(()),
            line => Err(MspDebugError::UnexpectedShellMessage("\\ready\n", line)),
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
