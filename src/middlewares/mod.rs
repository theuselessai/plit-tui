pub mod api;
pub mod command;
pub mod debug;
pub mod keyboard;
pub mod websocket;

use tokio::sync::mpsc;

use crate::actions::Action;
use crate::structs::AppState;

pub trait Middleware {
    fn process(
        &self,
        state: &AppState,
        action: &Action,
        tx: &mpsc::UnboundedSender<Action>,
    ) -> Option<Action>;
}

pub enum DispatchResult {
    Continue(Option<Action>),
    Quit,
}

pub fn dispatch(
    state: &mut AppState,
    action: Action,
    middlewares: &[Box<dyn Middleware>],
    tx: &mpsc::UnboundedSender<Action>,
) -> DispatchResult {
    let mut current = Some(action);

    for mw in middlewares {
        if let Some(ref act) = current {
            current = mw.process(state, act, tx);
        }
    }

    let Some(action) = current else {
        return DispatchResult::Continue(None);
    };

    if matches!(action, Action::Quit) {
        return DispatchResult::Quit;
    }

    crate::reducers::reduce(state, &action);

    if let Action::CommandBarEnqueueCmd(ref uuid) = action
        && let Some(cmd_str) = state.cmd_str_queue.get(uuid).cloned()
        && crate::structs::command_handler::execute(state, &cmd_str, tx)
    {
        return DispatchResult::Quit;
    }

    DispatchResult::Continue(Some(action))
}
