use std::{ffi::OsStr, path::PathBuf};

use crate::scalper::{grab, ParsedValue};
use crate::structure::{Config, ConfigFormat};
use anyhow::{anyhow, Ok, Result};
use clap::{value_parser, Parser};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use serde::Serialize;
use serde_json::json;

/// Parse multiple data fields from a N resources defined in a config file
#[derive(Parser)]
pub struct Args {
    /// Path to the config file.
    #[arg(short, long, value_name = "PATH", value_parser = value_parser!(PathBuf))]
    path: PathBuf,

    /// (Optional) Parameters to be passed to the resources separated by spaces.
    ///
    /// Example:
    ///
    /// ```
    /// --params param1 param2 param3
    /// ```
    ///
    /// More complex example, if parameter is needed only for the first and the third resource:
    ///
    /// ```
    /// --params param1 _ param3
    /// ```
    ///
    /// In this case, you can pass any value for the second parameter, because it will be ignored.
    #[arg(long, num_args(0..))]
    params: Option<Vec<String>>,

    /// Output the data in JSON format
    #[arg(long)]
    json: bool,
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

    if config.needs_parameters() {
        if args.params.is_none() {
            return Err(anyhow!(
                "This config needs parameters!\nMore info: rvp batch --help"
            ));
        }

        let params = args.params.unwrap();

        if config.resources.len() != params.len() {
            return Err(anyhow!(
                "The number of parameters does not match the number of resources!"
            ));
        }

        for (i, param) in params.iter().enumerate() {
            config.resources[i].mut_url_with_param(param);
        }
    }

    // TODO: parse in a thread pool
    for resource in config.resources {
        let parsed_values = grab(resource.selectors, &resource.url).await?;

        // TODO: Combine resources into one object and print it at the end (not in the loop)
        if args.json {
            println!("{}", generate_json(&parsed_values, resource.url));
        } else {
            println!("Table for resource:\n{}", resource.url);
            println!("{}", generate_table(&parsed_values));
        }
    }

    Ok(())
}

/// Generate table from parsed values
fn generate_table(parsed_values: &Vec<ParsedValue>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Name", "Value"]);
    for parsed_value in parsed_values {
        table.add_row(vec![&parsed_value.name, &parsed_value.value.to_string()]);
    }

    table
}

/// Generate json from parsed values
fn generate_json(parsed_values: &Vec<ParsedValue>, url: String) -> String {
    // TODO: Create json with indentation
    let mut json_data = JsonData {
        url,
        data: Vec::new(),
    };

    for parsed_value in parsed_values {
        json_data.data.push(parsed_value.clone());
    }

    json!(json_data).to_string()
}

#[derive(Serialize)]
struct JsonData {
    url: String,
    data: Vec<ParsedValue>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Number, Value};

    #[test]
    fn test_generate_table() {
        let parsed_values = vec![
            ParsedValue {
                name: "name1".to_string(),
                value: Value::String("value1".to_string()),
            },
            ParsedValue {
                name: "name2".to_string(),
                value: Value::Number(Number::from_f64(2.2).unwrap()),
            },
        ];

        let table = generate_table(&parsed_values);

        assert_eq!(
            table.to_string(),
            "\
            ╭───────┬──────────╮\n\
            │ Name  ┆ Value    │\n\
            ╞═══════╪══════════╡\n\
            │ name1 ┆ \"value1\" │\n\
            ├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤\n\
            │ name2 ┆ 2.2      │\n\
            ╰───────┴──────────╯"
        );
    }

    #[test]
    fn test_generate_json() {
        let parsed_values = vec![
            ParsedValue {
                name: "name1".to_string(),
                value: Value::String("value1".to_string()),
            },
            ParsedValue {
                name: "name2".to_string(),
                value: Value::Number(Number::from_f64(25.6).unwrap()),
            },
        ];

        let json = generate_json(&parsed_values, "url".to_string());

        assert_eq!(
            json,
            r#"{"data":[{"name":"name1","value":"value1"},{"name":"name2","value":25.6}],"url":"url"}"#
        );
    }
}
