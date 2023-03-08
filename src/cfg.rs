use std::convert::AsRef;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use command_group::CommandGroup;

#[cfg(windows)]
use winapi::um::winbase::CREATE_NEW_PROCESS_GROUP;

#[cfg(feature = "msprun")]
use clap::ValueEnum;
use strum_macros::AsRefStr;

use super::{MspDebug, Error};

#[derive(Clone, Copy, AsRefStr)]
#[cfg_attr(feature = "msprun", derive(ValueEnum))]
pub enum TargetDriver {
    #[strum(serialize = "rf2500")]
    Rf2500,
    #[strum(serialize = "sim")]
    Sim,
    #[strum(serialize = "tilib")]
    Tilib,
}

pub struct Cfg {
    binary: PathBuf,
    driver: TargetDriver,
    quiet: bool,
    pub(crate) group: bool,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            binary: "mspdebug".into(),
            driver: TargetDriver::Sim,
            quiet: true,
            group: false
        }
    }

    pub fn binary<P>(self, binary: P) -> Cfg where P: Into<PathBuf> {
        let binary = binary.into();
        Cfg { binary, ..self }
    }

    pub fn driver(self, driver: TargetDriver) -> Cfg {
        Cfg { driver, ..self }
    }

    pub fn group(self, group: bool) -> Cfg {
        Cfg { group, ..self }
    }

    // Not part of public API for now. For testing.
    #[allow(unused)]
    fn quiet(self, quiet: bool) -> Cfg {
        Cfg { quiet, ..self }
    }

    pub fn run(self) -> Result<MspDebug, Error> {
        let mut cmd = Command::new(self.binary.clone());

        cmd.args(["--embedded", self.driver.as_ref()]);

        if self.quiet {
            cmd.arg("-q");
        }

        let child_cfg = cmd
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut child = if self.group {
            let mut child_group_cfg = child_cfg.group();

            // Process groups and job objects are separate on Windows, but might
            // as well use the command_group crate to abstract-away *nix.
            #[cfg(windows)]
            child_group_cfg.creation_flags(CREATE_NEW_PROCESS_GROUP);

            child_group_cfg.spawn()
                           .map_err(Error::SpawnError)?
                           .into_inner()
        } else {
            child_cfg.spawn()
                     .map_err(Error::SpawnError)?
        };

        let stdin = child
            .stdin
            .take()
            .ok_or(Error::StreamError("stdin"))?;
        let stdout = 
            child
                .stdout
                .take()
                .ok_or(Error::StreamError("stdout"))?;

        Ok(MspDebug::new(child, stdin, stdout, self))
    }
}
