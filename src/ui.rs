use crate::app::App;
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    text::Spans,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
    let commits = &app.commits;
    let text: Vec<Spans> =
        commits.iter().map(|s| Spans::from(s.clone())).collect();
    let block = Block::default().borders(Borders::ALL).title("Commits");
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, f.size());
}
