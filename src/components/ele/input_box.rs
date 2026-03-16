use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::structs::AppState;

pub fn render(frame: &mut Frame, store: &AppState, area: Rect) {
    let mode = store.json_store["mode"].as_str().unwrap_or("normal");
    let text = &store.input.text;
    let cursor = store.input.cursor;

    if text.is_empty() && mode != "insert" {
        let mut ph_lines = vec![Line::from("")];
        ph_lines.push(Line::from(vec![
            Span::styled("  \u{258E} ", Style::default().fg(Color::DarkGray)),
            Span::styled("Type a message...", Style::default().fg(Color::DarkGray)),
        ]));
        let placeholder = Paragraph::new(ph_lines);
        frame.render_widget(placeholder, area);
        return;
    }

    let content = if text.is_empty() { "" } else { text.as_str() };
    let input_lines: Vec<&str> = if content.is_empty() {
        vec![""]
    } else {
        content.split('\n').collect()
    };

    let mut lines: Vec<Line> = vec![Line::from("")];
    let mut char_offset = 0;

    for input_line in &input_lines {
        let line_len = input_line.len();
        let line_start = char_offset;
        let line_end = char_offset + line_len;

        let bar = Span::styled("  \u{258E} ", Style::default().fg(Color::Gray));

        if cursor >= line_start && cursor <= line_end {
            let pos_in_line = cursor - line_start;
            let before = &input_line[..pos_in_line];
            let after = if pos_in_line < line_len {
                &input_line[pos_in_line..]
            } else {
                ""
            };

            if pos_in_line < line_len {
                let cursor_char = &input_line[pos_in_line..pos_in_line + 1];
                let rest = &input_line[pos_in_line + 1..];
                lines.push(Line::from(vec![
                    bar,
                    Span::raw(before.to_string()),
                    Span::styled(
                        cursor_char.to_string(),
                        Style::default().fg(Color::Black).bg(Color::White),
                    ),
                    Span::raw(rest.to_string()),
                ]));
            } else {
                lines.push(Line::from(vec![
                    bar,
                    Span::raw(before.to_string()),
                    Span::styled("_", Style::default().fg(Color::DarkGray)),
                    Span::raw(after.to_string()),
                ]));
            }
        } else {
            lines.push(Line::from(vec![bar, Span::raw(input_line.to_string())]));
        }

        char_offset = line_end + 1;
    }

    let max_lines = area.height as usize;
    if lines.len() > max_lines {
        let skip = lines.len() - max_lines;
        lines = lines.into_iter().skip(skip).collect();
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
