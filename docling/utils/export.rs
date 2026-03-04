use crate::datamodel::base_models::Formatting;
use crate::datamodel::document::{Annotation, AnnotationKind, DocItem, DoclingDocument};

fn get_marker(kind: &AnnotationKind, start: bool) -> &str {
    match kind {
        AnnotationKind::Bold => "**",
        AnnotationKind::Italic => "*",
        AnnotationKind::Strikethrough => "~~",
        AnnotationKind::Underline => {
            if start {
                "<u>"
            } else {
                "</u>"
            }
        }
        AnnotationKind::Code => "`",
        AnnotationKind::Link { url: _ } => {
            if start {
                "["
            } else {
                "]"
            }
        }
    }
}

fn escape_text(text: &str) -> String {
    // Basic escaping for parity with Python docling exporter
    // Avoid double escaping by checking if the ampersand is already part of an entity
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '&' => {
                // Check if it's already an entity (e.g., &amp;, &lt;, etc.)
                let rest: String = chars.clone().take(4).collect();
                if rest.starts_with("amp;")
                    || rest.starts_with("lt;")
                    || rest.starts_with("gt;")
                    || rest.starts_with("quot;")
                    || rest.starts_with("apos;")
                {
                    result.push('&');
                } else {
                    result.push_str("&amp;");
                }
            }
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '_' => result.push_str("\\_"),
            _ => result.push(c),
        }
    }
    result
}

fn apply_annotations(text: &str, annotations: &[Annotation]) -> String {
    if annotations.is_empty() {
        return escape_text(text);
    }
    let mut events: Vec<(usize, bool, usize, usize, usize, &AnnotationKind)> = Vec::new();
    for (i, a) in annotations.iter().enumerate() {
        events.push((a.start, true, i, a.start, a.end, &a.kind));
        events.push((a.end, false, i, a.start, a.end, &a.kind));
    }
    // Sort:
    // 1. Position asc
    // 2. Ends (false) before Starts (true) at the same position
    // 3. For multiple Starts: longest range first (outermost), then lower index first
    // 4. For multiple Ends: shortest range first (innermost), then higher index first
    events.sort_by(|a, b| {
        if a.0 != b.0 {
            return a.0.cmp(&b.0);
        }
        if a.1 != b.1 {
            return a.1.cmp(&b.1); // false < true
        }
        if a.1 {
            let len_a = a.4.saturating_sub(a.3);
            let len_b = b.4.saturating_sub(b.3);
            if len_a != len_b {
                return len_b.cmp(&len_a);
            }
            a.2.cmp(&b.2)
        } else {
            let len_a = a.4.saturating_sub(a.3);
            let len_b = b.4.saturating_sub(b.3);
            if len_a != len_b {
                return len_a.cmp(&len_b);
            }
            b.2.cmp(&a.2)
        }
    });

    let mut result = String::new();
    let mut last_pos = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut active_code_count = 0;
    let mut just_ended_annotation = false;

    for (pos, is_start, _idx, _start, _end, kind) in events {
        // Add text before this event
        if pos > last_pos {
            let chunk: String = chars[last_pos..pos.min(chars.len())].iter().collect();
            // Parity space normalization: space before punctuation after annotation
            if just_ended_annotation
                && (chunk.starts_with('.') || chunk.starts_with(':') || chunk.starts_with(')'))
            {
                result.push(' ');
            }
            // Parity space normalization: space inside ( ... )
            if is_start && result.ends_with('(') && !chunk.starts_with(' ') {
                result.push(' ');
            }

            if active_code_count > 0 {
                result.push_str(&chunk);
            } else {
                result.push_str(&escape_text(&chunk));
            }
            last_pos = pos;
            just_ended_annotation = false;
        }

        if is_start && result.ends_with('(') {
            result.push(' ');
        }

        let marker = get_marker(kind, is_start);
        result.push_str(marker);
        if is_start {
            if let AnnotationKind::Code = kind {
                active_code_count += 1;
            }
        } else {
            just_ended_annotation = true;
            if let AnnotationKind::Code = kind {
                active_code_count -= 1;
            }
            if let AnnotationKind::Link { url } = kind {
                result.push_str(&format!("({})", url));
            }
        }
    }
    if last_pos < chars.len() {
        let chunk: String = chars[last_pos..].iter().collect();
        if just_ended_annotation
            && (chunk.starts_with('.') || chunk.starts_with(':') || chunk.starts_with(')'))
        {
            result.push(' ');
        }
        if active_code_count > 0 {
            result.push_str(&chunk);
        } else {
            result.push_str(&escape_text(&chunk));
        }
    }
    result
}

/// Export document to Markdown.
pub fn to_markdown(doc: &DoclingDocument) -> String {
    let mut md = String::new();
    let mut list_counters: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    let mut last_was_list_item: Option<bool> = None; // None: not list, Some(true): enumerated, Some(false): bullet

    for item in &doc.body {
        let current_is_list_item = match item {
            DocItem::ListItem(li) => Some(li.enumerated.unwrap_or(false)),
            _ => None,
        };

        if !md.is_empty() {
            let needs_double = match (last_was_list_item, current_is_list_item) {
                (Some(e1), Some(e2)) => e1 != e2,
                _ => true,
            };

            if needs_double {
                if !md.ends_with("\n\n") {
                    if md.ends_with('\n') {
                        md.push('\n');
                    } else {
                        md.push_str("\n\n");
                    }
                }
            } else {
                if !md.ends_with('\n') {
                    md.push('\n');
                }
            }
        }
        last_was_list_item = current_is_list_item;
        match item {
            DocItem::SectionHeader(h) => {
                let prefix = "#".repeat(h.level as usize);
                let text = apply_annotations(&h.text, &h.annotations);
                md.push_str(&format!("{} {}\n", prefix, text));
            }
            DocItem::Text(t) => {
                let text = apply_annotations(&t.text, &t.annotations);
                md.push_str(&format!("{}\n", text));
            }
            DocItem::ListItem(li) => {
                let marker = if li.enumerated == Some(true) {
                    let parent_id = li.id.rsplit_once('/').map(|(p, _)| p).unwrap_or("root");
                    let count = list_counters.entry(parent_id.to_string()).or_insert(0);
                    *count += 1;
                    format!("{}.", count)
                } else {
                    li.marker.as_deref().unwrap_or("-").to_string()
                };
                let mut text = apply_annotations(&li.text, &li.annotations);
                // Parity: special spacing for digit followed by period at start of list item text
                if let Some(c) = text.chars().next() {
                    if c.is_ascii_digit() && text.chars().nth(1) == Some('.') {
                        text.insert(1, ' ');
                    }
                }
                let indent = "    ".repeat(li.level as usize);
                md.push_str(&format!("{}{} {}\n", indent, marker, text));
            }
            DocItem::Code(c) => {
                let lang = c.code_language.as_deref().unwrap_or("");
                md.push_str(&format!("```{}\n{}\n```\n", lang, c.text));
            }
            DocItem::Table(t) => {
                // Proper table export with padding for parity
                let num_rows = t.data.num_rows as usize;
                let num_cols = t.data.num_cols as usize;
                if num_rows > 0 && num_cols > 0 {
                    let mut grid: Vec<String> = vec![String::new(); num_rows * num_cols];
                    for cell in &t.data.table_cells {
                        let r = cell.start_row as usize;
                        let c = cell.start_col as usize;
                        if r < num_rows && c < num_cols {
                            grid[r * num_cols + c] = cell.text.clone();
                        }
                    }

                    let mut col_widths = vec![0; num_cols];
                    for r in 0..num_rows {
                        for c in 0..num_cols {
                            let cell_text = grid[r * num_cols + c].replace('|', "&#124;");
                            col_widths[c] = col_widths[c].max(cell_text.len());
                        }
                    }

                    for r in 0..num_rows {
                        md.push_str("| ");
                        for c in 0..num_cols {
                            let cell_text = grid[r * num_cols + c].replace('|', "&#124;");
                            md.push_str(&cell_text);
                            md.push_str(
                                &" ".repeat((col_widths[c] + 1).saturating_sub(cell_text.len())),
                            );
                            md.push_str("|");
                            if c < num_cols - 1 {
                                md.push(' ');
                            }
                        }
                        md.push_str("\n");
                        if r == 0 {
                            md.push_str("|");
                            for c in 0..num_cols {
                                md.push_str(&"-".repeat(col_widths[c] + 2));
                                md.push_str("|");
                            }
                            md.push_str("\n");
                        }
                    }
                    md.push_str("\n");
                }
            }
            DocItem::Picture(p) => {
                let desc = p.description.as_deref().unwrap_or("picture");
                md.push_str(&format!("![{}]({})\n", desc, p.id));
            }
            _ => {
                last_was_list_item = None;
            }
        }
    }
    md.trim().to_string()
}

/// Export document to plain text.
pub fn to_text(doc: &DoclingDocument) -> String {
    let mut text = String::new();
    for item in &doc.body {
        match item {
            DocItem::SectionHeader(h) => text.push_str(&format!("{}\n\n", h.text)),
            DocItem::Text(t) => text.push_str(&format!("{}\n\n", t.text)),
            DocItem::ListItem(li) => text.push_str(&format!("* {}\n", li.text)),
            DocItem::Code(c) => text.push_str(&format!("{}\n\n", c.text)),
            DocItem::Table(t) => {
                for cell in &t.data.table_cells {
                    text.push_str(&format!("{}\t", cell.text));
                }
                text.push('\n');
            }
            _ => {}
        }
    }
    text.trim().to_string()
}

/// Export document to JSON.
pub fn to_json(doc: &DoclingDocument) -> String {
    serde_json::to_string_pretty(doc).unwrap_or_default()
}

/// Export document to HTML.
pub fn to_html(doc: &DoclingDocument) -> String {
    let mut html = String::new();
    html.push_str("<html><body>\n");
    for item in &doc.body {
        match item {
            DocItem::SectionHeader(h) => {
                let lvl = if h.level > 6 { 6 } else { h.level };
                let text = apply_annotations(&h.text, &h.annotations);
                html.push_str(&format!("<h{}>{}</h{}>\n", lvl, text, lvl));
            }
            DocItem::Text(t) => {
                let text = apply_annotations(&t.text, &t.annotations);
                html.push_str(&format!("<p>{}</p>\n", text));
            }
            DocItem::ListItem(li) => html.push_str(&format!("<li>{}</li>\n", li.text)),
            DocItem::Code(c) => html.push_str(&format!("<pre><code>{}</code></pre>\n", c.text)),
            _ => {}
        }
    }
    html.push_str("</body></html>");
    html
}

/// Export document to DocTags.
pub fn to_doctags(doc: &DoclingDocument) -> String {
    let mut tags = String::new();
    for item in &doc.body {
        match item {
            DocItem::SectionHeader(h) => {
                tags.push_str(&format!("<section_header>{}</section_header>\n", h.text))
            }
            DocItem::Text(t) => tags.push_str(&format!("<text>{}</text>\n", t.text)),
            _ => {}
        }
    }
    tags
}
