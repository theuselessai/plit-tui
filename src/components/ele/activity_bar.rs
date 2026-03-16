use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::structs::AppState;

fn status_icon(status: &str) -> (&'static str, Color) {
    match status {
        "completed" | "done" | "success" => ("\u{2713}", Color::Green),
        "running" | "active" => ("\u{27F3}", Color::Yellow),
        "failed" | "error" => ("\u{2717}", Color::Red),
        _ => ("\u{25CB}", Color::DarkGray),
    }
}

pub fn render(frame: &mut Frame, store: &AppState, area: Rect) {
    if store.activity.is_empty() {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            "",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(paragraph, area);
        return;
    }

    let cols = area.width as usize;
    let arrow = Span::styled(" \u{2192} ", Style::default().fg(Color::DarkGray));

    let mut spans: Vec<Span> = Vec::new();

    for (i, node) in store.activity.iter().enumerate() {
        if i > 0 {
            spans.push(arrow.clone());
        }

        let (icon, color) = status_icon(&node.status);

        if cols < 60 {
            spans.push(Span::styled(icon, Style::default().fg(color)));
        } else if cols < 100 {
            let short_name: String = node
                .node_name
                .split('_')
                .next()
                .unwrap_or(&node.node_name)
                .chars()
                .take(8)
                .collect();
            spans.push(Span::styled(
                format!("{short_name} "),
                Style::default().fg(Color::White),
            ));
            spans.push(Span::styled(icon, Style::default().fg(color)));
        } else {
            spans.push(Span::styled(
                format!("{} ", node.node_name),
                Style::default().fg(Color::White),
            ));
            spans.push(Span::styled(icon, Style::default().fg(color)));
            let status_label = match node.status.as_str() {
                "completed" | "done" | "success" => "",
                "running" | "active" => " running",
                "failed" | "error" => " failed",
                other => {
                    spans.push(Span::styled(
                        format!(" {other}"),
                        Style::default().fg(Color::DarkGray),
                    ));
                    ""
                }
            };
            if !status_label.is_empty() {
                spans.push(Span::styled(
                    status_label,
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}
