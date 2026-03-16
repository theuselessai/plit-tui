use crate::structs::{AppState, Command};

pub fn create(state: &mut AppState, uuid: &str, failed: bool) {
    if let Some(cmd_str) = state.cmd_str_queue.remove(uuid) {
        let cmd = Command::new(cmd_str, uuid.to_string(), failed);
        if failed {
            state.cmd_ended.push(cmd);
        } else {
            state.cmd_running.push(cmd);
        }
    }
}

pub fn end(state: &mut AppState, uuid: &str, success: bool) {
    if let Some(pos) = state.cmd_running.iter().position(|c| c.id == uuid) {
        let mut cmd = state.cmd_running.remove(pos);
        cmd.failed = !success;
        state.cmd_ended.push(cmd);
    }
}
