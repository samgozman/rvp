use crate::structure::{Config, ConfigFormat, Resource, Selector};
use anyhow::Result;
use clap::Parser;
use validator::validate_url;

use inquire::{
    required,
    validator::Validation::{Invalid, Valid},
    Confirm, Select, Text,
};

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

    let options = vec!["TOML", "JSON"];
    let format = Select::new("Save configuration in:", options).prompt()?;

    println!(
        "Creating new config file {}.{}",
        name,
        format.to_lowercase()
    );

    let description = Text::new("Config description:")
        .with_help_message("(Optional) Create a helpful description for this config file")
        .with_default("")
        .prompt()?;

    let resources = add_resource()?;

    let config = Config::new(name, description, resources);

    // todo: check if file exists before saving it

    let path = config.save(match format {
        "TOML" => ConfigFormat::TOML,
        "JSON" => ConfigFormat::JSON,
        _ => unreachable!(),
    })?;

    println!("Config file saved to {}", path.display());
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
        let name = Text::new("Selector name:")
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
            .with_validator(|input: &str| match validate_url(input) {
                true => Ok(Valid),
                false => Ok(Invalid("must be a valid URL!".into())),
            })
            .prompt()?;

        let selectors = add_selectors()?;
        resources.push(Resource::new(url, selectors));

        println!("New Resource added!");

        let add_another = Confirm::new("Add another Resource?")
            .with_default(false)
            .prompt()?;

        if !add_another {
            break 'resource_loop;
        }
    }

    Ok(resources)
}
