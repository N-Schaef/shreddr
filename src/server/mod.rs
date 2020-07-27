mod api;
mod assets;
mod documents;
mod pages;

use crate::index::JobType;
use crossbeam_channel::Sender;
use rocket_contrib::serve::StaticFiles;

use std::sync::Arc;
use std::sync::Mutex;

pub struct Server {}

impl Server {
    pub fn start(
        cfg: crate::cli::ShreddrConfig,
        index: Arc<super::index::Index>,
        job_queue: Mutex<Sender<JobType>>,
    ) {
        rocket::ignite()
            .manage(index)
            .manage(job_queue)
            .manage(cfg.clone())
            .mount(
                "/",
                routes![
                    pages::index,
                    pages::tags,
                    pages::edit_tag,
                    pages::create_or_update_tag,
                    pages::assets
                ],
            )
            .mount(
                "/documents",
                routes![
                    documents::index_get,
                    documents::index_get_json_fail,
                    documents::index_get_json,
                    documents::upload,
                    documents::document,
                    documents::document_json,
                    documents::document_download,
                    documents::document_remove,
                    documents::document_reimport,
                    documents::document_delete_tag,
                    documents::document_add_tag,
                    documents::document_patch,
                    // documents::document_edit,
                    //documents::document,
                ],
            )
            .mount(
                "/api",
                routes![api::job_status, api::tag, api::tags, api::remove_tag,],
            )
            .mount(
                "/thumbnails",
                StaticFiles::from(cfg.data_dir.join("thumbnails")),
            )
            .launch();
    }
}
