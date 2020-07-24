use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub mod file_repository;
use file_repository::{FileRepository, FileRepositoryError};

pub mod document_repository;
use document_repository::{
    DocumentData, DocumentRepository, DocumentRepositoryError, FilterOptions,
};

use crate::metadata::content::ContentExtractor;
use crate::metadata::file_extractor::{FileExtractError, FileExtractor};
use crate::metadata::tag::{TagConfig, TagId, Tagger, TaggingError};

/// Identification type of documents
pub type DocId = u64;

/// The Index class encapsulates all of the storage/retrieval backend of Shreddr
pub struct Index {
    file_repo: Arc<RwLock<dyn FileRepository + Send + Sync>>,
    doc_repo: Arc<RwLock<dyn DocumentRepository + Send + Sync>>,
    tagger: Arc<RwLock<Tagger>>,
    extractor: Arc<RwLock<ContentExtractor>>,
    data_dir: PathBuf,
    thumbnails_dir: PathBuf,
    tmp_dir: PathBuf,
}

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("error while interacting with file repository: `{0}`")]
    FileRepoError(#[from] FileRepositoryError),
    #[error("error while interacting with document repository: `{0}`")]
    DocRepoError(#[from] DocumentRepositoryError),
    #[error("error while interacting with tagger: `{0}`")]
    TaggerError(#[from] TaggingError),
    #[error("could not extract text from pdf file")]
    PDFError(),
    #[error("could not get lock on {0}")]
    LockError(String),
    #[error("error during IO operation `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("could not convert OSString to string")]
    OSStringError(),
    #[error("could not parse id from ID file")]
    IDError(#[from] std::num::ParseIntError),
    #[error("error during file metadata extraction")]
    FileExtractorError(#[from] FileExtractError),
}

impl Index {
    /// Creates a new index with the given repositories and languages.
    /// It uses the data_directory to store temporary data and thumbnails
    pub fn new(
        data_dir: &Path,
        tesseract_languages: &[String],
        file_repo: Arc<RwLock<dyn FileRepository + Send + Sync>>,
        doc_repo: Arc<RwLock<dyn DocumentRepository + Send + Sync>>,
    ) -> Result<Index, IndexError> {
        let tmp_dir = data_dir.join("tmp");
        std::fs::create_dir_all(&tmp_dir)?;
        let thumbnails_dir = data_dir.join("thumbnails");
        std::fs::create_dir_all(&thumbnails_dir)?;
        let tagger = Arc::new(RwLock::new(Tagger::new(data_dir)?));
        let extractor = Arc::new(RwLock::new(ContentExtractor::new(
            &tmp_dir,
            tesseract_languages,
        )));

        Ok(Index {
            file_repo,
            doc_repo,
            tagger,
            extractor,
            thumbnails_dir,
            data_dir: data_dir.into(),
            tmp_dir,
        })
    }

    pub fn get_tmp_dir(&self) -> &Path {
        &self.tmp_dir
    }

    /// Returns the next ID
    fn get_next_id(&self) -> Result<DocId, IndexError> {
        let id_file = self.data_dir.join("id.dat");

        //Create ID file if it does not exist
        if !id_file.exists() {
            info!(
                "ID file at `{:#?}` does not yet exist, creating with content `0`",
                &id_file
            );
            std::fs::write(&id_file, 0.to_string())?;
        }

        //Read content of ID file
        let id_content = std::fs::read(&id_file);
        let mut curr_id: DocId = match id_content {
            Ok(i) => String::from_utf8_lossy(&i).parse()?,
            Err(e) => {
                return Err(e.into());
            }
        };

        curr_id += 1;

        //Write ID file
        std::fs::write(id_file, curr_id.to_string())?;
        Ok(curr_id)
    }

    /// Imports a new document
    /// This function computes the hash value of each document and skips the file, if it is already contained in the repo
    /// To reimport/reprocess a document use the `reprocess_document` function
    pub fn import_document(&self, file: &Path) -> Result<DocId, IndexError> {
        let hash = FileExtractor::get_file_hash(file)?;
        if let Ok(Some(found_id)) = self
            .doc_repo
            .read()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .contains_hash(&hash)
        {
            debug!(
                "File {:?} already contained in repo with id {}",
                file, found_id
            );
            return Ok(found_id);
        }
        info!("Importing file {:?}", file);
        let id = self.get_next_id()?;
        let original_name = file.file_name().ok_or(IndexError::OSStringError())?;
        //Import
        (*self.file_repo)
            .write()
            .map_err(|_| IndexError::LockError("file repository".into()))?
            .add_document(id, file)?;

        //Extract
        let body = self
            .extractor
            .write()
            .map_err(|_| IndexError::LockError("extractor".into()))?
            .extract_body(file);
        //Create thumbnail
        let mut thumbnail_file = self.thumbnails_dir.join("tmp");
        thumbnail_file.set_file_name(format!("{}.jpg", id.to_string()));

        ContentExtractor::render_thumbnail(&file, &thumbnail_file);

        let original_filename: String = original_name
            .to_str()
            .ok_or(IndexError::OSStringError())?
            .into();
        let mut doc_data = DocumentData {
            id,
            title: original_filename.clone(),
            original_filename,
            body,
            tags: vec![],
            language: None,
            imported_date: chrono::Utc::now(),
            inferred_date: None,
            file_size: FileExtractor::get_file_size(file)?,
            hash,
        };
        //Tag
        self.tagger
            .write()
            .map_err(|_| IndexError::LockError("tagger".into()))?
            .tag_document(&mut doc_data)?;
        //Doc Repo
        self.doc_repo
            .write()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .add_document(&doc_data)?;

        Ok(id)
    }

    /// Updates the metadata of a given document
    pub fn update_doc_metadata(&self, doc: DocumentData) -> Result<(), IndexError> {
        self.doc_repo
            .write()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .update_metadata(&doc)?;
        Ok(())
    }

    /// Reprocesses/Reimports a document
    pub fn reprocess_document(&self, id: DocId) -> Result<(), IndexError> {
        let mut doc = self
            .doc_repo
            .write()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .get_document(id)?;
        let doc_path = (*self.file_repo)
            .read()
            .map_err(|_| IndexError::LockError("file repository".into()))?
            .get_document(id)?;
        //Extract
        let body = self
            .extractor
            .write()
            .map_err(|_| IndexError::LockError("extractor".into()))?
            .extract_body(&doc_path);
        //Create thumbnail
        let mut thumbnail_file = self.thumbnails_dir.join("tmp");
        thumbnail_file.set_file_name(format!("{}.jpg", id.to_string()));
        ContentExtractor::render_thumbnail(&doc_path, &thumbnail_file);
        //reset inferred data
        doc.tags = vec![];
        doc.inferred_date = None;
        doc.body = body;
        //Tag
        self.tagger
            .write()
            .map_err(|_| IndexError::LockError("tagger".into()))?
            .tag_document(&mut doc)?;
        //Index
        self.doc_repo
            .write()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .add_document(&doc)?;
        Ok(())
    }

    /// Reprocesses a document, but forces the pipeline to OCR the document.
    /// Useful if the normal extractor cannot correctly extract the text.
    pub fn reprocess_document_force_ocr(&self, id: DocId) -> Result<(), IndexError> {
        let mut doc = self
            .doc_repo
            .write()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .get_document(id)?;
        let doc_path = (*self.file_repo)
            .read()
            .map_err(|_| IndexError::LockError("file repository".into()))?
            .get_document(id)?;
        //Extract
        let body = self
            .extractor
            .write()
            .map_err(|_| IndexError::LockError("extractor".into()))?
            .ocr(&doc_path);
        //Create thumbnail
        let mut thumbnail_file = self.thumbnails_dir.join("tmp");
        thumbnail_file.set_file_name(format!("{}.jpg", id.to_string()));
        ContentExtractor::render_thumbnail(&doc_path, &thumbnail_file);
        //reset inferred data
        doc.tags = vec![];
        doc.inferred_date = None;
        doc.body = body;
        //Tag
        self.tagger
            .write()
            .map_err(|_| IndexError::LockError("tagger".into()))?
            .tag_document(&mut doc)?;
        //Index
        self.doc_repo
            .write()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .add_document(&doc)?;
        Ok(())
    }

    /// Returns the total number of documents
    #[allow(dead_code)]
    pub fn len(&self) -> Result<usize, IndexError> {
        self.doc_repo
            .read()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .len()
            .map_err(|e| e.into())
    }

    /// Returns the storage location of a document.
    /// If the documents are stored off-site, then a copy will be retrieved and a path to the document is given
    pub fn get_document_path(&self, id: DocId) -> Result<PathBuf, IndexError> {
        self.file_repo
            .read()
            .map_err(|_| IndexError::LockError("file repository".into()))?
            .get_document(id)
            .map_err(|e| e.into())
    }

    /// Returns all documents in the given range without any filters
    #[allow(dead_code)]
    pub fn get_documents(
        &self,
        offset: usize,
        count: usize,
    ) -> Result<Vec<DocumentData>, IndexError> {
        self.doc_repo
            .read()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .get_documents(offset, count)
            .map_err(|e| e.into())
    }

    pub fn get_document(&self, id: DocId) -> Result<DocumentData, IndexError> {
        self.doc_repo
            .read()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .get_document(id)
            .map_err(|e| e.into())
    }

    /// Returns a subset of documents after filtering them.
    pub fn get_sorted_documents(
        &self,
        offset: usize,
        count: usize,
        filter: FilterOptions,
    ) -> Result<Vec<DocumentData>, IndexError> {
        self.doc_repo
            .read()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .get_filtered_documents(offset, count, filter)
            .map_err(|e| e.into())
    }

    /// Removes a document (file and metadata) from the repository
    pub fn remove_document(&self, id: DocId) -> Result<(), IndexError> {
        self.file_repo
            .write()
            .map_err(|_| IndexError::LockError("file repository".into()))?
            .remove_document(id)?;
        self.doc_repo
            .write()
            .map_err(|_| IndexError::LockError("document repository".into()))?
            .remove_document(id)?;
        Ok(())
    }

    ///////////////////////////////////////////////////////////////
    /////////////////          Tags         ///////////////////////
    ///////////////////////////////////////////////////////////////

    /// Adds a new tag to the system.
    /// Documents already stored in the system will not be tagged automatically, they have to be reprocessed.
    pub fn add_tag(&self, tag: TagConfig) -> Result<(), IndexError> {
        self.tagger
            .write()
            .map_err(|_| IndexError::LockError("tagger".into()))?
            .add_tag(tag)
            .map_err(|x| x.into())
    }

    /// Adds or replaces a tag to the system.
    /// Documents already stored in the system will not be tagged automatically, they have to be reprocessed.
    pub fn add_or_replace_tag(&self, tag: TagConfig) -> Result<(), IndexError> {
        self.tagger
            .write()
            .map_err(|_| IndexError::LockError("tagger".into()))?
            .add_or_replace_tag(tag)
            .map_err(|x| x.into())
    }

    /// Removes a tag from the repository
    pub fn remove_tag(&self, id: TagId) -> Result<(), IndexError> {
        let mut read = self
            .tagger
            .write()
            .map_err(|_| IndexError::LockError("tagger".into()))?;
        read.remove_tag(id).map_err(|x| x.into())
    }

    /// Retrieves the tag configuration of a tag given its ID
    pub fn get_tag(&self, id: TagId) -> Option<TagConfig> {
        let read = match self.tagger.read() {
            Ok(r) => r,
            Err(e) => {
                error!("Could not lock tagger {}", e);
                return None;
            }
        };
        read.get_tag(id)
    }

    /// Retrieves the configuration of all tags in the system
    pub fn get_tags(&self) -> Vec<TagConfig> {
        let read = match self.tagger.read() {
            Ok(r) => r,
            Err(e) => {
                error!("Could not lock tagger {}", e);
                return vec![];
            }
        };
        read.get_tags()
    }
}
