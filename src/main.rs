mod cli;
mod display;
mod finder;
mod tui;

use cli::Args;
use clap::Parser;
use display::Display;
use regex::Regex;
use std::io;

fn main() -> io::Result<()> {
    let args = Args::parse();
    
    if args.tui {
        println!("Using gui");
        let mut app = tui::TuiApp::new(args.root, args.content , args.name);
        return app.run();
    }

    let name_regex = Regex::new(&args.name).expect("Invalid filename pattern");
    let content_regex = Regex::new(&args.content).expect("Invalid content pattern");

    let matches = finder::search_files(
        &args.root,
        &name_regex,
        &content_regex,
        args.context,
        args.buffer_size,
        args.jobs,
    );

    let mut display = Display::new();
    display.print_matches(&matches)?;

    Ok(())
}
