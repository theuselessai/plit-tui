use tokio::sync::mpsc;

use crate::actions::Action;
use crate::middlewares::Middleware;
use crate::structs::AppState;

pub struct CommandMiddleware;

impl Middleware for CommandMiddleware {
    fn process(
        &self,
        _state: &AppState,
        action: &Action,
        _tx: &mpsc::UnboundedSender<Action>,
    ) -> Option<Action> {
        Some(action.clone())
    }
}
