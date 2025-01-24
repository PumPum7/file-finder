use ignore::Walk;
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
    if let Some(num_threads) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap();
    }

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
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(buffer_size, file);
    let mut matches = Vec::new();
    let mut line_buffer = Vec::with_capacity(context * 2 + 1);
    let mut line_num = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        line_num += 1;

        // Add the line to our circular buffer
        if line_buffer.len() >= context * 2 + 1 {
            line_buffer.remove(0);
        }
        line_buffer.push((line_num, line.clone()));

        // Check if the current line matches
        if content_regex.is_match(&line) {
            let context_start = line_buffer.len().saturating_sub(context + 1);
            let context_lines: Vec<_> = line_buffer[context_start..line_buffer.len()-1]
                .iter()
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