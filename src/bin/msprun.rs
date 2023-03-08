use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eyre::Result;
use mspdebug_embedded::*;

#[derive(clap::Parser)]
#[clap(author, version)]
/// "cargo run"-friendly driver program for mspdebug.
pub struct Args {
    pub driver: TargetDriver,
    #[clap(subcommand)]
    pub cmd: Cmd,
    #[arg(short = 'b')]
    pub binary: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Program attached msp430 microcontroller with given ELF file.
    Prog { filename: PathBuf },
    /// Use mspdebug to create a `gdb` server; spawn an interactive `msp430-elf-gdb` session.
    Gdb {
        filename: PathBuf,
        #[arg(short = 'e')]
        erase: bool,
        #[arg(short = 'x')]
        erase_extra: bool,
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
            erase,
            erase_extra,
        } => {
            let msp = cfg.driver(args.driver).group(true).run()?;

            let cfg = match (erase, erase_extra) {
                (true, false) => GdbCfg::default().erase_and_load(),
                (_, true) => GdbCfg::default().erase_all_and_load(),
                (_, _) => GdbCfg::default(),
            };
            msp.gdb(filename, cfg)?;
        }
    }

    Ok(())
}
