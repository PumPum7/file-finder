use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Fast file finder with content preview")]
pub struct Args {
    /// Root directory to search
    pub root: PathBuf,

    /// Filename regex pattern
    #[arg(short = 'n', long)]
    pub name: String,

    /// Content regex pattern
    #[arg(short = 'c', long)]
    pub content: String,

    /// Context lines around matches
    #[arg(short = 'C', long, default_value = "1")]
    pub context: usize,

    /// Number of parallel workers (default: number of CPU cores)
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Buffer size for reading files (in bytes)
    #[arg(short = 'b', long, default_value = "8192")]
    pub buffer_size: usize,
}