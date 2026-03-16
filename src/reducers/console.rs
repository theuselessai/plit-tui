use crate::structs::AppState;
use serde_json::Value;

pub fn push(state: &mut AppState, line: &str) {
    let value = state.json_store["console_output_lines"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut process_value = value;
    process_value.push(Value::String(line.to_string()));
    state.json_store["console_output_lines"] = Value::Array(process_value);
}
