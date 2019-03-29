//! Mdbookshelf's configuration.
//!
//! # Examples
//!
//! ```rust
//! # extern crate mdbook;
//! # use mdbook::errors::*;
//! # extern crate toml;
//! use std::path::PathBuf;
//! use mdbook::Config;
//! use toml::Value;
//!
//! # fn run() -> Result<()> {
//! let src = r#"
//! [book]
//! title = "My Book"
//! authors = ["Michael-F-Bryan"]
//!
//! [build]
//! src = "out"
//!
//! [other-table.foo]
//! bar = 123
//! "#;
//!
//! // load the `Config` from a toml string
//! let mut cfg = Config::from_str(src)?;
//!
//! // retrieve a nested value
//! let bar = cfg.get("other-table.foo.bar").cloned();
//! assert_eq!(bar, Some(Value::Integer(123)));
//!
//! // Set the `output.html.theme` directory
//! assert!(cfg.get("output.html").is_none());
//! cfg.set("output.html.theme", "./themes");
//!
//! // then load it again, automatically deserializing to a `PathBuf`.
//! let got: PathBuf = cfg.get_deserialized("output.html.theme")?;
//! assert_eq!(got, PathBuf::from("./themes"));
//! # Ok(())
//! # }
//! # fn main() { run().unwrap() }
//! ```

#![deny(missing_docs)]

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use toml::value::Table;
use toml::{self, Value};
use toml_query::delete::TomlValueDeleteExt;
use toml_query::insert::TomlValueInsertExt;
use toml_query::read::TomlValueReadExt;

use errors::*;

/// The overall configuration object for MDBook, essentially an in-memory
/// representation of `bookshelf.toml`.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Destination directory.
    pub destination_dir: String,
    /// Working directory.
    pub working_dir: String,
    /// An array of BookRepoConfig
    pub repo_urls: Vec<String>,
    rest: Value,
}

impl Config {
    /// Load a `Config` from some string.
    pub fn from_str(src: &str) -> Result<Config> {
        toml::from_str(src).chain_err(|| Error::from("Invalid configuration file"))
    }

    /// Load the configuration file from disk.
    pub fn from_disk<P: AsRef<Path>>(config_file: P) -> Result<Config> {
        let mut buffer = String::new();
        File::open(config_file)
            .chain_err(|| "Unable to open the configuration file")?
            .read_to_string(&mut buffer)
            .chain_err(|| "Couldn't read the file")?;

        Config::from_str(&buffer)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            destination_dir: String::default(),
            working_dir: String::default(),
            repo_urls: Vec::new(),
            rest: Value::Table(Table::default()),
        }
    }
}
impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(de: D) -> ::std::result::Result<Self, D::Error> {
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

        let book: BookConfig = table
            .remove("book")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();

        let build: BuildConfig = table
            .remove("build")
            .and_then(|value| value.try_into().ok())
            .unwrap_or_default();

        Ok(Config {
            book,
            build,
            rest: Value::Table(table),
        })
    }
}

impl Serialize for Config {
    fn serialize<S: Serializer>(&self, s: S) -> ::std::result::Result<S::Ok, S::Error> {
        use serde::ser::Error;

        let mut table = self.rest.clone();

        let book_config = match Value::try_from(self.book.clone()) {
            Ok(cfg) => cfg,
            Err(_) => {
                return Err(S::Error::custom("Unable to serialize the BookConfig"));
            }
        };

        table.insert("book", book_config).expect("unreachable");
        table.serialize(s)
    }
}

/// Configuration options which are specific to the book and required for
/// loading it from disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BookRepoConfig {
    /// The book's title.
    // :TODO: get it from book itself?
    pub title: Option<String>,
    /// The books root directories.
    pub src_dirs: Option<Vec<PathBuf>>,
    /// The git repository url.
    pub url: String,
}

impl Default for BookRepoConfig {
    fn default() -> BookConfig {
        BookConfig {
            title: None,
            src_dirs: Some(vec!(PathBuf::from(".")),
            url: String::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLEX_CONFIG: &'static str = r#"
        [book]
        title = "Some Book"
        authors = ["Michael-F-Bryan <michaelfbryan@gmail.com>"]
        description = "A completely useless book"
        multilingual = true
        src = "source"

        [build]
        build-dir = "outputs"
        create-missing = false
        use-default-preprocessors = true
        "#;

    #[test]
    fn load_a_complex_config_file() {
        let src = COMPLEX_CONFIG;

        let book_should_be = BookConfig {
            title: Some(String::from("Some Book")),
            authors: vec![String::from("Michael-F-Bryan <michaelfbryan@gmail.com>")],
            description: Some(String::from("A completely useless book")),
            multilingual: true,
            src: PathBuf::from("source"),
            ..Default::default()
        };
        let build_should_be = BuildConfig {
            build_dir: PathBuf::from("outputs"),
            create_missing: false,
            use_default_preprocessors: true,
        };
        let playpen_should_be = Playpen {
            editable: true,
            copy_js: true,
        };
        let html_should_be = HtmlConfig {
            curly_quotes: true,
            google_analytics: Some(String::from("123456")),
            additional_css: vec![PathBuf::from("./foo/bar/baz.css")],
            theme: Some(PathBuf::from("./themedir")),
            default_theme: Some(String::from("rust")),
            playpen: playpen_should_be,
            git_repository_url: Some(String::from("https://foo.com/")),
            git_repository_icon: Some(String::from("fa-code-fork")),
            ..Default::default()
        };

        let got = Config::from_str(src).unwrap();

        assert_eq!(got.book, book_should_be);
        assert_eq!(got.build, build_should_be);
        assert_eq!(got.html_config().unwrap(), html_should_be);
    }
}
