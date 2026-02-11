use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::Screen;

pub fn draw_statusbar(
    frame: &mut Frame,
    area: Rect,
    screen: &Screen,
    in_composer: bool,
    error: Option<&str>,
) {
    if let Some(err) = error {
        let error_bar = Paragraph::new(format!(" Error: {}", err))
            .style(Style::default().fg(Color::White).bg(Color::Red));
        frame.render_widget(error_bar, area);
        return;
    }

    let hints = if in_composer {
        "Ctrl+Enter: post | Esc: cancel"
    } else {
        match screen {
            Screen::Login => "Tab: switch fields | Enter: login | Esc: quit",
            Screen::Timeline => {
                "j/k: navigate | Enter: thread | n: post | r: reply | l: like | t: repost | R: refresh | q: quit"
            }
            Screen::Thread => {
                "Esc: back | r: reply | l: like | t: repost | u: profile | q: quit"
            }
            Screen::Profile => {
                "j/k: navigate | Enter: thread | Esc: back | q: quit"
            }
        }
    };

    let bar = Paragraph::new(format!(" {}", hints))
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black));
    frame.render_widget(bar, area);
}
