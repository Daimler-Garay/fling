use crate::job::{Job, JobExecutionError, JobId, JobOutput, JobType};

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

    pub fn execute(&self, job: &Job) -> Result<JobOutput, JobExecutionError> {
        match job.job_type() {
            JobType::PrintMessage => Ok(JobOutput::Text(print_message(job.parameters())?)),
            JobType::AlwaysFail => Err(JobExecutionError::Failed {
                message: "intentional test failure".to_string(),
            }),
        }
    }
}

impl Default for Worker {
    fn default() -> Self {
        Self::new()
    }
}

fn print_message(param: &str) -> Result<String, JobExecutionError> {
    if param.is_empty() {
        return Err(JobExecutionError::InvalidParameter {
            message: "message cannot be empty".to_string(),
        });
    }
    Ok(param.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("cannot assign job to a busy worker.")]
    AlreadyBusy,
}
