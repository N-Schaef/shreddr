use std::path::Path;

mod pdf;

/// Extracts text (and other content) from files
pub struct ContentExtractor {
    tesseract_languages: Vec<String>,
}

impl ContentExtractor {
    /// Initializes a new ContentExtractor with an temporary directory, in which it will create a OCR subdirectory if required, and the configured tesseract languages.
    pub fn new(tesseract_languages: &[String]) -> ContentExtractor {
        ContentExtractor {
            tesseract_languages: tesseract_languages.into(),
        }
    }

    /// Extracts the text from a file.
    /// If the file does not contain any text, the text is extracted by OCR.
    pub fn extract_body(&self, file: &Path) -> Option<String> {
        if let Some(ext) = ContentExtractor::extract_extension(file) {
            let text = self._extract_body(file, &ext);
            if text.is_none() || text.as_ref().unwrap().is_empty() {
                info!("Could not extract text => OCR");
                return self._ocr(file, &ext);
            } else {
                return text;
            }
        } else {
            error!("Could not extract extension`{:#?}`", file);
        }
        None
    }

    fn _extract_body(&self, file: &Path, extension: &str) -> Option<String> {
        match extension {
            "pdf" => Some(pdf::extractor::extract_body(file)),
            _ => {
                error!(
                    "Trying to extract text from unsupported file format `{:#?}`",
                    file
                );
                None
            }
        }
    }

    /// Forces extraction of the body via OCR
    pub fn ocr(&self, file: &Path) -> Option<String> {
        if let Some(ext) = ContentExtractor::extract_extension(file) {
            self._ocr(file, &ext)
        } else {
            None
        }
    }

    fn _ocr(&self, file: &Path, extension: &str) -> Option<String> {
        // OCR file
        let _res = match extension {
            "pdf" => pdf::renderer::ocr_file(file, &self.tesseract_languages),
            _ => {
                error!("OCR not supported for file `{:#?}`", file);
                return None;
            }
        };

        let text = self._extract_body(file, extension).unwrap_or_default();

        if text.is_empty() {
            error!("OCR attempt did not yield text.");
            return None;
        }
        Some(text)
    }

    /// Renders a thumbnail of the image
    pub fn render_thumbnail(file: &Path, thumbnail_file: &Path) {
        debug!(
            "Rendering thumbnail for `{:#?}` in `{:#?}`",
            file, thumbnail_file
        );

        if let Some(ext) = ContentExtractor::extract_extension(file) {
            match ext.as_str() {
                "pdf" => pdf::renderer::render_thumbnail(file, thumbnail_file),
                _ => {
                    error!(
                        "Trying to render thumbnail of unsupported file format `{:#?}`",
                        file
                    );
                }
            }
        };
    }

    fn extract_extension(file: &Path) -> Option<String> {
        if let Some(os_str) = file.extension() {
            if let Some(ext_str) = os_str.to_str() {
                return Some(String::from(ext_str).to_lowercase());
            }
        }
        None
    }
}
