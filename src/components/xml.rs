use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use serde_json::Value;

use crate::components::ele::powerline_tab::Tabs;
use crate::components::parsing::xml::{
    alignment_from_text, extract_text, get_ratio_value, get_u16_value, parse_attr, parse_str_list,
    parse_styles, parse_text_attr, parse_usize, XmlElement,
};

#[derive(Clone)]
pub enum El {
    Paragraph(Paragraph<'static>),
    Line(Line<'static>),
    Span(Span<'static>),
    Tabs(Tabs<'static>),
    Layout(Layout, Vec<Option<Box<El>>>),
    Constraint(Constraint, Option<Box<El>>),
    Component(Option<String>),
    MessageList,
    InputBox,
    ActivityBar,
    ToolBar,
    ProgressBar,
}

pub fn create_element(el: &XmlElement) -> El {
    let children: Vec<El> = if !el.children.is_empty() {
        el.children.iter().map(create_element).collect()
    } else {
        vec![]
    };

    let style = parse_styles(el, "styles");

    match el.name.as_str() {
        "paragraph" => {
            let wrap_json: Option<Value> = parse_attr(el, "wrap");
            let alignment_json: Option<Value> = parse_attr(el, "alignment");

            let el_list: Vec<Line> = if !children.is_empty() {
                children
                    .into_iter()
                    .map(|child| match child {
                        El::Line(s) => s,
                        _ => panic!("Not a Text Node!"),
                    })
                    .collect()
            } else {
                vec![]
            };
            let mut paragraph_el = Paragraph::new(el_list);

            if let Some(v_wrap) = wrap_json
                && let Some(trim) = v_wrap.get("trim").and_then(|value| value.as_bool())
            {
                paragraph_el = paragraph_el.wrap(Wrap { trim });
            }

            if let Some(v_alignment) = alignment_json
                && let Some(alignment_str) =
                    v_alignment.get("position").and_then(|value| value.as_str())
            {
                paragraph_el = paragraph_el.alignment(alignment_from_text(alignment_str));
            }

            paragraph_el = paragraph_el.style(style);
            El::Paragraph(paragraph_el)
        }
        "line" => {
            if !children.is_empty() {
                let span_list: Vec<Span> = children
                    .into_iter()
                    .map(|child| match child {
                        El::Span(s) => s,
                        _ => panic!("Not a Text Node!"),
                    })
                    .collect();
                El::Line(Line::from(span_list))
            } else {
                El::Line(Line::from(extract_text(el)))
            }
        }
        "span" => {
            let span_el = Span::styled(extract_text(el), style);
            El::Span(span_el)
        }
        "tabs" => {
            let mut tabs_el = Tabs::default();
            let tabs_titles = parse_str_list(el, "tab_titles").unwrap();
            let tabs_selection = parse_usize(el, "tab_selection").unwrap();
            let highlight_style = parse_styles(el, "highlight_styles");
            let divider_style = parse_styles(el, "divider_styles");
            tabs_el = tabs_el
                .titles(tabs_titles)
                .style(style)
                .highlight_style(highlight_style)
                .divider_style(divider_style)
                .select(tabs_selection);
            El::Tabs(tabs_el)
        }
        "layout" => {
            let direction_str = parse_text_attr(el, "direction");
            let mut layout_el = Layout::default();
            match direction_str.as_deref() {
                Some("vertical") => layout_el = layout_el.direction(Direction::Vertical),
                Some("horizontal") => layout_el = layout_el.direction(Direction::Horizontal),
                _ => {}
            }

            let el_list: Vec<Constraint> = if !children.is_empty() {
                children
                    .iter()
                    .map(|child| match child {
                        El::Constraint(c, _) => *c,
                        _ => panic!("Not a Constraint Node!"),
                    })
                    .collect()
            } else {
                vec![]
            };

            let child_list: Vec<Option<Box<El>>> = if !children.is_empty() {
                children
                    .into_iter()
                    .map(|child| match child {
                        El::Constraint(_, child) => child,
                        _ => panic!("Not a Constraint Node!"),
                    })
                    .collect()
            } else {
                vec![]
            };

            layout_el = layout_el.constraints(el_list);
            El::Layout(layout_el, child_list)
        }
        "constraint" => {
            if let Some(value) = parse_attr(el, "type") {
                if value.is_object() {
                    let obj = value.as_object().expect("object values are wrong");
                    let key = obj.keys().next().unwrap().as_str();
                    let constraint_el = match key {
                        "length" => Constraint::Length(get_u16_value(obj, key)),
                        "min" => Constraint::Min(get_u16_value(obj, key)),
                        "max" => Constraint::Max(get_u16_value(obj, key)),
                        "percentage" => Constraint::Percentage(get_u16_value(obj, key)),
                        "ratio" => {
                            let rv = get_ratio_value(obj, key);
                            Constraint::Ratio(rv[0], rv[1])
                        }
                        _ => panic!("Wrong type for constraint"),
                    };
                    if !children.is_empty() {
                        let child = children.into_iter().next().unwrap();
                        El::Constraint(constraint_el, Some(Box::new(child)))
                    } else {
                        El::Constraint(constraint_el, None)
                    }
                } else {
                    panic!("constraint type value must be a json object");
                }
            } else {
                panic!("constraint type value must be a json object");
            }
        }
        "component" => {
            let template = parse_text_attr(el, "template");
            El::Component(template)
        }
        "message_list" => El::MessageList,
        "input_box" => El::InputBox,
        "activity_bar" => El::ActivityBar,
        "tool_bar" => El::ToolBar,
        "progress_bar" => El::ProgressBar,
        _ => panic!("Unknown Element: {}", el.name),
    }
}
