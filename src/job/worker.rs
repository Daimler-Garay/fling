use crate::job::JobId;

#[derive(Debug)]
struct Worker {
    job: Option<JobId>,
    status: WorkerStatus,
}

impl Worker {
    fn new() -> Self {
        Self {
            job: None,
            status: WorkerStatus::Idle,
        }
    }

    fn assign(&mut self, job_id: JobId) {
        self.job = Some(job_id);
    }

    fn remove(&mut self) {
        self.job = None;
    }
}

#[derive(Debug)]
enum WorkerStatus {
    Idle,
    Busy,
}

impl WorkerStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!((self, next), (Self::Idle, Self::Busy))
    }
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error("invalid job status transition from {from:?} to {to:?}")]
    InvalidStatusTransition {
        from: WorkerStatus,
        to: WorkerStatus,
    },
}
