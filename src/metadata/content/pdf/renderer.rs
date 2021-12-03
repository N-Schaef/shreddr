use std::path::{Path, PathBuf};
use std::process::Command;

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum OCRError {
    #[error("could not initialize ocrmypdf `{0}`")]
    Ocrmypdf(String),
    #[error("could not interact with temporary ocr directory")]
    IO(#[from] std::io::Error),
    #[error("could not extract text as UTF-8")]
    UTF8(#[from] std::str::Utf8Error),
    #[error("could not extract ocr-image name `{0}`")]
    Image(String),
}

/// OCRs the given file and replaces it with an optimized version where the text is inserted as copyable metadata
pub fn ocr_file(file: &Path, tesseract_languages: &[String]) -> Result<(), OCRError> {
    let languages = tesseract_languages.join("+");
    let file = file
        .to_str()
        .ok_or_else(|| OCRError::Image(format!("{:#?}", file)))?;
    let ocr_mypdfoutput = Command::new("ocrmypdf")
        .arg("--deskew") //Fix skewed images
        .arg("--clean") // Use unpaper to improve OCR (does not edit final PDF)
        .arg("--force-ocr")
        .arg("-l")
        .arg(languages)
        .arg(file)
        .arg(file)
        .output();
    match ocr_mypdfoutput {
        Ok(output) => {
            debug!("{}", std::str::from_utf8(&output.stdout).unwrap());
            if !output.status.success() {
                let msg = std::str::from_utf8(&output.stderr).unwrap();
                let e = Err(OCRError::Ocrmypdf(msg.into()));
                error!("{:?}", e);
                return e;
            }
        }
        Err(e) => {
            let e = Err(OCRError::Ocrmypdf(format!("{:?}", e)));
            error!("{:?}", e);
            return e;
        }
    }
    Ok(())
}

pub fn render_thumbnail(file: &Path, thumbnail_file: &Path) {
    //Adapt input filename
    let mut input: PathBuf = file.into();
    input.set_file_name(format!(
        "{}[0]",
        input.file_name().unwrap().to_str().unwrap()
    ));
    let mut cmd = Command::new("convert");
    cmd.arg("-colorspace")
        .arg("RGB")
        .arg(input)
        .arg("-trim")
        .arg("+repage")
        .arg("-background")
        .arg("white")
        .arg("-flatten")
        .arg(thumbnail_file);
    debug!("Executing command `{:#?}`", cmd);
    let convert_output = cmd.output();
    match convert_output {
        Ok(output) => {
            debug!("{}", std::str::from_utf8(&output.stdout).unwrap());
            if !output.status.success() {
                error!(
                    "Could not execute convert command: {}",
                    std::str::from_utf8(&output.stderr).unwrap()
                );
            }
        }
        Err(e) => {
            error!("Could not execute convert command: {}", e);
        }
    }
}
