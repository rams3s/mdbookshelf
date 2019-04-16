extern crate chrono;
#[macro_use]
extern crate failure;
extern crate git2;
#[macro_use]
extern crate log;
extern crate mdbook;
extern crate mdbook_epub;
extern crate serde;
extern crate serde_json;

pub mod config;

use chrono::{TimeZone, Utc};
use config::Config;
use failure::Error;
use git2::Repository;
use mdbook::renderer::RenderContext;
use mdbook::MDBook;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct ManifestEntry {
    pub commit_sha: String,
    pub epub_size: u64,
    pub last_modified: String,
    pub path: PathBuf,
    pub repo_url: String,
    pub title: String,
    pub url: String,
}

impl Default for ManifestEntry {
    fn default() -> Self {
        ManifestEntry {
            commit_sha: String::default(),
            epub_size: 0,
            last_modified: String::default(),
            path: PathBuf::default(),
            repo_url: String::default(),
            title: String::default(),
            url: String::default(),
        }
    }
}

#[derive(Default, Serialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
    pub timestamp: String,
    pub title: String,
}

impl Manifest {
    pub fn new() -> Manifest {
        let entries = Vec::new();
        let timestamp = Utc::now().to_rfc2822();
        Manifest {
            entries,
            timestamp,
            title: String::default(),
        }
    }
}

pub fn run(config: Config) -> Result<Manifest, Error> {
    let mut manifest = Manifest::new();
    manifest.entries.reserve(config.book_repo_configs.len());
    manifest.title = config.title;

    for repo_config in config.book_repo_configs {
        let mut manifest_entry = ManifestEntry::default();

        let repo_url = &repo_config.repo_url;
        let folder = repo_url.split('/').last().unwrap();
        let (_repo, mut repo_path) =
            clone_or_fetch_repo(&mut manifest_entry, repo_url, &config.working_dir)?;

        if let Some(repo_folder) = repo_config.folder {
            repo_path = repo_path.join(repo_folder);
        }

        let dest = Path::new(&config.destination_dir).join(folder);
        if let Err(e) = generate_epub(&mut manifest_entry, repo_path, dest) {
            error!("Epub generation failed {}", e);
            continue;
        }

        manifest_entry.title = repo_config.title;
        manifest_entry.repo_url = repo_config.repo_url;
        manifest_entry.url = repo_config.url;

        manifest.entries.push(manifest_entry);
    }

    Ok(manifest)
}

fn generate_epub(entry: &mut ManifestEntry, path: PathBuf, dest: PathBuf) -> Result<(), Error> {
    let md = MDBook::load(path).map_err(|e| format_err!("Could not load mdbook: {}", e))?;

    let ctx = RenderContext::new(
        md.root.clone(),
        md.book.clone(),
        md.config.clone(),
        dest.to_path_buf(),
    );

    mdbook_epub::generate(&ctx).unwrap_or_else(|e| {
        error!("{}", e);
    });

    let output_file = mdbook_epub::output_filename(&dest, &ctx.config);
    info!("Generated epub into {}", output_file.display());

    let metadata = std::fs::metadata(&output_file)?;
    let epub_size = metadata.len();
    entry.epub_size = epub_size;
    entry.path = output_file;

    if let Some(title) = md.config.book.title {
        entry.title = title;
    }

    Ok(())
}

fn clone_or_fetch_repo(
    entry: &mut ManifestEntry,
    url: &str,
    working_dir: &str,
) -> Result<(Repository, PathBuf), Error> {
    let folder = url.split('/').last().unwrap();
    let mut dest = Path::new(working_dir).join(folder);

    // :TRICKY: can't use \ as path separator here because of improper native path handling in some parts of libgit2
    // see https://github.com/libgit2/libgit2/issues/3012
    if cfg!(windows) {
        dest = PathBuf::from(dest.to_str().unwrap().replace('\\', "/"));
    }

    let repo = match Repository::open(&dest) {
        Ok(repo) => {
            info!("Found {:?}. Fetching {}", dest, url);
            repo.find_remote("origin")?
                .fetch(&["master"], None, None)
                .unwrap();
            repo
        }
        Err(_err) => {
            info!("Cloning {} to {:?}", url, dest);
            Repository::clone(url, &dest)?
        }
    };

    {
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        let commit_sha = commit.id();
        let last_modified = Utc.timestamp(commit.time().seconds(), 0);

        entry.commit_sha = commit_sha.to_string();
        entry.last_modified = last_modified.to_rfc2822();
    }

    Ok((repo, dest))
}

#[test]
fn test_generate_epub() {
    let mut entry = ManifestEntry::default();
    let path = Path::new("tests").join("dummy");
    let dest = Path::new("tests").join("book");

    generate_epub(&mut entry, path, dest).unwrap();

    assert!(entry.epub_size > 0, "Epub size should be bigger than 0");
    assert_eq!(entry.title, "Hello Rust", "Title doesn't match");
    assert_eq!(
        entry.path,
        Path::new("tests").join("book").join("Hello Rust.epub"),
        "Manifest entry path should be filled"
    );
}
