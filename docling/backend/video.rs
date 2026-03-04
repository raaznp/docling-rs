use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, TextItem};
use crate::errors::Result;

/// Video backend — MP4, AVI, MOV. Audio track extracted and transcribed via ASR.
pub struct VideoBackend {
    source: BackendSource,
    valid: bool,
}

impl VideoBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for VideoBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Mp4, InputFormat::Avi, InputFormat::Mov]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for VideoBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "video/*".into(),
            binary_hash: None,
            uri: None,
        });
        // TODO(asr): extract audio via ffmpeg, then run Whisper ONNX.
        doc.add_text(TextItem {
            id: "#/texts/0".to_string(),
            text: format!("<video source=\"{}\">", self.source.name()),
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
