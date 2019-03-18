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

    let destination_dir = matches.value_of("DESTINATION").unwrap();

    info!(
        "Running mdbook-library with destination {}",
        destination_dir
    );

    let working_dir = matches.value_of("working_dir").unwrap_or("./repos");

    info!("Cloning repositories to {}", working_dir);

    // :TODO: read from config file

    let repo_urls = vec![
        "https://github.com/rust-lang/book.git",
        "https://github.com/rust-lang/async-book.git",
    ];

    let config = Config {
        destination_dir,
        working_dir,
        repo_urls,
    };

    if let Err(e) = mdbook_library::run(config) {
        error!("Application error: {}", e);

        process::exit(1);
    }
}
