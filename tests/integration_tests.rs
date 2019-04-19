use mdbookshelf;
use mdbookshelf::config::Config;
use std::str::FromStr;

const CONFIG: &'static str = r#"
    title = "My eBookshelf"
    destination-dir = "tests/out"
    working-dir = "tests/repos"
    templates-dir = "tests/templates"

    [[book]]
    repo-url = "https://github.com/rams3s/mdbook-dummy.git"
    url = "https://rams3s.github.io/mdbook-dummy/index.html"
    "#;

#[test]
fn generate_epub_library() {
    let config = Config::from_str(CONFIG).unwrap();

    std::fs::remove_dir_all(config.destination_dir.as_ref().unwrap()).unwrap_or_default();
    std::fs::remove_dir_all(config.working_dir.as_ref().unwrap()).unwrap_or_default();

    let output_file = config
        .destination_dir
        .as_ref()
        .unwrap()
        .join("mdbook-dummy.git")
        .join("Hello Rust.epub");

    assert!(!output_file.exists());
    let manifest = mdbookshelf::run(&config).unwrap();
    assert!(output_file.exists());
    assert_eq!(1, manifest.entries.len());
    let entry = &manifest.entries[0];
    assert_eq!("Hello Rust", entry.title);
}
