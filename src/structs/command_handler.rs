use tokio::sync::mpsc;

use crate::actions::Action;
use crate::structs::AppState;

pub fn execute(state: &mut AppState, cmd: &str, tx: &mpsc::UnboundedSender<Action>) -> bool {
    let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
    let command = parts[0];
    let _args = parts.get(1).copied().unwrap_or("");

    match command {
        "q" | "quit" => return true,
        "clear" => {
            state.messages.clear();
            state.scroll_offset = 0;
            state.sticky_bottom = true;
        }
        "sessions" => {
            let _ = tx.send(Action::ConsolePush(
                "Sessions not yet implemented".to_string(),
            ));
        }
        "connect" => {
            let _ = tx.send(Action::ConsolePush(
                "Connect not yet implemented".to_string(),
            ));
        }
        "theme" => {
            let _ = tx.send(Action::ConsolePush(
                "Themes not yet implemented".to_string(),
            ));
        }
        "help" => {
            let help_lines = [
                ":q         quit",
                ":clear     clear chat history",
                ":sessions  manage sessions",
                ":connect   connect to backend",
                ":theme     switch theme",
                ":help      show this help",
            ];
            for line in help_lines {
                let _ = tx.send(Action::ConsolePush(line.to_string()));
            }
        }
        other => {
            let _ = tx.send(Action::ConsolePush(format!("Unknown command: {other}")));
        }
    }

    let _ = tx.send(Action::SetMode("normal".to_string()));
    false
}
