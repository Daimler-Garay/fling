use jiff::Zoned;

use crate::job::{Job, JobId, JobQueue, job::JobStore, worker::Worker};

pub mod job;

fn main() {
    let job: Job = Job::new(
        "test".to_string(),
        "description".to_string(),
        "param".to_string(),
        Zoned::now(),
    );

    println!("{job:#?}");

    let mut storage: JobStore = JobStore::new();
    let mut queue: JobQueue = JobQueue::new();
    let mut worker: Worker = Worker::new();

    let job_id = storage.add(job).unwrap();

    queue.push(job_id);
    println!("{queue:#?}");

    let runnable: JobId = queue.take().unwrap();

    worker.assign(runnable);

    println!("{storage:#?}");
    println!("{queue:#?}");
    println!("{runnable:#?}");
}
