use std::{
    cmp::min,
    path::{Path, PathBuf},
};

use list_helper_core::ListInfo;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};

use crate::{
    git::{git_diff_file, CommitRange, DiffFile, DiffLine, Stat},
    views::statusline::Status,
};

#[derive(Debug, Clone)]
pub struct Diff {
    height: usize,
    offset: usize,
    diff: DiffFile,
    range: CommitRange,
    stat: Stat,
}

impl Diff {
    pub fn new(stat: &Stat, range: &CommitRange) -> Diff {
        let diff = git_diff_file(&stat.path, &stat.old_path, &range, None);

        Diff {
            diff,
            height: 0,
            offset: 0,
            stat: stat.clone(),
            range: range.clone(),
        }
    }

    /// Return true if the file represented by this diff is in the given list of
    /// paths
    pub fn is_in_list(&self, paths: &Vec<PathBuf>) -> bool {
        let buf = Path::new(&self.stat.path).canonicalize().unwrap();
        paths.contains(&buf)
    }

    /// Re-diff the file; call this when the underlying file may have changed
    pub fn refresh(&mut self) {
        self.diff = git_diff_file(
            &self.stat.path,
            &self.stat.old_path,
            &self.range,
            None,
        );
    }

    pub fn scroll_up(&mut self) {
        let delta = min(1, self.offset);
        self.offset -= delta;
    }

    pub fn page_up(&mut self) {
        let delta = min(self.height, self.offset);
        self.offset -= delta;
    }

    pub fn scroll_down(&mut self) {
        if self.diff.lines.len() - self.offset > self.height {
            let limit = self.diff.lines.len() - self.offset - self.height;
            let delta = min(limit, 1);
            self.offset += delta;
        }
    }

    pub fn page_down(&mut self) {
        if self.diff.lines.len() - self.offset > self.height {
            let limit = self.diff.lines.len() - self.offset - self.height;
            let delta = min(limit, self.height);
            self.offset += delta;
        }
    }
}

impl ListInfo for Diff {
    fn list_count(&self) -> usize {
        self.diff.lines.len()
    }

    fn list_pos(&self) -> usize {
        min(self.offset + self.height, self.diff.lines.len())
    }
}

impl Status for Diff {
    fn status(&self) -> String {
        format!("{}: {}", self.range, self.stat.path)
    }
}

/// The Widget used to render a Diff
pub struct DiffView<'a> {
    diff: &'a mut Diff,
}

impl<'a> DiffView<'a> {
    pub fn new(diff: &'a mut Diff) -> DiffView {
        DiffView { diff }
    }
}

fn line_spans<'a>(
    old_color: u8,
    new_color: u8,
    line_color: u8,
    old: u32,
    new: u32,
    line: &str,
) -> Vec<Span<'a>> {
    [
        Span::styled(
            old.to_string(),
            Style::default().fg(Color::Indexed(old_color)),
        ),
        Span::from(" "),
        Span::styled(
            new.to_string(),
            Style::default().fg(Color::Indexed(new_color)),
        ),
        Span::from(" "),
        Span::styled(
            String::from(&line[1..]),
            Style::default().fg(Color::Indexed(line_color)),
        ),
    ]
    .into()
}

impl<'a> Widget for DiffView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let diff = self.diff;
        diff.height = area.height as usize;

        let lines: Vec<Spans> = diff
            .diff
            .lines
            .iter()
            .enumerate()
            .map(|(line_nr, line)| {
                if line.len() > 0 {
                    Spans::from(match &diff.diff.line_meta[line_nr] {
                        DiffLine::Add(meta) => {
                            line_spans(8, 7, 2, meta.old, meta.new, line)
                        }
                        DiffLine::Del(meta) => {
                            line_spans(7, 8, 1, meta.old, meta.new, line)
                        }
                        DiffLine::Same(meta) => {
                            line_spans(7, 7, 17, meta.old, meta.new, line)
                        }
                        DiffLine::Start => [Span::styled(
                            line.clone(),
                            Style::default().fg(Color::Indexed(3)),
                        )]
                        .into(),
                        DiffLine::Hunk => [Span::styled(
                            line.clone(),
                            Style::default().fg(Color::Indexed(6)),
                        )]
                        .into(),
                        _ => [Span::from(line.clone())].into(),
                    })
                } else {
                    Spans::from(vec![Span::from("")])
                }
            })
            .collect();

        let view = Paragraph::new(lines).scroll((diff.offset as u16, 0));
        Widget::render(view, area, buf);
    }
}
