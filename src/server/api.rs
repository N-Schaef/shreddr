use crate::index::document_repository::{DocumentData, FilterOptions, SortOrder};
use crate::index::DocId;
use crate::metadata::tag::{TagConfig, TagId};
use rocket::http::ContentType;
use rocket::Data;
use rocket::State;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::sync::Arc;

use crate::index::Index;
use rocket_contrib::json::Json;

//////////////////////////////////////////////
//////////        Tags        ////////////////
//////////////////////////////////////////////

#[get("/tags/<id>")]
pub fn tag(index: State<Arc<Index>>, id: TagId) -> Option<Json<TagConfig>> {
    let tag = (*index).get_tag(id);
    match tag {
        None => None,
        Some(t) => Some(Json(t)),
    }
}

#[delete("/tags/<id>")]
pub fn remove_tag(index: State<Arc<Index>>, id: TagId) -> Result<(), crate::index::IndexError> {
    (*index).remove_tag(id)?;
    Ok(())
}

#[get("/tags")]
pub fn tags(index: State<Arc<Index>>) -> Json<Vec<TagConfig>> {
    let tags = (*index).get_tags();
    Json(tags)
}

//////////////////////////////////////////////
//////////        Documents   ////////////////
//////////////////////////////////////////////

#[get("/documents?<offset>&<count>&<order>&<tag>&<query>")]
pub fn documents(
    index: State<Arc<Index>>,
    offset: usize,
    count: usize,
    order: Option<usize>,
    tag: String,
    query: Option<String>,
) -> Result<Json<Vec<DocumentData>>, Box<dyn std::error::Error>> {
    let order_parsed = match order {
        Some(0) => SortOrder::ImportedDate,
        Some(1) => SortOrder::InferredDate,
        _ => SortOrder::NoOrder,
    };
    let mut tags = std::collections::HashSet::<TagId>::new();
    if !tag.is_empty() {
        let tag_strs = tag.split(',');
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

#[get("/documents/<id>/download")]
pub fn download_document(
    index: State<Arc<Index>>,
    id: DocId,
) -> Result<rocket::response::NamedFile, Box<dyn std::error::Error>> {
    let path = index.get_document_path(id)?;
    Ok(rocket::response::NamedFile::open(&path)?)
}

#[get("/documents/<id>/reimport")]
pub fn reimport_document(
    index: State<Arc<Index>>,
    id: DocId,
) -> Result<(), Box<dyn std::error::Error>> {
    index.reprocess_document(id)?;
    Ok(())
}

#[get("/documents/<id>/reimport?<ocr>")]
pub fn reimport_document_ocr(
    index: State<Arc<Index>>,
    id: DocId,
    ocr: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if ocr {
        index.reprocess_document_force_ocr(id)?;
    } else {
        index.reprocess_document(id)?;
    }
    Ok(())
}

#[delete("/documents/<docid>/tags/<tagid>")]
pub fn delete_tag_from_document(
    index: State<Arc<Index>>,
    docid: DocId,
    tagid: TagId,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = index.get_document(docid)?;
    doc.tags.retain(|t| t != &tagid);
    index.update_doc_metadata(doc)?;
    Ok(())
}

#[put("/documents/<docid>/tags/<tagid>")]
pub fn add_tag_to_document(
    index: State<Arc<Index>>,
    docid: DocId,
    tagid: TagId,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = index.get_document(docid)?;

    if !doc.tags.contains(&tagid) {
        doc.tags.push(tagid);
    }

    index.update_doc_metadata(doc)?;
    Ok(())
}

#[post("/documents", data = "<data>")]
pub fn upload_document(
    index: State<Arc<Index>>,
    content_type: &ContentType,
    data: Data,
) -> &'static str {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file"),
    ]);

    let multipart_form_data = MultipartFormData::parse(content_type, data, options).unwrap();
    let file = multipart_form_data.files.get("file");

    if let Some(file_fields) = file {
        let file_field = &file_fields[0]; // Because we only put one "file" field to the allowed_fields, the max length of this file_fields is 1.

        let _content_type = &file_field.content_type;
        let file_name = &file_field.file_name;
        let path = &file_field.path;

        let base_dir = index.get_tmp_dir();
        let tmp_dir = tempfile::tempdir_in(base_dir).unwrap();
        let tmp_file = tmp_dir.path().join(file_name.as_ref().unwrap());
        debug!(
            "Copying uploaded file from {:#?} to {:#?}",
            &path, &tmp_file
        );
        std::fs::copy(path, &tmp_file).unwrap();

        index.import_document(&tmp_file).unwrap();
    }

    "ok"
}
