use jiff::Zoned;

use crate::job::{
    Job, JobId, JobQueue,
    job::{JobStatus, JobStore, JobStoreError},
    worker::Worker,
};

pub mod job;

fn schedule(
    job_id: JobId,
    store: &mut JobStore,
    queue: &mut JobQueue,
) -> Result<(), SchedulerError> {
    store.update_status(&job_id, JobStatus::Scheduled)?;

    queue.push(job_id);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum SchedulerError {
    #[error(transparent)]
    Job(#[from] JobStoreError),
}

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

    let job_id = storage.add(job);

    schedule(job_id, &mut storage, &mut queue);
    println!("{queue:#?}");

    let runnable: JobId = queue.take().unwrap();

    worker.assign(runnable);

    println!("{storage:#?}");
    println!("{queue:#?}");
    println!("{runnable:#?}");
}
