use serde::{Deserialize, Serialize};

// ============================================================
// Base backend options
// ============================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaseBackendOptions {}

/// Options for declarative backends (HTML, MD, DOCX, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeclarativeBackendOptions {}

/// PDF-specific backend options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfBackendOptions {
    /// Page rendering DPI (affects image quality for OCR).
    pub dpi: u32,
    /// Whether to load the document's embedded fonts.
    pub load_fonts: bool,
}

impl Default for PdfBackendOptions {
    fn default() -> Self {
        Self {
            dpi: 144,
            load_fonts: true,
        }
    }
}

/// HTML-specific backend options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlBackendOptions {
    /// Resolve relative URLs against this base.
    pub base_url: Option<String>,
    /// Whether to inline linked CSS.
    pub resolve_css: bool,
}

impl Default for HtmlBackendOptions {
    fn default() -> Self {
        Self {
            base_url: None,
            resolve_css: false,
        }
    }
}

/// Markdown-specific backend options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownBackendOptions {
    /// Treat YAML front matter as metadata.
    pub parse_front_matter: bool,
}

impl Default for MarkdownBackendOptions {
    fn default() -> Self {
        Self {
            parse_front_matter: true,
        }
    }
}

/// LaTeX-specific backend options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexBackendOptions {
    /// Whether to expand custom macros.
    pub expand_macros: bool,
}

impl Default for LatexBackendOptions {
    fn default() -> Self {
        Self {
            expand_macros: true,
        }
    }
}

/// XBRL-specific backend options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XbrlBackendOptions {}

/// Generic backend options union.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendOptions {
    Base(BaseBackendOptions),
    Pdf(PdfBackendOptions),
    Html(HtmlBackendOptions),
    Markdown(MarkdownBackendOptions),
    Latex(LatexBackendOptions),
    Xbrl(XbrlBackendOptions),
    Declarative(DeclarativeBackendOptions),
}

impl Default for BackendOptions {
    fn default() -> Self {
        Self::Declarative(DeclarativeBackendOptions::default())
    }
}
