use ratatui::prelude::*;
use ratatui::text::Line;
use unicode_width::UnicodeWidthStr;

use crate::models::post::{Facet, FacetKind};

/// Count how many terminal rows `text` occupies when wrapped at `max_width` columns.
/// Uses Unicode display width so emoji and CJK characters are measured correctly.
pub fn wrapped_line_count(text: &str, max_width: u16) -> u16 {
    let max_width = max_width as usize;
    if max_width == 0 {
        return 1;
    }
    text.split('\n')
        .map(|line| {
            let w = UnicodeWidthStr::width(line);
            if w == 0 { 1 } else { ((w - 1) / max_width + 1) as u16 }
        })
        .sum()
}

pub fn styled_text<'a>(text: &str, facets: &[Facet]) -> Vec<Line<'a>> {
    if facets.is_empty() {
        return text
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();
    }

    let mut spans: Vec<Span<'a>> = Vec::new();
    let bytes = text.as_bytes();
    let mut pos = 0;

    let mut sorted_facets: Vec<&Facet> = facets.iter().collect();
    sorted_facets.sort_by_key(|f| f.start);

    for facet in sorted_facets {
        let start = facet.start.min(bytes.len());
        let end = facet.end.min(bytes.len());

        if start > pos {
            if let Ok(s) = std::str::from_utf8(&bytes[pos..start]) {
                spans.push(Span::raw(s.to_string()));
            }
        }

        if start < end {
            if let Ok(s) = std::str::from_utf8(&bytes[start..end]) {
                let style = match &facet.kind {
                    FacetKind::Mention(_) => Style::default().fg(Color::Cyan),
                    FacetKind::Link(_) => Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                    FacetKind::Tag(_) => Style::default().fg(Color::Magenta),
                };
                spans.push(Span::styled(s.to_string(), style));
            }
        }

        pos = end;
    }

    if pos < bytes.len() {
        if let Ok(s) = std::str::from_utf8(&bytes[pos..]) {
            spans.push(Span::raw(s.to_string()));
        }
    }

    // Split spans across newline boundaries into separate Lines
    if spans.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines: Vec<Line<'a>> = Vec::new();
    let mut current_spans: Vec<Span<'a>> = Vec::new();

    for span in spans {
        let style = span.style;
        let content = span.content.to_string();
        let mut parts = content.split('\n').peekable();

        while let Some(part) = parts.next() {
            if !part.is_empty() {
                current_spans.push(Span::styled(part.to_string(), style));
            }
            // If there's another part after this, we hit a newline — flush the current line
            if parts.peek().is_some() {
                lines.push(Line::from(std::mem::take(&mut current_spans)));
            }
        }
    }

    // Flush remaining spans
    lines.push(Line::from(current_spans));

    lines
}
