#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use env_logger::Env;
use std::fs::File;
use std::process;
use std::path::Path;

use mdbookshelf;
use mdbookshelf::Config;

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
            Arg::with_name("DESTINATION")
                .help("Sets the input file to use")
                .required(true),
        )
        .get_matches();

    let destination_dir = matches.value_of("DESTINATION").unwrap();

    info!(
        "Running mdbookshelf with destination {}",
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

    match mdbookshelf::run(config) {
        Ok( manifest ) => {
            let manifest_path = Path::new(destination_dir).join("manifest.json");
            let f = File::create(manifest_path).expect("Could not create manifest file");
            serde_json::to_writer_pretty(f, &manifest).expect("Error while writing manifest to file");
        }
        Err( e ) => {
            error!("Application error: {}", e);
            process::exit(1);
        }
    }
}
