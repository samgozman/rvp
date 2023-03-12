use anyhow::Result;
use clap::Parser;

/// Simply grab one value from a web page.
#[derive(Parser)]
pub struct Args {
    /// Selector path to grab from the page.
    ///
    /// You can get it from the browser's developer tools
    /// by right clicking on the HTML element and selecting "Copy > Selector Path" (or similar).
    ///
    /// Example: `-s="#search > div"`
    #[arg(short, long, value_name = "PATH")]
    selector: String,

    /// URL to web page to grab from.
    ///
    /// Example: `-f="https://example.com"`
    #[arg(short, long, value_name = "URL")]
    from: String,
}

pub async fn command(args: Args) -> Result<()> {
    println!("Grab command");
    println!("Selector: {}", args.selector);
    println!("From: {}", args.from);
    Ok(())
}
