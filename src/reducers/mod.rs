mod chat;
mod command_bar;
mod commands;
mod console;
mod mode;

use crate::actions::Action;
use crate::structs::AppState;

pub fn reduce(state: &mut AppState, action: &Action) {
    match action {
        Action::SetMode(m) => {
            mode::set(state, m);
            command_bar::set_mode(state, m);
        }
        Action::CommandBarPush(c) => command_bar::push(state, *c),
        Action::CommandBarPop(n) => command_bar::pop(state, *n),
        Action::CommandBarEnqueueCmd(uuid) => command_bar::enqueue_cmd(state, uuid),
        Action::ConsolePush(line) => console::push(state, line),
        Action::CommandCreate(uuid) => commands::create(state, uuid, false),
        Action::CommandInvalid(uuid) => commands::create(state, uuid, true),
        Action::CommandEnd {
            uuid,
            success,
            reason: _,
        } => commands::end(state, uuid, *success),
        Action::TabNext => {
            let len = state.json_store["tabs_titles"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(1);
            let current = state.json_store["tabs_selection"].as_u64().unwrap_or(0) as usize;
            let next = (current + 1) % len;
            state.json_store["tabs_selection"] = serde_json::Value::from(next);
        }
        Action::TabPrev => {
            let len = state.json_store["tabs_titles"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(1);
            let current = state.json_store["tabs_selection"].as_u64().unwrap_or(0) as usize;
            let prev = if current == 0 { len - 1 } else { current - 1 };
            state.json_store["tabs_selection"] = serde_json::Value::from(prev);
        }
        Action::ScrollDown(n) => {
            state.scroll_offset = state.scroll_offset.saturating_add(*n);
            state.sticky_bottom = false;
            state.json_store["scroll_indicator"] = serde_json::Value::String("↑ scrolled".into());
        }
        Action::ScrollUp(n) => {
            state.scroll_offset = state.scroll_offset.saturating_sub(*n);
            state.sticky_bottom = false;
            let indicator = if state.scroll_offset == 0 {
                "top"
            } else {
                "↑ scrolled"
            };
            state.json_store["scroll_indicator"] = serde_json::Value::String(indicator.into());
        }
        Action::ScrollToTop => {
            state.scroll_offset = 0;
            state.sticky_bottom = false;
            state.json_store["scroll_indicator"] = serde_json::Value::String("top".into());
        }
        Action::ScrollToBottom => {
            state.sticky_bottom = true;
            state.unread_count = 0;
            state.json_store["scroll_indicator"] = serde_json::Value::String(String::new());
        }
        Action::WorkflowsLoaded(workflows) => {
            state.workflows = workflows.clone();
            state.selected_agent = 0;
            state.rebuild_agent_lines();
            if let Some(w) = state.workflows.get(state.selected_agent) {
                state.json_store["agent_name"] = serde_json::Value::String(w.name.clone());
                state.json_store["model_name"] =
                    serde_json::Value::String(w.model_name().unwrap_or_default());
            }
        }
        Action::AgentDown => {
            if !state.workflows.is_empty() {
                state.selected_agent = (state.selected_agent + 1).min(state.workflows.len() - 1);
                state.rebuild_agent_lines();
            }
        }
        Action::AgentUp => {
            state.selected_agent = state.selected_agent.saturating_sub(1);
            state.rebuild_agent_lines();
        }
        Action::AgentSelect => {
            if let Some(w) = state.workflows.get(state.selected_agent) {
                state.json_store["agent_name"] = serde_json::Value::String(w.name.clone());
                let model = w.model_name().unwrap_or_default();
                state.json_store["model_name"] = serde_json::Value::String(model);
                state.json_store["tabs_selection"] = serde_json::Value::from(1);
            }
        }
        Action::InputChar(c) => chat::input_char(state, *c),
        Action::InputBackspace => chat::input_backspace(state),
        Action::InputNewline => chat::input_newline(state),
        Action::InputSend => {
            let text = state.input.text.trim().to_string();
            if !text.is_empty() && state.json_store["nodes_running"].as_bool() == Some(true) {
                state.message_queue.push(text);
                state.input.text.clear();
                state.input.cursor = 0;
            } else {
                if !text.is_empty() {
                    state.json_store["nodes_running"] = serde_json::Value::Bool(true);
                }
                chat::input_send(state);
            }
        }
        Action::ChatMessageReceived { role, content } => {
            chat::chat_message_received(state, role, content);
            if !state.sticky_bottom {
                state.unread_count += 1;
            }
        }
        Action::ChatHistoryLoaded(messages) => {
            chat::chat_history_loaded(state, messages.clone());
            state.json_store["nodes_running"] = serde_json::Value::Bool(false);
            state.sticky_bottom = true;
            state.activity.clear();
        }
        Action::WsStatusChanged(status) => {
            state.json_store["ws_status"] = serde_json::Value::String(status.clone());
        }
        Action::WsNodeStatus {
            node_name,
            status,
            model_name,
        } => {
            if let Some(model) = model_name {
                state.json_store["model_name"] = serde_json::Value::String(model.clone());
            }
            state.json_store["nodes_running"] = serde_json::Value::Bool(true);
            if let Some(existing) = state
                .activity
                .iter_mut()
                .find(|a| a.node_name == *node_name)
            {
                existing.status = status.clone();
            } else {
                state.activity.push(crate::structs::NodeActivity {
                    node_name: node_name.clone(),
                    status: status.clone(),
                });
            }
        }
        Action::WsToolCall {
            tool_name,
            node_id,
            status,
        } => {
            if let Some(existing) = state.tool_calls.iter_mut().find(|t| t.node_id == *node_id) {
                existing.status = status.clone();
            } else {
                state.tool_calls.push(crate::structs::ToolCall {
                    tool_name: tool_name.clone(),
                    node_id: node_id.clone(),
                    status: status.clone(),
                });
            }
        }
        Action::WsChildTask {
            execution_id,
            status,
        } => {
            if let Some(existing) = state
                .child_tasks
                .iter_mut()
                .find(|t| t.execution_id == *execution_id)
            {
                existing.status = status.clone();
            } else {
                state.child_tasks.push(crate::structs::ChildTask {
                    execution_id: execution_id.clone(),
                    status: status.clone(),
                });
            }
        }
        Action::WsExecutionStarted => {
            state.json_store["nodes_running"] = serde_json::Value::Bool(true);
            state.activity.clear();
            state.tool_calls.clear();
            state.child_tasks.clear();
        }
        Action::WsExecutionDone { .. } => {
            state.activity.clear();
            state.tool_calls.clear();
            state.child_tasks.clear();
            state.json_store["nodes_running"] = serde_json::Value::Bool(false);
        }
        Action::SpinnerTick => {
            let frame = state.json_store["spinner_frame"].as_u64().unwrap_or(0);
            state.json_store["spinner_frame"] = serde_json::Value::from((frame + 1) % 10);
        }
        _ => {}
    }
}
