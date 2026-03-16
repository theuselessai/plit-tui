use crate::structs::AppState;
use serde_json::Value;

pub fn set_mode(state: &mut AppState, mode: &str) {
    match mode {
        "normal" | "insert" => state.json_store["command"] = Value::from(""),
        "command" => state.json_store["command"] = Value::from(":"),
        _ => {}
    }
}

pub fn push(state: &mut AppState, c: char) {
    let value = state.json_store["command"]
        .as_str()
        .expect("command is not str");
    let mut process_value = value.to_string();
    process_value.push(c);
    state.json_store["command"] = Value::String(process_value);
}

pub fn pop(state: &mut AppState, _pop_index: u16) {
    let value = state.json_store["command"]
        .as_str()
        .expect("command is not str");
    let mut process_value = value.to_string();
    if process_value.len() > 1 {
        process_value.pop();
        state.json_store["command"] = Value::String(process_value);
    }
}

pub fn enqueue_cmd(state: &mut AppState, uuid: &str) {
    let value = state.json_store["command"]
        .as_str()
        .expect("command is not str");
    let mut process_value = value.to_string();
    if process_value.len() > 1 {
        let cmd = process_value.split_off(1);
        state.cmd_str_queue.insert(uuid.to_string(), cmd);
    }
    state.json_store["command"] = Value::String(process_value);
}
