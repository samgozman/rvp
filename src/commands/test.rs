use anyhow::Result;
use clap::Parser;

/// This is a test command and should be removed.
#[derive(Parser)]
pub struct Args {
    /// lists test values
    #[arg(short, long)]
    list: bool,
}

pub async fn command(args: Args) -> Result<()> {
    if args.list {
        println!("Printing testing lists...");
    } else {
        println!("Not printing testing lists...");
    }

    Ok(())
}
