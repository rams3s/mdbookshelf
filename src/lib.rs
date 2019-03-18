extern crate git2;
#[macro_use]
extern crate log;
extern crate mdbook;
extern crate mdbook_epub;

use git2::Repository;
use mdbook::renderer::RenderContext;
use mdbook::MDBook;
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct Config<'a> {
    pub destination_dir: &'a str,
    pub working_dir: &'a str,
    pub repo_urls: Vec<&'a str>,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for repo_url in &config.repo_urls {
        let folder = repo_url.split('/').last().unwrap();
        let (_repo, repo_path) = clone_or_fetch_repo(repo_url, &config.working_dir)?;

        let md = MDBook::load(repo_path)?;
        let dest = Path::new(config.destination_dir).join(folder);

        let ctx = RenderContext::new(
            md.root.clone(),
            md.book.clone(),
            md.config.clone(),
            dest.to_path_buf(),
        );

        mdbook_epub::generate(&ctx)?;

        let output_file = mdbook_epub::output_filename(&dest, &ctx.config);
        info!("Generated epub into {}", output_file.display());
    }

    Ok(())
}

fn clone_or_fetch_repo(url: &str, working_dir: &str) -> Result<(Repository, PathBuf), git2::Error> {
    let folder = url.split('/').last().unwrap();
    let dest = Path::new(working_dir).join(folder);

    match Repository::open(&dest) {
        Ok(repo) => {
            info!("Found {:?}. Fetching {}", dest, url);
            repo.find_remote("origin")?
                .fetch(&["master"], None, None)
                .unwrap();
            Ok((repo, dest))
        }
        Err(_err) => {
            info!("Cloning {} to {:?}", url, dest);
            let repo = Repository::clone(url, &dest)?;
            Ok((repo, dest))
        }
    }
}
