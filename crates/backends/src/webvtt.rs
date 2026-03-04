use docling_core::{
    base_models::InputFormat,
    doc_types::{
        DoclingDocument, DocumentOrigin, ListItem, SectionHeaderItem, TableData, TableItem,
        TextItem,
    },
    errors::{DoclingError, Result},
    LayoutLabel,
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// WebVTT backend.
/// Converts WebVTT subtitle/caption files into a DoclingDocument.
/// Mirrors `docling/backend/webvtt_backend.py`.
pub struct WebVttBackend {
    source: BackendSource,
    valid: bool,
}

impl WebVttBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }

    fn parse(&self, content: &str, name: &str) -> Result<DoclingDocument> {
        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "text/vtt".to_string(),
            binary_hash: None,
            uri: None,
        });

        let mut lines = content.lines().peekable();
        let mut cue_text_lines: Vec<String> = Vec::new();
        let mut in_cue = false;

        // Skip "WEBVTT" header
        if let Some(first) = lines.peek() {
            if first.trim_start().starts_with("WEBVTT") {
                lines.next();
            }
        }

        let flush_cue = |doc: &mut DoclingDocument, lines: &mut Vec<String>| {
            let text = lines.join(" ").trim().to_string();
            if !text.is_empty() {
                let id = format!("#/texts/{}", doc.body.len());
                doc.add_text(TextItem {
                    id,
                    text,
                    label: LayoutLabel::Text,
                    prov: vec![],
                    orig: None,
                    enumerated: None,
                    marker: None,
                });
            }
            lines.clear();
        };

        for line in lines {
            let trimmed = line.trim();

            // Empty line: end of cue
            if trimmed.is_empty() {
                if in_cue {
                    flush_cue(&mut doc, &mut cue_text_lines);
                    in_cue = false;
                }
                continue;
            }

            // Cue timing line: contains " --> "
            if trimmed.contains(" --> ") {
                in_cue = true;
                continue;
            }

            // Numeric-only or NOTE / STYLE / REGION blocks — skip
            if trimmed.chars().all(|c| c.is_numeric()) {
                continue;
            }
            if trimmed.starts_with("NOTE")
                || trimmed.starts_with("STYLE")
                || trimmed.starts_with("REGION")
            {
                in_cue = false;
                continue;
            }

            if in_cue {
                // Strip VTT tags like <c>, <v Speaker>, etc.
                let clean = strip_vtt_tags(trimmed);
                cue_text_lines.push(clean);
            }
        }

        flush_cue(&mut doc, &mut cue_text_lines);

        Ok(doc)
    }
}

/// Strip VTT inline tags from a cue text string.
fn strip_vtt_tags(text: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

impl DocumentBackend for WebVttBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Vtt]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for WebVttBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let content = String::from_utf8_lossy(&bytes).into_owned();
        let name = self.source.name().to_string();
        self.parse(&content, &name)
    }
}
