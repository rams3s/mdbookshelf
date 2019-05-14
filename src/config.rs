//! Mdbookshelf's configuration.
//!
//! Heavily inspired by mdbook's Config.

#![deny(missing_docs)]

use failure::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml::{self, Value};

/// The overall configuration object for MDBookshelf, essentially an in-memory
/// representation of `bookshelf.toml`.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Additional CSS files.
    pub additional_css: Vec<PathBuf>,
    /// An array of BookRepoConfig.
    pub book_repo_configs: Vec<BookRepoConfig>,
    /// Destination directory.
    pub destination_dir: Option<PathBuf>,
    /// Templates directory (if not set, will generate manifest.json).
    pub templates_dir: Option<PathBuf>,
    /// Title of the book collection.
    pub title: String,
    /// Working directory.
    pub working_dir: Option<PathBuf>,
}

impl Config {
    /// Load the configuration file from disk.
    pub fn from_disk<P: AsRef<Path>>(config_file: P) -> Result<Config, Error> {
        let mut buffer = String::new();
        File::open(config_file)?.read_to_string(&mut buffer)?;

        Config::from_str(&buffer)
    }
}

impl FromStr for Config {
    type Err = Error;

    /// Load a `Config` from some string.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        toml::from_str(src).map_err(|e| format_err!("{}", e))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            additional_css: Vec::new(),
            book_repo_configs: Vec::new(),
            destination_dir: None,
            templates_dir: None,
            title: String::default(),
            working_dir: None,
        }
    }
}
impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let raw = Value::deserialize(de)?;

        let mut table = match raw {
            Value::Table(t) => t,
            _ => {
                use serde::de::Error;
                return Err(D::Error::custom(
                    "A config file should always be a toml table",
                ));
            }
        };

        let book_repo_configs: Vec<BookRepoConfig> = table
            .remove("book")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();
        let additional_css: Vec<PathBuf> = table
            .remove("additional-css")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();
        let destination_dir: Option<PathBuf> = table
            .remove("destination-dir")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();
        let templates_dir: Option<PathBuf> = table
            .remove("templates-dir")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();
        let title: String = table
            .remove("title")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();
        let working_dir: Option<PathBuf> = table
            .remove("working-dir")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();

        Ok(Config {
            additional_css,
            book_repo_configs,
            destination_dir,
            templates_dir,
            title,
            working_dir,
        })
    }
}

/// The configuration for a single book
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BookRepoConfig {
    /// The book's title.  
    /// If set, overwrites the value read from the book itself when generating the manifest.
    pub title: Option<String>,
    /// The book root directory.
    pub folder: Option<PathBuf>,
    /// The git repository url.
    pub repo_url: String,
    /// The online rendered book url.
    pub url: String,
}

impl Default for BookRepoConfig {
    fn default() -> Self {
        BookRepoConfig {
            title: None,
            folder: None,
            repo_url: String::default(),
            url: String::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLEX_CONFIG: &'static str = r#"
        title = "My bookshelf"
        templates-dir = "templates/"

        [[book]]
        title = "Some Book"
        repo-url = "git_source"
        url = "source"
        folder = "./foo"

        [[book]]
        repo-url = "git_source2"
        url = "source2"
        "#;

    #[test]
    fn load_config_file() {
        let src = COMPLEX_CONFIG;

        let book_repo_configs = vec![
            BookRepoConfig {
                title: Some(String::from("Some Book")),
                folder: Some(PathBuf::from("./foo")),
                repo_url: String::from("git_source"),
                url: String::from("source"),
                ..Default::default()
            },
            BookRepoConfig {
                repo_url: String::from("git_source2"),
                url: String::from("source2"),
                ..Default::default()
            },
        ];

        let got = Config::from_str(src).unwrap();

        assert_eq!(got.title, "My bookshelf");
        assert_eq!(got.templates_dir.unwrap().to_str().unwrap(), "templates/");
        assert_eq!(got.book_repo_configs, book_repo_configs);
    }
}
