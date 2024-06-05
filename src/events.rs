use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use notify::{
    event::ModifyKind, recommended_watcher, EventKind, RecommendedWatcher,
    RecursiveMode, Watcher,
};

use crate::error::AppError;

#[derive(Debug)]
pub enum Key {
    Enter,
    Escape,
    Backspace,
    Up,
    Down,
    Ctrl(char),
    Char(char),
    Unknown,
}

impl Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: String = match self {
            Key::Enter => "Enter".into(),
            Key::Escape => "Escape".into(),
            Key::Backspace => "Backspace".into(),
            Key::Up => "Up".into(),
            Key::Down => "Down".into(),
            Key::Char(char) => char.to_string(),
            Key::Ctrl(char) => format!("Ctrl+{}", char),
            Key::Unknown => "unknown".into(),
        };
        write!(f, "Key({})", name)
    }
}

impl From<event::KeyEvent> for Key {
    fn from(key_event: KeyEvent) -> Key {
        match key_event {
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => Key::Ctrl(c),

            KeyEvent {
                code: KeyCode::Esc, ..
            } => Key::Escape,

            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => Key::Backspace,

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Key::Enter,

            KeyEvent {
                code: KeyCode::Up, ..
            } => Key::Up,

            KeyEvent {
                code: KeyCode::Down,
                ..
            } => Key::Down,

            KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => Key::Char(c),

            _ => Key::Unknown,
        }
    }
}

pub enum AppEvent {
    Input(Key),
    FilesChanged(Vec<PathBuf>),
    Resize,
}

pub struct Events {
    rx: Receiver<AppEvent>,
    tx: Sender<AppEvent>,
    watcher: RecommendedWatcher,
}

impl Events {
    pub fn new() -> Result<Events, AppError> {
        let (tx, rx) = mpsc::channel();

        let watch_tx = tx.clone();
        let watcher = recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let evt = event.clone();
                    if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
                        tracing::debug!("files changed: {:?}", evt.paths);
                        watch_tx
                            .send(AppEvent::FilesChanged(evt.paths))
                            .err()
                            .map(|err| {
                                tracing::error!(
                                    "Error sending files changed event: {:?}",
                                    err
                                );
                            });
                    }
                }
            },
        )?;

        Ok(Events { rx, tx, watcher })
    }

    pub fn start(&mut self) {
        let input_tx = self.tx.clone();
        thread::spawn(move || loop {
            if let Ok(event) = event::read() {
                match event {
                    Event::Key(key) => {
                        input_tx.send(AppEvent::Input(Key::from(key))).unwrap();
                    }

                    Event::Resize(_, _) => {
                        input_tx.send(AppEvent::Resize).unwrap();
                    }

                    _ => {}
                }
            }
        });
    }

    pub fn watch_file(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)
    }

    pub fn unwatch_file(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.unwatch(path)
    }

    pub fn next(&self) -> Result<AppEvent, mpsc::RecvError> {
        self.rx.recv()
    }
}
