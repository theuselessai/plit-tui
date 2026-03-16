use handlebars::Handlebars;
use ratatui::layout::Rect;
use serde_json::{json, Value};

const EMBEDDED_TEMPLATES: &[(&str, &str)] = &[
    (
        "activity_bar",
        include_str!("../../packages/default/templates/activity_bar.hbs"),
    ),
    (
        "agent_list",
        include_str!("../../packages/default/templates/agent_list.hbs"),
    ),
    (
        "app",
        include_str!("../../packages/default/templates/app.hbs"),
    ),
    (
        "chat",
        include_str!("../../packages/default/templates/chat.hbs"),
    ),
    (
        "command_bar",
        include_str!("../../packages/default/templates/command_bar.hbs"),
    ),
    (
        "input_box",
        include_str!("../../packages/default/templates/input_box.hbs"),
    ),
    (
        "input_separator",
        include_str!("../../packages/default/templates/input_separator.hbs"),
    ),
    (
        "messages",
        include_str!("../../packages/default/templates/messages.hbs"),
    ),
    (
        "status_bar",
        include_str!("../../packages/default/templates/status_bar.hbs"),
    ),
    (
        "tabs",
        include_str!("../../packages/default/templates/tabs.hbs"),
    ),
];

pub fn register_embedded_templates(reg: &mut Handlebars<'_>) {
    for (name, source) in EMBEDDED_TEMPLATES {
        reg.register_template_string(name, *source)
            .unwrap_or_else(|e| panic!("Failed to register template '{name}': {e}"));
    }
}

pub fn props(store: &Value, area: Option<Rect>) -> Value {
    match area {
        Some(v) => {
            json!({
                "props": {
                    "store": store,
                    "area": {
                        "height": v.height,
                        "width": v.width
                    }
                }
            })
        }
        None => {
            json!({"props": {"store": store, "area": {}}})
        }
    }
}
