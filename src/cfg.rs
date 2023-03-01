use std::convert::AsRef;
use std::io;
use std::process::{Command, Stdio};

use strum_macros::AsRefStr;

use super::{MspDebugDriver, MspDebugError};

#[derive(AsRefStr)]
pub enum Driver {
    #[strum(serialize = "rf2500")]
    Rf2500,
    #[strum(serialize = "sim")]
    Sim,
    #[strum(serialize = "tilib")]
    Tilib,
}

pub struct MspDebugCfg {
    driver: Driver,
    quiet: bool,
}

impl MspDebugCfg {
    pub fn new() -> Self {
        MspDebugCfg {
            driver: Driver::Sim,
            quiet: true,
        }
    }

    pub fn driver(self, driver: Driver) -> MspDebugCfg {
        MspDebugCfg { driver, ..self }
    }

    // Not part of public API for now. For testing.
    #[allow(unused)]
    fn quiet(self, quiet: bool) -> MspDebugCfg {
        MspDebugCfg { quiet, ..self }
    }

    pub fn run(self) -> Result<MspDebugDriver, MspDebugError> {
        let mut cmd = Command::new("mspdebug");

        cmd.args(["--embedded", self.driver.as_ref()]);

        if self.quiet {
            cmd.arg("-q");
        }

        let mut child = cmd
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(MspDebugError::SpawnError)?;

        let stdin = child
            .stdin
            .take()
            .ok_or(MspDebugError::StreamError("stdin"))?;
        let stdout = io::BufReader::new(
            child
                .stdout
                .take()
                .ok_or(MspDebugError::StreamError("stdout"))?,
        );

        Ok(MspDebugDriver {
            stdin,
            stdout,
            cfg: self,
            last_shelltype: None
        })
    }
}
