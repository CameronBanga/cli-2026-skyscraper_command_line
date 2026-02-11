use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::action::Action;

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        if tx.send(evt).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        EventHandler { rx, _task: task }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

pub fn key_to_action(key: KeyEvent, in_composer: bool, in_login: bool) -> Option<Action> {
    if in_composer || in_login {
        return None;
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
        (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Action::Quit),
        (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
            Some(Action::SelectNext)
        }
        (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
            Some(Action::SelectPrev)
        }
        (KeyModifiers::NONE, KeyCode::Enter) => Some(Action::OpenThread),
        (KeyModifiers::NONE, KeyCode::Esc) => Some(Action::GoBack),
        (KeyModifiers::NONE, KeyCode::Char('n')) => Some(Action::OpenComposer {
            reply_to: None,
            reply_to_author: None,
        }),
        (KeyModifiers::NONE, KeyCode::Char('r')) => {
            // Reply - actual reply_to will be filled by the app
            Some(Action::OpenComposer {
                reply_to: None,
                reply_to_author: None,
            })
        }
        (KeyModifiers::NONE, KeyCode::Char('l')) => Some(Action::ToggleLike),
        (KeyModifiers::NONE, KeyCode::Char('t')) => Some(Action::ToggleRepost),
        (KeyModifiers::NONE, KeyCode::Char('u')) => Some(Action::ViewAuthorProfile),
        (KeyModifiers::SHIFT, KeyCode::Char('R')) => Some(Action::RefreshTimeline),
        (KeyModifiers::NONE, KeyCode::Char('g')) => Some(Action::ScrollToTop),
        (KeyModifiers::SHIFT, KeyCode::Char('G')) => Some(Action::ScrollToBottom),
        (KeyModifiers::NONE, KeyCode::Char('1')) => Some(Action::SwitchTab(0)),
        (KeyModifiers::NONE, KeyCode::Char('2')) => Some(Action::SwitchTab(1)),
        _ => None,
    }
}
