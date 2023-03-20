use std::convert::AsRef;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use command_group::CommandGroup;

#[cfg(windows)]
use winapi::um::winbase::CREATE_NEW_PROCESS_GROUP;

#[cfg(feature = "msprun")]
use clap::ValueEnum;
use strum_macros::AsRefStr;

use super::{Error, MspDebug};

#[derive(Clone, Copy, AsRefStr, PartialEq)]
#[cfg_attr(feature = "msprun", derive(ValueEnum))]
pub enum TargetDriver {
    /// eZ430-RF2500 devices (USB)
    #[strum(serialize = "rf2500")]
    Rf2500,
    /// Olimex MSP-JTAG-TINY
    #[strum(serialize = "olimex")]
    Olimex,
    /// Olimex MSP-JTAG-TINY (V1)
    #[strum(serialize = "olimex-v1")]
    Olimexv1,
    /// Olimex MSP-JTAG-ISO
    #[strum(serialize = "olimex-iso")]
    OlimexIso,
    /// Olimex MSP430-JTAG-ISO-MK2
    #[strum(serialize = "olimex-iso-mk2")]
    OlimexIsoMk2,
    /// Simulation mode (standard CPU)
    #[strum(serialize = "sim")]
    Sim,
    /// CPUX Simulation mode
    #[strum(serialize = "simx")]
    SimX,
    /// TI FET430UIF and compatible devices (e.g. eZ430)
    #[strum(serialize = "uif")]
    Uif,
    /// TI FET430UIF bootloader
    #[strum(serialize = "uif-bsl")]
    UifBsl,
    /// TI generic flash-based bootloader via RS-232
    #[strum(serialize = "flash-bsl")]
    FlashBsl,
    /// GDB client mode
    #[strum(serialize = "gdbc")]
    GdbClient,
    /// TI MSP430 library
    #[strum(serialize = "tilib")]
    Tilib,
    /// GoodFET MSP430 JTAG
    #[strum(serialize = "goodfet")]
    GoodFet,
    /// Parallel Port JTAG
    #[strum(serialize = "pif")]
    Pif,
    /// /sys/class/gpio direct connect
    #[strum(serialize = "gpio")]
    Gpio,
    /// Loadable USB BSL driver (USB 5xx/6xx).
    #[strum(serialize = "load-bsl")]
    LoadBsl,
    /// Texas Instruments eZ-FET
    #[strum(serialize = "ezfet")]
    EzFet,
    /// ROM bootstrap loader
    #[strum(serialize = "rom-bsl")]
    RomBsl,
    /// Bus Pirate JTAG, MISO-TDO, MOSI-TDI, CS-TMS, AUX-RESET, CLK-TCK
    #[strum(serialize = "bus-pirate")]
    BusPirate,
    /// MehFET USB JTAG/SBW device
    #[strum(serialize = "mehfet")]
    MehFet,
}

pub struct Cfg {
    binary: PathBuf,
    pub(crate) driver: TargetDriver,
    quiet: bool,
    pub(crate) group: bool,
}

impl Cfg {
    pub fn new() -> Self {
        Cfg {
            binary: "mspdebug".into(),
            driver: TargetDriver::Sim,
            quiet: true,
            group: false,
        }
    }

    pub fn binary<P>(self, binary: P) -> Cfg
    where
        P: Into<PathBuf>,
    {
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

            child_group_cfg
                .spawn()
                .map_err(Error::SpawnError)?
                .into_inner()
        } else {
            child_cfg.spawn().map_err(Error::SpawnError)?
        };

        let stdin = child.stdin.take().ok_or(Error::StreamError("stdin"))?;
        let stdout = child.stdout.take().ok_or(Error::StreamError("stdout"))?;

        Ok(MspDebug::new(child, stdin, stdout, self))
    }
}
