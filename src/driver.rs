use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus};
use std::thread;

use io::Write as _;

use bitflags::bitflags;

use super::{Cfg, Error};

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

pub(crate) enum ShellType {
    Ready,
    Busy,
    PowerSampleUs,
    PowerSamples,
}

#[derive(PartialEq)]
enum WaitMode {
    Ready,
    Busy,
}

pub struct MspDebug {
    pub(crate) stdin: ChildStdin,
    pub(crate) stdout: io::BufReader<ChildStdout>,
    #[allow(unused)]
    pub(crate) cfg: Cfg,
    pub(crate) last_shelltype: Option<ShellType>,
    pub(crate) child: Child,
    pub(crate) need_drop: bool
}

bitflags! {
    struct GdbConfigFlags: u32 {
        const QUIET = 1;
        const RESET = 1 << 1;
        const ERASE = 1 << 2;
        const ERASE_INFOMEM = 1 << 3;
        const LOAD = 1 << 4;

        const ERASE_AND_LOAD = Self::ERASE.bits | Self::LOAD.bits;
        const ERASE_ALL_AND_LOAD = Self::ERASE.bits | Self::ERASE_INFOMEM.bits | Self::LOAD.bits;
        const DEFAULT = Self::QUIET.bits | Self::RESET.bits;
    }
}

pub struct GdbCfg {
    flags: GdbConfigFlags,
    port: u16
}

impl Default for GdbCfg {
    fn default() -> Self {
        Self {
            flags: GdbConfigFlags::DEFAULT,
            port: 2000
        } 
    }
}

impl GdbCfg {
    pub fn erase_and_load(mut self) -> Self {
        self.flags |= GdbConfigFlags::ERASE_AND_LOAD;
        self
    }

    pub fn erase_all_and_load(mut self) -> Self {
        self.flags |= GdbConfigFlags::ERASE_ALL_AND_LOAD;
        self
    }

    pub fn set_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

impl MspDebug {
    fn get_line<'a>(&mut self, line: &'a mut String) -> Result<OutputType<'a>, Error> {
        loop {
            self.stdout.read_line(line).map_err(Error::ReadError)?;

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
                    return Err(Error::UnexpectedSigil(un));
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

    fn get_shell_type(&self, line: &str) -> Result<ShellType, Error> {
        match line {
            line if line.eq("ready\n") || line.eq("ready\r\n") => Ok(ShellType::Ready),
            line if line.eq("busy\n") || line.eq("busy\r\n") => Ok(ShellType::Busy),
            line if line.eq("power-sample-us\n") || line.eq("power-sample-us\r\n") => {
                Ok(ShellType::PowerSampleUs)
            }
            line if line.eq("power-samples\n") || line.eq("power-samples\r\n") => {
                Ok(ShellType::PowerSamples)
            }
            line => Err(Error::UnexpectedShellMessage(line.to_owned())),
        }
    }

    pub fn wait_for_ready(&mut self) -> Result<(), Error> {
        self.wait_for_ready_or_busy(WaitMode::Ready)
    }

    pub fn wait_for_busy(&mut self) -> Result<(), Error> {
        self.wait_for_ready_or_busy(WaitMode::Busy)
    }

    fn wait_for_ready_or_busy(&mut self, mode: WaitMode) -> Result<(), Error> {
        // Every command in this driver waits for ready at the beginning and
        // end. Cache the value of ShellType, so we know what the last line was.
        match self.last_shelltype {
            Some(ShellType::Ready) if mode == WaitMode::Ready => return Ok(()),
            Some(ShellType::Busy) if mode == WaitMode::Busy => return Ok(()),
            _ => {}
        }

        let mut line = String::new();

        loop {
            self.last_shelltype = None;
            match self.get_line(&mut line)? {
                OutputType::Shell(s) => match s {
                    ShellType::Ready if mode == WaitMode::Ready => {
                        self.last_shelltype = Some(ShellType::Ready);
                        return Ok(());
                    }
                    ShellType::Busy if mode == WaitMode::Busy => {
                        self.last_shelltype = Some(ShellType::Busy);
                        return Ok(());
                    }
                    _stype => {}
                },
                OutputType::Error(ErrorSeverity::Warning(_w)) => { /* todo!() */ }
                OutputType::Error(ErrorSeverity::Error(e)) => match e {
                    e if e.starts_with("fet: FET returned error code")
                        || e.starts_with("fet: command C_IDENT1 failed")
                        || e.starts_with("fet: FET returned NAK") => {}
                    e => {
                        return Err(Error::CommsError(e.into()));
                    }
                },
                _ => {}
            }

            line.clear();
        }
    }

    pub fn program<F>(&mut self, filename: F) -> Result<(), Error>
    where
        F: Into<PathBuf>,
    {
        if self.cfg.group {
            return Err(Error::ExpectedNoProcessGroup);
        }

        let filename = filename.into();

        self.wait_for_ready()?;
        write!(self, ":prog {}\n", filename.display()).map_err(|e| Error::WriteError(e))?;
        self.wait_for_busy()?;
        self.wait_for_ready()?;

        Ok(())
    }

    /** Run `mspdebug` in `gdb` server mode and spawn a `msp430-elf-gdb` session.
    
    Shell equivalent:

    ```ignore
    set -m && { mspdebug [driver] gdb < /dev/null > /dev/null 2> /dev/null & } && msp430-elf-gdb -q -ex "target remote localhost:2000" [..] -ex "monitor reset" /path/to/elf
    ```
     
    Powershell equivalent:

    ```ignore
    Start-Job { $null | mspdebug -q --embedded [driver] gdb > $null } > $null; msp430-elf-gdb -q -ex "target remote localhost:2000" [..] -ex "monitor reset" /path/to/elf
    ```
    */
    pub fn gdb<F>(mut self, filename: F, cfg: GdbCfg) -> Result<ExitStatus, Error>
    where
        F: Into<PathBuf>,
    {
        if !self.cfg.group {
            return Err(Error::ExpectedProcessGroup);
        }

        let filename = filename.into();

        ctrlc::set_handler(move || {}).map_err(|e| Error::CtrlCError(e))?;
        self.wait_for_ready()?;
        write!(self, ":gdb {}\n", cfg.port).map_err(|e| Error::WriteError(e))?;
        self.wait_for_busy()?;

        // FIXME: Between here and gdb invocation, if this function panics,
        // mspdebug will not exit by itself. Figure out why.
        // Might be a small race here too (between wait_for_busy returning and
        // need_drop being set)?
        self.need_drop = true;
        let fn_string = filename.to_string_lossy();
        let mut args = Vec::new();

        if cfg.flags.contains(GdbConfigFlags::QUIET) {
            args.push("-q");
        }

        let target_str = format!("target remote localhost:{}", cfg.port);
        args.extend(["-ex", &target_str]);

        if cfg.flags.contains(GdbConfigFlags::ERASE) {
            args.extend(["-ex", "monitor erase"]);
        }

        let erase_infomem_str: String;
        if cfg.flags.contains(GdbConfigFlags::ERASE_INFOMEM) {
            erase_infomem_str = format!("monitor erase segrange {}", unimplemented!());
            args.extend(["-ex", &erase_infomem_str]);
        }

        if cfg.flags.contains(GdbConfigFlags::LOAD) {
            args.extend(["-ex", "load"]);
        }

        args.extend(["-ex", "monitor reset"]);
        args.push(&fn_string);

        let mut gdb = Command::new("msp430-elf-gdb")
            .args(&args)
            .spawn()
            .map_err(|e| Error::SpawnError(e))?;

        self.need_drop = false;
        // When gdb exits, mspdebug will too.
        let exit = gdb.wait()
            .map_err(|e| Error::GdbError(e))?;

        Ok(exit)
    }
}

impl io::Read for MspDebug {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdout.read(buf)
    }
}

impl io::Write for MspDebug {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdin.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdin.flush()
    }
}


impl Drop for MspDebug {
    fn drop(&mut self) {
        if self.need_drop {
            self.child.kill().unwrap()
        }
    }
}
