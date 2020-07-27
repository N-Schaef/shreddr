use super::pages::{get_content_page, get_content_page_with_named_template};
use crate::index::document_repository::{DocumentData, FilterOptions, SortOrder};
use crate::index::{DocId, Index};
use crate::metadata::tag::TagId;
use crate::JobType;
use rocket::http::ContentType;
use rocket::Data;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use chrono::serde::ts_seconds_option;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use rocket::response;
use rocket::State;
use rocket_contrib::json::Json;

//////////////////////////////////////////////
//////////        INDEX (/)   ////////////////
//////////////////////////////////////////////

/// GET the default documents overview
#[get("/")]
pub fn index_get<'r>() -> response::Result<'r> {
    get_content_page("documents.html")
}

/// GET a list of  documents as JSON document
#[get("/json", format = "json")]
pub fn index_get_json_fail() -> response::status::BadRequest<&'static str> {
    response::status::BadRequest(Some("Missing parameter(s): offset, count"))
}

/// GET a list of (filtered) documents as JSON document
#[get("/json?<offset>&<count>&<order>&<tag>&<query>", format = "json")]
pub fn index_get_json(
    index: State<Arc<Index>>,
    offset: usize,
    count: usize,
    order: Option<usize>,
    tag: Option<String>,
    query: Option<String>,
) -> Result<Json<Vec<DocumentData>>, Box<dyn std::error::Error>> {
    let order_parsed = match order {
        Some(0) => SortOrder::ImportedDate,
        Some(1) => SortOrder::InferredDate,
        _ => SortOrder::NoOrder,
    };
    let mut tags = std::collections::HashSet::<TagId>::new();
    let tag_str = tag.unwrap_or_default();
    if !tag_str.is_empty() {
        let tag_strs = tag_str.split(',');
        for tag_s in tag_strs {
            tags.insert(tag_s.parse::<TagId>()?);
        }
    }

    let filter = FilterOptions {
        sort: order_parsed,
        tags: tags.into_iter().collect(),
        query,
    };
    let docs = (*index).get_sorted_documents(offset, count, filter)?;
    Ok(Json(docs))
}

/// POST a new document
#[post("/", data = "<data>")]
pub fn upload(
    index: State<Arc<Index>>,
    send: State<Mutex<Sender<JobType>>>,
    cfg: State<crate::cli::ShreddrConfig>,
    content_type: &ContentType,
    data: Data,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file").size_limit(cfg.max_upload_size),
    ]);

    let multipart_form_data = MultipartFormData::parse(content_type, data, options)?;
    let file = multipart_form_data.files.get("file");

    if let Some(file_fields) = file {
        let file_field = &file_fields[0]; // Because we only put one "file" field to the allowed_fields, the max length of this file_fields is 1.

        let _content_type = &file_field.content_type;
        let file_name = &file_field.file_name;
        let path = &file_field.path;

        let base_dir = index.get_tmp_dir();
        let tmp_file = base_dir.join(file_name.as_ref().unwrap());
        debug!(
            "Copying uploaded file from {:#?} to {:#?}",
            &path, &tmp_file
        );
        std::fs::copy(path, &tmp_file)?;

        let guard = send.lock().unwrap();
        guard.send(JobType::ImportFile {
            path: tmp_file,
            copy: false,
        })?;
    }

    Ok(())
}

//////////////////////////////////////////////
//////////        DOCUMENT (/<id>)   /////////
//////////////////////////////////////////////

/// GET one specific document
#[get("/<id>")]
pub fn document<'r>(index: State<Arc<Index>>, id: DocId) -> response::Result<'r> {
    let doc = match index.get_document(id) {
        Ok(d) => d,
        Err(e) => {
            error!("Could not retrieve document {}", e);
            return Err(rocket::http::Status::new(404, "could not find document"));
        }
    };
    let mut map = HashMap::new();
    let extracted_obj = serde_json::to_string(&doc.extracted).unwrap();
    map.insert("extracted", extracted_obj.as_str());
    map.insert("title", doc.title.as_str());
    map.insert("original_filename", doc.original_filename.as_str());
    let doc_date = doc
        .extracted
        .doc_date
        .map(|d| d.timestamp())
        .unwrap_or(0)
        .to_string();
    map.insert("doc_date", doc_date.as_str());
    let imported_date = doc.imported_date.timestamp().to_string();
    map.insert("imported_date", imported_date.as_str());
    map.insert("tags", "");
    let id_str = id.to_string();
    map.insert("id", id_str.as_str());
    let lang = doc.language.unwrap_or_else(|| "-".into());
    map.insert("lang", &lang);
    let tags: Vec<String> = doc.tags.iter().map(|t| t.to_string()).collect();
    let tags_str = format!("[{}]", tags.join(","));
    map.insert("tags", &tags_str);
    get_content_page_with_named_template("show_document.html", &map)
}

/// GET one specific document as JSON
#[get("/<id>/json", format = "json")]
pub fn document_json(
    index: State<Arc<Index>>,
    id: DocId,
) -> Result<Json<DocumentData>, Box<dyn std::error::Error>> {
    let doc = index.get_document(id)?;
    Ok(Json(doc))
}

/// DELETE one specific document as JSON
#[delete("/<id>")]
pub fn document_remove(
    index: State<Arc<Index>>,
    id: DocId,
) -> Result<(), Box<dyn std::error::Error>> {
    index.remove_document(id)?;
    Ok(())
}

/// GET the document file
#[get("/<id>/download")]
pub fn document_download(
    index: State<Arc<Index>>,
    id: DocId,
) -> Result<rocket::response::NamedFile, Box<dyn std::error::Error>> {
    let path = index.get_document_path(id)?;
    Ok(rocket::response::NamedFile::open(&path)?)
}

/// PUT which starts the reimport of the file
#[put("/<id>/reimport?<ocr>")]
pub fn document_reimport(
    send: State<Mutex<Sender<JobType>>>,
    id: DocId,
    ocr: Option<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let guard = send.lock().unwrap();
    guard.send(JobType::ReprocessFile {
        id,
        force_ocr: ocr.unwrap_or_default(),
    })?;
    Ok(())
}

///////////////// UPDATE //////////////////////

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchDocumentData {
    pub title: Option<String>,
    pub tags: Option<Vec<TagId>>,
    pub language: Option<String>,
    // Extracted metadata
    pub extracted: Option<PatchExtractedData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PatchExtractedData {
    #[serde(default)]
    pub phone: Option<Vec<String>>,
    #[serde(default)]
    pub email: Option<Vec<String>>,
    #[serde(default)]
    pub link: Option<Vec<String>>,
    #[serde(default)]
    pub iban: Option<Vec<String>>,
    #[serde(with = "ts_seconds_option")]
    pub doc_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<DocumentData> for PatchDocumentData {
    fn from(doc: DocumentData) -> Self {
        PatchDocumentData {
            language: doc.language,
            title: Some(doc.title),
            tags: Some(doc.tags),
            extracted: Some(PatchExtractedData {
                phone: Some(doc.extracted.phone),
                email: Some(doc.extracted.email),
                link: Some(doc.extracted.link),
                iban: Some(doc.extracted.iban),
                doc_date: doc.extracted.doc_date,
            }),
        }
    }
}

impl DocumentData {
    pub fn patch(&mut self, patch: PatchDocumentData) {
        if let Some(title) = patch.title {
            self.title = title;
        }

        if let Some(language) = patch.language {
            self.language = Some(language);
        }

        if let Some(tags) = patch.tags {
            self.tags = tags;
        }

        if let Some(extracted) = patch.extracted {
            let my_extracted = &mut self.extracted;
            if let Some(phone) = extracted.phone {
                my_extracted.phone = phone;
            }
            if let Some(email) = extracted.email {
                my_extracted.email = email;
            }
            if let Some(link) = extracted.link {
                my_extracted.link = link;
            }
            if let Some(iban) = extracted.iban {
                my_extracted.iban = iban;
            }
            if let Some(doc_date) = extracted.doc_date {
                my_extracted.doc_date = Some(doc_date);
            }
        }
    }
}

/// PATCH the document data
#[patch("/<id>", format = "json", data = "<patch>")]
pub fn document_patch(
    index: State<Arc<Index>>,
    id: DocId,
    patch: Json<PatchDocumentData>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = index.get_document(id)?;
    doc.patch(patch.into_inner());
    index.update_doc_metadata(doc)?;
    Ok(())
}

///////////////// Tags //////////////////////

/// DELETE tag from document
#[delete("/<id>/tags/<tagid>")]
pub fn document_delete_tag(
    index: State<Arc<Index>>,
    id: DocId,
    tagid: TagId,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = index.get_document(id)?;
    doc.tags.retain(|t| t != &tagid);
    index.update_doc_metadata(doc)?;
    Ok(())
}

#[post("/<id>/tags/<tagid>")]
pub fn document_add_tag(
    index: State<Arc<Index>>,
    id: DocId,
    tagid: TagId,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = index.get_document(id)?;

    if !doc.tags.contains(&tagid) {
        doc.tags.push(tagid);
    }

    index.update_doc_metadata(doc)?;
    Ok(())
}
