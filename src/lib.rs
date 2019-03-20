extern crate chrono;
extern crate git2;
#[macro_use]
extern crate log;
extern crate mdbook;
extern crate mdbook_epub;
extern crate serde;
extern crate serde_json;

use chrono::Utc;
use git2::Repository;
use mdbook::renderer::RenderContext;
use mdbook::MDBook;
use serde::Serialize;
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct Config<'a> {
    pub destination_dir: &'a str,
    pub working_dir: &'a str,
    pub repo_urls: Vec<&'a str>,
}

#[derive(Serialize)]
pub struct ManifestEntry {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Default, Serialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
    pub timestamp: String,
}

impl Manifest {
    pub fn new() -> Manifest {
        let entries = Vec::new();
        let timestamp = Utc::now().to_rfc2822();
        Manifest { entries, timestamp }
    }
}

pub fn run(config: Config) -> Result<Manifest, Box<dyn Error>> {
    let mut manifest = Manifest::new();
    manifest.entries.reserve(config.repo_urls.len());

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

        let entry = ManifestEntry {
            name: folder.to_string(),
            path: output_file,
        };

        manifest.entries.push(entry);
    }

    Ok(manifest)
}

fn clone_or_fetch_repo(url: &str, working_dir: &str) -> Result<(Repository, PathBuf), git2::Error> {
    let folder = url.split('/').last().unwrap();
    let mut dest = Path::new(working_dir).join(folder);

    // :TRICKY: can't use \ as path separator here because of improper native path handling in some parts of libgit2
    // see https://github.com/libgit2/libgit2/issues/3012
    if cfg!(windows) {
        dest = PathBuf::from(dest.to_str().unwrap().replace('\\', "/"));
    }

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
