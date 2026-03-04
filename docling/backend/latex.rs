use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{
    CodeItem, DoclingDocument, DocumentOrigin, FormulaItem, SectionHeaderItem, TextItem,
};
use crate::errors::Result;

pub struct LatexBackend {
    source: BackendSource,
    valid: bool,
}
impl LatexBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for LatexBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Latex]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for LatexBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let content = String::from_utf8_lossy(&bytes).to_string();
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name,
            mime_type: "application/x-latex".into(),
            binary_hash: None,
            uri: None,
        });

        let mut idx = 0usize;
        let mut in_verbatim = false;
        let mut verbatim_buf = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\\begin{verbatim}")
                || trimmed.starts_with("\\begin{lstlisting}")
            {
                in_verbatim = true;
                verbatim_buf.clear();
                continue;
            }
            if trimmed.starts_with("\\end{verbatim}") || trimmed.starts_with("\\end{lstlisting}") {
                doc.add_code(CodeItem {
                    id: format!("#/texts/{}", idx),
                    text: verbatim_buf.trim().to_string(),
                    label: LayoutLabel::Code,
                    prov: vec![],
                    code_language: None,
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
                in_verbatim = false;
                idx += 1;
                continue;
            }
            if in_verbatim {
                verbatim_buf.push_str(line);
                verbatim_buf.push('\n');
                continue;
            }

            if let Some(rest) = trimmed
                .strip_prefix("\\section{")
                .and_then(|s| s.strip_suffix('}'))
            {
                doc.add_header(SectionHeaderItem {
                    id: format!("#/texts/{}", idx),
                    text: rest.to_string(),
                    level: 1,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else if let Some(rest) = trimmed
                .strip_prefix("\\subsection{")
                .and_then(|s| s.strip_suffix('}'))
            {
                doc.add_header(SectionHeaderItem {
                    id: format!("#/texts/{}", idx),
                    text: rest.to_string(),
                    level: 2,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else if trimmed.contains("\\[") || trimmed.contains("\\begin{equation}") {
                doc.add_formula(FormulaItem {
                    id: format!("#/texts/{}", idx),
                    text: trimmed.to_string(),
                    label: LayoutLabel::Formula,
                    prov: vec![],
                });
            } else if !trimmed.is_empty() && !trimmed.starts_with('%') && !trimmed.starts_with('\\')
            {
                doc.add_text(TextItem {
                    id: format!("#/texts/{}", idx),
                    text: line.trim().to_string(),
                    label: LayoutLabel::Text,
                    prov: vec![],
                    orig: None,
                    enumerated: None,
                    marker: None,
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else {
                continue;
            }
            idx += 1;
        }
        Ok(doc)
    }
}
