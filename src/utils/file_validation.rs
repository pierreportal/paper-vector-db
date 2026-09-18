use anyhow::{Result, bail};
use std::path::Path;

pub fn validate_file(absolute_path: &Path) -> Result<()> {
    if !absolute_path.exists() {
        bail!("file does not exist: {}", absolute_path.display());
    }

    if !absolute_path.is_file() {
        bail!("path is not a file: {}", absolute_path.display());
    }

    let extension = absolute_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    if !is_supported_extension(extension) {
        bail!("unsupported file type: .{}", extension);
    }

    Ok(())
}

fn is_supported_extension(extension: &str) -> bool {
    matches!(
        extension,
        "txt" | "md" | "pdf" // | "rs"
                             // | "js"
                             // | "ts"
                             // | "jsx"
                             // | "tsx"
                             // | "json"
                             // | "toml"
                             // | "yaml"
                             // | "yml"
                             // | "html"
                             // | "css"
                             // | "scss"
                             // | "xml"
                             // | "csv"
    )
}
