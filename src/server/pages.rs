use super::assets::{Asset, Pages};
use rocket::http::{ContentType, Status};
use rocket::request::Form;
use rocket::response;
use rocket::response::Redirect;
use rocket::State;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use string_template::Template;

use crate::index::Index;
use crate::metadata::tag::{MatcherConfig, TagConfig, TagId};

//////////////////////////////////////////////
//////////        Helpers   ////////////////
//////////////////////////////////////////////

fn get_content_page<'r>(page_name: &str) -> response::Result<'r> {
    //Header
    let mut document = match Pages::get("header.html") {
        Some(h) => h.into_owned(),
        None => return Err(Status::NotFound),
    };

    //Content
    match Pages::get(page_name) {
        Some(h) => document.append(&mut h.into_owned()),
        None => return Err(Status::NotFound),
    };

    //Footer
    match Pages::get("footer.html") {
        Some(h) => document.append(&mut h.into_owned()),
        None => return Err(Status::NotFound),
    };

    response::Response::build()
        .header(ContentType::HTML)
        .sized_body(Cursor::new(document))
        .ok()
}

fn get_content_page_with_template<'r>(page_name: &str, values: &[String]) -> response::Result<'r> {
    //Header
    let mut document = match Pages::get("header.html") {
        Some(h) => h.into_owned(),
        None => return Err(Status::NotFound),
    };

    //Content
    match Pages::get(page_name) {
        Some(h) => {
            let mut content = h.into_owned();
            let template = Template::new(&std::str::from_utf8_mut(&mut content).unwrap());
            let v: Vec<&str> = values.iter().map(|x| &**x).collect();
            document.append(&mut template.render_positional(&v).into_bytes());
        }
        None => return Err(Status::NotFound),
    };

    //Footer
    match Pages::get("footer.html") {
        Some(h) => document.append(&mut h.into_owned()),
        None => return Err(Status::NotFound),
    };

    response::Response::build()
        .header(ContentType::HTML)
        .sized_body(Cursor::new(document))
        .ok()
}

fn get_content_page_with_named_template<'r>(
    page_name: &str,
    values: &HashMap<&str, &str>,
) -> response::Result<'r> {
    //Header
    let mut document = match Pages::get("header.html") {
        Some(h) => h.into_owned(),
        None => return Err(Status::NotFound),
    };

    //Content
    match Pages::get(page_name) {
        Some(h) => {
            let mut content = h.into_owned();
            let template = Template::new(&std::str::from_utf8_mut(&mut content).unwrap());
            document.append(&mut template.render_named(values).into_bytes());
        }
        None => return Err(Status::NotFound),
    };

    //Footer
    match Pages::get("footer.html") {
        Some(h) => document.append(&mut h.into_owned()),
        None => return Err(Status::NotFound),
    };

    response::Response::build()
        .header(ContentType::HTML)
        .sized_body(Cursor::new(document))
        .ok()
}

//////////////////////////////////////////////
//////////        Index   ////////////////
//////////////////////////////////////////////

#[get("/")]
pub fn index() -> Redirect {
    /*
    let mut map = HashMap::new();
    let doclen = index.len().unwrap().to_string();
    map.insert("docs", doclen.as_str());
    let tags = index.get_tags();
    let taglen = tags.len().to_string();
    map.insert("tags",taglen.as_str());
    get_content_page_with_named_template("index.html",&map)
    */
    Redirect::to(uri!(documents))
}

//////////////////////////////////////////////
//////////        Documents   ////////////////
//////////////////////////////////////////////

#[get("/documents")]
pub fn documents<'r>() -> response::Result<'r> {
    get_content_page("documents.html")
}

#[get("/documents/<id>")]
pub fn document<'r>(index: State<Arc<Index>>, id: crate::index::DocId) -> response::Result<'r> {
    let doc = match index.get_document(id) {
        Ok(d) => d,
        Err(e) => {
            error!("Could not retrieve document {}", e);
            return Err(rocket::http::Status::new(404, "could not find document"));
        }
    };
    let mut map = HashMap::new();
    map.insert("title", doc.title.as_str());
    map.insert("original_filename", doc.original_filename.as_str());
    let inferred_date = doc
        .inferred_date
        .map(|d| d.timestamp())
        .unwrap_or(0)
        .to_string();
    map.insert("inferred_date", inferred_date.as_str());
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

#[derive(FromForm, Debug)]
pub struct DocForm {
    title: Option<String>,
    inferred_date: Option<i64>,
    lang: Option<String>,
}

#[post(
    "/documents/<id>",
    format = "application/x-www-form-urlencoded",
    data = "<doc_data>"
)]
pub fn document_edit(
    index: State<Arc<Index>>,
    id: crate::index::DocId,
    doc_data: Form<DocForm>,
) -> Redirect {
    let mut doc = match index.get_document(id) {
        Ok(d) => d,
        Err(e) => {
            error!("Could not retrieve document {}", e);
            return Redirect::to(uri!(document: id));
        }
    };

    if let Some(title) = &doc_data.title {
        doc.title = title.clone();
    }

    if let Some(lang) = &doc_data.lang {
        if lang.is_empty() || lang == "-" {
            doc.language = None;
        } else {
            doc.language = Some(lang.clone());
        }
    }

    if let Some(inferred_date) = &doc_data.inferred_date {
        doc.inferred_date = Some(chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp(*inferred_date, 0),
            chrono::Utc,
        ));
    }
    index.update_doc_metadata(doc).unwrap();

    Redirect::to(uri!(document: id))
}

//////////////////////////////////////////////
//////////        Tags        ////////////////
//////////////////////////////////////////////

#[get("/tags")]
pub fn tags<'r>() -> response::Result<'r> {
    get_content_page("tags.html")
}

#[get("/tags/<id>/edit")]
pub fn edit_tag<'r>(index: State<Arc<Index>>, id: TagId) -> response::Result<'r> {
    match index.get_tag(id) {
        Some(t) => edit_tag_from_config(&t),
        None => {
            let t = TagConfig {
                id: 0,
                color: None,
                name: "".into(),
                matcher: MatcherConfig::FullMatcher {
                    match_str: "".into(),
                    case_insensitive: false,
                },
            };
            edit_tag_from_config(&t)
        }
    }
}

#[post(
    "/tags/<id>/edit",
    format = "application/x-www-form-urlencoded",
    data = "<tag_form>"
)]
pub fn create_or_update_tag(
    index: State<Arc<Index>>,
    tag_form: Form<TagForm>,
    id: TagId,
) -> Result<Redirect, Box<dyn std::error::Error>> {
    let form = tag_form.0;
    let matcher = match form.matcher_type {
        1 => Some(MatcherConfig::FullMatcher {
            match_str: form.full_matcher_match_str,
            case_insensitive: form.full_matcher_case_insensitive,
        }),
        2 => Some(MatcherConfig::RegexMatcher {
            match_str: form.regex_match_string,
        }),
        3 => Some(MatcherConfig::AnyMatcher {
            match_str: form.any_matcher_match_str,
            case_insensitive: form.any_matcher_case_insensitive,
        }),
        x => {
            error!("Got unknown matcher type `{}`", x);
            None
        }
    };
    let tag = TagConfig {
        id,
        name: form.name,
        color: Some(form.color),
        matcher: matcher.unwrap(),
    };
    (*index).add_or_replace_tag(tag)?;
    Ok(Redirect::to(uri!(tags)))
}

#[derive(FromForm, Debug)]
pub struct TagForm {
    name: String,
    color: String,
    matcher_type: u64,
    full_matcher_match_str: String,
    full_matcher_case_insensitive: bool,
    any_matcher_match_str: String,
    any_matcher_case_insensitive: bool,
    regex_match_string: String,
}

fn edit_tag_from_config<'r>(tag: &TagConfig) -> response::Result<'r> {
    let mut map: HashMap<&'static str, &str> = HashMap::new();
    map.insert("name", &tag.name);
    map.insert("color", tag.color.as_deref().unwrap_or("default string"));
    // Default values
    map.insert("full_match_str", "");
    map.insert("regex_str", "");
    map.insert("any_match_str", "");
    map.insert("any_checked", "");
    map.insert("checked", "");
    match &tag.matcher {
        MatcherConfig::FullMatcher {
            match_str,
            case_insensitive,
        } => {
            map.insert("type", "1");
            map.insert("full_match_str", &match_str);

            if *case_insensitive {
                map.insert("checked", "checked");
            }
        }
        MatcherConfig::RegexMatcher { match_str } => {
            map.insert("type", "2");
            map.insert("regex_str", &match_str);
        }
        MatcherConfig::AnyMatcher {
            match_str,
            case_insensitive,
        } => {
            map.insert("type", "3");
            map.insert("any_match_str", &match_str);

            if *case_insensitive {
                map.insert("any_checked", "checked");
            }
        }
    };
    get_content_page_with_named_template("edit_tag.html", &map)
}

//////////////////////////////////////
///     Assets
//////////////////////////////////////

#[get("/assets/<file..>")]
pub fn assets<'r>(file: PathBuf) -> response::Result<'r> {
    let filename = file.display().to_string();
    Asset::get(&filename).map_or_else(
        || Err(Status::NotFound),
        |d| {
            let ext = file
                .as_path()
                .extension()
                .and_then(OsStr::to_str)
                .ok_or_else(|| Status::new(400, "Could not get file extension"))?;
            let content_type = ContentType::from_extension(ext)
                .ok_or_else(|| Status::new(400, "Could not get file content type"))?;
            response::Response::build()
                .header(content_type)
                .sized_body(Cursor::new(d))
                .ok()
        },
    )
}
