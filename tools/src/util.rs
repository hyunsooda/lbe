use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn create_dir(path: &str) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn extract_filename(file: &str) -> &str {
    Path::new(file)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
}
