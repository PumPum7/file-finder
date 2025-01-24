use criterion::{black_box, criterion_group, criterion_main, Criterion};
use file_finder::finder::search_files;
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;
use regex::Regex;
use std::fs;

fn create_test_file(dir: &PathBuf, size_kb: usize, name: &str) -> PathBuf {
    let file_path = dir.join(name);
    let mut file = File::create(&file_path).unwrap();
    let content = "This is a test line that will be repeated many times to create a large file.\n".repeat(size_kb * 20);
    file.write_all(content.as_bytes()).unwrap();
    file_path
}

fn create_nested_structure(root: &PathBuf, depth: usize, files_per_level: usize, size_kb: usize) {
    if depth == 0 { return; }
    
    // Create files at current level
    for i in 0..files_per_level {
        create_test_file(root, size_kb, &format!("test_{}.txt", i));
    }
    
    // Create subdirectories and recurse
    for i in 0..3 {
        let subdir = root.join(format!("level_{}", i));
        fs::create_dir_all(&subdir).unwrap();
        create_nested_structure(&subdir, depth - 1, files_per_level, size_kb);
    }
}

fn benchmark_search(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path().to_path_buf();
    
    // Test configurations
    let configs = [
        ("shallow", 2, 5),    // 2 levels deep, 5 files per level
        ("medium", 4, 3),     // 4 levels deep, 3 files per level
        ("deep", 6, 2),       // 6 levels deep, 2 files per level
    ];
    
    let mut group = c.benchmark_group("directory_traversal");
    
    for (name, depth, files_per_level) in configs {
        // Create a fresh directory structure for each test
        let test_dir = root.join(name);
        fs::create_dir_all(&test_dir).unwrap();
        create_nested_structure(&test_dir, depth, files_per_level, 10); // 10KB files
        
        group.bench_function(format!("search_{}_structure", name), |b| {
            b.iter(|| {
                search_files(
                    black_box(&test_dir),
                    black_box(&Regex::new(".*").unwrap()),
                    black_box(&Regex::new("test").unwrap()),
                    black_box(0),
                    black_box(8192),
                    black_box(None)
                )
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);