pub mod actions;
pub mod auth;
pub mod client;
pub mod components;
pub mod middlewares;
pub mod reducers;
pub mod structs;
pub mod types;
pub mod utils;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures_util::StreamExt;
use handlebars::Handlebars;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio_util::sync::CancellationToken;

use crate::actions::Action;
use crate::auth::AuthConfig;
use crate::client::PipelitClient;
use crate::components::app;
use crate::components::helpers::height_buffer::HEIGHT_BUFFER_HELPER;
use crate::components::helpers::{hb_macros, hb_utils};
use crate::components::utils::register_embedded_templates;
use crate::middlewares::websocket::spawn_ws_task;
use crate::middlewares::{DispatchResult, dispatch};
use crate::structs::AppState;

pub async fn run(url: Option<String>) -> Result<()> {
    let auth = AuthConfig::load().ok();
    let base_url = url
        .or_else(|| auth.as_ref().map(|a| a.pipelit_url.clone()))
        .unwrap_or_else(|| "http://localhost:8000".to_string());
    let token = auth.map(|a| a.token).unwrap_or_default();

    let api: Option<Arc<PipelitClient>> = if !token.is_empty() {
        Some(Arc::new(PipelitClient::new(&base_url, &token)))
    } else {
        None
    };

    if let Some(ref api) = api {
        match api.verify_token().await {
            Ok(_username) => {
                tracing::info!("Token verified");
            }
            Err(e) => {
                tracing::warn!("Token verification failed: {e}");
            }
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut reg = Handlebars::new();
    reg.register_helper("stringify", Box::new(hb_macros::stringify));
    reg.register_helper("gt", Box::new(hb_macros::gt));
    reg.register_helper("height_buffer", Box::new(HEIGHT_BUFFER_HELPER));
    reg.register_escape_fn(hb_utils::escape_nothing);

    register_embedded_templates(&mut reg);

    let mut state = AppState::new();
    let host_display = base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string();
    state.json_store["host_display"] = serde_json::Value::String(host_display);
    let middlewares = utils::init_middlewares(api.clone());
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut events = crossterm::event::EventStream::new();
    let cancel = CancellationToken::new();
    let (ws_cmd_tx, ws_cmd_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let ws_handle = if api.is_some() {
        Some(spawn_ws_task(
            base_url.clone(),
            token.clone(),
            action_tx.clone(),
            ws_cmd_rx,
            cancel.clone(),
        ))
    } else {
        None
    };

    if let Some(ref api) = api {
        let api = Arc::clone(api);
        let tx = action_tx.clone();
        let ws_tx = ws_cmd_tx.clone();
        tokio::spawn(async move {
            match api.list_workflows().await {
                Ok(workflows) => {
                    let mut detailed = Vec::new();
                    for w in &workflows {
                        match api.get_workflow(&w.slug).await {
                            Ok(detail) if detail.has_trigger_chat() => detailed.push(detail),
                            Ok(_) => {}
                            Err(e) => tracing::warn!("Failed to get workflow {}: {e}", w.slug),
                        }
                    }
                    if let Some(first) = detailed.first() {
                        let slug = first.slug.clone();
                        ws_tx.send(format!("workflow:{slug}")).ok();
                        let _ = tx.send(Action::WorkflowsLoaded(detailed));
                        match api.get_chat_history(&slug).await {
                            Ok(history) => {
                                let messages = history
                                    .into_iter()
                                    .map(|m| crate::structs::ChatMessage {
                                        role: m.role,
                                        content: m.text,
                                    })
                                    .collect();
                                let _ = tx.send(Action::ChatHistoryLoaded(messages));
                            }
                            Err(e) => tracing::warn!("Failed to load initial history: {e}"),
                        }
                    } else {
                        let _ = tx.send(Action::WorkflowsLoaded(detailed));
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to load workflows: {e}");
                }
            }
        });
    }

    let mut tick = tokio::time::interval(Duration::from_millis(80));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let res: Result<()> = loop {
        terminal.draw(|f| {
            let area = f.area();
            app::render(f, &mut state, &mut reg, area, "app");
        })?;

        let reduced_action: Option<Action>;

        tokio::select! {
            event = events.next() => {
                match event {
                    Some(Ok(Event::Key(key))) => {
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            break Ok(());
                        }
                        match dispatch(&mut state, Action::Keyboard(key), &middlewares, &action_tx) {
                            DispatchResult::Quit => break Ok(()),
                            DispatchResult::Continue(a) => reduced_action = a,
                        }
                    }
                    Some(Err(e)) => break Err(e.into()),
                    None => break Ok(()),
                    _ => { reduced_action = None; }
                }
            }
            action = action_rx.recv() => {
                match action {
                    Some(action) => {
                        match dispatch(&mut state, action, &middlewares, &action_tx) {
                            DispatchResult::Quit => break Ok(()),
                            DispatchResult::Continue(a) => reduced_action = a,
                        }
                    }
                    None => break Ok(()),
                }
            }
            _ = tick.tick() => {
                if state.json_store["nodes_running"].as_bool() == Some(true) {
                    dispatch(&mut state, Action::SpinnerTick, &middlewares, &action_tx);
                }
                reduced_action = None;
            }
            _ = tokio::signal::ctrl_c() => {
                break Ok(());
            }
        }

        if let Some(ref action) = reduced_action
            && matches!(
                action,
                Action::AgentSelect | Action::WsExecutionDone { success: true }
            )
            && let Some(ref api) = api
            && let Some(w) = state.workflows.get(state.selected_agent)
        {
            let slug = w.slug.clone();
            ws_cmd_tx.send(format!("workflow:{slug}")).ok();

            if matches!(action, Action::WsExecutionDone { .. }) && !state.message_queue.is_empty() {
                let queued = state.message_queue.remove(0);
                state.messages.push(crate::structs::ChatMessage {
                    role: "user".to_string(),
                    content: queued.clone(),
                });
                state.json_store["nodes_running"] = serde_json::Value::Bool(true);
                let client = Arc::clone(api);
                tokio::spawn(async move {
                    if let Err(e) = client.send_chat_message(&slug, &queued).await {
                        tracing::warn!("Failed to send queued message: {e}");
                    }
                });
            } else {
                let client = Arc::clone(api);
                let tx = action_tx.clone();
                tokio::spawn(async move {
                    match client.get_chat_history(&slug).await {
                        Ok(history) => {
                            let messages = history
                                .into_iter()
                                .map(|m| crate::structs::ChatMessage {
                                    role: m.role,
                                    content: m.text,
                                })
                                .collect();
                            let _ = tx.send(Action::ChatHistoryLoaded(messages));
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load chat history: {e}");
                        }
                    }
                });
            }
        }
    };

    cancel.cancel();
    if let Some(handle) = ws_handle {
        let _ = handle.await;
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}
