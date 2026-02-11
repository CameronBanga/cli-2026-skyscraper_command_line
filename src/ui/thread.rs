use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::models::thread::ThreadViewModel;
use crate::ui::post_widget;

pub fn draw_thread(frame: &mut Frame, area: Rect, thread: Option<&ThreadViewModel>) {
    let thread = match thread {
        Some(t) => t,
        None => {
            let loading = Paragraph::new("Loading thread...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(loading, area);
            return;
        }
    };

    let mut y = area.y;
    let max_y = area.bottom();

    // Parent chain
    for parent in &thread.parents {
        if y >= max_y {
            break;
        }
        let h = post_widget::post_height(parent, area.width).min(max_y - y);
        let post_area = Rect::new(area.x, y, area.width, h);
        post_widget::draw_post(frame, post_area, parent, false);
        y += h;

        // Thread connector
        if y < max_y {
            let connector = Paragraph::new("│")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(connector, Rect::new(area.x + 1, y, 1, 1));
            y += 1;
        }
    }

    // Focal post (highlighted)
    if y < max_y {
        let h = post_widget::post_height(&thread.focal, area.width).min(max_y - y);
        let post_area = Rect::new(area.x, y, area.width, h);
        post_widget::draw_post(frame, post_area, &thread.focal, true);
        y += h;
    }

    // Separator
    if y < max_y {
        let sep = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(sep, Rect::new(area.x, y, area.width, 1));
        y += 1;
    }

    // Replies header
    if y < max_y && !thread.replies.is_empty() {
        let header = Paragraph::new(format!(
            " {} {}",
            thread.replies.len(),
            if thread.replies.len() == 1 { "reply" } else { "replies" }
        ))
        .style(Style::default().fg(Color::Gray));
        frame.render_widget(header, Rect::new(area.x, y, area.width, 1));
        y += 1;
    }

    // Replies
    for reply in &thread.replies {
        if y >= max_y {
            break;
        }
        let indented_x = area.x + 2;
        let indented_w = area.width.saturating_sub(2);
        let h = post_widget::post_height(reply, indented_w).min(max_y - y);
        let reply_area = Rect::new(indented_x, y, indented_w, h);
        post_widget::draw_post(frame, reply_area, reply, false);
        y += h;
    }
}
