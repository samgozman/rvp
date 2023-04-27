use std::{ffi::OsStr, path::PathBuf};

use crate::structure::{
    Config, ConfigFormat, Position, Resource, Selector, SelectorType, URL_PARAM_PLACEHOLDER,
};
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

    let mut config = Config::from_file(&args.path, &config_format)?;

    'resource_loop: loop {
        let resource =
            Select::new("Select resource to edit:", config.resources.clone()).prompt()?;

        let actions = vec!["Edit URL", "Edit selectors", "Delete", "↩ Back", "⏹ Exit"];
        let action = Select::new("Select action:", actions).prompt()?;

        match action {
            "Edit URL" => {
                config.resources[&resource].url = Text::new("Site URL:")
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
                edit_selectors(&mut config, &resource)?;
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
            config.save(&config_format)?;
            println!("Config file saved!");
        }
        false => println!("Changes discarded."),
    }

    Ok(())
}

fn edit_selectors(config: &mut Config, resource: &Resource) -> Result<()> {
    'edit_selectors: loop {
        let action = Select::new(
            "Select action:",
            vec!["Add selector", "Edit selectors", "⏹ Exit"],
        )
        .prompt()?;

        match action {
            "Add selector" => {
                let path = Text::new("Selector path:")
                    .with_validator(required!("This field is required"))
                    .with_help_message("e.g. body > div > h1")
                    .prompt()?;
                let name = Text::new("Selector name:")
                    .with_validator(required!("This field is required"))
                    .with_help_message("e.g. title")
                    .prompt()?;
                let parsed_type = Select::new("Selector type:", SelectorType::list_as_vec()).prompt()?;
                config.resources[resource]
                    .selectors
                    .push(Selector::new(path, name, parsed_type));
            }
            "Edit selectors" => 'selectors_loop: loop {
                let selector = Select::new(
                    "Choose selector to edit:",
                    // TODO: Fix bug: selector names are not updated in the cloned vector
                    config.resources[resource].selectors.clone(),
                )
                .with_help_message("Choose selector to edit or delete.")
                .prompt()?;

                let actions = vec![
                    "Rename",
                    "Edit",
                    "Change type",
                    "Delete",
                    "↩ Back",
                    "⏹ Exit",
                ];
                let action = Select::new("Select action:", actions).prompt()?;

                match action {
                    "Rename" => {
                        config.resources[resource].selectors[&selector].name = Text::new("Name:")
                            .with_validator(required!("This field is required"))
                            .with_help_message("e.g. title")
                            .with_initial_value(&selector.name)
                            .prompt()?;
                        break 'selectors_loop;
                    }
                    "Edit" => {
                        config.resources[resource].selectors[&selector].path = Text::new("Path:")
                            .with_validator(required!("This field is required"))
                            .with_help_message("e.g. body > div > h1")
                            .with_initial_value(&selector.path)
                            .prompt()?;
                        break 'selectors_loop;
                    }
                    "Change type" => {
                        config.resources[resource].selectors[&selector].parsed_type =
                            Select::new("Selector type:", SelectorType::list_as_vec()).prompt()?;
                        break 'selectors_loop;
                    }
                    "Delete" => {
                        if Confirm::new("Are you sure you want to delete this selector?")
                            .with_default(false)
                            .prompt()?
                        {
                            let index = config.resources[resource].selectors.position(&selector);
                            config.resources[resource].selectors.remove(index);
                            break 'selectors_loop;
                        }
                    }
                    "↩ Back" => continue 'selectors_loop,
                    "⏹ Exit" => break 'selectors_loop,
                    _ => unreachable!(),
                }
            },
            "⏹ Exit" => break 'edit_selectors,
            _ => unreachable!(),
        }

        match Confirm::new("Edit more selectors?")
            .with_default(true)
            .prompt()?
        {
            true => {
                continue 'edit_selectors;
            }
            false => break 'edit_selectors,
        }
    }

    Ok(())
}
