use crossterm::event::KeyEvent;

use crate::structs::ChatMessage;
use crate::types::WorkflowDetail;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Action {
    Keyboard(KeyEvent),
    SetMode(String),
    CommandBarPush(char),
    CommandBarPop(u16),
    CommandBarEnqueueCmd(String),
    CommandConsume(String),
    CommandCreate(String),
    CommandInvalid(String),
    CommandEnd {
        uuid: String,
        success: bool,
        reason: String,
    },
    ConsolePush(String),
    Quit,
    TabNext,
    TabPrev,
    ScrollUp(usize),
    ScrollDown(usize),
    ScrollToTop,
    ScrollToBottom,
    InputChar(char),
    InputBackspace,
    InputNewline,
    InputSend,
    WorkflowsLoaded(Vec<WorkflowDetail>),
    AgentDown,
    AgentUp,
    AgentSelect,
    ChatMessageReceived {
        role: String,
        content: String,
    },
    ChatHistoryLoaded(Vec<ChatMessage>),
    WsStatusChanged(String),
    WsNodeStatus {
        node_name: String,
        status: String,
        model_name: Option<String>,
    },
    WsToolCall {
        tool_name: String,
        node_id: String,
        status: String,
    },
    WsChildTask {
        execution_id: String,
        status: String,
    },
    WsExecutionStarted,
    WsExecutionDone {
        success: bool,
    },
    WsSubscribe(String),
    SpinnerTick,
}
