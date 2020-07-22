use glob::glob;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Renders every page in the file as a separate grayscale image preprocessed with unpaper.
/// The images are stored in the passed tmp_directory.
pub fn render_pages_for_ocr(file: &Path, tmp_dir: &Path) -> Vec<PathBuf> {
    let convert_output = Command::new("convert")
        .arg("-density")
        .arg(format!("{}", 300))
        .arg("-depth")
        .arg(format!("{}", 8))
        .arg("-type")
        .arg("grayscale")
        .arg(file)
        .arg(tmp_dir.join("convert-%04d.pnm").to_str().unwrap())
        .output();
    match convert_output {
        Ok(output) => {
            debug!("{}", std::str::from_utf8(&output.stdout).unwrap());
            if !output.status.success() {
                error!(
                    "Could not execute convert command: {}",
                    std::str::from_utf8(&output.stderr).unwrap()
                );
                return vec![];
            }
        }
        Err(e) => {
            error!("Could not execute convert command: {}", e);
            return vec![];
        }
    }

    let pnms = glob(tmp_dir.join("*.pnm").to_str().unwrap())
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    for image in &pnms {
        let unpaper_output = Command::new("unpaper")
            .arg("--overwrite")
            .arg(image)
            .arg(image)
            .output();
        match unpaper_output {
            Ok(output) => {
                debug!("{}", std::str::from_utf8(&output.stdout).unwrap());
                if !output.status.success() {
                    error!(
                        "Could not execute convert command: {}",
                        std::str::from_utf8(&output.stderr).unwrap()
                    );
                    return vec![];
                }
            }
            Err(e) => {
                error!("Could not execute convert command: {}", e);
                return vec![];
            }
        }
    }
    pnms
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
                return;
            }
        }
        Err(e) => {
            error!("Could not execute convert command: {}", e);
        }
    }
}
