use tokio::sync::mpsc;

use crate::actions::Action;
use crate::middlewares::Middleware;
use crate::structs::AppState;

pub struct DebugMiddleware;

impl Middleware for DebugMiddleware {
    fn process(
        &self,
        _state: &AppState,
        action: &Action,
        _tx: &mpsc::UnboundedSender<Action>,
    ) -> Option<Action> {
        tracing::debug!(?action, "dispatch");
        Some(action.clone())
    }
}
