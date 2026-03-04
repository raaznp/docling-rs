use std::path::PathBuf;

pub fn get_test_data_path() -> PathBuf {
    // Relative to docling-rs/
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("docling-main");
    path.push("tests");
    path.push("data");
    path
}

pub fn get_groundtruth_path() -> PathBuf {
    let mut path = get_test_data_path();
    path.push("groundtruth");
    path.push("docling_v2");
    path
}
