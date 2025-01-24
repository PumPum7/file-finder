use ignore::Walk;
use memmap2::Mmap;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

pub struct FileMatch {
    pub path: PathBuf,
    pub line_num: usize,
    pub line: String,
    pub context_lines: Vec<(usize, String)>,
}

pub fn search_files(root: &PathBuf, name_regex: &Regex, content_regex: &Regex, context: usize, buffer_size: usize, jobs: Option<usize>) -> Vec<FileMatch> {
    // Configure thread pool based on available CPU cores if not specified
    let num_threads = jobs.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    });
    
    // Initialize thread pool with optimal configuration
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .stack_size(8 * 1024 * 1024) // 8MB stack size
        .build_global();

    Walk::new(root)
        .par_bridge()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
        .filter(|entry| {
            name_regex.is_match(
                entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            )
        })
        .filter_map(|entry| {
            let path = entry.path().to_owned();
            search_file(&path, content_regex, context, buffer_size).ok()
        })
        .flatten()
        .collect()
}

fn search_file(path: &PathBuf, content_regex: &Regex, context: usize, buffer_size: usize) -> io::Result<Vec<FileMatch>> {
    const LARGE_FILE_THRESHOLD: u64 = 10 * 1024 * 1024; // 10MB
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();

    if file_size > LARGE_FILE_THRESHOLD {
        // Use memory mapping for large files
        let mmap = unsafe { Mmap::map(&file)? };
        return search_mmap(&mmap, path, content_regex, context);
    }

    // Use buffered reading for smaller files with optimized circular buffer
    let reader = BufReader::with_capacity(buffer_size, file);
    let mut matches = Vec::new();
    let mut line_buffer = CircularBuffer::new(context * 2 + 1);
    let mut line_num = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        line_num += 1;
        line_buffer.push((line_num, line.clone()));

        if content_regex.is_match(&line) {
            let context_lines = line_buffer
                .iter()
                .take(line_buffer.len().saturating_sub(1))
                .map(|(num, text)| (*num, text.clone()))
                .collect();

            matches.push(FileMatch {
                path: path.clone(),
                line_num,
                line,
                context_lines,
            });
        }
    }

    Ok(matches)
}

fn search_mmap(mmap: &Mmap, path: &PathBuf, content_regex: &Regex, context: usize) -> io::Result<Vec<FileMatch>> {
    let mut matches = Vec::with_capacity(1024);
    let mut line_buffer = CircularBuffer::new(context * 2 + 1);
    let mut line_num = 0;
    let mut start = 0;
    let mut line_start = 0;
    let data = mmap.as_ref();

    // Pre-allocate string buffer to reduce allocations
    let mut line_string = String::with_capacity(256);

    while start < data.len() {
        if start < data.len() && data[start] == b'\n' {
            unsafe {
                line_string.clear();
                line_string.push_str(std::str::from_utf8_unchecked(&data[line_start..start]));
            }
            line_num += 1;
            line_buffer.push((line_num, line_string.clone()));

            if content_regex.is_match(&line_string) {
                let context_lines = line_buffer
                    .iter()
                    .take(line_buffer.len().saturating_sub(1))
                    .map(|(num, text)| (*num, text.clone()))
                    .collect();

                matches.push(FileMatch {
                    path: path.clone(),
                    line_num,
                    line: line_string.clone(),
                    context_lines,
                });
            }
            start += 1;
            line_start = start;
        } else {
            start += 1;
        }
    }

    // Handle the last line if it doesn't end with a newline
    if line_start < data.len() {
        unsafe {
            line_string.clear();
            line_string.push_str(std::str::from_utf8_unchecked(&data[line_start..]));
        }
        line_num += 1;
        line_buffer.push((line_num, line_string.clone()));

        if content_regex.is_match(&line_string) {
            let context_lines = line_buffer
                .iter()
                .take(line_buffer.len().saturating_sub(1))
                .map(|(num, text)| (*num, text.clone()))
                .collect();

            matches.push(FileMatch {
                path: path.clone(),
                line_num,
                line: line_string,
                context_lines,
            });
        }
    }

    Ok(matches)
}

struct CircularBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    start: usize,
    size: usize,
}

impl<T> CircularBuffer<T> {
    #[inline(always)]
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.reserve_exact(capacity);
        CircularBuffer {
            buffer,
            capacity,
            start: 0,
            size: 0,
        }
    }

    #[inline(always)]
    fn push(&mut self, item: T) {
        if self.size < self.capacity {
            self.buffer.push(item);
            self.size += 1;
        } else {
            self.buffer[self.start] = item;
            self.start = (self.start + 1) % self.capacity;
        }
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = &T> {
        let (right, left) = self.buffer.split_at(self.start);
        left.iter().chain(right.iter()).take(self.size)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.size
    }
}