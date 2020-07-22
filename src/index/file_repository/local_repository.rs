use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{FileRepository, FileRepositoryError};
use crate::index::DocId;

pub struct LocalFileRepository {
    document_dir: PathBuf,
    documents: HashMap<DocId, PathBuf>,
}

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum LocalFileRepositoryError {
    #[error("could not read file directory `{0}`")]
    DocumentDirectoryRead(#[from] std::io::Error),
    #[error("could not extract filename from `{0}`")]
    ExtractFilenameError(PathBuf),
    #[error("could not convert filename `{0}` to string")]
    ConvertFilenameError(PathBuf),
    #[error("could not parse filename to ID {0}")]
    ParseIdError(#[from] std::num::ParseIntError),
    #[error("could not find document with ID {0}")]
    DocumentNotFoundError(DocId),
}

impl LocalFileRepository {
    pub fn new(document_dir: &Path) -> Result<LocalFileRepository, LocalFileRepositoryError> {
        let docs = LocalFileRepository::load_documents(&document_dir)?;
        Ok(LocalFileRepository {
            document_dir: document_dir.into(),
            documents: docs,
        })
    }

    fn load_documents(dir: &Path) -> Result<HashMap<DocId, PathBuf>, LocalFileRepositoryError> {
        let mut map = HashMap::new();
        let paths = std::fs::read_dir(dir)?;
        let mut max: u64 = 0;
        for path in paths {
            let p = path?.path();
            let filename = p
                .file_stem()
                .ok_or_else(|| LocalFileRepositoryError::ExtractFilenameError(p.clone()))?;
            if filename == "documents" {
                continue;
            }
            let id = filename
                .to_str()
                .ok_or_else(|| LocalFileRepositoryError::ConvertFilenameError(p.clone()))?
                .parse::<DocId>()?;
            map.insert(id, p);
            if id > max {
                max = id;
            }
        }
        Ok(map)
    }

    fn _add_document(&mut self, id: DocId, file: &Path) -> Result<(), LocalFileRepositoryError> {
        let new_path = self.document_dir.join(format!("{}.pdf", id));
        std::fs::copy(file, &new_path)?;
        self.documents.insert(id, new_path);
        Ok(())
    }

    fn _remove_document(&mut self, id: DocId) -> Result<(), LocalFileRepositoryError> {
        match self.documents.remove(&id) {
            None => {
                debug!("No file with id {} in repository", id);
                Ok(())
            },
            Some(f) => {
                info!("Removing file `{:#?} for document {}`", &f, id);
                match std::fs::remove_file(&f) {
                    Ok(_) => {
                        debug!("Removed file `{:#?}`", &f);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Could not remove file `{:#?}`: {}", &f, e);
                        Err(e.into())
                    }
                }
            }
        }
    }

    fn _get_document(&self, id: DocId) -> Result<PathBuf, LocalFileRepositoryError> {
        Ok(self
            .documents
            .get(&id)
            .ok_or(LocalFileRepositoryError::DocumentNotFoundError(id))?
            .clone())
    }
}

impl FileRepository for LocalFileRepository {
    fn add_document(&mut self, id: DocId, file: &Path) -> Result<(), FileRepositoryError> {
        self._add_document(id, file).map_err(|e| e.into())
    }

    fn remove_document(&mut self, id: DocId) -> Result<(), FileRepositoryError> {
        self._remove_document(id).map_err(|e| e.into())
    }

    fn get_document(&self, id: DocId) -> Result<PathBuf, FileRepositoryError> {
        self._get_document(id).map_err(|e| e.into())
    }
}
