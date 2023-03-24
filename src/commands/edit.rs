use std::{ffi::OsStr, path::PathBuf};

use crate::structure::{Config, ConfigFormat, Position, URL_PARAM_PLACEHOLDER};
use anyhow::{anyhow, Result};
use clap::{value_parser, Parser};
use inquire::{
    required,
    validator::Validation::{Invalid, Valid},
    Confirm, Select, Text,
};
use validator::validate_url;

/// Edit config file
#[derive(Parser)]
pub struct Args {
    /// Path to the config file.
    #[arg(short, long, value_name = "PATH", value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

pub async fn command(args: Args) -> Result<()> {
    if !args.path.exists() {
        return Err(anyhow!("File does not exist!"));
    }

    let config_format = match args.path.extension().and_then(OsStr::to_str) {
        Some("json") => ConfigFormat::Json,
        Some("toml") => ConfigFormat::Toml,
        _ => return Err(anyhow!("Invalid file format!")),
    };

    let mut config = Config::from_file(&args.path, config_format.clone())?;

    'resource_loop: loop {
        let resource =
            Select::new("Select resource to edit:", config.resources.clone()).prompt()?;

        let actions = vec!["Edit URL", "Edit selectors", "Delete", "↩ Back", "⏹ Exit"];
        let action = Select::new("Select action:", actions).prompt()?;

        match action {
            "Edit URL" => {
                config.resources[resource].url = Text::new("Site URL:")
                    .with_validator(required!("This field is required"))
                    .with_initial_value(&resource.url)
                    .with_help_message(
                        format!("e.g. http://example.com?id={}", URL_PARAM_PLACEHOLDER).as_str(),
                    )
                    .with_validator(|input: &str| match validate_url(input) {
                        true => Ok(Valid),
                        false => Ok(Invalid("must be a valid URL!".into())),
                    })
                    .prompt()?;
            }
            "Edit selectors" => {
                unimplemented!()
            }
            "Delete" => {
                if Confirm::new("Are you sure you want to delete this resource?")
                    .with_default(false)
                    .prompt()?
                {
                    let index = config.resources.position(&resource);
                    config.resources.remove(index);
                }
            }
            "↩ Back" => continue 'resource_loop,
            "⏹ Exit" => break 'resource_loop,
            _ => unreachable!(),
        }

        match Confirm::new("Edit more resources?")
            .with_default(true)
            .prompt()?
        {
            true => {
                continue 'resource_loop;
            }
            false => break 'resource_loop,
        }
    }

    match Confirm::new("Save changes?").with_default(true).prompt()? {
        true => {
            config.save(config_format)?;
            println!("Config file saved!");
        }
        false => println!("Changes discarded."),
    }

    // ? All of this will be performed in multiple levels of loops
    // TODO: If selectors, choose action: add, edit, delete, back

    Ok(())
}
