use crate::chunking::base::{estimate_tokens, BaseChunker, DocChunk};
use crate::chunking::hierarchical::HierarchicalChunker;
use crate::datamodel::document::DoclingDocument;

/// HybridChunker — combines hierarchical splits with sentence boundaries.
pub struct HybridChunker {
    pub max_tokens: usize,
    pub min_tokens: usize,
}

impl HybridChunker {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            min_tokens: 64,
        }
    }
}

impl Default for HybridChunker {
    fn default() -> Self {
        Self::new(512)
    }
}

impl BaseChunker for HybridChunker {
    fn chunk(&self, doc: &DoclingDocument) -> Vec<DocChunk> {
        let coarse = HierarchicalChunker::new(self.max_tokens).chunk(doc);
        let mut refined: Vec<DocChunk> = Vec::new();
        for chunk in coarse {
            if chunk.token_count <= self.max_tokens {
                refined.push(chunk);
                continue;
            }
            let mut sub = String::new();
            for sentence in split_sentences(&chunk.text) {
                if estimate_tokens(&sub) + estimate_tokens(sentence) > self.max_tokens {
                    if !sub.trim().is_empty() {
                        refined.push(DocChunk::new(
                            sub.trim().to_string(),
                            chunk.item_ids.clone(),
                            chunk.headings.clone(),
                        ));
                        sub.clear();
                    }
                }
                sub.push_str(sentence);
                sub.push(' ');
            }
            if !sub.trim().is_empty() {
                refined.push(DocChunk::new(
                    sub.trim().to_string(),
                    chunk.item_ids.clone(),
                    chunk.headings.clone(),
                ));
            }
        }
        merge_small(refined, self.max_tokens, self.min_tokens)
    }
}

fn split_sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        if matches!(bytes[i], b'.' | b'!' | b'?') && bytes[i + 1] == b' ' {
            out.push(&text[start..=i + 1]);
            start = i + 2;
        }
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    if out.is_empty() {
        out.push(text);
    }
    out
}

fn merge_small(chunks: Vec<DocChunk>, max: usize, min: usize) -> Vec<DocChunk> {
    let mut out: Vec<DocChunk> = Vec::new();
    for chunk in chunks {
        if let Some(last) = out.last_mut() {
            let combined = last.token_count + chunk.token_count;
            if last.headings == chunk.headings
                && (last.token_count < min || chunk.token_count < min)
                && combined <= max
            {
                last.text.push('\n');
                last.text.push_str(&chunk.text);
                last.token_count = combined;
                last.item_ids.extend(chunk.item_ids);
                continue;
            }
        }
        out.push(chunk);
    }
    out
}
