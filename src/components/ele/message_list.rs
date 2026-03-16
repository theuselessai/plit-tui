use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::structs::AppState;

/// Return the byte offset in `s` that fits within `max_cols` display columns.
fn byte_offset_for_cols(s: &str, max_cols: usize) -> usize {
    let mut cols = 0;
    let mut byte_pos = 0;
    for ch in s.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if cols + w > max_cols {
            break;
        }
        cols += w;
        byte_pos += ch.len_utf8();
    }
    byte_pos
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            result.push(String::new());
            continue;
        }
        let mut remaining = line;
        while !remaining.is_empty() {
            if UnicodeWidthStr::width(remaining) <= max_width {
                result.push(remaining.to_string());
                break;
            }
            let byte_limit = byte_offset_for_cols(remaining, max_width);
            let split_at = remaining[..byte_limit]
                .rfind(' ')
                .map(|p| p + 1)
                .unwrap_or(byte_limit);
            // Guarantee forward progress when no space and first char exceeds width
            let split_at = if split_at == 0 {
                remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(0)
            } else {
                split_at
            };
            result.push(remaining[..split_at].to_string());
            remaining = &remaining[split_at..];
        }
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

fn calc_height(content: &str, width: usize) -> usize {
    let usable = width.saturating_sub(4);
    let wrapped = wrap_text(content, usable);
    1 + wrapped.len() + 1
}

fn build_user_msg(msg: &crate::structs::ChatMessage, usable_width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let accent = Style::default().fg(Color::DarkGray);

    lines.push(Line::from(vec![
        Span::styled("  \u{2503} ", accent),
        Span::styled("you", Style::default().fg(Color::DarkGray)),
    ]));

    for wrapped in wrap_text(&msg.content, usable_width) {
        lines.push(Line::from(vec![
            Span::styled("  \u{2503} ", accent),
            Span::raw(wrapped),
        ]));
    }

    lines
}

fn build_agent_msg(
    msg: &crate::structs::ChatMessage,
    model: &str,
    usable_width: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let mut meta = vec![Span::styled(
        "  \u{25A3} agent",
        Style::default().fg(Color::DarkGray),
    )];
    if !model.is_empty() {
        meta.push(Span::styled(
            format!(" \u{00B7} {model}"),
            Style::default().fg(Color::DarkGray),
        ));
    }
    lines.push(Line::from(meta));

    for wrapped in wrap_text(&msg.content, usable_width) {
        lines.push(Line::from(Span::raw(format!("  {wrapped}"))));
    }

    lines
}

pub fn render(frame: &mut Frame, store: &mut AppState, area: Rect) {
    if store.messages.is_empty() {
        let placeholder = Paragraph::new(Line::from(vec![Span::styled(
            "  No messages yet",
            Style::default().fg(Color::DarkGray),
        )]));
        frame.render_widget(placeholder, area);
        return;
    }

    let width = area.width as usize;
    let view_height = area.height as usize;
    let usable_width = width.saturating_sub(4);

    let heights: Vec<usize> = store
        .messages
        .iter()
        .map(|m| calc_height(&m.content, width))
        .collect();
    let mut prefix = Vec::with_capacity(heights.len() + 1);
    prefix.push(0usize);
    for h in &heights {
        prefix.push(prefix.last().unwrap() + h);
    }
    let total_height = *prefix.last().unwrap();

    let scroll_offset = if store.sticky_bottom {
        let bottom = total_height.saturating_sub(view_height);
        store.scroll_offset = bottom;
        bottom
    } else {
        let clamped = store
            .scroll_offset
            .min(total_height.saturating_sub(view_height));
        store.scroll_offset = clamped;
        clamped
    };

    let max_scroll = total_height.saturating_sub(view_height);
    let pct = if max_scroll == 0 {
        100
    } else {
        (scroll_offset * 100) / max_scroll
    };
    store.json_store["scroll_indicator"] = serde_json::Value::String(format!("{}%", pct.min(100)));

    let first_msg = match prefix.binary_search(&scroll_offset) {
        Ok(i) => i.min(store.messages.len().saturating_sub(1)),
        Err(i) => i.saturating_sub(1),
    };

    let model = store.json_store["model_name"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let mut lines: Vec<Line> = Vec::new();
    let mut y_accumulated: usize = prefix[first_msg];

    for (idx, (msg, &h)) in store
        .messages
        .iter()
        .zip(heights.iter())
        .enumerate()
        .skip(first_msg)
    {
        let msg_start = y_accumulated;

        if msg_start >= scroll_offset + view_height {
            break;
        }

        let mut msg_lines: Vec<Line> = Vec::new();

        if idx > 0 {
            msg_lines.push(Line::from(""));
        }

        let is_user = msg.role == "user";
        if is_user {
            msg_lines.extend(build_user_msg(msg, usable_width));
        } else {
            msg_lines.extend(build_agent_msg(msg, &model, usable_width));
        }

        let skip_top = scroll_offset.saturating_sub(msg_start);
        let remaining_view = (scroll_offset + view_height).saturating_sub(msg_start);
        let take = remaining_view.min(msg_lines.len());

        for line in msg_lines
            .into_iter()
            .skip(skip_top)
            .take(take.saturating_sub(skip_top))
        {
            lines.push(line);
        }

        y_accumulated += h;
    }

    lines.truncate(view_height);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);

    if store.unread_count > 0 && !store.sticky_bottom {
        let indicator = format!(" \u{2193} {} new ", store.unread_count);
        let indicator_width = UnicodeWidthStr::width(indicator.as_str()) as u16;
        let x = area.right().saturating_sub(indicator_width + 1);
        let y = area.bottom().saturating_sub(1);
        if y >= area.y && x >= area.x {
            let indicator_area = Rect::new(x, y, indicator_width, 1);
            let widget = Paragraph::new(Line::from(Span::styled(
                indicator,
                Style::default().fg(Color::Yellow).bg(Color::DarkGray),
            )));
            frame.render_widget(widget, indicator_area);
        }
    }
}
