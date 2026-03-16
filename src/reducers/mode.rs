use crate::structs::AppState;
use serde_json::Value;

pub fn set(state: &mut AppState, mode: &str) {
    state.json_store["mode"] = Value::from(mode);
}
