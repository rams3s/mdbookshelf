use mdbookshelf;
use mdbookshelf::config::{BookRepoConfig, Config};
use std::path::Path;

#[test]
fn generate_epub_library() {
    let destination_dir = "epubs".to_string();
    let working_dir = "repos".to_string();

    let book_repo_configs = vec![BookRepoConfig {
        title: "The book".to_string(),
        repo_url: "https://github.com/rust-lang/book.git".to_string(),
        ..Default::default()
    }];

    let config = Config {
        destination_dir,
        working_dir,
        title: "My bookshelf".to_string(),
        book_repo_configs,
    };

    std::fs::remove_dir_all(&config.destination_dir).unwrap_or_default();
    std::fs::remove_dir_all(&config.working_dir).unwrap_or_default();

    let output_file = Path::new(&config.destination_dir)
        .join("book.git")
        .join("The Rust Programming Language.epub");

    assert!(!output_file.exists());
    let manifest = mdbookshelf::run(config).unwrap();
    assert!(output_file.exists());
    assert_eq!(1, manifest.entries.len());

    // :TODO: check second call updates files and uses fetch

    let book_repo_configs = vec![BookRepoConfig {
        title: "The book".to_string(),
        repo_url: "https://github.com/rust-lang/book.git".to_string(),
        ..Default::default()
    }];

    let destination_dir = "epubs".to_string();
    let working_dir = "repos".to_string();

    let config = Config {
        destination_dir,
        working_dir,
        title: "My bookshelf".to_string(),
        book_repo_configs,
    };

    mdbookshelf::run(config).unwrap();
}
