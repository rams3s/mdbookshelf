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
    /// Destination directory.
    pub destination_dir: String,
    /// Working directory.
    pub working_dir: String,
    /// An array of BookRepoConfig
    pub book_repo_configs: Vec<BookRepoConfig>,
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
            destination_dir: String::default(),
            working_dir: String::default(),
            book_repo_configs: Vec::new(),
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

        Ok(Config {
            book_repo_configs,
            ..Default::default()
        })
    }
}

/// Configuration options which are specific to the book and required for
/// loading it from disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BookRepoConfig {
    /// The book's title.
    // :TODO: get it from book itself?
    pub title: String,
    /// The book root directory.
    pub folder: Option<PathBuf>,
    /// The git repository url.
    pub repo_url: String,
    /// The online rendered book url.
    pub url: String,
}

impl Default for BookRepoConfig {
    fn default() -> BookRepoConfig {
        BookRepoConfig {
            title: String::default(),
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
        [[book]]
        title = "Some Book"
        repo-url = "git_source"
        url = "source"
        folder = "./foo"

        [[book]]
        title = "Some Book2"
        repo-url = "git_source2"
        url = "source2"
        "#;

    #[test]
    fn load_a_complex_config_file() {
        let src = COMPLEX_CONFIG;

        let book_repo_configs = vec![
            BookRepoConfig {
                title: String::from("Some Book"),
                folder: Some(PathBuf::from("./foo")),
                repo_url: String::from("git_source"),
                url: String::from("source"),
                ..Default::default()
            },
            BookRepoConfig {
                title: String::from("Some Book2"),
                repo_url: String::from("git_source2"),
                url: String::from("source2"),
                ..Default::default()
            },
        ];

        let got = Config::from_str(src).unwrap();

        assert_eq!(got.book_repo_configs, book_repo_configs);
    }
}
