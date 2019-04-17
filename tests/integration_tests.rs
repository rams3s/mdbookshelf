use mdbookshelf;
use mdbookshelf::config::{BookRepoConfig, CollectionConfig, Config};
use std::path::Path;

#[test]
fn generate_epub_library() {
    let destination_dir = Some( "epubs".to_string() );
    let working_dir = Some( "repos".to_string() );

    // :TODO: use bookshelf.toml + templates dir

    let book_repo_configs = vec![BookRepoConfig {
        title: Some( "The book".to_string() ),
        repo_url: "https://github.com/rust-lang/book.git".to_string(),
        ..Default::default()
    }];

    let collections = vec![CollectionConfig {
        title: "The Rust Collection".to_string(),
        book_repo_configs
    }];

    let config = Config {
        destination_dir,
        working_dir,
        title: "My bookshelf".to_string(),
        collections,
        ..Default::default()
    };

    std::fs::remove_dir_all(config.destination_dir.as_ref().unwrap()).unwrap();
    std::fs::remove_dir_all(config.working_dir.as_ref().unwrap()).unwrap();

    let output_file = Path::new(config.destination_dir.as_ref().unwrap())
        .join("book.git")
        .join("The Rust Programming Language.epub");

    assert!(!output_file.exists());
    let manifest = mdbookshelf::run(config).unwrap();
    assert!(output_file.exists());
    assert_eq!(1, manifest.collections.len());

    let collection = &manifest.collections[0];
    assert_eq!(1, collection.entries.len());
    let entry = &collection.entries[0];
    assert_eq!("The Rust Programming Language", entry.title);
}
