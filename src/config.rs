//! Mdbookshelf's configuration.
//!
//! Heavily inspired by mdbook's Config.

#![deny(missing_docs)]

use failure::Error;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// The overall configuration object for MDBookshelf, essentially an in-memory
/// representation of `bookshelf.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Config {
    /// Destination directory.
    pub destination_dir: Option<String>,
    /// Title of the bookshelf.
    pub title: String,
    /// Working directory.
    pub working_dir: Option<String>,
    /// An array of collections
    pub collections: Vec<CollectionConfig>,
    /// Templates directory.
    pub templates_dir: Option<String>,
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
            destination_dir: None,
            title: String::default(),
            working_dir: None,
            collections: Vec::new(),
            templates_dir: None,
        }
    }
}

/// Configuration for collections (of books)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct CollectionConfig {
    /// The collection's title
    pub title: String,
    /// An array of BookRepoConfig
    pub book_repo_configs: Vec<BookRepoConfig>,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            book_repo_configs: Vec::new(),
        }
    }
}

/// Configuration options which are specific to the book and required for
/// loading it from disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BookRepoConfig {
    /// The book's title
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

        [[collections]]
        title = "My collection"

        [[collections.entries]]
        title = "Some Book"
        repo-url = "git_source"
        url = "source"
        folder = "./foo"

        [[collections.entries]]
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
        assert_eq!(got.templates_dir.unwrap(), "templates/");

        assert_eq!(got.collections.len(), 1);
        let collection = &got.collections[0];
        assert_eq!(collection.title, "My collection");
        assert_eq!(collection.book_repo_configs, book_repo_configs);
    }
}
