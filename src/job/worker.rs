use crate::job::JobId;

#[derive(Debug)]
pub struct Worker {
    pub job: Option<JobId>,
}

impl Worker {
    #[must_use]
    pub fn new() -> Self {
        Self { job: None }
    }

    pub fn assign(&mut self, job_id: JobId) -> Result<(), WorkerError> {
        if self.job.is_some() {
            return Err(WorkerError::AlreadyBusy);
        }
        self.job = Some(job_id);
        Ok(())
    }

    pub fn remove(&mut self) {
        self.job = None;
    }

    pub fn current_job(&self) -> Option<JobId> {
        self.job
    }
}

impl Default for Worker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Cannot assign job to a busy worker.")]
    AlreadyBusy,
}
