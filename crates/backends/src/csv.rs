use csv::Reader;
use docling_core::{
    base_models::InputFormat,
    doc_types::{DoclingDocument, DocumentOrigin, TextItem},
    errors::{DoclingError, Result},
    LayoutLabel,
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// CSV document backend.
/// Converts CSV data to a simple `DoclingDocument` — the entire CSV is
/// represented as a raw table item. Mirrors `docling/backend/csv_backend.py`.
pub struct CsvBackend {
    source: BackendSource,
    valid: bool,
    delimiter: u8,
    has_header: bool,
}

impl CsvBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
            delimiter: b',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }
}

impl DocumentBackend for CsvBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }

    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Csv]
    }

    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for CsvBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "text/csv".to_string(),
            binary_hash: None,
            uri: None,
        });

        let mut rdr = Reader::from_reader(bytes.as_slice());
        let mut rows: Vec<Vec<String>> = Vec::new();

        // Read headers if present
        let headers = if self.has_header {
            rdr.headers()
                .map_err(|e| DoclingError::backend(e.to_string()))?
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if !headers.is_empty() {
            rows.push(headers);
        }

        for result in rdr.records() {
            let record = result.map_err(|e| DoclingError::backend(e.to_string()))?;
            rows.push(record.iter().map(|f| f.to_string()).collect());
        }

        if rows.is_empty() {
            return Ok(doc);
        }

        let num_rows = rows.len() as u32;
        let num_cols = rows.first().map(|r| r.len() as u32).unwrap_or(0);

        let cells: Vec<docling_core::Cell> = rows
            .iter()
            .enumerate()
            .flat_map(|(row_i, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(col_i, cell_text)| docling_core::Cell {
                        id: col_i as u32,
                        text: cell_text.clone(),
                        bbox: docling_core::BoundingBox::new(0.0, 0.0, 0.0, 0.0),
                        row_span: 1,
                        col_span: 1,
                        start_row: row_i as u32,
                        end_row: row_i as u32,
                        start_col: col_i as u32,
                        end_col: col_i as u32,
                        column_header: row_i == 0 && self.has_header,
                        row_header: false,
                        row_section: false,
                    })
            })
            .collect();

        doc.add_table(docling_core::doc_types::TableItem {
            id: "#/tables/0".to_string(),
            label: LayoutLabel::Table,
            prov: vec![],
            data: docling_core::doc_types::TableData {
                num_rows,
                num_cols,
                table_cells: cells,
                grid: None,
            },
            captions: None,
        });

        Ok(doc)
    }
}
