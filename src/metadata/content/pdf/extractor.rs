use std::path::Path;

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("could not interact with temporary ocr directory")]
    IOError(#[from] std::io::Error),
    #[error("could not extract text as UTF-8")]
    UTF8Error(#[from] std::str::Utf8Error),
}

/// Extracts the text from a PDF file.
pub fn extract_body(file: &Path) -> String {
    debug!("Extracting text from file {:?}", file);
    let result = match std::panic::catch_unwind(|| pdf_extract::extract_text(&file)) {
        Ok(r) => r,
        Err(e) => {
            error!("Extractor panicked for file {:#?}: {:#?}", file, e);
            return String::new();
        }
    };
    match result {
        Ok(text) => text,
        Err(e) => {
            error!("Could not extract text from file {:#?}: {}", file, e);
            String::new()
        }
    }
}
