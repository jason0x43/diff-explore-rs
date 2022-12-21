use std::{
    fmt::{self, Display},
    path::Path,
    sync::mpsc::{self, Receiver},
    thread,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::git::git_root;

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

pub enum InputEvent {
    Input(Key),
    FileChange(notify::Event),
    Resize,
}

pub struct Events {
    rx: Receiver<InputEvent>,
    _watcher: Box<dyn Watcher>,
}

impl Events {
    pub fn new() -> Events {
        let (sender, receiver) = mpsc::channel();

        let input_sender = sender.clone();
        thread::spawn(move || loop {
            if let Ok(event) = event::read() {
                match event {
                    Event::Key(key) => {
                        let key = Key::from(key);
                        input_sender.send(InputEvent::Input(key)).unwrap();
                    }
                    Event::Resize(_, _) => {
                        input_sender.send(InputEvent::Resize).unwrap();
                    }
                    _ => {}
                }
            }
        });

        let watch_sender = sender.clone();
        let mut watcher: Box<dyn Watcher> = Box::new(
            RecommendedWatcher::new(
                move |res: Result<notify::Event, notify::Error>| match res {
                    Ok(event) => {
                        watch_sender
                            .send(InputEvent::FileChange(event))
                            .unwrap();
                    }
                    _ => {}
                },
                notify::Config::default(),
            )
            .unwrap(),
        );

        match git_root() {
            Ok(path) => {
                watcher
                    .watch(Path::new(&path), RecursiveMode::Recursive)
                    .unwrap();
            }
            _ => {}
        }

        Events {
            rx: receiver,
            _watcher: watcher,
        }
    }

    pub fn next(&self) -> Result<InputEvent, mpsc::RecvError> {
        self.rx.recv()
    }
}
