use anyhow::Result;
use clap::Parser;

use inquire::{required, Confirm, Text};

/// Create new config file to grab multiple values from a web page at once.
#[derive(Parser)]
pub struct Args {
    /// Name of the config file to create.
    ///
    ///  *Optional.* If not provided, the default name will be used.
    #[arg(short, long, value_name = "NAME")]
    name: Option<String>,
}

pub async fn command(args: Args) -> Result<()> {
    println!(
        "Creating new config file {}.toml",
        args.name.unwrap_or("default".to_string())
    );

    let url = Text::new("Site URL:")
        .with_validator(required!("This field is required"))
        .with_help_message("http://example.com")
        .prompt()?;

    println!("Now you need to add some selectors to grab from the page:");

    let selectors = add_selectors()?;

    // TODO: Add outer loop to add more websites and selectors

    println!("URL: {}", url);
    selectors
        .iter()
        .for_each(|s| println!("{}: {}", s.name, s.path));
    Ok(())
}

fn add_selectors() -> Result<Vec<Selector>> {
    let mut selectors: Vec<Selector> = Vec::new();

    'selector_loop: loop {
        let path = Text::new("Selector path:")
            .with_validator(required!("This field is required"))
            .with_help_message("body > div > h1")
            .prompt()?;
        let name = Text::new("Name:")
            .with_validator(required!("This field is required"))
            .with_help_message("Title")
            .prompt()?;
        selectors.push(Selector::new(path, name));

        let add_another = Confirm::new("Add another selector?")
            .with_default(false)
            .prompt()?;

        if !add_another {
            break 'selector_loop;
        }
    }

    Ok(selectors)
}

struct Selector {
    path: String,
    name: String,
}

impl Selector {
    fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}
