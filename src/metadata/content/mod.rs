use std::path::{Path, PathBuf};

mod ocr;
mod pdf;

/// Extracts text (and other content) from files
pub struct ContentExtractor {
  tmp_dir: PathBuf,
  tesseract_languages: Vec<String>,
}

impl ContentExtractor {
  /// Initializes a new ContentExtractor with an temporary directory, in which it will create a OCR subdirectory if required, and the configured tesseract languages.
  pub fn new(tmp_dir: &Path, tesseract_languages: &[String]) -> ContentExtractor {
    let ocr_dir = tmp_dir.join("ocr");
    ContentExtractor {
      tmp_dir: ocr_dir,
      tesseract_languages: tesseract_languages.into(),
    }
  }

  /// Extracts the text from a file.
  /// If the file does not contain any text, the text is extracted by OCR.
  pub fn extract_body(&self, file: &Path) -> Option<String> {
    if let Some(ext) = ContentExtractor::extract_extension(file) {
      let text = match ext.as_ref() {
        "pdf" => pdf::extractor::extract_body(file),
        _ => {
          error!(
            "Trying to extract text from unsupported file format `{:#?}`",
            file
          );
          return None;
        }
      };
      if text.is_empty()  {
        info!("Could not extract text => OCR");
        return self._ocr(file, &ext);
      } else {
        return Some(text);
      }
    }

    None
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
    //Create temp dir
    if std::fs::create_dir_all(&self.tmp_dir).is_err() {
      error!("Could not create directory `{:#?}`", self.tmp_dir);
      return None;
    }

    // Create OCR files
    let files = match extension {
      "pdf" => pdf::renderer::render_pages_for_ocr(file, &self.tmp_dir),
      _ => {
        error!("OCR not supported for file `{:#?}`", file);
        return None;
      }
    };

    // OCR files
    let text = match ocr::ocr_files(&files, &self.tesseract_languages) {
      Err(e) => {
        error!("Could not OCR the files: `{}`", e);
        None
      }
      Ok(t) => Some(t),
    };

    //Remove temp dir
    if std::fs::remove_dir_all(&self.tmp_dir).is_err() {
      error!("Could not create directory `{:#?}`", self.tmp_dir);
      return None;
    }

    text
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
