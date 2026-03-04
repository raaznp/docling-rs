use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, TextItem};
use crate::errors::Result;

/// Audio backend — WAV, MP3, M4A, AAC, OGG, FLAC.
/// Requires the 'asr' feature for actual transcription.
pub struct AudioBackend {
    source: BackendSource,
    valid: bool,
}

impl AudioBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for AudioBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[
            InputFormat::Wav,
            InputFormat::Mp3,
            InputFormat::M4a,
            InputFormat::Aac,
            InputFormat::Ogg,
            InputFormat::Flac,
        ]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for AudioBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "audio/*".into(),
            binary_hash: None,
            uri: None,
        });
        // TODO(asr): run Whisper ONNX model for transcription.
        doc.add_text(TextItem {
            id: "#/texts/0".to_string(),
            text: format!("<audio source=\"{}\">", name),
            label: LayoutLabel::Text,
            prov: vec![],
            orig: None,
            enumerated: None,
            marker: None,
            formatting: None,
            hyperlink: None,
            annotations: vec![],
        });
        Ok(doc)
    }
}
