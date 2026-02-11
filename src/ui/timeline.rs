use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::models::feed::FeedState;
use crate::ui::post_widget;

pub fn draw_timeline(frame: &mut Frame, area: Rect, feed: &FeedState) {
    if feed.loading && feed.posts.is_empty() {
        let loading = Paragraph::new("Loading timeline...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(loading, area);
        return;
    }

    if feed.posts.is_empty() {
        let empty = Paragraph::new("No posts yet. Press R to refresh.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    // Virtual scrolling: only render visible posts
    let mut y = area.y;
    let max_y = area.bottom();

    // Calculate scroll offset to keep selected post visible
    let mut offset = feed.scroll_offset;
    let visible_height = area.height as usize;

    // Adjust offset so selected post is visible
    // First, calculate heights of posts up to selected
    let mut cumulative_height: usize = 0;
    let mut selected_start: usize = 0;
    let mut selected_height: usize = 0;

    for (i, post) in feed.posts.iter().enumerate() {
        let h = post_widget::post_height(post, area.width) as usize;
        if i == feed.selected_index {
            selected_start = cumulative_height;
            selected_height = h;
            break;
        }
        cumulative_height += h;
    }

    // Ensure selected post is within view
    if selected_start < offset {
        offset = selected_start;
    } else if selected_start + selected_height > offset + visible_height {
        offset = (selected_start + selected_height).saturating_sub(visible_height);
    }

    // Render from offset
    let mut running_height: usize = 0;
    for (i, post) in feed.posts.iter().enumerate() {
        let h = post_widget::post_height(post, area.width);

        if running_height + h as usize <= offset {
            running_height += h as usize;
            continue;
        }

        if y >= max_y {
            break;
        }

        let available_h = (max_y - y).min(h);
        let post_area = Rect::new(area.x, y, area.width, available_h);

        let selected = i == feed.selected_index;
        post_widget::draw_post(frame, post_area, post, selected);

        y += available_h;
        running_height += h as usize;
    }

    // Loading indicator at bottom
    if feed.loading {
        if y < max_y {
            frame.render_widget(
                Paragraph::new("Loading more...")
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center),
                Rect::new(area.x, y, area.width, 1),
            );
        }
    }
}
