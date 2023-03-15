/// This file contains the structure of the config file.
/// It is used to create and serialize the config file.

use serde::Serialize;

/// A selector is named a path to a value on a web page
#[derive(Serialize)]
pub struct Selector {
    path: String,
    name: String,
}

impl Selector {
    pub fn new(path: String, name: String) -> Self {
        Self { path, name }
    }
}

// A resource is a website with a list of selectors
#[derive(Serialize)]
pub struct Resource {
    url: String,
    selectors: Vec<Selector>,
}

impl Resource {
    pub fn new(url: String, selectors: Vec<Selector>) -> Self {
        Self { url, selectors }
    }
}

// A config is a list of resources
#[derive(Serialize)]
pub struct Config {
    name: String,
    description: String,
    resources: Vec<Resource>,
}

impl Config {
    pub fn new(name: String, description: String, resources: Vec<Resource>) -> Self {
        Self {
            name,
            description,
            resources,
        }
    }
}