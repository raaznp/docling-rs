use crate::datamodel::base_models::{ConversionStatus, Page};
use crate::datamodel::document::ConversionResult;
use crate::errors::Result;
use crate::models::{
    layout::LayoutModel,
    ocr::OcrModel,
    picture_classifier::PictureClassifierModel,
    picture_description::PictureDescriptionModel,
    table::{TableFormerMode, TableStructureModel},
};
use crate::pipeline::base::{BasePipeline, PaginatedPipeline};
use std::path::PathBuf;

/// StandardPdfPipeline — ML-driven PDF processing.
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
        let build_pipe: Vec<Box<dyn crate::models::BuildModel>> = vec![
            Box::new(OcrModel::new(
                do_ocr,
                false,
                vec!["en".into()],
                artifacts_path.clone(),
            )?),
            Box::new(LayoutModel::new(true, artifacts_path.clone())?),
            Box::new(TableStructureModel::new(
                do_table_structure,
                TableFormerMode::Fast,
                artifacts_path,
            )?),
        ];

        let enrichment_pipe: Vec<Box<dyn crate::models::EnrichmentModel>> = vec![
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
        let total = conv_res.input.page_count;
        let (start_page, end_page) = conv_res.input.limits.page_range;

        for i in 0..total {
            let page_no = i + 1;
            if page_no >= start_page && page_no <= end_page {
                conv_res.pages.push(Page::new(page_no as u32));
            }
        }

        let start_time = std::time::Instant::now();
        let batches: Vec<Vec<Page>> = conv_res
            .pages
            .chunks(self.page_batch_size)
            .map(|c| c.to_vec())
            .collect();

        let mut all_pages: Vec<Page> = Vec::new();
        for mut batch in batches {
            if let Some(timeout) = self.document_timeout {
                if start_time.elapsed().as_secs_f64() > timeout {
                    conv_res.status = ConversionStatus::PartialSuccess;
                    break;
                }
            }
            self.inner.run_build_pipe(&mut conv_res, &mut batch)?;
            all_pages.extend(batch);
        }
        conv_res.pages = all_pages;

        let doc = assemble_from_pages(&conv_res);
        conv_res.document = Some(doc);
        Ok(conv_res)
    }

    fn enrich_document(&self, mut conv_res: ConversionResult) -> Result<ConversionResult> {
        self.inner.run_enrichment_pipe(&mut conv_res)?;
        Ok(conv_res)
    }

    fn determine_status(&self, conv_res: &ConversionResult) -> ConversionStatus {
        match conv_res.status.clone() {
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

fn assemble_from_pages(conv_res: &ConversionResult) -> crate::datamodel::document::DoclingDocument {
    use crate::datamodel::base_models::LayoutLabel;
    use crate::datamodel::document::*;

    let name = conv_res
        .input
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document")
        .to_string();
    let mut doc = DoclingDocument::new(&name);

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

        let clusters = match page.predictions.layout.as_ref() {
            Some(lp) => &lp.clusters,
            None => continue,
        };

        let mut sorted = clusters.clone();
        sorted.sort_by(|a, b| {
            a.bbox
                .t
                .partial_cmp(&b.bbox.t)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for cluster in &sorted {
            let text: String = cluster
                .cells
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let prov = vec![ProvenanceRef {
                page_no: page.page_no,
                bbox: cluster.bbox.clone(),
                charspan: [0, text.len()],
            }];
            let id = format!("#/items/{}", doc.body.len());

            match cluster.label {
                LayoutLabel::Title | LayoutLabel::SectionHeader => {
                    let level = if cluster.label == LayoutLabel::Title {
                        1
                    } else {
                        2
                    };
                    doc.add_header(SectionHeaderItem {
                        id,
                        text,
                        level,
                        label: cluster.label.clone(),
                        prov,
                        formatting: None,
                        hyperlink: None,
                        annotations: vec![],
                    });
                }
                LayoutLabel::ListItem => {
                    doc.add_list_item(ListItem {
                        id,
                        text,
                        level: 0,
                        label: cluster.label.clone(),
                        prov,
                        enumerated: Some(false),
                        marker: Some("-".into()),
                        formatting: None,
                        hyperlink: None,
                        annotations: vec![],
                    });
                }
                LayoutLabel::Table => {
                    doc.add_table(TableItem {
                        id,
                        label: cluster.label.clone(),
                        prov,
                        data: TableData {
                            num_rows: 0,
                            num_cols: 0,
                            table_cells: vec![],
                            grid: None,
                        },
                        captions: None,
                    });
                }
                LayoutLabel::Figure | LayoutLabel::Picture => {
                    doc.add_picture(PictureItem {
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
                    doc.add_code(CodeItem {
                        id,
                        text,
                        label: cluster.label.clone(),
                        prov,
                        code_language: None,
                        formatting: None,
                        hyperlink: None,
                        annotations: vec![],
                    });
                }
                LayoutLabel::Formula => {
                    doc.add_formula(FormulaItem {
                        id,
                        text,
                        label: cluster.label.clone(),
                        prov,
                    });
                }
                _ => {
                    if !text.trim().is_empty() {
                        doc.add_text(TextItem {
                            id,
                            text,
                            label: cluster.label.clone(), // Use cluster.label.clone() instead of LayoutLabel::Text
                            prov,
                            orig: None,
                            enumerated: None,
                            marker: None,
                            formatting: None,
                            hyperlink: None,
                            annotations: vec![],
                        });
                    }
                }
            }
        }
    }
    doc
}
