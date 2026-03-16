use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::actions::Action;
use crate::middlewares::Middleware;
use crate::structs::AppState;

pub struct KeyboardMiddleware;

impl Middleware for KeyboardMiddleware {
    fn process(
        &self,
        state: &AppState,
        action: &Action,
        _tx: &mpsc::UnboundedSender<Action>,
    ) -> Option<Action> {
        if let Action::Keyboard(key) = action {
            map_key(key, state)
        } else {
            Some(action.clone())
        }
    }
}

fn map_key(key: &KeyEvent, state: &AppState) -> Option<Action> {
    let mode = state.json_store["mode"].as_str().unwrap_or("normal");
    match mode {
        "normal" => normal_key(key, state),
        "command" => command_key(key),
        "insert" => insert_key(key),
        _ => None,
    }
}

fn normal_key(key: &KeyEvent, state: &AppState) -> Option<Action> {
    let tab = state.json_store["tabs_selection"].as_u64().unwrap_or(0);
    match key.code {
        KeyCode::Char(':') => Some(Action::SetMode("command".to_string())),
        KeyCode::Char('i') | KeyCode::Char('a') => Some(Action::SetMode("insert".to_string())),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('j') if tab == 0 => Some(Action::AgentDown),
        KeyCode::Char('k') if tab == 0 => Some(Action::AgentUp),
        KeyCode::Enter if tab == 0 => Some(Action::AgentSelect),
        KeyCode::Char('j') => Some(Action::ScrollDown(1)),
        KeyCode::Char('k') => Some(Action::ScrollUp(1)),
        KeyCode::Char('G') => Some(Action::ScrollToBottom),
        KeyCode::Char('g') => Some(Action::ScrollToTop),
        KeyCode::PageUp => Some(Action::ScrollUp(15)),
        KeyCode::PageDown => Some(Action::ScrollDown(15)),
        KeyCode::Tab => Some(Action::TabNext),
        KeyCode::BackTab => Some(Action::TabPrev),
        _ => None,
    }
}

fn command_key(key: &KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::SetMode("normal".to_string())),
        KeyCode::Backspace => Some(Action::CommandBarPop(1)),
        KeyCode::Enter => Some(Action::CommandBarEnqueueCmd(
            uuid::Uuid::new_v4().to_string(),
        )),
        KeyCode::Char(c) => Some(Action::CommandBarPush(c)),
        _ => None,
    }
}

fn insert_key(key: &KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::SetMode("normal".to_string())),
        KeyCode::Enter => Some(Action::InputSend),
        KeyCode::Backspace => Some(Action::InputBackspace),
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::InputNewline)
        }
        KeyCode::Char(c) => Some(Action::InputChar(c)),
        _ => None,
    }
}
