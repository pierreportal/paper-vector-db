use std::fs::read_to_string;
use std::path::PathBuf;

use crate::text_processing::chunker::{TextChunk, chunk_text};

pub fn extract_text_chunks_from_file(absolute_path: &PathBuf) -> Vec<TextChunk> {
    let extension = absolute_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap();

    let doc = match extension {
        "pdf" => pdf_extract::extract_text(&absolute_path).unwrap(),
        _ => read_to_string(&absolute_path).unwrap(),
    };

    let chunks = chunk_text(extension, &doc);

    chunks
}
