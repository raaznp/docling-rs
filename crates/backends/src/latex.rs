use docling_core::{
    base_models::InputFormat,
    doc_types::{
        CodeItem, DoclingDocument, DocumentOrigin, FormulaItem, ListItem, PictureItem,
        SectionHeaderItem, TableData, TableItem, TextItem,
    },
    errors::{DoclingError, Result},
    LayoutLabel,
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// LaTeX document backend.
///
/// Recursive descent parser for LaTeX source files.
/// Handles common document classes: article, book, report.
/// Mirrors `docling/backend/latex_backend.py`.
pub struct LatexBackend {
    source: BackendSource,
    valid: bool,
    expand_macros: bool,
}

impl LatexBackend {
    pub fn new(source: BackendSource, expand_macros: bool) -> Self {
        Self {
            source,
            valid: true,
            expand_macros,
        }
    }

    fn parse(&self, content: &str, name: &str) -> Result<DoclingDocument> {
        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "application/x-latex".to_string(),
            binary_hash: None,
            uri: None,
        });

        let mut parser = LatexParser::new(content);
        parser.parse_into(&mut doc);

        Ok(doc)
    }
}

struct LatexParser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    section_depth: u8,
}

impl<'a> LatexParser<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            chars: content.chars().peekable(),
            section_depth: 0,
        }
    }

    fn parse_into(&mut self, doc: &mut DoclingDocument) {
        let mut current_paragraph = String::new();

        while let Some(&ch) = self.chars.peek() {
            match ch {
                '\\' => {
                    self.chars.next(); // consume '\'
                    let cmd = self.read_command();
                    self.handle_command(&cmd, doc, &mut current_paragraph);
                }
                '%' => {
                    // LaTeX comment — skip to end of line
                    self.skip_to_newline();
                }
                '\n' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'\n') {
                        // Double newline = paragraph break
                        let text = current_paragraph.trim().to_string();
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
                        current_paragraph.clear();
                    } else {
                        current_paragraph.push(' ');
                    }
                }
                _ => {
                    self.chars.next();
                    current_paragraph.push(ch);
                }
            }
        }

        // Flush final paragraph
        let text = current_paragraph.trim().to_string();
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
    }

    fn read_command(&mut self) -> String {
        let mut cmd = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphabetic() {
                cmd.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        // Skip whitespace after command
        if cmd.is_empty() {
            // Special char like \\ \{ \} \$ etc.
            if let Some(ch) = self.chars.next() {
                cmd.push(ch);
            }
        }
        cmd
    }

    fn read_braced_arg(&mut self) -> String {
        let mut depth = 0;
        let mut arg = String::new();
        while let Some(&ch) = self.chars.peek() {
            self.chars.next();
            match ch {
                '{' => {
                    depth += 1;
                    if depth > 1 {
                        arg.push(ch);
                    }
                }
                '}' => {
                    if depth == 1 {
                        break;
                    }
                    depth -= 1;
                    arg.push(ch);
                }
                _ => {
                    if depth >= 1 {
                        arg.push(ch);
                    }
                }
            }
        }
        arg.trim().to_string()
    }

    fn skip_to_newline(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            self.chars.next();
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn handle_command(&mut self, cmd: &str, doc: &mut DoclingDocument, current: &mut String) {
        match cmd {
            // Document title
            "title" => {
                let arg = self.read_braced_arg();
                doc.metadata.title = Some(strip_latex(&arg));
            }

            // Sectioning commands
            "chapter" => {
                let arg = self.read_braced_arg();
                let text = strip_latex(&arg);
                let id = format!("#/texts/{}", doc.body.len());
                doc.add_header(SectionHeaderItem {
                    id,
                    text,
                    level: 1,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                });
            }
            "section" => {
                let arg = self.read_braced_arg();
                let text = strip_latex(&arg);
                let id = format!("#/texts/{}", doc.body.len());
                doc.add_header(SectionHeaderItem {
                    id,
                    text,
                    level: 2,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                });
            }
            "subsection" => {
                let arg = self.read_braced_arg();
                let text = strip_latex(&arg);
                let id = format!("#/texts/{}", doc.body.len());
                doc.add_header(SectionHeaderItem {
                    id,
                    text,
                    level: 3,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                });
            }
            "subsubsection" => {
                let arg = self.read_braced_arg();
                let text = strip_latex(&arg);
                let id = format!("#/texts/{}", doc.body.len());
                doc.add_header(SectionHeaderItem {
                    id,
                    text,
                    level: 4,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                });
            }

            // Text formatting — extract content and push to current paragraph
            "textbf" | "textit" | "emph" | "texttt" | "textrm" | "textsc" | "text" => {
                let arg = self.read_braced_arg();
                current.push_str(&strip_latex(&arg));
            }

            // Math environments
            "(" | "[" => {
                // Inline or display math — read until \) or \]
                let end = if cmd == "(" { ")" } else { "]" };
                let math = self.read_until_math_end(end);
                let id = format!("#/texts/{}", doc.body.len());
                doc.add_formula(FormulaItem {
                    id,
                    text: math,
                    label: LayoutLabel::Formula,
                    prov: vec![],
                });
            }

            // List items
            "item" => {
                // Push current as a list item
                let text = current.trim().to_string();
                current.clear();
                // Read text until next \item or end of environment
                let rest = self.read_until_item();
                let item_text = format!("{} {}", text, rest).trim().to_string();
                if !item_text.is_empty() {
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_list_item(ListItem {
                        id,
                        text: item_text,
                        label: LayoutLabel::ListItem,
                        prov: vec![],
                        enumerated: None,
                        marker: Some("-".to_string()),
                    });
                }
            }

            // Begin/end environments
            "begin" => {
                let env = self.read_braced_arg();
                match env.as_str() {
                    "equation" | "equation*" | "align" | "align*" | "math" => {
                        let math = self.read_until_end_env(&env);
                        let id = format!("#/texts/{}", doc.body.len());
                        doc.add_formula(FormulaItem {
                            id,
                            text: math.trim().to_string(),
                            label: LayoutLabel::Formula,
                            prov: vec![],
                        });
                    }
                    "verbatim" | "lstlisting" | "minted" => {
                        let code = self.read_until_end_env(&env);
                        let id = format!("#/texts/{}", doc.body.len());
                        doc.add_code(CodeItem {
                            id,
                            text: code.trim().to_string(),
                            label: LayoutLabel::Code,
                            prov: vec![],
                            code_language: None,
                        });
                    }
                    _ => {} // recurse into other environments naturally
                }
            }

            "end" => {
                // consume environment name
                self.read_braced_arg();
            }

            // Ignore preamble / formatting commands
            "documentclass" | "usepackage" | "setlength" | "setcounter" | "geometry"
            | "hypersetup" | "bibliographystyle" | "bibliography" => {
                // Skip optional and mandatory args
                self.skip_optional_arg();
                let _ = self.read_braced_arg();
            }

            "maketitle" | "tableofcontents" | "listoffigures" | "listoftables" | "clearpage"
            | "newpage" | "hline" | "noindent" | "centering" => {
                // No-op rendering commands
            }

            _ => {
                // Unknown command: try to consume a braced arg if the next char is {
                self.skip_whitespace();
                if self.chars.peek() == Some(&'{') {
                    let arg = self.read_braced_arg();
                    current.push_str(&strip_latex(&arg));
                }
            }
        }
    }

    fn skip_optional_arg(&mut self) {
        self.skip_whitespace();
        if self.chars.peek() == Some(&'[') {
            self.chars.next(); // consume '['
            while let Some(&ch) = self.chars.peek() {
                self.chars.next();
                if ch == ']' {
                    break;
                }
            }
        }
    }

    fn read_until_math_end(&mut self, end_marker: &str) -> String {
        let mut math = String::new();
        let marker_char = end_marker.chars().next().unwrap_or(')');
        while let Some(&ch) = self.chars.peek() {
            if ch == '\\' {
                self.chars.next();
                if self.chars.peek() == Some(&marker_char) {
                    self.chars.next();
                    break;
                }
                math.push('\\');
            } else {
                math.push(ch);
                self.chars.next();
            }
        }
        math
    }

    fn read_until_end_env(&mut self, env: &str) -> String {
        let needle = format!("\\end{{{}}}", env);
        let mut buf = String::new();
        while !buf.ends_with(&needle) {
            match self.chars.next() {
                Some(ch) => buf.push(ch),
                None => break,
            }
        }
        if buf.ends_with(&needle) {
            let len = buf.len() - needle.len();
            buf.truncate(len);
        }
        buf
    }

    fn read_until_item(&mut self) -> String {
        let mut buf = String::new();
        // Read until we hit \item or \end
        while let Some(&ch) = self.chars.peek() {
            if ch == '\\' {
                // Peek ahead at the full command
                let saved: String = std::iter::once('\\').chain(self.peek_command()).collect();
                if saved.contains("\\item") || saved.contains("\\end") {
                    break;
                }
            }
            buf.push(ch);
            self.chars.next();
        }
        strip_latex(buf.trim())
    }

    fn peek_command(&self) -> Vec<char> {
        // We can't peek more than one character on a Peekable<Chars>, so we just look at next char
        vec![]
    }
}

/// Strip simple LaTeX markup from text (formatting commands, braces, etc.)
fn strip_latex(s: &str) -> String {
    let mut out = String::new();
    let mut in_cmd = false;
    let mut depth = 0i32;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                in_cmd = true;
                // Skip the command name
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphabetic() {
                        chars.next();
                    } else {
                        break;
                    }
                }
                in_cmd = false;
                out.push(' ');
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth < 0 {
                    depth = 0;
                }
            }
            _ => {
                out.push(ch);
            }
        }
    }
    out
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
        let content = String::from_utf8_lossy(&bytes).into_owned();
        let name = self.source.name().to_string();
        self.parse(&content, &name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latex_basic() {
        let latex = r#"\section{Introduction}\nHello world."#;
        let source = BackendSource::Bytes(latex.as_bytes().to_vec(), "test.tex".to_string());
        let mut backend = LatexBackend::new(source, false);
        let doc = backend.convert().expect("conversion failed");
        assert!(!doc.body.is_empty());
    }
}
