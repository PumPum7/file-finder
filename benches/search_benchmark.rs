use criterion::{black_box, criterion_group, criterion_main, Criterion};
use file_finder::finder::search_files;
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;
use regex::Regex;

fn create_test_file(dir: &tempfile::TempDir, size_kb: usize) -> PathBuf {
    let file_path = dir.path().join(format!("test_{}.txt", size_kb));
    let mut file = File::create(&file_path).unwrap();
    let content = "This is a test line that will be repeated many times to create a large file.\n".repeat(size_kb * 20);
    file.write_all(content.as_bytes()).unwrap();
    file_path
}

fn benchmark_search(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    
    // Create test files of different sizes
    let small_file = create_test_file(&temp_dir, 10);   // 10KB
    let medium_file = create_test_file(&temp_dir, 100); // 100KB
    let large_file = create_test_file(&temp_dir, 1000); // 1MB

    let mut group = c.benchmark_group("search_performance");

    // Test different file sizes
    for file in [&small_file, &medium_file, &large_file] {
        let size = file.metadata().unwrap().len() / 1024;
        group.bench_function(format!("search_{}_kb", size), |b| {
            b.iter(|| {
                search_files(
                    black_box(&file.to_path_buf()),
                    black_box(&Regex::new(".*").unwrap()),
                    black_box(&Regex::new("test").unwrap()),
                    black_box(2),
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