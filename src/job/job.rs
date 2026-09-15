use std::collections::{HashMap, VecDeque};

use jiff::Zoned;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct JobId {
    id: Uuid,
}

impl JobId {
    pub fn new() -> Self {
        Self { id: Uuid::now_v7() }
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    pub id: JobId,
    pub job_type: JobType,
    pub name: String,
    pub description: String,
    pub parameters: String,
    pub status: JobStatus,
    pub created_at: Zoned,
    pub starts_at: Zoned,
}

impl Job {
    #[must_use]
    pub fn new(name: String, description: String, parameters: String, starts_at: Zoned) -> Self {
        Self {
            id: JobId::new(),
            job_type: JobType::JobOne,
            name,
            description,
            parameters,
            status: JobStatus::Pending,
            created_at: Zoned::now(),
            starts_at,
        }
    }

    pub fn check_status(&self) -> &JobStatus {
        &self.status
    }

    pub fn update_status(&mut self, next: JobStatus) -> Result<(), JobError> {
        if !self.status.can_transition_to(next) {
            return Err(JobError::InvalidStatusTransition {
                from: self.status,
                to: next,
            });
        }

        self.status = next;
        Ok(())
    }
}

#[derive(Debug)]
pub struct JobStore {
    jobs: HashMap<JobId, Job>,
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    pub fn add(&mut self, job: Job) -> Result<JobId, JobStoreError> {
        let id = job.id.clone();

        if self.jobs.contains_key(&id) {
            return Err(JobStoreError::AlreadyExists { id });
        }

        self.jobs.insert(id.clone(), job);

        Ok(id)
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    pub fn get(&self, id: &JobId) -> Option<&Job> {
        self.jobs.get(id)
    }
}

#[derive(Debug)]
pub struct JobQueue {
    job: VecDeque<JobId>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            job: VecDeque::new(),
        }
    }

    pub fn push(&mut self, job_id: JobId) {
        self.job.push_back(job_id);
    }

    pub fn take(&mut self) -> Option<JobId> {
        self.job.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.job.is_empty()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// State Machine and valid state transitions
impl JobStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Scheduled | Self::Cancelled)
                | (Self::Scheduled, Self::Running | Self::Cancelled)
                | (
                    Self::Running,
                    Self::Completed | Self::Failed | Self::Cancelled
                )
        )
    }
}

// Placeholder
#[derive(Debug, PartialEq, Eq)]
pub enum JobType {
    JobOne,
}

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("invalid job status transition from {from:?} to {to:?}")]
    InvalidStatusTransition { from: JobStatus, to: JobStatus },
}

#[derive(Debug, thiserror::Error)]
pub enum JobStoreError {
    #[error("job id: {id} already exists")]
    AlreadyExists { id: JobId },
    #[error("cannot find job id: {id}")]
    NotFound { id: JobId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions_are_allowed() {
        let valid = [
            (JobStatus::Pending, JobStatus::Scheduled),
            (JobStatus::Pending, JobStatus::Cancelled),
            (JobStatus::Scheduled, JobStatus::Running),
            (JobStatus::Scheduled, JobStatus::Cancelled),
            (JobStatus::Running, JobStatus::Completed),
            (JobStatus::Running, JobStatus::Failed),
            (JobStatus::Running, JobStatus::Cancelled),
        ];

        for (from, to) in valid {
            assert!(
                from.can_transition_to(to),
                "{from:?} should be allowed to transition to {to:?}"
            );
        }
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        let invalid = [
            (JobStatus::Pending, JobStatus::Completed),
            (JobStatus::Pending, JobStatus::Failed),
            (JobStatus::Scheduled, JobStatus::Completed),
            (JobStatus::Completed, JobStatus::Running),
            (JobStatus::Failed, JobStatus::Running),
            (JobStatus::Cancelled, JobStatus::Running),
        ];

        for (from, to) in invalid {
            assert!(
                !from.can_transition_to(to),
                "{from:?} should not be allowed to transition to {to:?}"
            );
        }
    }
}
