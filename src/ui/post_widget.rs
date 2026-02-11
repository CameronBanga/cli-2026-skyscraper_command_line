use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::models::post::PostViewModel;
use crate::utils::text::{styled_text, wrapped_line_count};
use crate::utils::time::relative_time;

pub fn post_height(post: &PostViewModel, width: u16) -> u16 {
    let text_width = width.saturating_sub(4);
    let text_lines = wrapped_line_count(&post.text, text_width);

    let mut height = 1  // author line
        + text_lines     // post text
        + 1              // stats line
        + 1;             // bottom border/padding

    if post.reply_parent_author.is_some() {
        height += 1;
    }
    if post.reposted_by.is_some() {
        height += 1;
    }
    if post.embed_summary.is_some() {
        height += 1;
    }

    height
}

pub fn draw_post(frame: &mut Frame, area: Rect, post: &PostViewModel, selected: bool) {
    let border_style = if selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut y = inner.y;
    let x = inner.x + 1;
    let w = inner.width.saturating_sub(1);
    let bottom = inner.bottom();

    // Repost indicator
    if let Some(ref reposted_by) = post.reposted_by {
        if y >= bottom { return; }
        let repost_line = Line::from(vec![
            Span::styled("⟳ ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("Reposted by {}", reposted_by),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(repost_line),
            Rect::new(x, y, w, 1),
        );
        y += 1;
    }

    // Reply indicator
    if let Some(ref parent_author) = post.reply_parent_author {
        if y >= bottom { return; }
        let reply_line = Line::from(vec![
            Span::styled("↩ ", Style::default().fg(Color::Blue)),
            Span::styled(
                format!("Reply to {}", parent_author),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(reply_line),
            Rect::new(x, y, w, 1),
        );
        y += 1;
    }

    // Author line
    if y >= bottom { return; }
    let time_str = relative_time(&post.created_at);
    let author_line = Line::from(vec![
        Span::styled(
            &post.author_display_name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  @{}", post.author_handle),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("  {}", time_str),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(author_line),
        Rect::new(x, y, w, 1),
    );
    y += 1;

    // Post text
    if y >= bottom { return; }
    let text_lines = styled_text(&post.text, &post.facets);
    let remaining = bottom.saturating_sub(y);
    let text_height = remaining.saturating_sub(2).max(1).min(remaining);
    frame.render_widget(
        Paragraph::new(text_lines).wrap(Wrap { trim: false }),
        Rect::new(x, y, w, text_height),
    );
    let wrap_lines = wrapped_line_count(&post.text, w);
    y += wrap_lines.min(remaining);

    // Embed summary
    if let Some(ref embed) = post.embed_summary {
        if y < bottom {
            if let crate::models::post::EmbedKind::Images(n) = embed.kind {
                draw_embed_images(frame, x, y, w, n);
                y += 1;
            } else {
                let embed_text = match (&embed.title, &embed.description) {
                    (Some(t), _) => format!("📎 {}", t),
                    (_, Some(d)) => format!("📎 {}", d),
                    _ => format!("📎 [{}]", match embed.kind {
                        crate::models::post::EmbedKind::ExternalLink => "link",
                        crate::models::post::EmbedKind::Video => "video",
                        crate::models::post::EmbedKind::Record => "quote",
                        crate::models::post::EmbedKind::RecordWithMedia => "quote+media",
                        crate::models::post::EmbedKind::Images(_) => unreachable!(),
                    }),
                };
                frame.render_widget(
                    Paragraph::new(embed_text).style(Style::default().fg(Color::DarkGray)),
                    Rect::new(x, y, w, 1),
                );
                y += 1;
            }
        }
    }

    // Stats line
    if y < bottom {
        let like_style = if post.is_liked {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let repost_style = if post.is_reposted {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let stats = Line::from(vec![
            Span::styled(
                if post.is_liked { "♥ " } else { "♡ " },
                like_style,
            ),
            Span::styled(format!("{}", post.like_count), like_style),
            Span::raw("  "),
            Span::styled(
                if post.is_reposted { "⟳ " } else { "⟳ " },
                repost_style,
            ),
            Span::styled(format!("{}", post.repost_count), repost_style),
            Span::raw("  "),
            Span::styled(
                format!("💬 {}", post.reply_count),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        frame.render_widget(Paragraph::new(stats), Rect::new(x, y, w, 1));
    }
}

fn draw_embed_images(frame: &mut Frame, x: u16, y: u16, w: u16, count: usize) {
    let text = format!("🖼 {} image{}", count, if count != 1 { "s" } else { "" });
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        Rect::new(x, y, w, 1),
    );
}
