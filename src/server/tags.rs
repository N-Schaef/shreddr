use super::pages::{get_content_page, get_content_page_with_named_template};
use crate::index::Index;
use crate::metadata::tag::{TagConfig, TagId};
use rocket::State;
use rocket_contrib::json::Json;
use std::sync::Arc;

use rocket::request::Form;
use rocket::response;
use rocket::response::Redirect;

use std::collections::HashMap;

use crate::metadata::tag::MatcherConfig;

/// GET a specific tag as JSON config
#[get("/<id>/json")]
pub fn tag_json(index: State<Arc<Index>>, id: TagId) -> Json<Option<TagConfig>> {
    Json((*index).get_tag(id))
}

/// DELETE a specific tag
#[delete("/<id>")]
pub fn remove_tag(index: State<Arc<Index>>, id: TagId) -> Result<(), crate::index::IndexError> {
    (*index).remove_tag(id)?;
    Ok(())
}

/// GET all tags as JSON config
#[get("/json")]
pub fn tags_json(index: State<Arc<Index>>) -> Json<Vec<TagConfig>> {
    let tags = (*index).get_tags();
    Json(tags)
}

/// GET the tags page
#[get("/")]
pub fn tags<'r>() -> response::Result<'r> {
    get_content_page("tags.html")
}

/// GET all tags as JSON config
#[get("/<id>")]
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
    "/<id>",
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
    Ok(Redirect::to(uri!("/tags", tags)))
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
