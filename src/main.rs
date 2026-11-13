use jiff::Zoned;

use crate::job::{Job, JobId, JobType, Scheduler, SchedulerError};

pub mod job;

fn main() -> Result<(), SchedulerError> {
    let mut scheduler = Scheduler::new();

    // Submit several jobs. AlwaysFail is queued first so the drain shows a
    // failure followed by two successes, in FIFO order.
    let make = |job_type, param: &str| {
        Job::new(
            job_type,
            "demo".to_string(),
            "description".to_string(),
            param.to_string(),
            Zoned::now(),
        )
    };
    let specs = [
        (JobType::AlwaysFail, "boom"),
        (JobType::PrintMessage, "hello"),
        (JobType::PrintMessage, "world"),
    ];

    let mut ids = Vec::new();
    for (job_type, param) in specs {
        let id = scheduler.add(make(job_type, param))?;
        scheduler.schedule(id)?;
        println!("Submitted {id}");
        ids.push(id);
    }

    println!("\n-- draining queue --");
    while let Some(id) = scheduler.run_next()? {
        println!("Ran {id} -> {:?}", scheduler.get(&id).map(Job::status));
    }

    println!("\n-- invalid transition --");
    match scheduler.schedule(ids[0]) {
        Ok(()) => println!("unexpected: re-scheduled a finished job"),
        Err(e) => println!("rejected as expected: {e}"),
    }

    println!("\n-- unknown id --");
    println!("get(random) = {:?}", scheduler.get(&JobId::new()));

    Ok(())
}
