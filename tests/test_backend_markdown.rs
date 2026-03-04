mod common;

use docling::backend::markdown::MarkdownBackend;
use docling::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use docling::document_converter::DocumentConverter;

#[test]
fn test_markdown_backend_is_valid() {
    let mut path = common::get_test_data_path();
    path.push("md");
    path.push("mixed_without_h1.md");

    let source = BackendSource::Path(path);
    let backend = MarkdownBackend::new(source);
    assert!(backend.is_valid());
}

#[test]
fn test_markdown_conversion_parity() {
    let mut md_dir = common::get_test_data_path();
    md_dir.push("md");
    let gt_dir = common::get_groundtruth_path();

    let files = vec![
        "inline_and_formatting.md",
        "mixed_without_h1.md",
        "escaped_characters.md",
    ];

    for file_name in files {
        let mut in_path = md_dir.clone();
        in_path.push(file_name);

        let mut gt_path = gt_dir.clone();
        gt_path.push(format!("{}.md", file_name));

        if !in_path.exists() || !gt_path.exists() {
            println!("Skipping {}, missing input or groundtruth", file_name);
            continue;
        }

        let source = BackendSource::Path(in_path);
        let mut backend = MarkdownBackend::new(source);
        let doc = backend.convert().unwrap();

        let act_md = doc.export_to_markdown().trim().to_string();
        let exp_md = std::fs::read_to_string(gt_path).unwrap().trim().to_string();

        assert_eq!(act_md, exp_md, "Parity failed for {}", file_name);
    }
}

#[test]
fn test_document_converter_markdown() {
    let mut in_path = common::get_test_data_path();
    in_path.push("md");
    in_path.push("mixed_without_h1.md");

    let converter = DocumentConverter::new();
    let result = converter.convert(&in_path).unwrap();

    assert!(result.document.is_some());
    let doc = result.document.unwrap();
    assert!(!doc.export_to_markdown().is_empty());
}
