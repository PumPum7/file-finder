use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crate::finder::{FileMatch, search_files};
use regex::Regex;
use std::path::PathBuf;

pub struct TuiApp {
    search_results: Vec<FileMatch>,
    selected_index: usize,
    search_path: PathBuf,
    name_pattern: String,
    content_pattern: String,
}

impl TuiApp {
    pub fn new(search_path: PathBuf, content_pattern: String, name_pattern: String) -> Self {
        Self {
            search_results: Vec::new(),
            selected_index: 0,
            search_path,
            name_pattern,
            content_pattern,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Perform initial search
        self.perform_search()?;

        let res = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res
    }

    fn run_app<B: tui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        if !self.search_results.is_empty() {
                            self.selected_index = (self.selected_index + 1) % self.search_results.len();
                        }
                    }
                    KeyCode::Up => {
                        if !self.search_results.is_empty() {
                            self.selected_index = self.selected_index.saturating_sub(1);
                        }
                    }
                    KeyCode::Enter => self.perform_search()?,
                    _ => {}
                }
            }
        }
    }

    fn perform_search(&mut self) -> io::Result<()> {
        let name_regex = Regex::new(&self.name_pattern).unwrap_or(Regex::new(".*").unwrap());
        let content_regex = Regex::new(&self.content_pattern).unwrap_or(Regex::new("").unwrap());
        
        self.search_results = search_files(
            &self.search_path,
            &name_regex,
            &content_regex,
            0,
            8192,
            None
        );
        Ok(())
    }

    fn ui<B: tui::backend::Backend>(&self, f: &mut tui::Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(f.size());

        // Results list
        let items: Vec<ListItem> = self
            .search_results
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = format!("{}: {}", m.path.display(), m.line);
                let style = if i == self.selected_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(vec![Spans::from(vec![Span::styled(content, style)])])
            })
            .collect();

        let results_list = List::new(items)
            .block(Block::default().title("Search Results").borders(Borders::ALL));

        f.render_widget(results_list, chunks[0]);

        // Preview panel
        let preview_content = if let Some(selected) = self.search_results.get(self.selected_index) {
            let mut content = vec![];
            content.push(Spans::from(format!("File: {}", selected.path.display())));
            content.push(Spans::from(""));
            content.push(Spans::from(vec![Span::styled(
                "Search Queries:",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            )]));
            content.push(Spans::from(format!("  Name pattern: {}", self.name_pattern)));
            content.push(Spans::from(format!("  Content pattern: {}", self.content_pattern)));
            content.push(Spans::from(""));
            
            // Display context lines before match
            let mut context_lines_iter = selected.context_lines.iter().peekable();
            while let Some((num, line)) = context_lines_iter.next() {
                if *num >= selected.line_num {
                    break;
                }
                content.push(Spans::from(vec![Span::styled(
                    format!("  {}: {}", num, line),
                    Style::default().fg(Color::DarkGray)
                )]));
            }
            
            // Display matched line with highlighting
            content.push(Spans::from(vec![Span::styled(
                format!("→ {}: {}", selected.line_num, selected.line),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            )]));
            
            // Display context lines after match
            for (num, line) in context_lines_iter {
                content.push(Spans::from(vec![Span::styled(
                    format!("  {}: {}", num, line),
                    Style::default().fg(Color::DarkGray)
                )]));
            }
            content
        } else {
            vec![Spans::from("No file selected")]
        };

        let preview = Paragraph::new(preview_content)
            .block(Block::default().title("Preview").borders(Borders::ALL));

        f.render_widget(preview, chunks[1]);
    }
}