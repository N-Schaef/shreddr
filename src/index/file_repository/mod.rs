use std::path::{Path, PathBuf};

pub mod local_repository;

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum FileRepositoryError {
    #[error("error in local file repository: `{0}`")]
    FileRepoError(#[from] local_repository::LocalFileRepositoryError),
}

/// Implements a location where the document files can be stored, like disk or cloud Repository
pub trait FileRepository {
    /// Adds a document to the FileRepository
    fn add_document(&mut self, id: super::DocId, file: &Path) -> Result<(), FileRepositoryError>;

    /// Removes a document from the FileRepository
    fn remove_document(&mut self, id: super::DocId) -> Result<(), FileRepositoryError>;

    /// Retrieves a document from the FileRepository
    /// Returns the location to the file
    fn get_document(&self, id: super::DocId) -> Result<PathBuf, FileRepositoryError>;
}
