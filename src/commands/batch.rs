use std::{ffi::OsStr, path::PathBuf};

use crate::scalper::{grab, ParsedValue};
use crate::structure::{Config, ConfigFormat};
use anyhow::{anyhow, Result};
use clap::{value_parser, Parser};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use serde::Serialize;
use serde_json::{json, to_string_pretty};

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

    /// (Optional) Single parameter to be passed to all resources.
    ///
    /// This argument is mutually exclusive with `params`.
    ///
    /// Example:
    ///
    /// ```
    /// --one-param param1
    /// ```
    ///
    /// This argument is useful when you want to pass the same parameter to all resources.
    #[arg(long, conflicts_with = "params")]
    one_param: Option<String>,

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
        if args.params.is_none() && args.one_param.is_none() {
            return Err(anyhow!(
                "This config needs parameters!\nMore info: rvp batch --help"
            ));
        }

        if args.params.is_some() {
            let params = args.params.unwrap();
            let resources_len = config.resources.len();
            if resources_len != params.len() {
                return Err(anyhow!(
                    "The number of parameters does not match the number of resources ({})!",
                    resources_len
                ));
            }

            for (i, param) in params.iter().enumerate() {
                config.resources[i].mut_url_with_param(param);
            }
        }

        if args.one_param.is_some() {
            let param = args.one_param.unwrap();

            for resource in config.resources.iter_mut() {
                resource.mut_url_with_param(&param);
            }
        }
    }

    // TODO: parse in a thread pool
    let mut tasks = Vec::default();
    for r in config.resources {
        tasks.push(tokio::spawn(grab(r.selectors, r.url)));
    }

    let mut outputs = Vec::default();
    for task in tasks {
        let mut parsed = match task.await.unwrap() {
            Ok(v) => v,
            Err(e) => {
                panic!("Error while processing request to one of the URLs: {}", e);
            }
        };
        outputs.append(&mut parsed);
    }

    if args.json {
        println!("{}", generate_json(&outputs));
    } else {
        println!("{}", generate_table(&outputs));
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
fn generate_json(parsed_values: &Vec<ParsedValue>) -> String {
    let json_str = json!(parsed_values);
    to_string_pretty(&json_str).expect("Error while prettifying json!")
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

        let json = generate_json(&parsed_values);

        assert_eq!(
            json,
            "[\n  {\n    \"name\": \"name1\",\n    \"value\": \"value1\"\n  },\n  \
            {\n    \"name\": \"name2\",\n    \"value\": 25.6\n  }\n]"
        );
    }
}
