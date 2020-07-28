use crate::index::JobType;
use crossbeam_channel::Sender;
use rocket::State;

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
