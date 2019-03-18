use mdbook_library;
use mdbook_library::Config;
use std::path::Path;

#[test]
fn generate_epub_library() {
    let destination_dir = "epubs";
    let working_dir = "repos";

    let repo_urls = vec!["https://github.com/rust-lang/book.git"];

    let config = Config {
        destination_dir,
        working_dir,
        repo_urls,
    };

    std::fs::remove_dir_all(destination_dir).unwrap_or_default();
    std::fs::remove_dir_all(working_dir).unwrap_or_default();

    let output_file = Path::new(destination_dir)
        .join("book.git")
        .join("The Rust Programming Language.epub");

    assert!(!output_file.exists());
    let manifest = mdbook_library::run(config).unwrap();
    assert!(output_file.exists());
    assert_eq!(1, manifest.entries.len());

    // :TODO: check second call updates files and uses fetch

    let repo_urls = vec!["https://github.com/rust-lang/book.git"];
    let config = Config {
        destination_dir,
        working_dir,
        repo_urls,
    };

    mdbook_library::run(config).unwrap();
}
