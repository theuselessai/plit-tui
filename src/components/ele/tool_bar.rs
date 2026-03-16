use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::structs::AppState;

fn status_icon(status: &str) -> (&'static str, Color) {
    match status {
        "success" | "done" => ("\u{2713}", Color::Green),
        "running" | "active" => ("\u{27F3}", Color::Yellow),
        "failed" | "error" => ("\u{2717}", Color::Red),
        "waiting" => ("\u{25CB}", Color::DarkGray),
        _ => ("\u{25CB}", Color::DarkGray),
    }
}

const BRAILLE_FRAMES: [char; 10] = [
    '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}',
    '\u{2807}', '\u{280F}',
];

pub fn render(frame: &mut Frame, store: &AppState, area: Rect) {
    let bg = Style::default().bg(Color::Black);
    let nodes_running = store.json_store["nodes_running"].as_bool() == Some(true);

    if store.tool_calls.is_empty() && !nodes_running {
        let paragraph = Paragraph::new(Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green).bg(Color::Black)),
            Span::styled(
                "ready ",
                Style::default().fg(Color::DarkGray).bg(Color::Black),
            ),
        ]))
        .style(bg)
        .alignment(ratatui::layout::Alignment::Right);
        frame.render_widget(paragraph, area);
        return;
    }

    let mut spans: Vec<Span> = vec![Span::styled(" ", bg)];

    if nodes_running {
        let frame_idx = store.json_store["spinner_frame"].as_u64().unwrap_or(0) as usize;
        let spinner = BRAILLE_FRAMES[frame_idx % BRAILLE_FRAMES.len()];
        let agent_name = store.json_store["agent_name"]
            .as_str()
            .unwrap_or("agent")
            .to_string();
        spans.push(Span::styled(
            format!("{spinner} {agent_name} "),
            Style::default().fg(Color::Yellow).bg(Color::Black),
        ));
    }

    if !store.tool_calls.is_empty() {
        spans.push(Span::styled(" ", Style::default().bg(Color::Black)));
        for (i, tool) in store.tool_calls.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(
                    " \u{2192} ",
                    Style::default().fg(Color::DarkGray).bg(Color::Black),
                ));
            }
            let (icon, color) = status_icon(&tool.status);
            spans.push(Span::styled(
                format!("{} ", tool.tool_name),
                Style::default().fg(Color::White).bg(Color::Black),
            ));
            spans.push(Span::styled(
                icon,
                Style::default().fg(color).bg(Color::Black),
            ));
        }
    }

    if !store.child_tasks.is_empty() {
        let done = store
            .child_tasks
            .iter()
            .filter(|t| t.status == "success")
            .count();
        let total = store.child_tasks.len();
        spans.push(Span::styled(" ", Style::default().bg(Color::Black)));
        if done == total {
            spans.push(Span::styled(
                format!("{total}/{total} tasks \u{2713}"),
                Style::default().fg(Color::Green).bg(Color::Black),
            ));
        } else {
            spans.push(Span::styled(
                format!("{done}/{total} tasks \u{27F3}"),
                Style::default().fg(Color::Yellow).bg(Color::Black),
            ));
        }
    }

    if !store.message_queue.is_empty() {
        spans.push(Span::styled(" ", Style::default().bg(Color::Black)));
        let count = store.message_queue.len();
        spans.push(Span::styled(
            format!("{count} queued"),
            Style::default().fg(Color::DarkGray).bg(Color::Black),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).style(bg);
    frame.render_widget(paragraph, area);
}
