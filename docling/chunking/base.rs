use crate::datamodel::document::DoclingDocument;

/// Estimate token count (GPT heuristic: ~4 chars/token).
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

/// A single document chunk.
#[derive(Debug, Clone)]
pub struct DocChunk {
    pub text: String,
    pub item_ids: Vec<String>,
    pub token_count: usize,
    pub headings: Vec<String>,
}

impl DocChunk {
    pub fn new(text: String, item_ids: Vec<String>, headings: Vec<String>) -> Self {
        let token_count = estimate_tokens(&text);
        Self {
            text,
            item_ids,
            token_count,
            headings,
        }
    }
}

/// Trait all chunkers implement.
pub trait BaseChunker: Send + Sync {
    fn chunk(&self, doc: &DoclingDocument) -> Vec<DocChunk>;
}
