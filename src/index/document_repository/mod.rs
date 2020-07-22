use crate::metadata::tag::TagId;

use chrono::serde::{ts_seconds, ts_seconds_option};

pub mod local_repository;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentData {
    pub id: super::DocId,
    pub original_filename: String,
    pub title: String,
    #[serde(skip)]
    pub body: Option<String>,
    pub tags: Vec<TagId>,
    #[serde(with = "ts_seconds")]
    pub imported_date: chrono::DateTime<chrono::Utc>,
    #[serde(with = "ts_seconds_option")]
    pub inferred_date: Option<chrono::DateTime<chrono::Utc>>,
    pub language: Option<String>,
    pub hash: String,
    pub file_size: u64,
}

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum DocumentRepositoryError {
    #[error("error in local document repository: `{0}`")]
    LocalRepoError(#[from] local_repository::IndexerError),
}
pub enum SortOrder {
    ImportedDate = 0,
    InferredDate = 1,
    NoOrder = 2,
}

pub struct FilterOptions {
    pub sort: SortOrder,
    pub tags: Vec<TagId>,
    pub query: Option<String>,
}

/// Implements a location where document data will be stored
pub trait DocumentRepository {
    /// Adds or replaces a document to the repository
    fn add_document(&mut self, doc: &DocumentData) -> Result<(), DocumentRepositoryError>;

    ///Updates the metadata of the document.
    ///The body attribute does not have to be set
    fn update_metadata(&mut self, doc: &DocumentData) -> Result<(), DocumentRepositoryError>;

    /// Removes a document from the repository
    fn remove_document(&mut self, id: super::DocId) -> Result<(), DocumentRepositoryError>;

    /// Retrieves a document from the repository
    fn get_document(&self, id: super::DocId) -> Result<DocumentData, DocumentRepositoryError>;

    /// Checks if the hash of a given file is already contained in the repository
    fn contains_hash(&self, hash: &str) -> Result<Option<super::DocId>, DocumentRepositoryError>;

    /// Returns the number of indexed documents
    fn len(&self) -> Result<usize, DocumentRepositoryError>;

    ///Gets all documents between [offset,offset+count]
    fn get_documents(
        &self,
        offset: usize,
        count: usize,
    ) -> Result<Vec<DocumentData>, DocumentRepositoryError>;

    //Gets the documents in sorted order
    fn get_filtered_documents(
        &self,
        offset: usize,
        count: usize,
        filter: FilterOptions,
    ) -> Result<Vec<DocumentData>, DocumentRepositoryError>;
}
