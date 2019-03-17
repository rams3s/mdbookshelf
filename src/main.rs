#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate git2;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use env_logger::Env;
use git2::Repository;
use std::path::{Path, PathBuf};
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

    info!(
        "Running mdbook-library with destination {}",
        matches.value_of("DESTINATION").unwrap()
    );

    let repos = matches.value_of("working_dir").unwrap_or("./book_repos");
    info!("Cloning repositories to {}", repos);

    let dest = Path::new(repos).join("book");

    let _repo =
        clone_or_fetch_repo("https://github.com/rust-lang/book.git", dest).unwrap_or_else(|err| {
            error!("Problem while cloning or fetching repo: {}", err);
            process::exit(1);
        });
}

fn clone_or_fetch_repo(url: &str, dest: PathBuf) -> Result<Repository, git2::Error> {
    match Repository::open(&dest) {
        Ok(repo) => {
            info!("Found {:?}. Fetching {}", dest, url);
            repo.find_remote("origin")?.fetch(&["master"], None, None).unwrap();
            Ok(repo)
        }
        Err(_err) => {
            info!("Cloning {} to {:?}", url, dest);
            Repository::clone(url, &dest)
        }
    }
}
