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

/// This is the format of the config file to be saved or read
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ConfigFormat {
    Toml,
    Json,
}

/// The type for parsed [Selector] values
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum SelectorType {
    String,
    Number,
}

impl SelectorType {
    /// It returns a vector of all the possible [ParsedType]s
    pub fn to_vec() -> Vec<SelectorType> {
        vec![SelectorType::String, SelectorType::Number]
    }

    /// It returns the string representation of the [ParsedType]
    fn as_str(&self) -> &'static str {
        match self {
            SelectorType::String => "String",
            SelectorType::Number => "Number",
        }
    }
}

impl fmt::Display for SelectorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A selector is named a path to a value on a web page
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Selector {
    pub path: String,
    pub name: String,
    pub parsed_type: SelectorType,
}

impl Selector {
    /// Create a new selector
    pub fn new(path: String, name: String, parsed_type: SelectorType) -> Self {
        Self {
            path,
            name,
            parsed_type,
        }
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

impl ops::Index<&Selector> for Vec<Selector> {
    type Output = Selector;

    fn index(&self, index: &Selector) -> &Self::Output {
        for selector in self.iter() {
            if selector == index {
                return selector;
            }
        }
        panic!("selector not found");
    }
}

impl ops::IndexMut<&Selector> for Vec<Selector> {
    fn index_mut(&mut self, index: &Selector) -> &mut Self::Output {
        for selector in self {
            if selector == index {
                return selector;
            }
        }
        panic!("selector not found");
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
        fs::write(full_path.clone(), data)?;

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
            Err(e) => Err(anyhow!("Failed parsing TOML config: {}", e)),
        }
    }

    /// Parse a JSON string into a [Config]
    fn from_json(data: &str) -> Result<Self> {
        match serde_json::from_str(data) {
            Ok(config) => Ok(config),
            Err(e) => Err(anyhow!("Failed parsing JSON config: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector() {
        let s0 = Selector::new("test".to_string(), "test".to_string(), SelectorType::String);
        let s1 = Selector::new(
            "test2".to_string(),
            "test2".to_string(),
            SelectorType::Number,
        );

        let selectors = vec![s0.clone(), s1.clone()];

        assert_eq!(selectors[0].name, "test");
        assert_eq!(selectors[1].path, "test2");

        // Test position
        assert_eq!(selectors.position(&s1), 1);

        // Test the Index trait
        assert_eq!(selectors[&s0].name, selectors[0].name);
        assert_eq!(selectors[&s1].path, selectors[1].path);

        // Test the IndexMut trait
        let mut selectors = vec![s0.clone(), s1.clone()];
        selectors[&s0].name = "test".to_string();
        selectors[&s1].path = "test2".to_string();
    }

    #[test]
    fn test_resource() {
        let s0 = Selector::new("test".to_string(), "test".to_string(), SelectorType::String);
        let s1 = Selector::new(
            "test2".to_string(),
            "test2".to_string(),
            SelectorType::Number,
        );

        let selectors = vec![s0, s1];

        let r0 = Resource::new("https://test.com/?id=%%".to_string(), selectors.clone());
        let r1 = Resource::new("https://test2.com".to_string(), selectors);

        let resources = vec![r0.clone(), r1.clone()];

        assert_eq!(resources[0].url, "https://test.com/?id=%%");
        assert_eq!(resources[1].selectors[0].name, "test");

        // Test position
        assert_eq!(resources.position(&r1), 1);

        // Test the Index trait
        assert_eq!(resources[&r0].url, resources[0].url);
        assert_eq!(
            resources[&r1].selectors[0].name,
            resources[1].selectors[0].name
        );

        // Test the IndexMut trait
        let mut resources = vec![r0.clone(), r1.clone()];
        resources[&r0].url = "https://test.com/?id=%%".to_string();
        resources[&r1].selectors[0].name = "test".to_string();

        // Test mut_url_with_param
        let mut r2 = r0.clone();
        r2.mut_url_with_param("test");
        assert_eq!(r2.url, "https://test.com/?id=test");

        // Test needs_parameter
        assert!(r0.needs_parameter());
        assert!(!r1.needs_parameter());
    }

    #[test]
    fn test_config() {
        let s0 = Selector::new("test".to_string(), "test".to_string(), SelectorType::String);
        let s1 = Selector::new(
            "test2".to_string(),
            "test2".to_string(),
            SelectorType::Number,
        );

        let selectors = vec![s0, s1];

        let r0 = Resource::new("https://test.com/?id=%%".to_string(), selectors.clone());
        let r1 = Resource::new("https://test2.com".to_string(), selectors);

        let resources = vec![r0, r1];

        let config = Config::new("test".to_string(), "".to_string(), resources);

        assert_eq!(config.name, "test");
        assert_eq!(config.resources[0].url, "https://test.com/?id=%%");

        // Test needs_parameters
        assert!(config.needs_parameters());
    }
}
