use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::models::feed::FeedState;
use crate::models::profile::ProfileViewModel;
use crate::ui::post_widget;

pub fn draw_profile(
    frame: &mut Frame,
    area: Rect,
    profile: Option<&ProfileViewModel>,
    feed: &FeedState,
) {
    let profile = match profile {
        Some(p) => p,
        None => {
            let loading = Paragraph::new("Loading profile...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(loading, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(1)])
        .split(area);

    // Profile header
    draw_profile_header(frame, chunks[0], profile);

    // Author feed
    draw_author_feed(frame, chunks[1], feed);
}

fn draw_profile_header(frame: &mut Frame, area: Rect, profile: &ProfileViewModel) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Display name
    frame.render_widget(
        Paragraph::new(profile.display_name.as_str())
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        chunks[0],
    );

    // Handle
    frame.render_widget(
        Paragraph::new(format!("@{}", profile.handle))
            .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );

    // Bio
    if !profile.description.is_empty() {
        frame.render_widget(
            Paragraph::new(profile.description.as_str())
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true }),
            chunks[2],
        );
    }

    // Stats
    let stats = Line::from(vec![
        Span::styled(
            format!("{}", profile.followers_count),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" followers  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", profile.follows_count),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" following  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", profile.posts_count),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" posts", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(stats), chunks[3]);
}

fn draw_author_feed(frame: &mut Frame, area: Rect, feed: &FeedState) {
    if feed.loading && feed.posts.is_empty() {
        let loading = Paragraph::new("Loading posts...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
        return;
    }

    if feed.posts.is_empty() {
        let empty = Paragraph::new("No posts")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let mut y = area.y;
    let max_y = area.bottom();

    for (i, post) in feed.posts.iter().enumerate() {
        if y >= max_y {
            break;
        }
        let h = post_widget::post_height(post, area.width).min(max_y - y);
        let post_area = Rect::new(area.x, y, area.width, h);
        post_widget::draw_post(frame, post_area, post, i == feed.selected_index);
        y += h;
    }
}
