use mdbookshelf;
use mdbookshelf::config::{BookRepoConfig, Config};

#[test]
fn generate_epub_library() {
    let destination_dir = Some("epubs".into());
    let working_dir = Some("repos".into());
    let templates_dir = Some("templates".into());

    // :TODO: use bookshelf.toml + templates dir

    let book_repo_configs = vec![BookRepoConfig {
        repo_url: "https://github.com/rust-lang/book.git".to_string(),
        ..Default::default()
    }];

    let config = Config {
        destination_dir,
        working_dir,
        title: "My bookshelf".to_string(),
        book_repo_configs,
        templates_dir,
    };

    std::fs::remove_dir_all(&config.destination_dir).unwrap_or_default();
    std::fs::remove_dir_all(&config.working_dir).unwrap_or_default();

    let output_file = config
        .destination_dir
        .as_ref()
        .unwrap()
        .join("book.git")
        .join("The Rust Programming Language.epub");

    assert!(!output_file.exists());
    let manifest = mdbookshelf::run(&config).unwrap();
    assert!(output_file.exists());
    assert_eq!(1, manifest.entries.len());
    let entry = &manifest.entries[0];
    assert_eq!("The Rust Programming Language", entry.title);
}
