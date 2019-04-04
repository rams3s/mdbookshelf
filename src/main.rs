#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate tera;

use clap::{App, Arg};
use env_logger::Env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
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
            Arg::with_name("DESTINATION")
                .help("Sets the input file to use")
                .required(true),
        )
        .get_matches();

    let destination_dir = matches.value_of("DESTINATION").unwrap().to_string();

    info!("Running mdbookshelf with destination {}", destination_dir);

    let working_dir = matches
        .value_of("working_dir")
        .unwrap_or("./repos")
        .to_string();

    info!("Cloning repositories to {}", working_dir);

    let manifest_path = Path::new(&destination_dir).join("manifest.json");
    let index_path = Path::new(&destination_dir).join("index.md");

    let config_location = Path::new(".").join("bookshelf.toml");
    let mut config = if config_location.exists() {
        info!("Loading config from {}", config_location.display());
        Config::from_disk(&config_location).unwrap_or_default()
    } else {
        Config::default()
    };

    config.destination_dir = destination_dir;
    config.working_dir = working_dir;

    match mdbookshelf::run(config) {
        Ok(manifest) => {
            info!("Writing manifest to {}", manifest_path.display());
            let f = File::create(&manifest_path).expect("Could not create manifest file");
            serde_json::to_writer_pretty(f, &manifest)
                .expect("Error while writing manifest to file");

            // :TODO: parametrize templates path
            let tera = compile_templates!("templates/**/*");
            let page = tera.render("index.md", &manifest).expect("Template error");

            let mut f = File::create(&index_path).expect("Could not create index file");

            f.write_all(page.as_bytes())
                .expect("Error while writing index file");
        }
        Err(e) => {
            error!("Application error: {}", e);
            process::exit(1);
        }
    }
}
