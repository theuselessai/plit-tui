use handlebars::Handlebars;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use crate::components::parsing::xml::parse;
use crate::components::utils::props;
use crate::components::xml::{El, create_element};
use crate::structs::AppState;

pub fn render(
    frame: &mut Frame,
    store: &mut AppState,
    reg: &mut Handlebars<'_>,
    area: Rect,
    template: &str,
) {
    let dom_root = parse(template, &props(&store.json_store, Some(area)), reg);
    let el = create_element(&dom_root);
    render_el(frame, store, reg, area, el);
}

fn render_el(
    frame: &mut Frame,
    store: &mut AppState,
    reg: &mut Handlebars<'_>,
    area: Rect,
    el: El,
) {
    match el {
        El::Paragraph(p) => {
            frame.render_widget(p, area);
        }
        El::Tabs(t) => {
            frame.render_widget(t, area);
        }
        El::Layout(l, children) => {
            let chunks = l.split(area);
            for (i, child) in children.into_iter().enumerate() {
                if let Some(child) = child {
                    render_child(frame, store, reg, chunks[i], *child);
                }
            }
        }
        El::Component(Some(name)) => {
            render(frame, store, reg, area, &name);
        }
        El::MessageList => {
            crate::components::ele::message_list::render(frame, store, area);
        }
        El::InputBox => {
            crate::components::ele::input_box::render(frame, store, area);
        }
        El::ActivityBar => {
            crate::components::ele::activity_bar::render(frame, store, area);
        }
        El::ToolBar => {
            crate::components::ele::tool_bar::render(frame, store, area);
        }
        El::ProgressBar => {
            frame.render_widget(Paragraph::new(""), area);
        }
        _ => {}
    }
}

fn render_child(
    frame: &mut Frame,
    store: &mut AppState,
    reg: &mut Handlebars<'_>,
    area: Rect,
    el: El,
) {
    match el {
        El::Component(Some(name)) => render(frame, store, reg, area, &name),
        other => render_el(frame, store, reg, area, other),
    }
}
