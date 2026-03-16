use std::sync::Arc;

use tokio::sync::mpsc;

use crate::actions::Action;
use crate::client::PipelitClient;
use crate::middlewares::Middleware;
use crate::structs::AppState;

pub struct ApiMiddleware {
    client: Option<Arc<PipelitClient>>,
}

impl ApiMiddleware {
    pub fn new(client: Option<Arc<PipelitClient>>) -> Self {
        Self { client }
    }
}

impl Middleware for ApiMiddleware {
    fn process(
        &self,
        state: &AppState,
        action: &Action,
        _tx: &mpsc::UnboundedSender<Action>,
    ) -> Option<Action> {
        if let Action::InputSend = action {
            let text = state.input.text.trim().to_string();
            if text.is_empty() {
                return None;
            }

            let nodes_running = state.json_store["nodes_running"].as_bool() == Some(true);
            if !nodes_running && let Some(ref client) = self.client {
                let slug = state
                    .workflows
                    .get(state.selected_agent)
                    .map(|w| w.slug.clone());

                if let Some(slug) = slug {
                    let client = Arc::clone(client);
                    tokio::spawn(async move {
                        if let Err(e) = client.send_chat_message(&slug, &text).await {
                            tracing::warn!("Failed to send message: {e}");
                        }
                    });
                }
            }
        }

        Some(action.clone())
    }
}
