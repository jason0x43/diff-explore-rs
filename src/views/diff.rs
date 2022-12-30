use std::{cmp::{max, min}, path::PathBuf};

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

    pub fn path(&self) -> PathBuf {
        self.stat.path()
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
        let delta = min(self.height - 1, self.offset);
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
            let delta = min(limit, self.height - 1);
            self.offset += delta;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.offset = self.diff.lines.len() - self.height;
    }

    pub fn scroll_to_top(&mut self) {
        self.offset = 0;
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
    tab_width: u8,
}

pub struct DiffViewOpts {
    pub tab_width: u8,
}

impl<'a> DiffView<'a> {
    pub fn new(diff: &'a mut Diff, options: Option<DiffViewOpts>) -> DiffView {
        DiffView {
            diff,
            tab_width: match options {
                Some(opts) => opts.tab_width,
                _ => 4,
            },
        }
    }
}

struct LineRenderer {
    line_nr_width: usize,
    tab_width: usize,
}

impl LineRenderer {
    fn new(line_nr_width: usize, tab_width: usize) -> LineRenderer {
        LineRenderer {
            line_nr_width,
            tab_width,
        }
    }

    fn render(
        &self,
        old_color: u8,
        new_color: u8,
        line_color: u8,
        old: u32,
        new: u32,
        line: &str,
    ) -> Vec<Span> {
        [
            Span::styled(
                format!("{:>width$}", old, width = self.line_nr_width),
                Style::default().fg(Color::Indexed(old_color)),
            ),
            Span::from(" "),
            Span::styled(
                format!("{:>width$}", new, width = self.line_nr_width),
                Style::default().fg(Color::Indexed(new_color)),
            ),
            Span::from(" "),
            Span::styled(
                String::from(
                    &line[1..].replace('\t', &" ".repeat(self.tab_width)),
                ),
                Style::default().fg(Color::Indexed(line_color)),
            ),
        ]
        .into()
    }
}

impl<'a> Widget for DiffView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let diff = self.diff;
        diff.height = area.height as usize;

        let line_nr_width = match diff.diff.line_meta.iter().last() {
            Some(DiffLine::Add(meta))
            | Some(DiffLine::Del(meta))
            | Some(DiffLine::Same(meta)) => {
                max(meta.old.to_string().len(), meta.new.to_string().len())
            }
            _ => 0,
        } as usize;
        let renderer =
            LineRenderer::new(line_nr_width, self.tab_width as usize);

        let lines: Vec<Spans> =
            diff.diff
                .lines
                .iter()
                .enumerate()
                .map(|(line_nr, line)| {
                    if line.len() > 0 {
                        Spans::from(match &diff.diff.line_meta[line_nr] {
                            DiffLine::Add(meta) => renderer
                                .render(16, 7, 2, meta.old, meta.new, line),
                            DiffLine::Del(meta) => renderer
                                .render(7, 16, 1, meta.old, meta.new, line),
                            DiffLine::Same(meta) => renderer
                                .render(7, 7, 17, meta.old, meta.new, line),
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
