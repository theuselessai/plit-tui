use crate::structs::{AppState, ChatMessage};

pub fn input_char(state: &mut AppState, c: char) {
    let cursor = state.input.cursor;
    let byte_pos = char_to_byte_pos(&state.input.text, cursor);
    state.input.text.insert(byte_pos, c);
    state.input.cursor += 1;
}

pub fn input_backspace(state: &mut AppState) {
    if state.input.cursor > 0 {
        state.input.cursor -= 1;
        let byte_pos = char_to_byte_pos(&state.input.text, state.input.cursor);
        let next_byte = char_to_byte_pos(&state.input.text, state.input.cursor + 1);
        state.input.text.drain(byte_pos..next_byte);
    }
}

pub fn input_newline(state: &mut AppState) {
    let byte_pos = char_to_byte_pos(&state.input.text, state.input.cursor);
    state.input.text.insert(byte_pos, '\n');
    state.input.cursor += 1;
}

pub fn input_send(state: &mut AppState) {
    let text = state.input.text.trim().to_string();
    if !text.is_empty() {
        state.messages.push(ChatMessage {
            role: "user".to_string(),
            content: text,
        });
    }
    state.input.text.clear();
    state.input.cursor = 0;
}

pub fn chat_message_received(state: &mut AppState, role: &str, content: &str) {
    state.messages.push(ChatMessage {
        role: role.to_string(),
        content: content.to_string(),
    });
}

pub fn chat_history_loaded(state: &mut AppState, messages: Vec<ChatMessage>) {
    state.messages = messages;
}

fn char_to_byte_pos(s: &str, char_pos: usize) -> usize {
    s.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
