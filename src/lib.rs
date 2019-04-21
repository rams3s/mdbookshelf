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
#[macro_use]
extern crate tera;
extern crate url;
extern crate walkdir;

pub mod config;

use chrono::{TimeZone, Utc};
use config::Config;
use failure::Error;
use git2::Repository;
use mdbook::renderer::RenderContext;
use mdbook::MDBook;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;
use walkdir::WalkDir;

/// A manifest entry for the generated EPUB
#[derive(Serialize)]
pub struct ManifestEntry {
    /// The commit sha
    pub commit_sha: String,
    /// The size of the EPUB in bytes
    pub epub_size: u64,
    /// The last modified date of the book (i.e. the datetime of the last commit)
    pub last_modified: String,
    /// The path to the generated EPUB
    pub path: PathBuf,
    /// The book repository URL
    pub repo_url: String,
    /// The book title
    pub title: String,
    /// The book online version URL
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

/// A Manifest contains the information about all EPUBs built
/// during one invocation of `mdbookshelf.run()`.
#[derive(Default, Serialize)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
    pub timestamp: String,
    pub title: String,
}

impl Manifest {
    pub fn new() -> Manifest {
        let entries = Vec::new();
        let timestamp = Utc::now().to_rfc3339();
        Manifest {
            entries,
            timestamp,
            title: String::default(),
        }
    }
}

/// Generates all EPUBs defined in `config` and returns a `Manifest` containing
/// information about all generated books.
pub fn run(config: &Config) -> Result<Manifest, Error> {
    let mut manifest = Manifest::new();
    manifest.entries.reserve(config.book_repo_configs.len());
    manifest.title = config.title.clone();

    for repo_config in &config.book_repo_configs {
        let mut manifest_entry = ManifestEntry::default();

        let repo_url = &repo_config.repo_url;
        let folder = repo_url.split('/').last().unwrap();
        let (_repo, mut repo_path) = clone_or_fetch_repo(
            &mut manifest_entry,
            repo_url,
            config.working_dir.as_ref().unwrap().to_str().unwrap(),
        )?;

        if let Some(repo_folder) = &repo_config.folder {
            repo_path = repo_path.join(repo_folder);
        }

        let dest = config.destination_dir.as_ref().unwrap().join(folder);
        if let Err(e) = generate_epub(&mut manifest_entry, repo_path, dest) {
            error!("Epub generation failed {}", e);
            continue;
        }

        if let Some(title) = &repo_config.title {
            manifest_entry.title = title.clone();
        }

        manifest_entry.repo_url = repo_config.repo_url.clone();
        manifest_entry.url = repo_config.url.clone();

        manifest.entries.push(manifest_entry);
    }

    let destination_dir = config.destination_dir.as_ref().unwrap();

    match config.templates_dir.as_ref() {
        Some(templates_dir) => {
            let templates_pattern = templates_dir.join("**/*");
            let tera = compile_templates!(templates_pattern.to_str().unwrap());

            for entry in WalkDir::new(templates_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|v| v.ok())
                .filter(|e| !e.file_type().is_dir())
            {
                let template_path = entry.path().strip_prefix(templates_dir).unwrap();
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
        None => {
            let manifest_path = Path::new(destination_dir).join("manifest.json");
            info!("Writing manifest to {}", manifest_path.display());

            let f = File::create(&manifest_path).expect("Could not create manifest file");
            serde_json::to_writer_pretty(f, &manifest)
                .expect("Error while writing manifest to file");
        }
    }

    Ok(manifest)
}

/// Generate an EPUB from `path` to `dest`. Also modify manifest `entry` accordingly.
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

/// Clones or fetches the repo at `url` inside `working_dir`.
fn clone_or_fetch_repo(
    entry: &mut ManifestEntry,
    url: &str,
    working_dir: &str,
) -> Result<(Repository, PathBuf), Error> {
    let parsed_url = Url::parse(url)?;
    let folder = parsed_url.path();
    // skip initial `/` in path
    let mut it = folder.chars();
    it.next();
    let folder = it.as_str();
    let mut dest = Path::new(working_dir).join(folder);

    // :TRICKY: can't use \ as path separator here because of improper native path handling in some parts of libgit2
    // see https://github.com/libgit2/libgit2/issues/3012
    if cfg!(windows) {
        dest = PathBuf::from(dest.to_str().unwrap().replace('\\', "/"));
    }

    let repo = match Repository::open(&dest) {
        Ok(repo) => {
            {
                let remote = repo.find_remote("origin")?;
                assert_eq!(
                    remote.url().unwrap(),
                    url,
                    "Remote url for origin and requested url do not match"
                );
            }
            info!("Found {:?}. Fetching {}", dest, url);
            repo.find_remote("origin")?
                .fetch(&["master"], None, None)
                .unwrap();
            repo
        }
        Err(_err) => {
            // :TODO: shallow clone when supported by libgit2 (https://github.com/libgit2/libgit2/issues/3058)
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
        entry.last_modified = last_modified.to_rfc3339();
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
