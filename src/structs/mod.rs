pub mod command_handler;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

use crate::types::WorkflowDetail;

const DATA: &str = r#"
{
    "mode": "normal",
    "tabs_titles": ["Workflows", "Chat"],
    "tabs_selection": 0,
    "command": "",
    "agent_name": "",
    "model_name": "",
    "ws_status": "disconnected",
    "host_display": "",
    "scroll_indicator": "",
    "spinner_frame": 0,
    "nodes_running": false,
    "console_output_lines": [],
    "agent_lines": []
}
"#;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub id: String,
    pub failed: bool,
}

impl Command {
    pub fn new(name: String, id: String, failed: bool) -> Command {
        Command { name, id, failed }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputState {
    pub text: String,
    pub cursor: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeActivity {
    pub node_name: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub node_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChildTask {
    pub execution_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum View {
    Chat,
    AgentList,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppState {
    pub json_store: Value,
    pub cmd_str_queue: HashMap<String, String>,
    pub cmd_running: Vec<Command>,
    pub cmd_ended: Vec<Command>,
    pub messages: Vec<ChatMessage>,
    pub input: InputState,
    pub workflows: Vec<WorkflowDetail>,
    pub selected_agent: usize,
    pub activity: Vec<NodeActivity>,
    pub tool_calls: Vec<ToolCall>,
    pub child_tasks: Vec<ChildTask>,
    pub message_queue: Vec<String>,
    pub scroll_offset: usize,
    pub sticky_bottom: bool,
    pub unread_count: usize,
    pub view_stack: Vec<View>,
}

impl AppState {
    pub fn new() -> AppState {
        let state: Value = serde_json::from_str(DATA).expect("JSON Error!");
        AppState {
            json_store: state,
            cmd_str_queue: HashMap::new(),
            cmd_running: Vec::new(),
            cmd_ended: Vec::new(),
            messages: Vec::new(),
            input: InputState::default(),
            workflows: Vec::new(),
            selected_agent: 0,
            activity: Vec::new(),
            tool_calls: Vec::new(),
            child_tasks: Vec::new(),
            message_queue: Vec::new(),
            scroll_offset: 0,
            sticky_bottom: true,
            unread_count: 0,
            view_stack: vec![View::Chat],
        }
    }
}

impl AppState {
    pub fn rebuild_agent_lines(&mut self) {
        let lines: Vec<Value> = self
            .workflows
            .iter()
            .enumerate()
            .map(|(i, w)| {
                if i == self.selected_agent {
                    Value::String(format!("  \u{25C9} {}", w.name))
                } else {
                    Value::String(format!("    {}", w.name))
                }
            })
            .collect();
        self.json_store["agent_lines"] = Value::Array(lines);
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState::new()
    }
}

impl fmt::Debug for AppState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("AppState")
            .field("json_store", &self.json_store)
            .finish()
    }
}
