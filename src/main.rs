mod commands;
mod scalper;
mod structure;
use commands::*;

#[macro_use]
mod macros;
use anyhow::Result;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    // Specify optional parameters here, before the subcommand
    #[command(subcommand)]
    command: Commands,
}

// Specify the command modules to be included in the CLI
commands_builder!(grab, new);

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    Commands::exec(cli).await?;
    Ok(())
}
