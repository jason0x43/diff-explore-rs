use std::{cmp::max, path::PathBuf};

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};

use crate::{
    git::{git_diff_file, CommitRange, DiffFile, DiffLine, Stat},
    list::{ListInfo, ListScroll},
    search::Search,
    views::statusline::Status, ui::highlight_spans,
};

#[derive(Debug, Clone)]
pub struct Diff {
    height: usize,
    offset: usize,
    diff: DiffFile,
    range: CommitRange,
    stat: Stat,
    search: Option<String>,
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
            search: None,
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
}

impl ListInfo for Diff {
    fn list_count(&self) -> usize {
        self.diff.lines.len()
    }

    fn list_pos(&self) -> usize {
        self.offset
    }

    fn set_list_pos(&mut self, pos: usize) {
        self.offset = pos;
    }
}

impl ListScroll for Diff {
    fn height(&self) -> usize {
        self.height
    }
}

impl Status for Diff {
    fn status(&self) -> String {
        format!("{}: {}", self.range, self.stat.path)
    }
}

impl Search for Diff {
    fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    fn get_search(&self) -> Option<String> {
        self.search.clone()
    }

    fn is_match(&self, idx: usize) -> bool {
        match &self.search {
            Some(search) => self.diff.lines[idx].contains(search),
            _ => false,
        }
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
    search: Option<String>,
}

impl LineRenderer {
    fn new(
        line_nr_width: usize,
        tab_width: usize,
        search: Option<String>,
    ) -> LineRenderer {
        LineRenderer {
            line_nr_width,
            tab_width,
            search,
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
        let mut spans: Vec<Span> = vec![
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
        ];

        let search = if self.search.is_some()
            && self.search.clone().unwrap().len() > 0
        {
            self.search.clone()
        } else {
            None
        };

        let line =
            String::from(&line[1..].replace('\t', &" ".repeat(self.tab_width)));

        spans.push(Span::styled(
            line,
            Style::default().fg(Color::Indexed(line_color)),
        ));

        if let Some(search) = search {
            spans = highlight_spans(
                spans,
                &search,
                Style::default().add_modifier(Modifier::REVERSED),
            );
        }

        spans
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
        let search = diff.search.clone();
        let renderer =
            LineRenderer::new(line_nr_width, self.tab_width as usize, search);

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
