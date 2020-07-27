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

pub fn get_content_page<'r>(page_name: &str) -> response::Result<'r> {
    //Header
    let mut document = match Pages::get("header.html") {
        Some(h) => {
            let mut content = h.into_owned();
            let template = Template::new(&std::str::from_utf8_mut(&mut content).unwrap());
            let v = vec![clap::crate_version!()];
            template.render_positional(&v).into_bytes()
        }
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

pub fn get_content_page_with_named_template<'r>(
    page_name: &str,
    values: &HashMap<&str, &str>,
) -> response::Result<'r> {
    //Header
    let mut document = match Pages::get("header.html") {
        Some(h) => {
            let mut content = h.into_owned();
            let template = Template::new(&std::str::from_utf8_mut(&mut content).unwrap());
            let v = vec![clap::crate_version!()];
            template.render_positional(&v).into_bytes()
        }
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
    Redirect::to(uri!(super::documents::index_get))
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
