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
    let name = args.name.unwrap_or("default".to_string());
    println!("Creating new config file {}.toml", name);

    let description = Text::new("Config description:")
        .with_help_message("(Optional) Create a helpful description for this config file")
        .with_default("")
        .prompt()?;

    let resources = add_resource()?;

    let config = Config::new(name, description, resources);

    println!("Done! Don't worry, you can edit the config file later.");

    Ok(())
}

/// Create list of selectors from user input
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

        let add_another = Confirm::new("Add another Selector?")
            .with_default(false)
            .prompt()?;

        if !add_another {
            break 'selector_loop;
        }
    }

    Ok(selectors)
}

/// Create list of resources from user input
fn add_resource() -> Result<Vec<Resource>> {
    let mut resources: Vec<Resource> = Vec::new();

    'resource_loop: loop {
        let url = Text::new("Site URL:")
            .with_validator(required!("This field is required"))
            .with_help_message("http://example.com")
            .prompt()?;
        let selectors = add_selectors()?;

        resources.push(Resource::new(url, selectors));

        let add_another = Confirm::new("Add another Resource?")
            .with_default(false)
            .prompt()?;

        if !add_another {
            break 'resource_loop;
        }
    }

    Ok(resources)
}

// TODO: Move these structs to a separate file, implement some methods (like 'add', 'remove', 'list', etc.)

/// A selector is named a path to a value on a web page
struct Selector {
    path: String,
    name: String,
}

impl Selector {
    fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}

// A resource is a website with a list of selectors
struct Resource {
    url: String,
    selectors: Vec<Selector>,
}

impl Resource {
    fn new(url: String, selectors: Vec<Selector>) -> Self {
        Self { url, selectors }
    }
}

// A config is a list of resources
struct Config {
    name: String,
    description: String,
    resources: Vec<Resource>,
}

impl Config {
    fn new(name: String, description: String, resources: Vec<Resource>) -> Self {
        Self {
            name,
            description,
            resources,
        }
    }
}
