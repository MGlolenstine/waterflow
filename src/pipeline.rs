use crate::job::{Job, JobStatus};
use crate::Result;
use tracing::trace;
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct Pipeline {
    pub(crate) jobs: Vec<Job>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline::default()
    }

    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub fn add_jobs(&mut self, mut jobs: Vec<Job>) {
        self.jobs.append(&mut jobs)
    }

    pub fn get_job_statuses(&self) -> Vec<(Uuid, JobStatus)> {
        self.jobs
            .iter()
            .map(|j| (j.get_id(), j.get_status()))
            .collect::<Vec<_>>()
    }

    /// Gets the tasks that can currently be run and haven't been run yet
    fn get_runnable_jobs(jobs: &[Job]) -> Vec<Uuid> {
        jobs.iter()
            // Only check the waiting jobs
            .filter(|j| j.get_status() == JobStatus::Waiting)
            // Only keep the jobs that can be executed
            .filter(|j| j.can_execute(jobs))
            .map(|j| j.get_id())
            .collect::<Vec<_>>()
    }

    fn all_jobs_completed(jobs: &[Job]) -> bool {
        jobs.iter().all(|j| !j.get_status().is_running())
    }

    fn get_mut_job(&mut self, job_id: Uuid) -> &mut Job {
        self.jobs
            .iter_mut()
            .find(|j| j.get_id() == job_id)
            .expect("Tried to get a job from a job_id, which was gotten from the jobs")
    }

    fn get_job(&self, job_id: Uuid) -> &Job {
        self.jobs
            .iter()
            .find(|j| j.get_id() == job_id)
            .expect("Tried to get a job from a job_id, which was gotten from the jobs")
    }

    fn get_dep_inputs(&self, job_id: Uuid) -> Vec<String> {
        let job = self.get_job(job_id);
        job.dependencies
            .iter()
            .map(|dep| {
                let job_status = self.get_job(*dep).get_status();
                let JobStatus::Succeeded { msg, duration: _ } = job_status else {
                    panic!("Tried to read output from a parent dependency that hasn't succeeded?");
                };
                msg
            })
            .collect::<Vec<String>>()
    }

    pub async fn execute(&mut self) -> Result<()> {
        loop {
            let runnable_jobs = Pipeline::get_runnable_jobs(&self.jobs);

            // If we don't have any more jobs to run and all of the jobs that we have been waiting for have completed, stop executing
            if runnable_jobs.is_empty() && Pipeline::all_jobs_completed(&self.jobs) {
                trace!("We ran out of jobs to run");
                break;
            }

            trace!("Running the following jobs: {:?}", runnable_jobs);

            // TODO: Make this be paralelizable
            // TODO: Make this run an even when a job completes
            for job_id in runnable_jobs {
                let inputs = self.get_dep_inputs(job_id);
                let job = self.get_mut_job(job_id);
                trace!("Executing: {:?}", job.name);

                job.set_input(inputs);

                job.execute().await?;

                trace!("Finished: {:?}", job.name);
            }
        }
        Ok(())
    }
}

#[test]
pub fn test_pipeline_execution() {
    use crate::job_type::JobType;
    let job1 = Job::new("Test job Hello", JobType::new_bash("echo -n 'Hello'"));
    let job2 = Job::new("Test job World", JobType::new_bash("echo -n 'World!'"));
    let mut job3 = Job::new(
        "Test job concatenate",
        JobType::new_bash("echo -n '{INPUT}'"),
    );

    job3.add_dependency(job1.get_id());
    job3.add_dependency(job2.get_id());

    let job1_id = job1.get_id();
    let job2_id = job2.get_id();
    let job3_id = job3.get_id();

    let mut pipeline = Pipeline::new();
    pipeline.add_jobs(vec![job1, job2, job3]);

    let pipeline_res = smol::block_on(pipeline.execute());

    assert!(pipeline_res.is_ok());

    let job1 = pipeline.get_job(job1_id);

    assert!(job1.input.is_empty());
    assert_eq!(job1.output, "Hello");

    let job2 = pipeline.get_job(job2_id);
    assert!(job2.input.is_empty());
    assert_eq!(job2.output, "World!");

    let job3 = pipeline.get_job(job3_id);
    assert_eq!(job3.input, ["Hello", "World!"]);
    assert_eq!(job3.output, "Hello World!");
}
