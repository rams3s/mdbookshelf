extern crate git2;
#[macro_use]
extern crate log;

use git2::Repository;
use std::error::Error;
use std::path::Path;

pub struct Config<'a> {
    pub destination_dir: &'a str,
    pub working_dir: &'a str,
    pub repo_urls: Vec<&'a str>,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for repo_url in &config.repo_urls {
        let _repo = clone_or_fetch_repo(repo_url, &config.working_dir)?;
        // :TODO:
    }

    Ok(())
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
