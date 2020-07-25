mod api;
mod assets;
mod pages;

use std::sync::Mutex;
use rocket_contrib::serve::StaticFiles;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use crate::index::JobType;


pub struct Server {}

impl Server {
    pub fn start(data_dir: &Path, index: Arc<super::index::Index>, job_queue: Mutex<Sender<JobType>>) {
        rocket::ignite()
            .manage(index)
            .manage(job_queue)
            .mount(
                "/",
                routes![
                    pages::index,
                    pages::documents,
                    pages::document,
                    pages::document_edit,
                    pages::tags,
                    pages::edit_tag,
                    pages::create_or_update_tag,
                    pages::assets
                ],
            )
            .mount(
                "/api",
                routes![
                    api::job_status,
                    api::tag,
                    api::tags,
                    api::remove_tag,
                    api::remove_document,
                    api::documents,
                    api::upload_document,
                    api::download_document,
                    api::reimport_document,
                    api::reimport_document_ocr,
                    api::add_tag_to_document,
                    api::delete_tag_from_document
                ],
            )
            .mount(
                "/thumbnails",
                StaticFiles::from(data_dir.join("thumbnails")),
            )
            .launch();
    }
}
