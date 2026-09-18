use jiff::Zoned;

use crate::job::{Job, Scheduler, SchedulerError};

pub mod job;

fn main() -> Result<(), SchedulerError> {
    let job: Job = Job::new(
        "test".to_string(),
        "description".to_string(),
        "param".to_string(),
        Zoned::now(),
    );

    println!("{job:#?}");

    let mut scheduler = Scheduler::new();
    let job_id = scheduler.add(job)?;
    println!("Added job {job_id}");
    scheduler.schedule(job_id)?;
    if let Some(runnable) = scheduler.dispatch()? {
        println!("Assigned job {runnable}");
    }
    println!("{scheduler:#?}");
    Ok(())
}
