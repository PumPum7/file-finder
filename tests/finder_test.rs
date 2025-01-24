use assert_fs::prelude::*;
use file_finder::finder::search_files;
use std::fs::File;
use std::io::Write;
use test_case::test_case;
use regex::Regex;

#[test]
fn test_basic_search() {
    let temp = assert_fs::TempDir::new().unwrap();
    let test_file = temp.child("test.txt");
    test_file.write_str("Hello World\nThis is a test\nHello again").unwrap();

    let name_pattern = Regex::new(".*").unwrap();
    let content_pattern = Regex::new("Hello").unwrap();
    let results = search_files(
        &test_file.path().to_path_buf(),
        &name_pattern,
        &content_pattern,
        0,
        8192,
        None
    );
    assert_eq!(results.len(), 2);
}

#[test_case("Hello", 0, 2 ; "basic match count")]
#[test_case("nonexistent", 0, 0 ; "no matches")]
#[test_case("test", 1, 1 ; "with context lines")]
fn test_search_variations(pattern: &str, context_lines: usize, expected_matches: usize) {
    let temp = assert_fs::TempDir::new().unwrap();
    let test_file = temp.child("test.txt");
    test_file.write_str("Hello World\nThis is a test\nHello again").unwrap();

    let name_pattern = Regex::new(".*").unwrap();
    let content_pattern = Regex::new(pattern).unwrap();
    let results = search_files(
        &test_file.path().to_path_buf(),
        &name_pattern,
        &content_pattern,
        context_lines,
        8192,
        None
    );
    assert_eq!(results.len(), expected_matches);
}

#[test]
fn test_large_file_search() {
    let temp = assert_fs::TempDir::new().unwrap();
    let test_file = temp.child("large.txt");
    let mut file = File::create(test_file.path()).unwrap();
    
    // Create a 1MB file
    let line = "This is a test line that will be repeated many times.\n".repeat(20000);
    file.write_all(line.as_bytes()).unwrap();

    let name_pattern = Regex::new(".*").unwrap();
    let content_pattern = Regex::new("test").unwrap();
    let results = search_files(
        &test_file.path().to_path_buf(),
        &name_pattern,
        &content_pattern,
        0,
        8192,
        None
    );
    assert!(results.len() > 0);
}

#[test]
fn test_case_sensitive_search() {
    let temp = assert_fs::TempDir::new().unwrap();
    let test_file = temp.child("case.txt");
    test_file.write_str("Hello World\nHELLO WORLD").unwrap();

    let name_pattern = Regex::new(".*").unwrap();
    let content_pattern = Regex::new("Hello").unwrap();
    let results = search_files(
        &test_file.path().to_path_buf(),
        &name_pattern,
        &content_pattern,
        0,
        8192,
        None
    );
    assert_eq!(results.len(), 1);
}