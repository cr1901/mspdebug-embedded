use std::convert::AsRef;
use std::io;
use std::process::{Command, Stdio};

use strum_macros::AsRefStr;

use super::{MspDebug, Error};

#[derive(AsRefStr)]
pub enum TargetDriver {
    #[strum(serialize = "rf2500")]
    Rf2500,
    #[strum(serialize = "sim")]
    Sim,
    #[strum(serialize = "tilib")]
    Tilib,
}

pub struct Cfg {
    driver: TargetDriver,
    quiet: bool,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            driver: TargetDriver::Sim,
            quiet: true,
        }
    }

    pub fn driver(self, driver: TargetDriver) -> Cfg {
        Cfg { driver, ..self }
    }

    // Not part of public API for now. For testing.
    #[allow(unused)]
    fn quiet(self, quiet: bool) -> Cfg {
        Cfg { quiet, ..self }
    }

    pub fn run(self) -> Result<MspDebug, Error> {
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
            .map_err(Error::SpawnError)?;

        let stdin = child
            .stdin
            .take()
            .ok_or(Error::StreamError("stdin"))?;
        let stdout = io::BufReader::new(
            child
                .stdout
                .take()
                .ok_or(Error::StreamError("stdout"))?,
        );

        Ok(MspDebug {
            stdin,
            stdout,
            cfg: self,
            last_shelltype: None
        })
    }
}
