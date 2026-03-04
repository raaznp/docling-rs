//! datamodel/backend_options.rs — per-backend options.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PdfBackendOptions {
    pub page_range: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBackendOptions {
    pub dpi: u32,
}

impl Default for ImageBackendOptions {
    fn default() -> Self {
        Self { dpi: 150 }
    }
}
