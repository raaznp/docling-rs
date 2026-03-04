use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

// ============================================================
// Performance settings
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSettings {
    /// Number of documents processed in one batch.
    pub doc_batch_size: usize,
    /// Maximum parallel document workers.
    pub doc_batch_concurrency: usize,
    /// Number of pages processed in one batch within a pipeline.
    pub page_batch_size: usize,
}

impl Default for PerfSettings {
    fn default() -> Self {
        Self {
            doc_batch_size: 4,
            doc_batch_concurrency: 2,
            page_batch_size: 4,
        }
    }
}

// ============================================================
// Global settings
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Performance tuning.
    pub perf: PerfSettings,
    /// Override path for all model artifacts (layout, OCR, table, etc.).
    pub artifacts_path: Option<std::path::PathBuf>,
    /// Default page range to process when none is specified.
    pub default_page_range: (usize, usize),
}

impl Settings {
    pub fn new() -> Self {
        Self {
            perf: PerfSettings::default(),
            artifacts_path: None,
            default_page_range: (1, usize::MAX),
        }
    }
}

/// Global settings singleton.
pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| RwLock::new(Settings::new()));

/// Convenience: read the current settings.
pub fn get_settings() -> Settings {
    SETTINGS.read().expect("Settings lock poisoned").clone()
}

/// Convenience: update settings.
pub fn update_settings<F>(f: F)
where
    F: FnOnce(&mut Settings),
{
    let mut s = SETTINGS.write().expect("Settings lock poisoned");
    f(&mut s);
}
