#![allow(dead_code)]

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

use crate::actions::Action;

pub fn spawn_ws_task(
    base_url: String,
    token: String,
    tx: mpsc::UnboundedSender<Action>,
    cmd_rx: mpsc::UnboundedReceiver<String>,
    cancel: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut cmd_rx = cmd_rx;
        let mut pending_subs: Vec<String> = vec![];
        let mut backoff = Duration::from_secs(1);
        let wslog = |msg: &str| {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/plit-ws.log")
            {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let _ = writeln!(f, "{ts} {msg}");
            }
        };
        wslog(&format!("WS task started, url={base_url}"));
        loop {
            tx.send(Action::WsStatusChanged("reconnecting".into())).ok();
            wslog("connecting...");
            match connect_ws(&base_url, &token).await {
                Ok(ws_stream) => {
                    backoff = Duration::from_secs(1);
                    wslog("connected, entering handle_ws");
                    tx.send(Action::WsStatusChanged("connected".into())).ok();
                    match handle_ws(ws_stream, &tx, &mut cmd_rx, &mut pending_subs, &cancel).await {
                        Ok(()) => wslog("handle_ws returned Ok"),
                        Err(e) => wslog(&format!("handle_ws returned Err: {e}")),
                    }
                }
                Err(e) => {
                    wslog(&format!("connect failed: {e}"));
                }
            }
            if cancel.is_cancelled() {
                wslog("cancelled, exiting");
                break;
            }
            wslog(&format!("reconnecting in {backoff:?}"));
            tx.send(Action::WsStatusChanged("disconnected".into())).ok();
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
        }
    })
}

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect_ws(base_url: &str, token: &str) -> anyhow::Result<WsStream> {
    let ws_base = base_url
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    let url = format!("{ws_base}/ws/?token={token}");
    let (ws_stream, _) = tokio_tungstenite::connect_async(&url).await?;
    Ok(ws_stream)
}

async fn handle_ws(
    ws_stream: WsStream,
    tx: &mpsc::UnboundedSender<Action>,
    cmd_rx: &mut mpsc::UnboundedReceiver<String>,
    pending_subs: &mut Vec<String>,
    cancel: &CancellationToken,
) -> anyhow::Result<()> {
    let (mut sink, mut stream) = ws_stream.split();

    for channel in pending_subs.iter() {
        send_subscribe(&mut sink, channel).await.ok();
    }

    let mut heartbeat = tokio::time::interval(Duration::from_secs(15));
    heartbeat.tick().await;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                sink.send(Message::Close(None)).await.ok();
                break;
            }
            _ = heartbeat.tick() => {
                let ping = serde_json::json!({"type": "ping"});
                if sink.send(Message::Text(ping.to_string().into())).await.is_err() {
                    break;
                }
            }
            cmd = cmd_rx.recv() => {
                if let Some(channel) = cmd {
                    send_subscribe(&mut sink, &channel).await.ok();
                    if !pending_subs.contains(&channel) {
                        pending_subs.push(channel);
                    }
                }
            }
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_text_message(&text, tx, &mut sink).await;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        sink.send(Message::Pong(data)).await.ok();
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => return Err(e.into()),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

async fn handle_text_message(
    text: &str,
    tx: &mpsc::UnboundedSender<Action>,
    sink: &mut futures_util::stream::SplitSink<WsStream, Message>,
) {
    let Ok(msg) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };
    let msg_type = msg["type"].as_str().unwrap_or_default();

    match msg_type {
        "ping" => {
            let pong = serde_json::json!({"type": "pong"});
            sink.send(Message::Text(pong.to_string().into())).await.ok();
        }
        "chat_message" => {
            let data = &msg["data"];
            let text = data["text"].as_str().unwrap_or_default().to_string();
            if !text.is_empty() {
                tx.send(Action::ChatMessageReceived {
                    role: "assistant".to_string(),
                    content: text,
                })
                .ok();
            }
        }
        "node_status" => {
            let data = &msg["data"];
            let is_tool_call = data["is_tool_call"].as_bool() == Some(true);

            if is_tool_call {
                let tool_name = data["tool_name"].as_str().unwrap_or_default().to_string();
                let node_id = data["node_id"].as_str().unwrap_or_default().to_string();
                let status = data["status"].as_str().unwrap_or_default().to_string();
                tx.send(Action::WsToolCall {
                    tool_name,
                    node_id,
                    status,
                })
                .ok();
            } else {
                let node_name = data["node_id"].as_str().unwrap_or_default().to_string();
                let status = data["status"].as_str().unwrap_or_default().to_string();
                let model_name = data["model_name"].as_str().map(String::from);
                if status == "waiting"
                    && let Some(ids) = data["child_execution_ids"].as_array()
                {
                    for id in ids {
                        if let Some(id) = id.as_str() {
                            tx.send(Action::WsChildTask {
                                execution_id: id.to_string(),
                                status: "running".to_string(),
                            })
                            .ok();
                        }
                    }
                }
                tx.send(Action::WsNodeStatus {
                    node_name,
                    status,
                    model_name,
                })
                .ok();
            }
        }
        "child_node_status" => {
            let data = &msg["data"];
            let child_exec_id = data["child_execution_id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let status = data["status"].as_str().unwrap_or_default().to_string();
            if !child_exec_id.is_empty() {
                tx.send(Action::WsChildTask {
                    execution_id: child_exec_id,
                    status,
                })
                .ok();
            }
        }
        "execution_started" => {
            tx.send(Action::WsExecutionStarted).ok();
        }
        "execution_completed" => {
            tx.send(Action::WsExecutionDone { success: true }).ok();
        }
        "execution_failed" => {
            tx.send(Action::WsExecutionDone { success: false }).ok();
        }
        _ => {}
    }
}

async fn send_subscribe(
    sink: &mut futures_util::stream::SplitSink<WsStream, Message>,
    channel: &str,
) -> anyhow::Result<()> {
    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/plit-ws.log")
        {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = writeln!(f, "{ts} SEND: subscribe {channel}");
        }
    }
    let msg = serde_json::json!({
        "type": "subscribe",
        "channel": channel,
    });
    sink.send(Message::Text(msg.to_string().into())).await?;
    Ok(())
}
