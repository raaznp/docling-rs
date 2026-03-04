use crate::chunking::base::{estimate_tokens, BaseChunker, DocChunk};
use crate::datamodel::document::{DocItem, DoclingDocument};

/// Splits documents into chunks that respect the heading hierarchy.
pub struct HierarchicalChunker {
    pub max_tokens: usize,
}

impl HierarchicalChunker {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }
}

impl Default for HierarchicalChunker {
    fn default() -> Self {
        Self::new(512)
    }
}

impl BaseChunker for HierarchicalChunker {
    fn chunk(&self, doc: &DoclingDocument) -> Vec<DocChunk> {
        let mut chunks: Vec<DocChunk> = Vec::new();
        let mut cur_text = String::new();
        let mut cur_ids: Vec<String> = Vec::new();
        let mut cur_headings: Vec<String> = Vec::new();

        let flush = |text: &mut String,
                     ids: &mut Vec<String>,
                     headings: &Vec<String>,
                     chunks: &mut Vec<DocChunk>| {
            if !text.trim().is_empty() {
                chunks.push(DocChunk::new(
                    text.trim().to_string(),
                    std::mem::take(ids),
                    headings.clone(),
                ));
                text.clear();
            }
        };

        for item in &doc.body {
            match item {
                DocItem::SectionHeader(h) => {
                    flush(&mut cur_text, &mut cur_ids, &cur_headings, &mut chunks);
                    let lvl = h.level as usize;
                    cur_headings.truncate(lvl.saturating_sub(1));
                    cur_headings.push(h.text.clone());
                }
                DocItem::Text(t) => {
                    if estimate_tokens(&cur_text) + estimate_tokens(&t.text) > self.max_tokens {
                        flush(&mut cur_text, &mut cur_ids, &cur_headings, &mut chunks);
                    }
                    if !cur_text.is_empty() {
                        cur_text.push('\n');
                    }
                    cur_text.push_str(&t.text);
                    cur_ids.push(t.id.clone());
                }
                DocItem::ListItem(l) => {
                    let bullet = format!("- {}", l.text);
                    if estimate_tokens(&cur_text) + estimate_tokens(&bullet) > self.max_tokens {
                        flush(&mut cur_text, &mut cur_ids, &cur_headings, &mut chunks);
                    }
                    cur_text.push('\n');
                    cur_text.push_str(&bullet);
                    cur_ids.push(l.id.clone());
                }
                DocItem::Table(t) => {
                    flush(&mut cur_text, &mut cur_ids, &cur_headings, &mut chunks);
                    chunks.push(DocChunk::new(
                        format!(
                            "[Table: {} rows × {} cols]",
                            t.data.num_rows, t.data.num_cols
                        ),
                        vec![t.id.clone()],
                        cur_headings.clone(),
                    ));
                }
                _ => {}
            }
        }
        flush(&mut cur_text, &mut cur_ids, &cur_headings, &mut chunks);
        chunks
    }
}
