use std::fs::File;
use std::path::{PathBuf,Path};
use sha2::{Sha256,Digest};

/// Extractor of generic file meta data
pub struct FileExtractor {}

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum FileExtractError {
  #[error("could not find or access file `{0}`")]
  FileNotFoundError(PathBuf),
}

impl FileExtractor {
  
    /// Returns the file size of the given file
    /// # Errors
    /// May return an error if the file was not found or cannot be accessed
    pub fn get_file_size(file: &Path) -> Result<u64,FileExtractError> {
      let meta = std::fs::metadata(file).map_err(|_|FileExtractError::FileNotFoundError(file.into()))?;
      Ok(meta.len())
  }

  /// Calculates the hash of a file
  pub fn get_file_hash(file_path: &Path) -> Result<String,FileExtractError> {
    let mut file = File::open(file_path).map_err(|_|FileExtractError::FileNotFoundError(file_path.into()))?;
    let mut sha256 = Sha256::new();
    std::io::copy(&mut file, &mut sha256).map_err(|_|FileExtractError::FileNotFoundError(file_path.into()))?;
    Ok(format!("{:x}",sha256.finalize()))
  }

}