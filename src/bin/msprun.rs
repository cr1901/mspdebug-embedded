use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use eyre::Result;
use mspdebug_embedded::*;

#[derive(clap::Parser)]
#[clap(author, version)]
/// "cargo run"-friendly driver program for mspdebug.
pub struct Args {
    pub driver: TargetDriver,
    #[clap(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Program attached msp430 microcontroller with given ELF file.
    Prog {
        filename: PathBuf
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.cmd {
        Cmd::Prog { filename } => {
            let mut msp = Cfg::new().driver(args.driver).run()?;
            msp.program(filename)?;
        }
    }

    Ok(())
}
