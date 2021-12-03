use super::assets::{Asset, Pages};
use rocket::http::{ContentType, Status};

use rocket::response;
use rocket::response::Redirect;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::PathBuf;

use string_template::Template;

//////////////////////////////////////////////
//////////        Helpers   ////////////////
//////////////////////////////////////////////

pub fn get_content_page<'r>(page_name: &str) -> response::Result<'r> {
    //Header
    let mut document = match Pages::get("header.html") {
        Some(h) => {
            let mut content = h.data.into_owned();
            let template = Template::new(&std::str::from_utf8_mut(&mut content).unwrap());
            let v = vec![clap::crate_version!()];
            template.render_positional(&v).into_bytes()
        }
        None => return Err(Status::NotFound),
    };

    //Content
    match Pages::get(page_name) {
        Some(h) => document.append(&mut h.data.into_owned()),
        None => return Err(Status::NotFound),
    };

    //Footer
    match Pages::get("footer.html") {
        Some(h) => document.append(&mut h.data.into_owned()),
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
            let mut content = h.data.into_owned();
            let template = Template::new(&std::str::from_utf8_mut(&mut content).unwrap());
            let v = vec![clap::crate_version!()];
            template.render_positional(&v).into_bytes()
        }
        None => return Err(Status::NotFound),
    };

    //Content
    match Pages::get(page_name) {
        Some(h) => {
            let mut content = h.data.into_owned();
            let template = Template::new(&std::str::from_utf8_mut(&mut content).unwrap());
            document.append(&mut template.render_named(values).into_bytes());
        }
        None => return Err(Status::NotFound),
    };

    //Footer
    match Pages::get("footer.html") {
        Some(h) => document.append(&mut h.data.into_owned()),
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
    Redirect::to(uri!("/documents", super::documents::index_get))
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
                .sized_body(Cursor::new(d.data))
                .ok()
        },
    )
}
