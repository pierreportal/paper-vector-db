use std::fs::read_to_string;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::text_processing::chunker::{TextChunk, chunk_text};

pub fn extract_text_chunks_from_file(absolute_path: &Path) -> Result<Vec<TextChunk>> {
    let extension = absolute_path
        .extension()
        .and_then(|ext| ext.to_str())
        .context("file has no valid extension")?;

    let doc = match extension {
        "pdf" => pdf_extract::extract_text(absolute_path)
            .with_context(|| format!("failed to extract text from PDF: {}", absolute_path.display()))?,
        _ => read_to_string(absolute_path)
            .with_context(|| format!("failed to read file: {}", absolute_path.display()))?,
    };

    if doc.trim().is_empty() {
        bail!("file contains no extractable text: {}", absolute_path.display());
    }

    Ok(chunk_text(extension, &doc))
}
