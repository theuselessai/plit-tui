use std::sync::Arc;

use crate::client::PipelitClient;
use crate::middlewares::{
    api::ApiMiddleware, command::CommandMiddleware, debug::DebugMiddleware,
    keyboard::KeyboardMiddleware, Middleware,
};

pub fn init_middlewares(client: Option<Arc<PipelitClient>>) -> Vec<Box<dyn Middleware>> {
    vec![
        Box::new(DebugMiddleware),
        Box::new(KeyboardMiddleware),
        Box::new(ApiMiddleware::new(client)),
        Box::new(CommandMiddleware),
    ]
}
