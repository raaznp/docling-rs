use docling_core::{
    base_models::{ConversionStatus, Page},
    errors::Result,
    ConversionResult,
};
use docling_models::{
    layout::LayoutModel, ocr::OcrModel, picture_classifier::PictureClassifierModel,
    picture_description::PictureDescriptionModel, table::TableStructureModel, BuildModel,
    EnrichmentModel,
};
use std::path::PathBuf;

use crate::base::{BasePipeline, PaginatedPipeline};

/// Standard PDF pipeline.
///
/// Orchestrates per-page processing:
///   1. Page initialization (load page from backend → bitmap + text layer)
///   2. OCR model (fills `page.cells`)
///   3. Layout detection model (fills `page.predictions.layout`)
///   4. Table structure model (fills table cluster cells)
///
/// Then assembles a DoclingDocument from the page-level predictions,
/// and runs enrichment models (picture classifier, picture description).
///
/// Mirrors `docling/pipeline/standard_pdf_pipeline.py`.
pub struct StandardPdfPipeline {
    inner: PaginatedPipeline,
    page_batch_size: usize,
    document_timeout: Option<f64>,
}

impl StandardPdfPipeline {
    pub fn new(
        artifacts_path: Option<PathBuf>,
        do_ocr: bool,
        do_table_structure: bool,
        do_picture_classification: bool,
        document_timeout: Option<f64>,
    ) -> Result<Self> {
        let layout_model = LayoutModel::new(true, artifacts_path.clone())?;
        let ocr_model = OcrModel::new(
            do_ocr,
            false,
            vec!["en".to_string()],
            artifacts_path.clone(),
        )?;
        let table_model = TableStructureModel::new(
            do_table_structure,
            docling_models::table::TableFormerMode::Fast,
            artifacts_path.clone(),
        )?;

        let build_pipe: Vec<Box<dyn BuildModel>> = vec![
            Box::new(ocr_model),
            Box::new(layout_model),
            Box::new(table_model),
        ];

        let enrichment_pipe: Vec<Box<dyn EnrichmentModel>> = vec![
            Box::new(PictureClassifierModel::new(do_picture_classification)),
            Box::new(PictureDescriptionModel::disabled()),
        ];

        Ok(Self {
            inner: PaginatedPipeline::new(build_pipe, enrichment_pipe),
            page_batch_size: 4,
            document_timeout,
        })
    }
}

impl BasePipeline for StandardPdfPipeline {
    fn name(&self) -> &str {
        "StandardPdfPipeline"
    }

    fn build_document(&self, mut conv_res: ConversionResult) -> Result<ConversionResult> {
        conv_res.status = ConversionStatus::Started;

        let total_pages = conv_res.input.page_count;
        let (start_page, end_page) = conv_res.input.limits.page_range;

        // Build the page list within the requested range
        for i in 0..total_pages {
            let page_no = i + 1;
            if page_no >= start_page && page_no <= end_page {
                conv_res.pages.push(Page::new(page_no as u32));
            }
        }

        let start_time = std::time::Instant::now();

        // Process pages in batches
        let page_batches: Vec<Vec<Page>> = conv_res
            .pages
            .chunks(self.page_batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut all_processed_pages: Vec<Page> = Vec::new();

        for mut batch in page_batches {
            // Check timeout
            if let Some(timeout) = self.document_timeout {
                if start_time.elapsed().as_secs_f64() > timeout {
                    log::warn!("StandardPdfPipeline: document timeout reached");
                    conv_res.status = ConversionStatus::PartialSuccess;
                    break;
                }
            }

            // Run the build pipe on this batch
            self.inner.run_build_pipe(&mut conv_res, &mut batch)?;
            all_processed_pages.extend(batch);
        }

        conv_res.pages = all_processed_pages;

        // Assemble DoclingDocument from page predictions
        let doc = assemble_document_from_pages(&conv_res);
        conv_res.document = Some(doc);

        Ok(conv_res)
    }

    fn enrich_document(&self, mut conv_res: ConversionResult) -> Result<ConversionResult> {
        self.inner.run_enrichment_pipe(&mut conv_res)?;
        Ok(conv_res)
    }

    fn determine_status(&self, conv_res: &ConversionResult) -> ConversionStatus {
        match conv_res.status {
            ConversionStatus::PartialSuccess => ConversionStatus::PartialSuccess,
            ConversionStatus::Pending | ConversionStatus::Started => {
                if conv_res.errors.is_empty() {
                    ConversionStatus::Success
                } else {
                    ConversionStatus::PartialSuccess
                }
            }
            other => other,
        }
    }
}

/// Assemble a DoclingDocument from page-level predictions.
///
/// Walks through all pages, collecting layout clusters in reading order,
/// and converts them to the appropriate DocItem types.
fn assemble_document_from_pages(conv_res: &ConversionResult) -> docling_core::DoclingDocument {
    use docling_core::{
        doc_types::{DoclingDocument, PageRef, SectionHeaderItem, TextItem},
        BoundingBox, LayoutLabel, PageSize,
    };

    let input_name = conv_res
        .input
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document")
        .to_string();

    let mut doc = DoclingDocument::new(&input_name);

    // Register page dimensions
    for page in &conv_res.pages {
        if let Some(ref size) = page.size {
            doc.pages.insert(
                page.page_no,
                PageRef {
                    page_no: page.page_no,
                    size: size.clone(),
                    image: None,
                },
            );
        }
    }

    // Convert layout clusters to document items
    for page in &conv_res.pages {
        let clusters = match page.predictions.layout.as_ref() {
            Some(lp) => &lp.clusters,
            None => continue,
        };

        // Sort clusters by top-to-bottom, left-to-right (reading order)
        let mut sorted_clusters = clusters.clone();
        sorted_clusters.sort_by(|a, b| {
            a.bbox
                .t
                .partial_cmp(&b.bbox.t)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for cluster in &sorted_clusters {
            let text: String = cluster
                .cells
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            let prov = vec![docling_core::doc_types::ProvenanceRef {
                page_no: page.page_no,
                bbox: cluster.bbox.clone(),
                charspan: [0, text.len()],
            }];

            let id = format!("#/texts/{}", doc.body.len());

            match cluster.label {
                LayoutLabel::Title | LayoutLabel::SectionHeader => {
                    doc.add_header(SectionHeaderItem {
                        id,
                        text,
                        level: if cluster.label == LayoutLabel::Title {
                            1
                        } else {
                            2
                        },
                        label: cluster.label.clone(),
                        prov,
                    });
                }
                LayoutLabel::ListItem => {
                    doc.add_list_item(docling_core::doc_types::ListItem {
                        id,
                        text,
                        label: cluster.label.clone(),
                        prov,
                        enumerated: Some(false),
                        marker: Some("-".to_string()),
                    });
                }
                LayoutLabel::Table => {
                    // Table structure details are filled by TableStructureModel
                    doc.add_table(docling_core::doc_types::TableItem {
                        id,
                        label: cluster.label.clone(),
                        prov,
                        data: docling_core::doc_types::TableData {
                            num_rows: 0,
                            num_cols: 0,
                            table_cells: vec![],
                            grid: None,
                        },
                        captions: None,
                    });
                }
                LayoutLabel::Figure | LayoutLabel::Picture => {
                    doc.add_picture(docling_core::doc_types::PictureItem {
                        id,
                        label: cluster.label.clone(),
                        prov,
                        captions: None,
                        description: None,
                        image_data: None,
                        classification: None,
                    });
                }
                LayoutLabel::Code => {
                    doc.add_code(docling_core::doc_types::CodeItem {
                        id,
                        text,
                        label: cluster.label.clone(),
                        prov,
                        code_language: None,
                    });
                }
                LayoutLabel::Formula => {
                    doc.add_formula(docling_core::doc_types::FormulaItem {
                        id,
                        text,
                        label: cluster.label.clone(),
                        prov,
                    });
                }
                _ => {
                    // Default: text item
                    if !text.trim().is_empty() {
                        doc.add_text(TextItem {
                            id,
                            text,
                            label: cluster.label.clone(),
                            prov,
                            orig: None,
                            enumerated: None,
                            marker: None,
                        });
                    }
                }
            }
        }
    }

    doc
}
