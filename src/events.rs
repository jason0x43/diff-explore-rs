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

#[derive(Debug)]
pub enum Key {
    Enter,
    Escape,
    Up,
    Down,
    Space,
    Ctrl(char),
    Char(char),
    Unknown,
}

impl Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Key::Enter => String::from("Enter"),
            Key::Escape => String::from("Escape"),
            Key::Up => String::from("Up"),
            Key::Down => String::from("Down"),
            Key::Space => String::from("Space"),
            Key::Char(char) => char.to_string(),
            Key::Ctrl(char) => format!("Ctrl+{}", char),
            Key::Unknown => String::from("unknown"),
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
                code: KeyCode::Char(' '),
                ..
            } => Key::Space,
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
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();

        let watch_tx = tx.clone();
        let watcher = recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| match res {
                Ok(event) => {
                    let evt = event.clone();
                    match event.kind {
                        EventKind::Modify(mod_kind) => match mod_kind {
                            ModifyKind::Data(_) => {
                                watch_tx
                                    .send(AppEvent::FilesChanged(evt.paths))
                                    .unwrap();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                _ => {}
            },
        )
        .unwrap();

        Events {
            rx,
            tx,
            watcher,
        }
    }

    pub fn start(&mut self) {
        let input_tx = self.tx.clone();
        thread::spawn(move || loop {
            if let Ok(event) = event::read() {
                match event {
                    Event::Key(key) => {
                        let key = Key::from(key);
                        input_tx.send(AppEvent::Input(key)).unwrap();
                    }
                    Event::Resize(_, _) => {
                        input_tx.send(AppEvent::Resize).unwrap();
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn watch_file(&mut self, path: &Path) {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .unwrap();
    }

    pub fn unwatch_file(&mut self, path: &Path) {
        self.watcher.unwatch(path).unwrap();
    }

    pub fn next(&self) -> Result<AppEvent, mpsc::RecvError> {
        self.rx.recv()
    }
}
