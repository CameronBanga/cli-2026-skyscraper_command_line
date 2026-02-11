use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Tabs as RatatuiTabs};

pub fn draw_tabs(frame: &mut Frame, area: Rect, active: usize) {
    let titles = vec!["[1] Timeline", "[2] Profile"];
    let tabs = RatatuiTabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .title(" Skyscraper "),
        )
        .select(active)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}
