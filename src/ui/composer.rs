use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use tui_textarea::TextArea;

use crate::action::Action;
use crate::api::client::ReplyRef;
use crate::ui::Component;

const MAX_CHARS: usize = 300;

pub struct Composer {
    textarea: TextArea<'static>,
    reply_to: Option<ReplyRef>,
    reply_to_author: Option<String>,
}

impl Composer {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Compose "),
        );
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("What's on your mind?");

        Composer {
            textarea,
            reply_to: None,
            reply_to_author: None,
        }
    }

    pub fn set_reply(&mut self, reply_to: Option<ReplyRef>, reply_to_author: Option<String>) {
        self.reply_to = reply_to;
        self.reply_to_author = reply_to_author;
        if self.reply_to.is_some() {
            self.textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(format!(
                        " Reply to {} ",
                        self.reply_to_author.as_deref().unwrap_or("post")
                    )),
            );
        }
    }

    fn char_count(&self) -> usize {
        self.textarea
            .lines()
            .iter()
            .map(|l| l.len())
            .sum::<usize>()
            + self.textarea.lines().len().saturating_sub(1)
    }

    fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }
}

impl Component for Composer {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => return Some(Action::CloseComposer),
            (KeyModifiers::CONTROL, KeyCode::Enter)
            | (KeyModifiers::ALT, KeyCode::Enter)
            | (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                let text = self.text();
                if text.is_empty() {
                    return None;
                }
                return Some(Action::SubmitPost {
                    text,
                    reply_to: self.reply_to.clone(),
                });
            }
            _ => {
                // Check character limit before allowing input
                if matches!(key.code, KeyCode::Char(_)) && self.char_count() >= MAX_CHARS {
                    return None;
                }
                self.textarea.input(key);
            }
        }
        None
    }

    fn update(&mut self, _action: &Action) {}

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let modal_width = 60.min(area.width.saturating_sub(4));
        let modal_height = 12.min(area.height.saturating_sub(4));
        let modal_area = Rect {
            x: (area.width.saturating_sub(modal_width)) / 2,
            y: (area.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        frame.render_widget(Clear, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(modal_area);

        frame.render_widget(&self.textarea, chunks[0]);

        // Character counter + hints
        let count = self.char_count();
        let counter_style = if count > MAX_CHARS {
            Style::default().fg(Color::Red)
        } else if count > MAX_CHARS - 20 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let status = Line::from(vec![
            Span::styled(
                format!("{}/{}", count, MAX_CHARS),
                counter_style,
            ),
            Span::raw("  "),
            Span::styled(
                "Ctrl+Enter/Ctrl+S: post  Esc: cancel",
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        frame.render_widget(Paragraph::new(status), chunks[1]);
    }
}
