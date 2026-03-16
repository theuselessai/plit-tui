#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowDetail {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub nodes: Vec<NodeInfo>,
    #[serde(default)]
    pub edges: Vec<EdgeInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    #[serde(default, alias = "node_id")]
    pub name: String,
    #[serde(default)]
    pub component_type: String,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeInfo {
    #[serde(default)]
    pub source_node_id: String,
    #[serde(default)]
    pub target_node_id: String,
    #[serde(default)]
    pub edge_label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatSendResponse {
    #[serde(default)]
    pub execution_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub timestamp: Option<String>,
}

impl WorkflowDetail {
    pub fn has_trigger_chat(&self) -> bool {
        self.nodes
            .iter()
            .any(|n| n.component_type == "trigger_chat")
    }

    pub fn model_name(&self) -> Option<String> {
        let trigger = self
            .nodes
            .iter()
            .find(|n| n.component_type == "trigger_chat")?;
        let agent_edge = self
            .edges
            .iter()
            .find(|e| e.source_node_id == trigger.name)?;
        let agent_id = &agent_edge.target_node_id;
        let llm_edge = self
            .edges
            .iter()
            .find(|e| e.target_node_id == *agent_id && e.edge_label == "llm")?;
        let llm_node = self
            .nodes
            .iter()
            .find(|n| n.name == llm_edge.source_node_id)?;
        llm_node
            .config
            .get("model_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }
}
