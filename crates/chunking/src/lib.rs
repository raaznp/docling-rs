//! Chunking crate for Docling-rs.
//!
//! Provides hierarchical chunking of DoclingDocument for downstream
//! LLM / RAG pipelines. Mirrors `docling/chunking/`.

use docling_core::{DocItem, DoclingDocument};
use serde::{Deserialize, Serialize};

/// A single chunk of document content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Chunk text content.
    pub text: String,
    /// Heading path above this chunk (for hierarchical context).
    pub headings: Vec<String>,
    /// Page number(s) this chunk spans.
    pub pages: Vec<u32>,
    /// Character start/end in the original document text.
    pub char_span: [usize; 2],
    /// Source document items included in this chunk.
    pub items: Vec<ChunkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkItem {
    pub label: String,
    pub text: String,
}

/// Chunking options.
#[derive(Debug, Clone)]
pub struct ChunkOptions {
    /// Maximum number of tokens per chunk (approximate).
    pub max_tokens: usize,
    /// Overlap between consecutive chunks (in tokens).
    pub overlap_tokens: usize,
    /// Whether to include heading context in chunk text.
    pub include_context_headings: bool,
}

impl Default for ChunkOptions {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            overlap_tokens: 64,
            include_context_headings: true,
        }
    }
}

/// Hierarchical chunker.
///
/// Splits a DoclingDocument into chunks that respect document structure:
/// chunks never split heading sections across chunks.
pub struct HierarchicalChunker {
    options: ChunkOptions,
}

impl HierarchicalChunker {
    pub fn new(options: ChunkOptions) -> Self {
        Self { options }
    }

    /// Chunk a document into a list of Chunk objects.
    pub fn chunk(&self, doc: &DoclingDocument) -> Vec<Chunk> {
        let mut chunks: Vec<Chunk> = Vec::new();
        let mut heading_stack: Vec<String> = Vec::new();
        let mut current_items: Vec<ChunkItem> = Vec::new();
        let mut current_text = String::new();
        let mut current_pages: Vec<u32> = Vec::new();

        let flush = |chunks: &mut Vec<Chunk>,
                     items: &mut Vec<ChunkItem>,
                     text: &mut String,
                     pages: &mut Vec<u32>,
                     headings: &Vec<String>| {
            if !items.is_empty() {
                chunks.push(Chunk {
                    text: text.trim().to_string(),
                    headings: headings.clone(),
                    pages: pages.clone(),
                    char_span: [0, text.len()],
                    items: items.drain(..).collect(),
                });
                text.clear();
                pages.clear();
            }
        };

        for item in doc.iter_items() {
            // Approximate token count (4 chars ≈ 1 token)
            let item_tokens = match item {
                DocItem::Text(t) => t.text.len() / 4,
                DocItem::SectionHeader(h) => h.text.len() / 4,
                DocItem::ListItem(li) => li.text.len() / 4,
                _ => 50,
            };

            // If adding this item would exceed max_tokens, flush current chunk
            if !current_items.is_empty()
                && (current_text.len() / 4) + item_tokens > self.options.max_tokens
            {
                flush(
                    &mut chunks,
                    &mut current_items,
                    &mut current_text,
                    &mut current_pages,
                    &heading_stack,
                );
            }

            match item {
                DocItem::SectionHeader(h) => {
                    // Flush before a new section
                    flush(
                        &mut chunks,
                        &mut current_items,
                        &mut current_text,
                        &mut current_pages,
                        &heading_stack,
                    );

                    // Update heading stack
                    let level = (h.level as usize).saturating_sub(1);
                    heading_stack.truncate(level);
                    heading_stack.push(h.text.clone());

                    if self.options.include_context_headings {
                        current_text.push_str(&h.text);
                        current_text.push('\n');
                    }
                    current_items.push(ChunkItem {
                        label: "section_header".to_string(),
                        text: h.text.clone(),
                    });
                }
                DocItem::Text(t) => {
                    current_text.push_str(&t.text);
                    current_text.push(' ');
                    let page = t.prov.first().map(|p| p.page_no);
                    if let Some(p) = page {
                        if !current_pages.contains(&p) {
                            current_pages.push(p);
                        }
                    }
                    current_items.push(ChunkItem {
                        label: "text".to_string(),
                        text: t.text.clone(),
                    });
                }
                DocItem::ListItem(li) => {
                    current_text.push_str(&li.text);
                    current_text.push('\n');
                    current_items.push(ChunkItem {
                        label: "list_item".to_string(),
                        text: li.text.clone(),
                    });
                }
                DocItem::Table(t) => {
                    // Each table is its own chunk
                    flush(
                        &mut chunks,
                        &mut current_items,
                        &mut current_text,
                        &mut current_pages,
                        &heading_stack,
                    );
                    let table_text: String = t
                        .data
                        .table_cells
                        .iter()
                        .map(|c| c.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    chunks.push(Chunk {
                        text: table_text.clone(),
                        headings: heading_stack.clone(),
                        pages: vec![],
                        char_span: [0, table_text.len()],
                        items: vec![ChunkItem {
                            label: "table".to_string(),
                            text: table_text,
                        }],
                    });
                }
                _ => {}
            }
        }

        flush(
            &mut chunks,
            &mut current_items,
            &mut current_text,
            &mut current_pages,
            &heading_stack,
        );

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docling_core::DoclingDocument;

    #[test]
    fn test_chunker_empty() {
        let doc = DoclingDocument::new("test");
        let chunker = HierarchicalChunker::new(ChunkOptions::default());
        let chunks = chunker.chunk(&doc);
        assert!(chunks.is_empty());
    }
}
