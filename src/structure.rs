use anyhow::{anyhow, Result};
/// This file contains the structure of the config file.
/// It is used to create and serialize the config file.
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub enum ConfigFormat {
    Toml,
    Json,
}

/// A selector is named a path to a value on a web page
#[derive(Serialize, Deserialize)]
pub struct Selector {
    path: String,
    name: String,
}

impl Selector {
    /// Create a new selector
    pub fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}

// A resource is a website with a list of selectors
#[derive(Serialize, Deserialize)]
pub struct Resource {
    url: String,
    selectors: Vec<Selector>,
}

impl Resource {
    /// Create a new resource
    pub fn new(url: String, selectors: Vec<Selector>) -> Self {
        Self { url, selectors }
    }
}

// A config is a list of resources
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    description: String,
    resources: Vec<Resource>,
}

impl Config {
    // Create a new config
    pub fn new(name: String, description: String, resources: Vec<Resource>) -> Self {
        Self {
            name,
            description,
            resources,
        }
    }

    /// It reads a file, then parses it as either TOML or JSON, and returns a [Config]
    ///
    /// Arguments:
    ///
    /// * `path`: The path to the file to read from.
    /// * `cf`: ConfigFormat - This is the format of the config file.
    ///
    /// Returns:
    ///
    /// A [Result<Self>]
    pub fn from_file(path: &Path, cf: ConfigFormat) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        match cf {
            ConfigFormat::Toml => Self::from_toml(&data),
            ConfigFormat::Json => Self::from_json(&data),
        }
    }

    /// Saves the [Config] structure to a file with the given name and specified format.
    ///
    /// Arguments:
    ///
    /// * `cf`: [ConfigFormat] - This is the format that you want to save the config in.
    ///
    /// Returns:
    ///
    /// A path to the saved config [Result<PathBuf, std::io::Error>]
    pub fn save(&self, cf: ConfigFormat) -> Result<PathBuf> {
        let data = match cf {
            ConfigFormat::Toml => self.to_toml(),
            ConfigFormat::Json => self.to_json(),
        };

        let full_path = self.get_full_path(cf);
        std::fs::write(full_path.clone(), data)?;

        Ok(full_path)
    }

    /// It returns the full path of the config file
    ///
    /// Arguments:
    ///
    /// * `cf`: [ConfigFormat] - This is the format that you want to save the config in.
    ///
    /// Returns:
    ///
    /// A [PathBuf]
    pub fn get_full_path(&self, cf: ConfigFormat) -> PathBuf {
        let file_name = match cf {
            ConfigFormat::Toml => format!("{}.toml", self.name),
            ConfigFormat::Json => format!("{}.json", self.name),
        };

        Path::new(&env::current_dir().unwrap()).join(file_name)
    }

    /// Convert config to TOML string
    fn to_toml(&self) -> String {
        toml::to_string(&self).unwrap_or("".to_string())
    }

    /// Convert config to JSON string
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or("".to_string())
    }

    /// Parse a TOML string into a [Config]
    fn from_toml(data: &str) -> Result<Self> {
        match toml::from_str(data) {
            Ok(config) => Ok(config),
            Err(_) => Err(anyhow!("Failed parsing TOML config!")),
        }
    }

    /// Parse a JSON string into a [Config]
    fn from_json(data: &str) -> Result<Self> {
        match serde_json::from_str(data) {
            Ok(config) => Ok(config),
            Err(_) => Err(anyhow!("Failed parsing JSON config!")),
        }
    }
}
