use crate::datamodel::document::ConversionResult;
use crate::errors::Result;

// ── BasePipeline ─────────────────────────────────────────────────

pub trait BasePipeline: Send + Sync {
    fn name(&self) -> &str;

    fn execute(&self, mut conv_res: ConversionResult, _raises_on_error: bool) -> ConversionResult {
        use crate::datamodel::base_models::ConversionStatus;
        use std::time::Instant;

        let start = Instant::now();
        conv_res.status = ConversionStatus::Started;

        let result = (|| -> Result<ConversionResult> {
            let conv_res = self.build_document(conv_res)?;
            let conv_res = self.assemble_document(conv_res)?;
            let conv_res = self.enrich_document(conv_res)?;
            Ok(conv_res)
        })();

        match result {
            Ok(mut cr) => {
                cr.timings
                    .record("pipeline_total", start.elapsed().as_secs_f64());
                cr.status = self.determine_status(&cr);
                cr
            }
            Err(e) => {
                let mut cr_fail = ConversionResult::empty_failure();
                cr_fail.status = ConversionStatus::Failure;
                cr_fail
                    .errors
                    .push(crate::datamodel::base_models::ErrorItem::new(
                        crate::datamodel::base_models::DoclingComponentType::Pipeline,
                        self.name(),
                        e.to_string(),
                    ));
                cr_fail
            }
        }
    }

    fn build_document(&self, conv_res: ConversionResult) -> Result<ConversionResult>;

    fn assemble_document(&self, conv_res: ConversionResult) -> Result<ConversionResult> {
        Ok(conv_res)
    }

    fn enrich_document(&self, conv_res: ConversionResult) -> Result<ConversionResult> {
        Ok(conv_res)
    }

    fn determine_status(
        &self,
        conv_res: &ConversionResult,
    ) -> crate::datamodel::base_models::ConversionStatus {
        use crate::datamodel::base_models::ConversionStatus;
        match conv_res.status.clone() {
            ConversionStatus::Pending | ConversionStatus::Started => ConversionStatus::Success,
            other => other,
        }
    }
}

// ── PaginatedPipeline ────────────────────────────────────────────

pub struct PaginatedPipeline {
    pub build_pipe: Vec<Box<dyn crate::models::BuildModel>>,
    pub enrichment_pipe: Vec<Box<dyn crate::models::EnrichmentModel>>,
}

impl PaginatedPipeline {
    pub fn new(
        build_pipe: Vec<Box<dyn crate::models::BuildModel>>,
        enrichment_pipe: Vec<Box<dyn crate::models::EnrichmentModel>>,
    ) -> Self {
        Self {
            build_pipe,
            enrichment_pipe,
        }
    }

    pub fn run_build_pipe(
        &self,
        conv_res: &mut ConversionResult,
        pages: &mut Vec<crate::datamodel::base_models::Page>,
    ) -> Result<()> {
        for model in &self.build_pipe {
            if model.is_enabled() {
                model.process_pages(conv_res, pages)?;
            }
        }
        Ok(())
    }

    pub fn run_enrichment_pipe(&self, conv_res: &mut ConversionResult) -> Result<()> {
        if let Some(doc) = conv_res.document.as_mut() {
            for model in &self.enrichment_pipe {
                if !model.is_enabled() {
                    continue;
                }
                let indices = doc
                    .body
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, item)| {
                        if model.prepare_element(item) {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if !indices.is_empty() {
                    model.process_batch(doc, &indices)?;
                }
            }
        }
        Ok(())
    }
}
