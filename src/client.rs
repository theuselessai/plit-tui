#![allow(dead_code)]

use anyhow::{Context, Result, bail};
use reqwest::header;

use crate::types::{ChatMessageResponse, ChatSendResponse, WorkflowDetail};

pub struct PipelitClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl PipelitClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    pub async fn verify_token(&self) -> Result<String> {
        let url = format!("{}/api/v1/auth/me/", self.base_url);
        let resp = self
            .http
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    anyhow::anyhow!(
                        "Cannot connect to {}. Is the server running?",
                        self.base_url
                    )
                } else {
                    anyhow::anyhow!("Request failed: {e}")
                }
            })?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            bail!("Token expired or invalid. Run `plit auth login` again.");
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("Token verification failed (HTTP {status}): {body}");
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse user info response")?;
        body["username"]
            .as_str()
            .map(String::from)
            .context("Missing 'username' in user info response")
    }

    pub async fn list_workflows(&self) -> Result<Vec<WorkflowDetail>> {
        let url = format!("{}/api/v1/workflows/", self.base_url);
        let resp = self
            .http
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to fetch workflows")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Failed to list workflows (HTTP {status}): {body}");
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse workflows response")?;
        let items = body
            .get("items")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));
        serde_json::from_value(items).context("Failed to deserialize workflow list")
    }

    pub async fn get_workflow(&self, slug: &str) -> Result<WorkflowDetail> {
        let url = format!("{}/api/v1/workflows/{}/", self.base_url, slug);
        let resp = self
            .http
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to fetch workflow detail")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Failed to get workflow (HTTP {status}): {body}");
        }

        resp.json::<WorkflowDetail>()
            .await
            .context("Failed to parse workflow detail response")
    }

    pub async fn send_chat_message(&self, slug: &str, message: &str) -> Result<ChatSendResponse> {
        let url = format!("{}/api/v1/workflows/{}/chat/", self.base_url, slug);
        let body = serde_json::json!({ "text": message });
        let resp = self
            .http
            .post(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .json(&body)
            .send()
            .await
            .context("Failed to send chat message")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Failed to send message (HTTP {status}): {body}");
        }

        resp.json::<ChatSendResponse>()
            .await
            .context("Failed to parse chat response")
    }

    pub async fn get_chat_history(&self, slug: &str) -> Result<Vec<ChatMessageResponse>> {
        let url = format!(
            "{}/api/v1/workflows/{}/chat/history?limit=200",
            self.base_url, slug
        );
        let resp = self
            .http
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .context("Failed to fetch chat history")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Failed to get chat history (HTTP {status}): {body}");
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse chat history response")?;
        let messages = body
            .get("messages")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));
        serde_json::from_value(messages).context("Failed to deserialize chat messages")
    }
}
