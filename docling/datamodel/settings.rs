//! datamodel/settings.rs — global settings.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoclingSettings {
    pub artifacts_path: Option<PathBuf>,
    pub log_level: String,
}

impl Default for DoclingSettings {
    fn default() -> Self {
        Self {
            artifacts_path: dirs::cache_dir().map(|p| p.join("docling").join("models")),
            log_level: "info".into(),
        }
    }
}
