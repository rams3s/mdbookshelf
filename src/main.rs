#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate git2;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use env_logger::Env;
use git2::Repository;
use std::path::Path;
use std::process;

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

    let _repo = clone_or_fetch_repo("https://github.com/rust-lang/book.git", &config.working_dir)
        .unwrap_or_else(|err| {
            error!("Problem while cloning or fetching repo: {}", err);
            process::exit(1);
        });
}

struct Config {
    pub destination_dir: String,
    pub working_dir: String,
}

fn clone_or_fetch_repo(url: &str, working_dir: &str) -> Result<Repository, git2::Error> {
    let folder = url.split('/').last().unwrap();
    let dest = Path::new(working_dir).join(folder);

    match Repository::open(&dest) {
        Ok(repo) => {
            info!("Found {:?}. Fetching {}", dest, url);
            repo.find_remote("origin")?
                .fetch(&["master"], None, None)
                .unwrap();
            Ok(repo)
        }
        Err(_err) => {
            info!("Cloning {} to {:?}", url, dest);
            Repository::clone(url, &dest)
        }
    }
}
