use std::path::Path;
use whatlang::{detect, Lang};

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum OCRError {
    #[error("could not initialize tesseract `{0}`")]
    TesseractError(String),
    #[error("could not interact with temporary ocr directory")]
    IOError(#[from] std::io::Error),
    #[error("could not extract text as UTF-8")]
    UTF8Error(#[from] std::str::Utf8Error),
    #[error("could not extract ocr-image name `{0}`")]
    ImageError(String),
    #[error("unknown language code `{0}`")]
    LanguageError(String),
}

/// Preprocesses and OCRs given files
pub fn ocr_files<'a, P>(
    ocr_images: &[P],
    tesseract_languages: &[String],
) -> Result<String, OCRError>
where
    P: 'a + AsRef<Path>,
{
    let mut text = String::new();
    for lang in tesseract_languages {
        let parsed_lang =
            Lang::from_code(lang).ok_or_else(|| OCRError::LanguageError(lang.into()))?;
        info!("Trying language {}", lang);
        let mut lt = leptess::LepTess::new(None, &lang)
            .map_err(|e| OCRError::TesseractError(format!("{}", e)))?;
        for image in ocr_images {
            let i = image.as_ref();
            lt.set_image(
                i.to_str()
                    .ok_or_else(|| OCRError::ImageError(format!("{:#?}", i)))?,
            )
            .map_err(|e| OCRError::TesseractError(format!("{}", e)))?;
            lt.set_fallback_source_resolution(300);
            let page_text = lt.get_utf8_text()?;
            text.push_str(&page_text);
        }
        if let Some(info) = detect(&text) {
            if info.lang() == parsed_lang {
                info!(
                    "Detected {} language with {} confidence",
                    parsed_lang,
                    info.confidence()
                );
                return Ok(text);
            }
        };
    }

    info!("Could not detect correct language with enough confidence. Returning empty body.");
    Ok(String::new())
}
