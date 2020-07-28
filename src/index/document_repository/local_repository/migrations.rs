use crate::index::document_repository::ExtractedData;
use crate::metadata::tag::TagId;
use chrono::serde::{ts_seconds, ts_seconds_option};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct VersionTest {
    version: usize,
}

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("unsupported repo version {0}")]
    VersionError(usize),
    #[error("could not load/write document file")]
    ConfigError(#[from] confy::ConfyError),
}

pub fn migrate(file: &Path) -> Result<(), MigrationError> {
    debug!("Checking migrations");
    if !file.exists(){
        return Ok(());
    }
    let version_object: VersionTest = confy::load_path(file).unwrap_or_default();
    let version = version_object.version;

    if version > 1 {
        return Err(MigrationError::VersionError(version));
    }

    if version == 0 {
        info!("Local Document Repository is stored in V0 format. Migrating to V1");
        let v0: RepoV0 = confy::load_path(file)?;
        let v1: RepoV1 = v0.into();
        confy::store_path(file, v1)?;
    }

    Ok(())
}

//////////////////////////////////////////////
//////////        V0          ////////////////
//////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DocumentDataV0 {
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
type RepoV0 = Vec<DocumentDataV0>;

//////////////////////////////////////////////
//////////        V1          ////////////////
//////////////////////////////////////////////

type RepoV1 = super::Documents;
impl From<RepoV0> for RepoV1 {
    fn from(repo: RepoV0) -> Self {
        RepoV1 {
            version: 1,
            docs: repo
                .into_iter()
                .map(|d| super::DocumentData {
                    id: d.id,
                    original_filename: d.original_filename,
                    title: d.title,
                    body: d.body,
                    tags: d.tags,
                    imported_date: d.imported_date,
                    hash: d.hash,
                    file_size: d.file_size,
                    language: d.language,
                    extracted: ExtractedData {
                        phone: vec![],
                        email: vec![],
                        link: vec![],
                        iban: vec![],
                        doc_date: d.inferred_date,
                    },
                })
                .collect(),
        }
    }
}
impl std::default::Default for RepoV1 {
    fn default() -> Self {
        RepoV1 {
            version: 1,
            docs: vec![],
        }
    }
}
