#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use env_logger::Env;
use std::path::{Path, PathBuf};
use std::process;

use mdbookshelf;
use mdbookshelf::config::Config;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    let matches = App::new("mdbookshelf")
        .about("Executes mdbook-epub on a collection of repositories")
        .version(concat!("v", crate_version!()))
        .author("Ramses Ladlani <rladlani@gmail.com>")
        .arg(
            Arg::with_name("working_dir")
                .short("w")
                .long("working_dir")
                .value_name("WORKING_DIR")
                .help("Sets a custom working directory where the book repositories will be cloned")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("destination_dir")
                .short("d")
                .long("destination_dir")
                .value_name("DESTINATION_DIR")
                .help("Sets the destination directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("templates_dir")
                .short("t")
                .long("templates_dir")
                .value_name("TEMPLATES_DIR")
                .help("Sets the templates directory")
                .takes_value(true),
        )
        .get_matches();

    // :TODO: add argument to set config path (bookshelf.toml)

    let config_location = Path::new(".").join("bookshelf.toml");
    let mut config = if config_location.exists() {
        info!("Loading config from {}", config_location.display());
        Config::from_disk(&config_location).unwrap_or_default()
    } else {
        Config::default()
    };

    if let Some(destination_dir) = matches.value_of("destination_dir") {
        config.destination_dir = Some(PathBuf::from(destination_dir));
    }

    assert!(
        config.destination_dir.is_some(),
        "Destination dir must be set in toml file or through command line"
    );

    info!(
        "Running mdbookshelf with destination {}",
        config.destination_dir.as_ref().unwrap().display()
    );

    if let Some(working_dir) = matches.value_of("working_dir") {
        config.working_dir = Some(PathBuf::from(working_dir));
    }

    config.working_dir = config.working_dir.or(Some(PathBuf::from("repos")));

    info!(
        "Cloning repositories to {}",
        config.working_dir.as_ref().unwrap().display()
    );

    if let Some(templates_dir) = matches.value_of("templates_dir") {
        config.templates_dir = Some(PathBuf::from(templates_dir));
    }

    match config.templates_dir.as_ref() {
        Some(templates_dir) => info!("Using templates in {}", templates_dir.display()),
        None => info!("No templates dir provided"),
    }

    if let Err(e) = mdbookshelf::run(&config) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}
