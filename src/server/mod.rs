mod api;
mod assets;
mod documents;
mod pages;
mod tags;

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
            .mount("/", routes![pages::index, pages::assets])
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
                "/tags",
                routes![
                    tags::tag_json,
                    tags::tags_json,
                    tags::remove_tag,
                    tags::tags,
                    tags::edit_tag,
                    tags::create_or_update_tag,
                ],
            )
            .mount("/api", routes![api::job_status,])
            .mount(
                "/thumbnails",
                StaticFiles::from(cfg.data_dir.join("thumbnails")),
            )
            .launch();
    }
}
