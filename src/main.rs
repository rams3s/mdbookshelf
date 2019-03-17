#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use env_logger::Env;
use std::process;

use mdbook_library;
use mdbook_library::Config;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    let matches = App::new("mdbook-library")
        .about("Executes mdbook on a collection of repositories")
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
            Arg::with_name("DESTINATION")
                .help("Sets the input file to use")
                .required(true),
        )
        .get_matches();

    let destination_dir = matches.value_of("DESTINATION").unwrap().to_string();

    info!(
        "Running mdbook-library with destination {}",
        destination_dir
    );

    let working_dir = matches
        .value_of("working_dir")
        .unwrap_or("./book_repos")
        .to_string();

    info!("Cloning repositories to {}", working_dir);

    let config = Config {
        destination_dir,
        working_dir,
    };

    if let Err(e) = mdbook_library::run(config) {
        error!("Application error: {}", e);

        process::exit(1);
    }
}
