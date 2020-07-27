use crate::index::document_repository::{DocumentData, FilterOptions, SortOrder};
use crate::index::DocId;
use crate::index::JobType;
use crate::metadata::tag::{TagConfig, TagId};
use crossbeam_channel::Sender;
use rocket::http::ContentType;
use rocket::Data;
use rocket::State;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::sync::Arc;
use std::sync::Mutex;

use crate::index::Index;
use rocket_contrib::json::Json;

//////////////////////////////////////////////
//////////        Status   ////////////////
//////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JobStatus {
    Idle,
    Busy {
        current: String,
        progress: i32,
        queue: usize,
    },
}

#[get("/job")]
pub fn job_status(
    index: State<Arc<Index>>,
    send: State<Mutex<Sender<JobType>>>,
) -> Json<JobStatus> {
    let guard = send.lock().unwrap();
    let queue = guard.len();
    match index.get_current_job() {
        Err(e) => {
            error!("Could not get job `{}`", e);
            Json(JobStatus::Idle)
        }
        Ok(job) => match job {
            Some(j) => Json(JobStatus::Busy {
                current: j.job.to_string(),
                progress: j.progress,
                queue,
            }),
            None => Json(JobStatus::Idle),
        },
    }
}

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
