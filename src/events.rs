use std::{
    fmt::{self, Display},
    sync::mpsc::{channel, Receiver, RecvError},
    thread,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};

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
    Resize,
}

pub struct Events {
    rx: Receiver<InputEvent>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone();
        thread::spawn(move || loop {
            if let Ok(event) = event::read() {
                match event {
                    Event::Key(key) => {
                        let key = Key::from(key);
                        event_tx.send(InputEvent::Input(key)).unwrap();
                    }
                    Event::Resize(_, _) => {
                        event_tx.send(InputEvent::Resize).unwrap();
                    }
                    _ => {}
                }
            }
        });

        Events { rx }
    }

    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }
}
