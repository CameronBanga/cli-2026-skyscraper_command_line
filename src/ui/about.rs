use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

const APP_STORE_PROMO: &str = "\
Skyscraper is also available on iOS!\n\
Download the companion app for a full Bluesky experience on your iPhone or iPad.";

pub fn draw_about(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);

    // Promo banner
    let promo = Paragraph::new(format!("{}\n\nPress Enter to open the App Store", APP_STORE_PROMO))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Skyscraper for iOS ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .padding(Padding::horizontal(1)),
        );
    frame.render_widget(promo, chunks[0]);

    // Key bindings
    let bindings = vec![
        Line::from(Span::styled(
            " Navigation",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  j / k      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move down / up"),
        ]),
        Line::from(vec![
            Span::styled("  g / G      ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to top / bottom"),
        ]),
        Line::from(vec![
            Span::styled("  Enter      ", Style::default().fg(Color::Yellow)),
            Span::raw("Open thread"),
        ]),
        Line::from(vec![
            Span::styled("  Esc        ", Style::default().fg(Color::Yellow)),
            Span::raw("Go back"),
        ]),
        Line::from(vec![
            Span::styled("  1 / 2      ", Style::default().fg(Color::Yellow)),
            Span::raw("Timeline / Profile tab"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Posts",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  n          ", Style::default().fg(Color::Yellow)),
            Span::raw("New post"),
        ]),
        Line::from(vec![
            Span::styled("  r          ", Style::default().fg(Color::Yellow)),
            Span::raw("Reply to selected post"),
        ]),
        Line::from(vec![
            Span::styled("  l          ", Style::default().fg(Color::Yellow)),
            Span::raw("Like / unlike"),
        ]),
        Line::from(vec![
            Span::styled("  t          ", Style::default().fg(Color::Yellow)),
            Span::raw("Repost / unrepost"),
        ]),
        Line::from(vec![
            Span::styled("  R          ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh timeline"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Other",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  u          ", Style::default().fg(Color::Yellow)),
            Span::raw("View author profile"),
        ]),
        Line::from(vec![
            Span::styled("  a          ", Style::default().fg(Color::Yellow)),
            Span::raw("This about screen"),
        ]),
        Line::from(vec![
            Span::styled("  q          ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
    ];

    let keybindings = Paragraph::new(bindings).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Key Bindings ")
            .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .padding(Padding::horizontal(1)),
    );
    frame.render_widget(keybindings, chunks[2]);
}
