use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tantivy::query::BooleanQuery;
use tantivy::tokenizer::{NgramTokenizer, Token, Tokenizer};

use crate::index::DocId;
use array_tool::vec::Intersect;

use tantivy::collector::{Count, TopDocs};
use tantivy::schema::*;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};

use super::{DocumentData, DocumentRepository, DocumentRepositoryError, FilterOptions, SortOrder};

pub struct LocalDocumentRepository {
    index_writer: IndexWriter,
    index_reader: IndexReader,
    doc_file: PathBuf,
    schema: Schema,
}

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("error in tantivy index `{0}`")]
    TantivyException(String),
    #[error("index schema does not contain field `{0}`")]
    UnknownField(String),
    #[error("document does not contain field `{0}`")]
    MissingFieldError(String),
    #[error("field `{0}` in document has wrong type")]
    TypeMismatch(String),
    #[error("document not found at address `{0:?}`")]
    DocumentNotFound(tantivy::DocAddress),
    #[error("document does not contain a body `{0}`")]
    DocumentMissingBody(DocId),
    #[error("could not fetch document with id `{0}`")]
    DocumentFetchError(DocId),
    #[error("function is not implemented")]
    NotImplementedError(),
    #[error("could not load/write document file")]
    ConfigError(#[from] confy::ConfyError),
}

impl LocalDocumentRepository {
    pub fn new(index_dir: &Path) -> Result<LocalDocumentRepository, IndexerError> {
        let i_dir: PathBuf = index_dir.into();
        let index = LocalDocumentRepository::init_index(&i_dir)?;
        let schema = index.schema();
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
            .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))?;

        let writer = index
            .writer(5000000)
            .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))?;

        Ok(LocalDocumentRepository {
            schema,
            index_reader: reader,
            index_writer: writer,
            doc_file: index_dir.join("docs.yaml"),
        })
    }

    fn init_index(index_dir: &Path) -> Result<Index, IndexerError> {
        info!("Initializing index");
        let tokenizer = NgramTokenizer::new(3, 6, false);

        let mut schema_builder = Schema::builder();
        let full_text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("ngram")
                .set_index_option(IndexRecordOption::WithFreqs),
        );
        let id_option = IntOptions::default()
            .set_stored()
            .set_fast(Cardinality::SingleValue)
            .set_indexed();

        schema_builder.add_text_field("body", full_text_options);
        schema_builder.add_u64_field("id", id_option);

        let schema = schema_builder.build();
        let dir = tantivy::directory::MmapDirectory::open(index_dir)
            .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))?;
        let index = Index::open_or_create(dir, schema)
            .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))?;
        index.tokenizers().register("ngram", tokenizer);
        Ok(index)
    }

    fn _add_document(&mut self, doc_data: &DocumentData) -> Result<(), IndexerError> {
        let mut doc = Document::default();

        let body = self
            .schema
            .get_field("body")
            .ok_or_else(|| IndexerError::UnknownField("body".into()))?;
        let id = self
            .schema
            .get_field("id")
            .ok_or_else(|| IndexerError::UnknownField("id".into()))?;

        debug!("Trying to delete {} from index if it exists", doc_data.id);
        self.index_writer
            .delete_term(tantivy::Term::from_field_u64(id, doc_data.id));

        let body_str = match &doc_data.body {
            Some(s) => s.clone(),
            None => {
                return Err(IndexerError::DocumentMissingBody(doc_data.id));
            }
        };
        doc.add_text(body, &body_str);
        doc.add_u64(id, doc_data.id);

        debug!("Adding document {} to index", doc_data.id);
        self.index_writer.add_document(doc);
        let mut cfg: Vec<DocumentData> = confy::load_path(&self.doc_file)?;
        cfg.retain(|d| d.id != doc_data.id);
        cfg.push(doc_data.clone());
        confy::store_path(&self.doc_file, cfg)?;
        self.index_writer
            .commit()
            .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))?;
        Ok(())
    }

    /// Removes a document from the repository
    fn _remove_document(&mut self, doc_id: DocId) -> Result<(), IndexerError> {
        let id = self
            .schema
            .get_field("id")
            .ok_or_else(|| IndexerError::UnknownField("id".into()))?;
        debug!("Deleting {} from index if it exists", doc_id);
        self.index_writer
            .delete_term(tantivy::Term::from_field_u64(id, doc_id));

        let mut cfg: Vec<DocumentData> = confy::load_path(&self.doc_file)?;
        cfg.retain(|d| d.id != doc_id);
        confy::store_path(&self.doc_file, cfg)?;
        Ok(())
    }

    /// Retrieves a document from the repository
    fn _get_document(&self, id: DocId) -> Result<DocumentData, IndexerError> {
        let cfg: Vec<DocumentData> = confy::load_path(&self.doc_file)?;
        for doc in &cfg {
            if doc.id == id {
                return Ok(doc.clone());
            }
        }
        Err(IndexerError::DocumentFetchError(id))
    }

    /// Returns the number of indexed documents
    fn _len(&self) -> Result<usize, IndexerError> {
        let searcher = self.index_reader.searcher();
        searcher
            .search(&tantivy::query::AllQuery {}, &Count)
            .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))
    }

    ///Gets all documents between [offset,offset+count]
    fn _get_documents(
        &self,
        offset: usize,
        count: usize,
    ) -> Result<Vec<DocumentData>, IndexerError> {
        let cfg: Vec<DocumentData> = confy::load_path(&self.doc_file)?;
        let slice = &cfg[offset..std::cmp::min(offset + count, cfg.len())];
        Ok(slice.to_vec())
    }

    ///
    fn _get_filtered_documents(
        &self,
        offset: usize,
        count: usize,
        filter: FilterOptions,
    ) -> Result<Vec<DocumentData>, IndexerError> {
        let mut cfg: Vec<DocumentData> = confy::load_path(&self.doc_file)?;
        let mut sorted = false;
        if let Some(query) = &filter.query {
            if !query.is_empty() {
                let query_result = self.query(&query)?;
                let mut max: f32 = 0.0;
                for v in query_result.values() {
                    if v > &max {
                        max = *v
                    }
                }
                cfg.retain(|d| {
                    query_result
                        .get(&d.id)
                        .map(|score| score > &(max * 0.1))
                        .unwrap_or(false)
                });
                cfg.sort_unstable_by(|a, b| {
                    query_result
                        .get(&a.id)
                        .unwrap()
                        .partial_cmp(query_result.get(&b.id).unwrap())
                        .unwrap()
                        .reverse()
                });
                sorted = true;
            }
        }
        if !filter.tags.is_empty() {
            cfg.retain(|d| filter.tags.intersect(d.tags.clone()).len() >= filter.tags.len())
        }
        match filter.sort {
            SortOrder::ImportedDate => cfg.sort_unstable_by(|a, b| {
                a.imported_date
                    .partial_cmp(&b.imported_date)
                    .unwrap()
                    .reverse()
            }),
            SortOrder::InferredDate => cfg.sort_unstable_by(|a, b| {
                a.inferred_date
                    .partial_cmp(&b.inferred_date)
                    .unwrap_or(std::cmp::Ordering::Less)
                    .reverse()
            }),
            SortOrder::NoOrder => {
                if !sorted {
                    cfg.sort_unstable_by(|a, b| {
                        a.imported_date
                            .partial_cmp(&b.imported_date)
                            .unwrap()
                            .reverse()
                    });
                }
            }
        }
        let slice =
            &cfg[std::cmp::min(offset, cfg.len())..std::cmp::min(offset + count, cfg.len())];
        Ok(slice.to_vec())
    }

    fn _update_metadata(&mut self, doc: &DocumentData) -> Result<(), IndexerError> {
        //Update metadata array
        let mut cfg: Vec<DocumentData> = confy::load_path(&self.doc_file)?;
        cfg.retain(|d| d.id != doc.id);
        cfg.push(doc.clone());
        confy::store_path(&self.doc_file, cfg)?;

        Ok(())
    }

    fn query(&self, query: &str) -> Result<HashMap<DocId, f32>, IndexerError> {
        let tokenizer = NgramTokenizer::new(3, 6, false);
        let mut queries: Vec<Term> = Vec::new();
        let mut stream = tokenizer.token_stream(&query);
        stream.process(&mut |token: &Token| {
            queries.push(Term::from_field_text(
                self.schema.get_field("body").unwrap(),
                &token.text,
            ))
        });
        let q = BooleanQuery::new_multiterms_query(queries);
        let searcher = self.index_reader.searcher();
        let result = searcher
            .search(&q, &TopDocs::with_limit(100))
            .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))?;
        let mut set = HashMap::new();
        for (score, doc_address) in result {
            let retrieved_doc = searcher
                .doc(doc_address)
                .map_err(|e| IndexerError::TantivyException(format!("{:?}", e)))?;
            let id = retrieved_doc
                .get_first(
                    self.schema
                        .get_field("id")
                        .ok_or_else(|| IndexerError::UnknownField("tags".into()))?,
                )
                .ok_or(IndexerError::DocumentNotFound(doc_address))?
                .u64_value();
            set.insert(id, score);
        }
        debug!("Got FTS results: {:#?}", set);
        Ok(set)
    }

    fn _contains_hash(&self, hash: &str) -> Result<Option<DocId>, IndexerError> {
        let cfg: Vec<DocumentData> = confy::load_path(&self.doc_file)?;
        match cfg.iter().find(|&d| d.hash == hash) {
            Some(d) => Ok(Some(d.id)),
            None => Ok(None),
        }
    }
}

impl DocumentRepository for LocalDocumentRepository {
    /// Adds or replaces a document to the repository
    fn add_document(&mut self, doc: &DocumentData) -> Result<(), DocumentRepositoryError> {
        self._add_document(doc).map_err(|e| e.into())
    }

    /// Adds or replaces a document to the repository
    fn update_metadata(&mut self, doc: &DocumentData) -> Result<(), DocumentRepositoryError> {
        self._update_metadata(doc).map_err(|e| e.into())
    }

    /// Removes a document from the repository
    fn remove_document(&mut self, id: DocId) -> Result<(), DocumentRepositoryError> {
        self._remove_document(id).map_err(|e| e.into())
    }

    /// Retrieves a document from the repository
    fn get_document(&self, id: DocId) -> Result<DocumentData, DocumentRepositoryError> {
        self._get_document(id).map_err(|e| e.into())
    }

    /// Returns the number of indexed documents
    fn len(&self) -> Result<usize, DocumentRepositoryError> {
        self._len().map_err(|e| e.into())
    }

    ///Gets all documents between [offset,offset+count]
    fn get_documents(
        &self,
        offset: usize,
        count: usize,
    ) -> Result<Vec<DocumentData>, DocumentRepositoryError> {
        self._get_documents(offset, count).map_err(|e| e.into())
    }

    fn contains_hash(&self, hash: &str) -> Result<Option<DocId>, DocumentRepositoryError> {
        self._contains_hash(hash).map_err(|e| e.into())
    }

    fn get_filtered_documents(
        &self,
        offset: usize,
        count: usize,
        filter: FilterOptions,
    ) -> Result<Vec<DocumentData>, DocumentRepositoryError> {
        self._get_filtered_documents(offset, count, filter)
            .map_err(|e| e.into())
    }
}

#[derive(Debug)]
pub struct SearchResult {
    id: u64,
    score: f32,
}
