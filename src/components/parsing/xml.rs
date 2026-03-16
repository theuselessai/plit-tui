use std::collections::HashMap;
use std::str::FromStr;

use handlebars::Handlebars;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use serde_json::{Map, Value};

#[derive(Clone, Debug)]
pub struct XmlElement {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<XmlElement>,
    pub text: Option<String>,
}

fn element_from_start(e: &BytesStart) -> XmlElement {
    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
    let mut attributes = HashMap::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = attr.unescape_value().unwrap_or_default().to_string();
        attributes.insert(key, value);
    }
    XmlElement {
        name,
        attributes,
        children: vec![],
        text: None,
    }
}

pub fn parse_xml(xml: &str) -> XmlElement {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<XmlElement> = vec![];
    let mut root: Option<XmlElement> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                stack.push(element_from_start(e));
            }
            Ok(Event::End(_)) => {
                let el = stack.pop().expect("XML Parse Error: unmatched end tag");
                if stack.is_empty() {
                    root = Some(el);
                } else {
                    stack.last_mut().unwrap().children.push(el);
                }
            }
            Ok(Event::Empty(ref e)) => {
                let el = element_from_start(e);
                if stack.is_empty() {
                    root = Some(el);
                } else {
                    stack.last_mut().unwrap().children.push(el);
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if let Some(current) = stack.last_mut() {
                    let preserve = matches!(current.name.as_str(), "span" | "line");
                    let stored = if preserve {
                        text.clone()
                    } else {
                        text.trim().to_string()
                    };
                    if !stored.is_empty() {
                        match &mut current.text {
                            Some(t) => t.push_str(&stored),
                            None => current.text = Some(stored),
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML Parse Error: {:?}", e),
            _ => {}
        }
    }

    root.expect("XML Parse Error: empty document")
}

pub fn parse(template_name: &str, v: &Value, reg: &mut Handlebars<'_>) -> XmlElement {
    let filled_template = reg.render(template_name, v).expect("Template Parse Error");
    parse_xml(&filled_template)
}

pub fn parse_attr(el: &XmlElement, attr_name: &str) -> Option<Value> {
    el.attributes
        .get(attr_name)
        .and_then(|v| serde_json::from_str(v).ok())
}

pub fn parse_text_attr(el: &XmlElement, attr_name: &str) -> Option<String> {
    el.attributes.get(attr_name).cloned()
}

pub fn parse_str_list(el: &XmlElement, attr_name: &str) -> Option<Vec<String>> {
    el.attributes
        .get(attr_name)
        .and_then(|v| serde_json::from_str(v).ok())
}

pub fn parse_usize(el: &XmlElement, attr_name: &str) -> Result<usize, String> {
    match el.attributes.get(attr_name) {
        Some(v) => v
            .parse::<usize>()
            .map_err(|e| format!("Attribute Parse Error: {:?}", e)),
        None => Err(format!("{} not found", attr_name)),
    }
}

pub fn extract_text(el: &XmlElement) -> String {
    el.text.clone().unwrap_or_default()
}

pub fn apply_color(style: Style, v_styles: &Value, color_attr: &str) -> Style {
    match v_styles.get(color_attr).and_then(|v| v.as_str()) {
        Some(color_str) => {
            let color = Color::from_str(color_str).unwrap();
            match color_attr {
                "fg" => style.fg(color),
                "bg" => style.bg(color),
                _ => style,
            }
        }
        None => style,
    }
}

pub fn parse_styles(el: &XmlElement, attr_name: &str) -> Style {
    let styles_json: Option<Value> = parse_attr(el, attr_name);
    let mut style = Style::default();
    if let Some(ref v_styles) = styles_json {
        style = apply_color(style, v_styles, "fg");
        style = apply_color(style, v_styles, "bg");
    }
    style
}

pub fn alignment_from_text(txt_alignment: &str) -> Alignment {
    match txt_alignment {
        "Center" => Alignment::Center,
        "Right" => Alignment::Right,
        _ => Alignment::Left,
    }
}

pub fn get_u16_value(obj: &Map<String, Value>, key: &str) -> u16 {
    let err_msg = format!("{} value error", key);
    obj.get(key).unwrap().as_u64().expect(&err_msg) as u16
}

pub fn get_ratio_value(obj: &Map<String, Value>, key: &str) -> Vec<u32> {
    let obj_value = obj.get(key).unwrap().as_str().expect("ratio value error");
    let ratio: Vec<u32> = obj_value
        .split(':')
        .map(|v| v.parse().expect("ratio value parse error"))
        .collect();
    assert!(ratio.len() == 2, "Ratio must be in the form of '1:2'");
    ratio
}
