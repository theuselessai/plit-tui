#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: String,
    pub username: String,
    pub pipelit_url: String,
}

fn auth_json_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("Could not determine config directory")?;
    Ok(base.join("plit").join("auth.json"))
}

impl AuthConfig {
    pub fn load() -> Result<AuthConfig> {
        let path = auth_json_path()?;
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("Failed to parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = auth_json_path()?;
        let dir = path.parent().context("Invalid auth.json path")?;
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory {}", dir.display()))?;

        let json = serde_json::to_string_pretty(self).context("Failed to serialize auth config")?;
        std::fs::write(&path, &json)
            .with_context(|| format!("Failed to write {}", path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .with_context(|| format!("Failed to set permissions on {}", path.display()))?;
        }

        Ok(())
    }
}
