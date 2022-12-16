use std::{
    fmt::{self, Display},
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};

pub enum Key {
    Enter,
    Escape,
    Up,
    Down,
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
            Key::Char(char) => char.to_string(),
            Key::Unknown => String::from("unknown"),
        };
        write!(f, "Key({})", name)
    }
}

impl From<event::KeyEvent> for Key {
    fn from(key_event: KeyEvent) -> Key {
        match key_event {
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
                code: KeyCode::Char(c),
                ..
            } => Key::Char(c),
            _ => Key::Unknown,
        }
    }
}

pub enum InputEvent {
    Input(Key),
}

pub struct Events {
    rx: Receiver<InputEvent>,
    _tx: Sender<InputEvent>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone();
        thread::spawn(move || loop {
            if let Event::Key(key) = event::read().unwrap() {
                let key = Key::from(key);
                event_tx.send(InputEvent::Input(key)).unwrap();
            }
        });

        Events { rx, _tx: tx }
    }

    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }
}
