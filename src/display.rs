use std::io::{self, Write};
use std::path::PathBuf;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::finder::FileMatch;

pub struct Display {
    stdout: StandardStream,
}

impl Display {
    pub fn new() -> Self {
        Self {
            stdout: StandardStream::stdout(ColorChoice::Auto),
        }
    }

    pub fn print_matches(&mut self, matches: &[FileMatch]) -> io::Result<()> {
        for file_match in matches {
            self.print_match(file_match)?;
        }
        Ok(())
    }

    fn print_match(&mut self, file_match: &FileMatch) -> io::Result<()> {
        // Print context lines before the match
        for (line_num, line) in &file_match.context_lines {
            self.print_line(&file_match.path, *line_num, line)?;
        }

        // Print the matching line with highlighting
        self.print_line(&file_match.path, file_match.line_num, &file_match.line)?;
        writeln!(self.stdout)?;

        Ok(())
    }

    fn print_line(&mut self, path: &PathBuf, line_num: usize, line: &str) -> io::Result<()> {
        let mut color_spec = ColorSpec::new();
        
        self.stdout.set_color(color_spec.set_fg(Some(termcolor::Color::Green)).set_bold(true))?;
        write!(self.stdout, "{}:", path.display())?;
        
        self.stdout.set_color(color_spec.set_fg(Some(termcolor::Color::Blue)).set_bold(true))?;
        write!(self.stdout, "{}:", line_num)?;
        
        self.stdout.reset()?;
        writeln!(self.stdout, " {}", line)?;
        
        Ok(())
    }
}