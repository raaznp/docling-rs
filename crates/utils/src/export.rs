use docling_core::DoclingDocument;
use std::fmt::Write as FmtWrite;

/// Export a DoclingDocument to Markdown.
/// More complete than the inline `export_to_markdown` on DoclingDocument.
pub fn to_markdown(doc: &DoclingDocument) -> String {
    doc.export_to_markdown()
}

/// Export a DoclingDocument to JSON (pretty-printed).
pub fn to_json(doc: &DoclingDocument) -> serde_json::Result<String> {
    doc.to_json_pretty()
}

/// Export a DoclingDocument to plain text (all text items joined by newlines).
pub fn to_text(doc: &DoclingDocument) -> String {
    use docling_core::DocItem;
    let mut out = String::new();
    for item in doc.iter_items() {
        match item {
            DocItem::Text(t) => {
                writeln!(out, "{}", t.text).ok();
            }
            DocItem::SectionHeader(h) => {
                writeln!(out, "{}", h.text).ok();
            }
            DocItem::ListItem(li) => {
                writeln!(out, "• {}", li.text).ok();
            }
            DocItem::Code(c) => {
                writeln!(out, "{}", c.text).ok();
            }
            DocItem::Formula(f) => {
                writeln!(out, "{}", f.text).ok();
            }
            _ => {}
        }
    }
    out
}
