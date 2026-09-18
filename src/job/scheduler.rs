use super::{Job, JobId, JobQueue, JobStatus, JobStore, JobStoreError};
use crate::job::worker::{Worker, WorkerError};

/// Coordinates the store, runnable queue, and a single sequential worker.
#[derive(Debug)]
pub struct Scheduler {
    store: JobStore,
    queue: JobQueue,
    worker: Worker,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            store: JobStore::new(),
            queue: JobQueue::new(),
            worker: Worker::new(),
        }
    }

    pub fn add(&mut self, job: Job) -> Result<JobId, SchedulerError> {
        Ok(self.store.add(job)?)
    }

    pub fn get(&self, id: &JobId) -> Option<&Job> {
        self.store.get(id)
    }

    pub fn current_job(&self) -> Option<JobId> {
        self.worker.current_job()
    }

    pub fn schedule(&mut self, id: JobId) -> Result<(), SchedulerError> {
        // Running jobs must use retry so the worker is released as well.
        let job = self.store.get(&id).ok_or(JobStoreError::NotFound { id })?;
        if job.status() != JobStatus::Pending {
            return Err(SchedulerError::NotPending { id });
        }
        self.store.update_status(&id, JobStatus::Scheduled)?;
        self.queue.push(id);
        Ok(())
    }

    pub fn cancel(&mut self, id: JobId) -> Result<(), SchedulerError> {
        self.store.update_status(&id, JobStatus::Cancelled)?;
        self.queue.remove(id);
        Ok(())
    }

    pub fn dispatch(&mut self) -> Result<Option<JobId>, SchedulerError> {
        if self.worker.current_job().is_some() {
            return Err(WorkerError::AlreadyBusy.into());
        }
        let Some(id) = self.queue.job.front().copied() else {
            return Ok(None);
        };
        self.worker.assign(id)?;
        if let Err(error) = self.store.update_status(&id, JobStatus::Running) {
            self.worker.remove();
            return Err(error.into());
        }
        self.queue.take();
        Ok(Some(id))
    }

    /// Report a retryable result after the execution attempt returns.
    /// This releases the assignment; it does not interrupt executing code.
    pub fn retry(&mut self, id: JobId) -> Result<(), SchedulerError> {
        self.finish_attempt(id, JobStatus::Scheduled)?;
        self.queue.push(id);
        Ok(())
    }

    /// Report successful completion after the attempt returns.
    pub fn complete(&mut self, id: JobId) -> Result<(), SchedulerError> {
        self.finish_attempt(id, JobStatus::Completed)
    }

    /// Report terminal failure after the attempt returns.
    pub fn fail(&mut self, id: JobId) -> Result<(), SchedulerError> {
        self.finish_attempt(id, JobStatus::Failed)
    }

    fn finish_attempt(&mut self, id: JobId, next: JobStatus) -> Result<(), SchedulerError> {
        if self.worker.current_job() != Some(id) {
            return Err(SchedulerError::NotAssigned { id });
        }
        self.store.update_status(&id, next)?;
        self.worker.remove();
        Ok(())
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error(transparent)]
    Job(#[from] JobStoreError),
    #[error(transparent)]
    Worker(#[from] WorkerError),
    #[error("job {id} is not pending")]
    NotPending { id: JobId },
    #[error("job {id} is not assigned to the worker")]
    NotAssigned { id: JobId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::Zoned;

    fn add(scheduler: &mut Scheduler) -> Result<JobId, SchedulerError> {
        scheduler.add(Job::new(
            "test".into(),
            String::new(),
            String::new(),
            Zoned::now(),
        ))
    }

    fn status(scheduler: &Scheduler, id: JobId) -> Option<JobStatus> {
        scheduler.get(&id).map(Job::status)
    }

    #[test]
    fn cancelled_jobs_never_dispatch() -> Result<(), SchedulerError> {
        let mut scheduler = Scheduler::new();
        let pending = add(&mut scheduler)?;
        let cancelled = add(&mut scheduler)?;
        let runnable = add(&mut scheduler)?;
        scheduler.schedule(cancelled)?;
        scheduler.schedule(runnable)?;
        scheduler.cancel(pending)?;
        scheduler.cancel(cancelled)?;
        assert_eq!(status(&scheduler, pending), Some(JobStatus::Cancelled));
        assert_eq!(status(&scheduler, cancelled), Some(JobStatus::Cancelled));
        assert_eq!(scheduler.dispatch()?, Some(runnable));
        assert_eq!(status(&scheduler, runnable), Some(JobStatus::Running));
        scheduler.complete(runnable)?;
        assert_eq!(scheduler.dispatch()?, None);
        Ok(())
    }

    #[test]
    fn retry_releases_worker_and_enqueues_once_at_back() -> Result<(), SchedulerError> {
        let mut scheduler = Scheduler::new();
        let first = add(&mut scheduler)?;
        let second = add(&mut scheduler)?;
        scheduler.schedule(first)?;
        scheduler.schedule(second)?;
        assert_eq!(scheduler.dispatch()?, Some(first));
        scheduler.retry(first)?;
        assert_eq!(scheduler.current_job(), None);
        assert_eq!(status(&scheduler, first), Some(JobStatus::Scheduled));
        assert!(scheduler.retry(first).is_err());
        assert!(scheduler.schedule(first).is_err());
        assert_eq!(scheduler.dispatch()?, Some(second));
        scheduler.complete(second)?;
        assert_eq!(scheduler.dispatch()?, Some(first));
        scheduler.fail(first)?;
        assert_eq!(status(&scheduler, first), Some(JobStatus::Failed));
        assert_eq!(scheduler.current_job(), None);
        assert!(scheduler.retry(first).is_err());
        assert_eq!(scheduler.dispatch()?, None);
        Ok(())
    }

    #[test]
    fn rejected_operations_preserve_assignment_and_queue() -> Result<(), SchedulerError> {
        let mut scheduler = Scheduler::new();
        let running = add(&mut scheduler)?;
        let queued = add(&mut scheduler)?;
        scheduler.schedule(running)?;
        scheduler.schedule(queued)?;
        assert_eq!(scheduler.dispatch()?, Some(running));
        assert!(scheduler.dispatch().is_err());
        assert!(scheduler.cancel(running).is_err());
        assert!(scheduler.schedule(running).is_err());
        assert!(scheduler.retry(queued).is_err());
        assert!(scheduler.complete(queued).is_err());
        assert!(scheduler.fail(queued).is_err());
        assert!(scheduler.cancel(JobId::new()).is_err());
        assert!(scheduler.schedule(JobId::new()).is_err());
        assert_eq!(scheduler.current_job(), Some(running));
        assert_eq!(status(&scheduler, running), Some(JobStatus::Running));
        assert_eq!(status(&scheduler, queued), Some(JobStatus::Scheduled));
        scheduler.complete(running)?;
        assert_eq!(status(&scheduler, running), Some(JobStatus::Completed));
        assert_eq!(scheduler.dispatch()?, Some(queued));
        scheduler.complete(queued)?;
        assert_eq!(scheduler.dispatch()?, None);
        Ok(())
    }
}
