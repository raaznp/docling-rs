use docling_core::{
    base_models::{ConversionStatus, DoclingComponentType, ErrorItem, Page},
    errors::Result,
    ConversionResult,
};
use docling_models::{BuildModel, EnrichmentModel};
use std::time::Instant;

/// Base pipeline trait — all pipelines implement this.
pub trait BasePipeline: Send + Sync {
    fn name(&self) -> &str;

    /// Execute the full pipeline: build → assemble → enrich.
    fn execute(&self, mut conv_res: ConversionResult, raises_on_error: bool) -> ConversionResult {
        let start = Instant::now();
        conv_res.status = ConversionStatus::Started;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let conv_res = self.build_document(conv_res)?;
            let conv_res = self.assemble_document(conv_res)?;
            let conv_res = self.enrich_document(conv_res)?;
            Ok::<ConversionResult, docling_core::DoclingError>(conv_res)
        }));

        match result {
            Ok(Ok(mut cr)) => {
                cr.timings
                    .record("pipeline_total", start.elapsed().as_secs_f64());
                cr.status = self.determine_status(&cr);
                cr
            }
            Ok(Err(e)) => {
                let mut cr_fail = ConversionResult::new(conv_res_input_placeholder());
                cr_fail.status = ConversionStatus::Failure;
                cr_fail.errors.push(ErrorItem::new(
                    DoclingComponentType::Pipeline,
                    self.name(),
                    e.to_string(),
                ));
                cr_fail
            }
            Err(_) => {
                conv_res.status = ConversionStatus::Failure;
                conv_res
            }
        }
    }

    fn build_document(&self, conv_res: ConversionResult) -> Result<ConversionResult>;

    fn assemble_document(&self, conv_res: ConversionResult) -> Result<ConversionResult> {
        Ok(conv_res)
    }

    fn enrich_document(&self, mut conv_res: ConversionResult) -> Result<ConversionResult> {
        // Run enrichment models on document items
        Ok(conv_res)
    }

    fn determine_status(&self, conv_res: &ConversionResult) -> ConversionStatus {
        match conv_res.status {
            ConversionStatus::Pending | ConversionStatus::Started => ConversionStatus::Success,
            other => other,
        }
    }

    fn unload(&self, conv_res: &mut ConversionResult) {}
}

// Placeholder to satisfy borrow checker in error paths
fn conv_res_input_placeholder() -> docling_core::InputDocument {
    panic!("Pipeline error path should carry the original ConversionResult")
}

/// Paginated pipeline — builds the document page-by-page using build_pipe models.
pub struct PaginatedPipeline {
    pub build_pipe: Vec<Box<dyn BuildModel>>,
    pub enrichment_pipe: Vec<Box<dyn EnrichmentModel>>,
}

impl PaginatedPipeline {
    pub fn new(
        build_pipe: Vec<Box<dyn BuildModel>>,
        enrichment_pipe: Vec<Box<dyn EnrichmentModel>>,
    ) -> Self {
        Self {
            build_pipe,
            enrichment_pipe,
        }
    }

    pub fn run_build_pipe(
        &self,
        conv_res: &mut ConversionResult,
        pages: &mut Vec<Page>,
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

                // Collect items that need enrichment
                let to_process: Vec<_> = doc
                    .body
                    .iter()
                    .filter_map(|item| model.prepare_element(item))
                    .collect();

                if !to_process.is_empty() {
                    model.process_batch(doc, &to_process)?;
                }
            }
        }
        Ok(())
    }
}
