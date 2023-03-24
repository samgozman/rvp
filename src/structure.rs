use anyhow::{anyhow, Result};
/// This file contains the structure of the config file.
/// It is used to create and serialize the config file.
use serde::{Deserialize, Serialize};
use std::{
    env, fmt, fs, ops,
    path::{Path, PathBuf},
};

/// This is the placeholder for the parameters in the URL
pub const URL_PARAM_PLACEHOLDER: &str = "%%";

pub trait Position<T> {
    /// It returns the position of the element in the [Vec]
    fn position(&self, element: T) -> usize;
}

pub enum ConfigFormat {
    Toml,
    Json,
}

/// A selector is named a path to a value on a web page
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Selector {
    pub path: String,
    pub name: String,
}

impl Selector {
    /// Create a new selector
    pub fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Position<&Selector> for Vec<Selector> {
    fn position(&self, element: &Selector) -> usize {
        self.iter().position(|s| s == element).unwrap()
    }
}

// A resource is a website with a list of selectors
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Resource {
    pub url: String,
    pub selectors: Vec<Selector>,
}

impl Resource {
    /// Create a new resource
    pub fn new(url: String, selectors: Vec<Selector>) -> Self {
        Self { url, selectors }
    }

    /// It replaces the parameter placeholder with the given parameter
    pub fn mut_url_with_param(&mut self, param: &str) {
        self.url = self.url.replace(URL_PARAM_PLACEHOLDER, param);
    }

    /// It checks if the URL contains the parameter placeholder
    fn needs_parameter(&self) -> bool {
        self.url.contains(URL_PARAM_PLACEHOLDER)
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" with {} selectors",
            self.url,
            self.selectors.len()
        )
    }
}

// Implement the Index trait for Vec<Resource> use Resource as index for the vector
impl ops::Index<&Resource> for Vec<Resource> {
    type Output = Resource;

    fn index(&self, index: &Resource) -> &Self::Output {
        for resource in self.iter() {
            if resource == index {
                return resource;
            }
        }
        panic!("resource not found");
    }
}

// Implement the IndexMut trait for Vec<Resource> use &mut Resource as index for the vector
// Requires the Index trait to be implemented.
impl ops::IndexMut<&Resource> for Vec<Resource> {
    fn index_mut(&mut self, index: &Resource) -> &mut Self::Output {
        for resource in self {
            if resource == index {
                return resource;
            }
        }
        panic!("resource not found");
    }
}

impl Position<&Resource> for Vec<Resource> {
    fn position(&self, element: &Resource) -> usize {
        self.iter().position(|r| r == element).unwrap()
    }
}

// A config is a list of resources
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    description: String,
    pub resources: Vec<Resource>,
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
    /// * `cf`: [ConfigFormat] - This is the format of the config file.
    ///
    /// Returns:
    ///
    /// A [Result<Self>]
    pub fn from_file(path: &Path, cf: &ConfigFormat) -> Result<Self> {
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
    pub fn save(&self, cf: &ConfigFormat) -> Result<PathBuf> {
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
    pub fn get_full_path(&self, cf: &ConfigFormat) -> PathBuf {
        let file_name = match cf {
            ConfigFormat::Toml => format!("{}.toml", self.name),
            ConfigFormat::Json => format!("{}.json", self.name),
        };

        Path::new(&env::current_dir().unwrap()).join(file_name)
    }

    /// It checks if the config resources need parameters
    pub fn needs_parameters(&self) -> bool {
        self.resources.iter().any(|r| r.needs_parameter())
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
