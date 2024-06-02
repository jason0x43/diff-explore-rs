use super::commits::DiffAction;

#[derive(Debug, Clone)]
struct ChunkInfo {
    old: u32,
    new: u32,
}

impl ChunkInfo {
    fn new(line: &str) -> ChunkInfo {
        let parts: Vec<&str> = line.split(' ').collect();
        let old: Vec<&str> = parts[1][1..].split(',').collect();
        let new: Vec<&str> = parts[2][1..].split(',').collect();
        let old_start: u32 = old[0].parse().unwrap();
        let new_start: u32 = new[0].parse().unwrap();
        ChunkInfo {
            old: old_start,
            new: new_start,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffLineNrs {
    pub old: u32,
    pub new: u32,
}

#[derive(Debug, Clone)]
pub enum DiffLine {
    Add(DiffLineNrs),
    Del(DiffLineNrs),
    Same(DiffLineNrs),
    Hunk,
    Start,
    None,
}

impl DiffLine {
    fn new_line(line: &str, old: u32, new: u32) -> DiffLine {
        match line.chars().nth(0) {
            Some('+') => DiffLine::Add(DiffLineNrs { old, new }),
            Some('-') => DiffLine::Del(DiffLineNrs { old, new }),
            _ => DiffLine::Same(DiffLineNrs { old, new }),
        }
    }

    fn new_meta(line: &str) -> DiffLine {
        match line.chars().nth(0) {
            Some('d') => DiffLine::Start,
            Some('@') => DiffLine::Hunk,
            _ => DiffLine::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    /// file path
    pub path: String,
    /// old path (if renamed)
    pub old_path: String,
    /// commit range for diff
    pub action: DiffAction,
    /// raw diff lines
    pub lines: Vec<String>,
    /// metadata about each line
    pub line_meta: Vec<DiffLine>,
}

impl FileDiff {
    pub fn new(text: &str, action: &DiffAction) -> FileDiff {
        let mut chunk_info: Option<ChunkInfo> = None;
        let mut path: &str = "";
        let mut old_path: &str = "";
        let lines: Vec<String> = text.lines().map(|s| s.into()).collect();
        let line_meta: Vec<DiffLine> = lines
            .iter()
            .map(|s| {
                if s.starts_with("diff ") {
                    chunk_info = None;
                    DiffLine::new_meta(s)
                } else if s.starts_with("@@") {
                    chunk_info = Some(ChunkInfo::new(s));
                    DiffLine::new_meta(s)
                } else if let Some(info) = &mut chunk_info {
                    let old = info.old;
                    let new = info.new;
                    match s.chars().nth(0) {
                        Some('+') => info.new += 1,
                        Some('-') => info.old += 1,
                        _ => {
                            info.new += 1;
                            info.old += 1;
                        }
                    }
                    DiffLine::new_line(s, old, new)
                } else if s.starts_with("---") {
                    old_path = s[4..].into();
                    DiffLine::new_meta(s)
                } else if s.starts_with("+++") {
                    path = s[4..].into();
                    DiffLine::new_meta(s)
                } else {
                    DiffLine::new_meta(s)
                }
            })
            .collect();

        FileDiff {
            path: path.into(),
            old_path: old_path.into(),
            action: action.clone(),
            lines,
            line_meta,
        }
    }
}

