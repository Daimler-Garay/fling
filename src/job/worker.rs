use crate::job::JobId;

#[derive(Debug)]
struct Worker {
    job: Option<JobId>,
}

impl Worker {
    fn new() -> Self {
        Self { job: None }
    }

    fn assign(&mut self, job_id: JobId) -> Result<(), WorkerError> {
        if self.job.is_some() {
            return Err(WorkerError::AlreadyBusy);
        }
        self.job = Some(job_id);
        Ok(())
    }

    fn remove(&mut self) {
        self.job = None;
    }
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error("Cannot assign job to a busy worker.")]
    AlreadyBusy,
}
