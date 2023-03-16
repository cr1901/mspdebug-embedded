use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eyre::Result;
use mspdebug_embedded::*;

#[derive(clap::Parser)]
#[clap(name = "msprun", author, version)]
/// `cargo run`-friendly driver program for `mspdebug`.
pub struct Args {
    /// Driver argument to pass to `mspdebug`
    pub driver: TargetDriver,
    /// High-level command to run (converted to multiple `mspdebug` commands)
    #[clap(subcommand)]
    pub cmd: Cmd,
    /// Explicit path to `mspdebug` binary (default to PATH)
    #[arg(short = 'b')]
    pub binary: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Cmd {
    /** Program attached msp430 microcontroller with given ELF file.

    This command is a wrapper over the `prog` command that is more friendly
    to being passed a file input argument to `cargo run`.
    
    Additionally, this command will check whether the ELF file is using msp430
    Information Memory segments D through B and automatically erase them if
    necessary. Information Memory segment A is untouched due to possibly
    containing calibration info.
    */
    Prog { filename: PathBuf },
    /** Use `mspdebug` to create a `gdb` server; spawn an interactive
    `msp430-elf-gdb` session.

    This command invokes `mspdebug` in a separate process group as
    `mspdebug [driver] gdb [port]`. The separate process group means that
    CTRL+C will be ignored by the spawned `mspdebug`. However, when
    `msp430-elf-gdb` exits, so will `mspdebug`.

    This command then invokes `msp430-elf-gdb` as `msp430-elf-gdb -q [-ex command ...] [file]`
    and gives control over to a `gdb` shell. The `-ex` arguments do initial setup.
    If `-r` was not passed, the `-ex` arguments also make sure that `msp430-elf-gdb`
    erases and then flashes the ELF file (via `mspdebug`'s `gdb` server) before
    giving control to the user.

    The same Information Memory detection logic is used here as in the `prog` command.
    */
    Gdb {
        /// Invoke `msp430-elf-gdb` with this ELF file as the debugged program.
        filename: PathBuf,
        /// Issue `monitor reset` only; do not program the ELF file from within `gdb`.
        #[arg(short = 'r')]
        reset_only: bool,
        /// TCP/IP port used by `mspdebug`'s `gdb` server.
        #[arg(short = 'p', default_value_t = 2000)]
        port: u16,
        /// Explicit path to `msp430-elf-gdb` binary (default to PATH)
        #[arg(short = 'b')]
        binary: Option<PathBuf>,

        #[arg(short = 'e')]
        gdb_init: Vec<String>
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut cfg = Cfg::new();
    if let Some(b) = args.binary {
        cfg = cfg.binary(b);
    }

    match args.cmd {
        Cmd::Prog { filename } => {
            let mut msp = cfg.driver(args.driver).run()?;
            msp.program(filename)?;
        }
        Cmd::Gdb {
            filename,
            reset_only,
            port,
            gdb_init,
            ..
        } => {
            let msp = cfg.driver(args.driver).group(true).run()?;

            let gdb = if reset_only {
                GdbCfg::default().set_port(port).extra_cmds(gdb_init)
            } else {
                GdbCfg::default().erase_and_load().set_port(port).extra_cmds(gdb_init)
            };

            msp.gdb(filename, gdb)?;
        }
    }

    Ok(())
}
