#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};
use env_logger::Env;

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

    info!(
        "Running mdbook-library with destination {}",
        matches.value_of("DESTINATION").unwrap()
    );

    let repos = matches.value_of("working_dir").unwrap_or("book_repos");
    info!("Cloning repositories to {}", repos);
}