#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate tera;
extern crate walkdir;

use clap::{App, Arg};
use env_logger::Env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

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
        .get_matches();

    // :TODO: add argument to set config path

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

    match mdbookshelf::run(&config) {
        Ok(manifest) => {
            let destination_dir = config.destination_dir.as_ref().unwrap();
            let manifest_path = Path::new(destination_dir).join("manifest.json");
            info!("Writing manifest to {}", manifest_path.display());

            let f = File::create(&manifest_path).expect("Could not create manifest file");
            serde_json::to_writer_pretty(f, &manifest)
                .expect("Error while writing manifest to file");

            // :TODO: parametrize templates path
            let tera = compile_templates!("templates/**/*");

            for entry in WalkDir::new("templates")
                .follow_links(true)
                .into_iter()
                .filter_map(|v| v.ok())
                .filter(|e| !e.file_type().is_dir())
            {
                let template_path = entry.path().strip_prefix("templates").unwrap();
                let template_path = template_path.to_str().unwrap();
                let output_path = Path::new(&destination_dir).join(template_path);

                info!(
                    "Rendering template {} to {}",
                    template_path,
                    output_path.display()
                );

                let page = tera
                    .render(template_path, &manifest)
                    .expect("Template error");
                let mut f = File::create(&output_path).expect("Could not create file");

                f.write_all(page.as_bytes())
                    .expect("Error while writing file");
            }
        }
        Err(e) => {
            error!("Application error: {}", e);
            process::exit(1);
        }
    }
}
