extern crate chrono;
extern crate git2;
#[macro_use]
extern crate log;
extern crate mdbook;
extern crate mdbook_epub;
extern crate serde;
extern crate serde_json;

use chrono::Utc;
use git2::build::CheckoutBuilder;
use git2::Repository;
use mdbook::renderer::RenderContext;
use mdbook::MDBook;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct Config<'a> {
    pub destination_dir: &'a str,
    pub working_dir: &'a str,
    pub repo_urls: Vec<&'a str>,
}

#[derive(Serialize)]
pub struct BookVersion {
    pub commit: String,
    pub commit_date: String,
    pub byte_count: usize,
    pub path: PathBuf,
}

#[derive(Serialize)]
pub struct ManifestEntry {
    pub name: String,
    pub versions: HashMap<String, BookVersion>,
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
        let (repo, repo_path) = clone_or_fetch_repo(repo_url, &config.working_dir)?;

        let tag_names = repo.tag_names(Option::None)?;
        let mut ref_names = Vec::with_capacity(tag_names.len() + 1);
        let mut versions = HashMap::new();

        ref_names.push("refs/heads/master".to_string());

        for tag_name in tag_names.iter() {
            let tag_name = tag_name.unwrap();
            ref_names.push(format!("refs/tags/{}", tag_name));
        }

        for ref_name in ref_names {
            let oid = repo.refname_to_id(&ref_name)?;
            let version_folder = ref_name.split('/').last().unwrap();
            info!(
                "Ref {} - id {} - version folder {}",
                ref_name, oid, version_folder
            );

            repo.set_head_detached(oid)?;
            let mut builder = CheckoutBuilder::new();
            builder.force();
            repo.checkout_head(Option::Some(&mut builder))?;

            let md = MDBook::load(&repo_path)?;
            let dest = Path::new(config.destination_dir)
                .join(folder)
                .join(version_folder);

            let ctx = RenderContext::new(
                md.root.clone(),
                md.book.clone(),
                md.config.clone(),
                dest.to_path_buf(),
            );

            mdbook_epub::generate(&ctx)?;

            let output_file = mdbook_epub::output_filename(&dest, &ctx.config);
            info!("Generated epub into {}", output_file.display());

            // :TODO: fill all version info

            let version = BookVersion {
                commit: "".to_string(),
                commit_date: "".to_string(),
                byte_count: 0,
                path: output_file,
            };

            versions.insert(version_folder.to_string(), version);
        }

        // :TODO: sort versions based on decreasing date

        let entry = ManifestEntry {
            name: folder.to_string(),
            versions,
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
