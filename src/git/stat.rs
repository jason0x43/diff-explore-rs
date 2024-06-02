use std::path::{PathBuf, Path};

#[derive(Debug, Clone)]
pub struct Stat {
    /// Number of added lines
    pub adds: u32,
    /// Number of deleted lines
    pub deletes: u32,
    /// Path of the modified file
    pub path: String,
    /// Original path of the modified file (if renamed)
    pub old_path: String,
}

impl Stat {
    pub fn new(stat_line: &str) -> Stat {
        let parts: Vec<&str> = stat_line.split('\t').collect();
        let adds: u32 = parts[0].parse().unwrap();
        let deletes: u32 = parts[1].parse().unwrap();
        let (path, old_path) = if parts[2].contains(" => ") {
            let path_parts: Vec<&str> = parts[2].split(" => ").collect();
            (path_parts[0].into(), path_parts[1].into())
        } else {
            (parts[2].into(), "".into())
        };

        Stat {
            adds,
            deletes,
            path,
            old_path,
        }
    }

    pub fn path(&self) -> Result<PathBuf, std::io::Error> {
        Path::new(&self.path).canonicalize()
    }
}
