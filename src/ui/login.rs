use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::action::Action;
use crate::ui::Component;

#[derive(Debug, Clone, PartialEq)]
enum LoginField {
    Handle,
    Password,
}

pub struct LoginForm {
    handle: String,
    password: String,
    focused_field: LoginField,
    error: Option<String>,
    submitting: bool,
}

impl LoginForm {
    pub fn new(default_handle: Option<String>) -> Self {
        LoginForm {
            handle: default_handle.unwrap_or_default(),
            password: String::new(),
            focused_field: LoginField::Handle,
            error: None,
            submitting: false,
        }
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
        self.submitting = false;
    }

    pub fn clear_error(&mut self) {
        self.error = None;
        self.submitting = false;
    }
}

impl Component for LoginForm {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        if self.submitting {
            return None;
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
            (KeyModifiers::NONE, KeyCode::Tab) => {
                self.focused_field = match self.focused_field {
                    LoginField::Handle => LoginField::Password,
                    LoginField::Password => LoginField::Handle,
                };
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if !self.handle.is_empty() && !self.password.is_empty() {
                    self.submitting = true;
                    self.error = None;
                    return Some(Action::SubmitLogin {
                        handle: self.handle.clone(),
                        password: self.password.clone(),
                    });
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => match self.focused_field {
                LoginField::Handle => {
                    self.handle.pop();
                }
                LoginField::Password => {
                    self.password.pop();
                }
            },
            (_, KeyCode::Char(c)) => match self.focused_field {
                LoginField::Handle => self.handle.push(c),
                LoginField::Password => self.password.push(c),
            },
            (KeyModifiers::NONE, KeyCode::Esc) => return Some(Action::Quit),
            _ => {}
        }
        None
    }

    fn update(&mut self, _action: &Action) {}

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let modal_width = 50.min(area.width.saturating_sub(4));
        let modal_height = 14.min(area.height.saturating_sub(4));
        let modal_area = Rect {
            x: (area.width.saturating_sub(modal_width)) / 2 + area.x,
            y: (area.height.saturating_sub(modal_height)) / 2 + area.y,
            width: modal_width,
            height: modal_height,
        };

        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(" Skyscraper - Login ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner);

        let title_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
        frame.render_widget(
            Paragraph::new("Sign in with app password").style(title_style).alignment(Alignment::Center),
            chunks[0],
        );

        let handle_label_style = if self.focused_field == LoginField::Handle {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        frame.render_widget(
            Paragraph::new("Handle:").style(handle_label_style),
            chunks[2],
        );

        let handle_style = if self.focused_field == LoginField::Handle {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let handle_text = if self.focused_field == LoginField::Handle {
            format!("{}█", &self.handle)
        } else {
            self.handle.clone()
        };
        frame.render_widget(
            Paragraph::new(handle_text).style(handle_style),
            chunks[3],
        );

        let pw_label_style = if self.focused_field == LoginField::Password {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        frame.render_widget(
            Paragraph::new("App Password:").style(pw_label_style),
            chunks[5],
        );

        let pw_style = if self.focused_field == LoginField::Password {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let pw_display = if self.focused_field == LoginField::Password {
            format!("{}█", "•".repeat(self.password.len()))
        } else {
            "•".repeat(self.password.len())
        };
        frame.render_widget(
            Paragraph::new(pw_display).style(pw_style),
            chunks[6],
        );

        if let Some(ref error) = self.error {
            frame.render_widget(
                Paragraph::new(error.as_str())
                    .style(Style::default().fg(Color::Red))
                    .wrap(Wrap { trim: true }),
                chunks[8],
            );
        } else if self.submitting {
            frame.render_widget(
                Paragraph::new("Signing in...")
                    .style(Style::default().fg(Color::Yellow)),
                chunks[8],
            );
        } else {
            frame.render_widget(
                Paragraph::new("Tab: switch fields  Enter: submit  Esc: quit")
                    .style(Style::default().fg(Color::DarkGray)),
                chunks[8],
            );
        }
    }
}
