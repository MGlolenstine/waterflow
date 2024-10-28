use std::time::{Duration, Instant};

use crate::{job_type::JobType, Result};
use tracing::trace;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum JobStatus {
    #[default]
    Waiting,
    InProgress {
        started_at: Instant,
    },
    Failed {
        msg: String,
        duration: Duration,
    },
    Succeeded {
        msg: String,
        duration: Duration,
    },
}

impl JobStatus {
    pub fn is_waiting(&self) -> bool {
        matches!(self, Self::Waiting)
    }
    pub fn is_running(&self) -> bool {
        matches!(self, Self::InProgress { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    pub fn is_succeeded(&self) -> bool {
        matches!(self, Self::Succeeded { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Job {
    /// Job's UUID, which can be used for dependencies
    pub job_id: Uuid,

    /// Job's dependencies
    /// When all of these are satisfied, we can run.
    /// If this is empty, run it in parallel.
    pub dependencies: Vec<Uuid>,

    /// Display name of the Job
    pub name: String,

    /// The action that the Job will execute
    pub job_type: JobType,

    /// Job's status
    pub status: JobStatus,

    /// Job's IO
    pub fixed_input: Vec<String>,
    pub input: Vec<String>,
    pub output: String,
}

// Builder pattern
impl Job {
    pub fn new(name: &str, job_type: JobType) -> Self {
        Job {
            job_id: Uuid::new_v4(),
            name: name.to_string(),
            job_type,
            ..Default::default()
        }
    }

    // TODO: Fix the input fetching from output of another job
    pub fn with_input(mut self, input: Vec<String>) -> Self {
        self.input = input;
        self
    }
}

// Dependency management
impl Job {
    pub fn set_dependencies(&mut self, dependencies: Vec<Uuid>) {
        if dependencies.contains(&self.job_id) {
            panic!("You cannot assign yourself as your own dependency!");
        }
        self.dependencies = dependencies;
    }

    pub fn add_dependency(&mut self, dependency: Uuid) {
        if dependency == self.job_id {
            panic!("You cannot assign yourself as your own dependency!");
        }
        self.dependencies.push(dependency);
    }
}

// Execution
impl Job {
    pub(crate) fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
    }

    pub(crate) fn set_status(&mut self, status: &JobStatus) {
        self.status = status.clone();
    }

    pub(crate) fn set_input(&mut self, input: Vec<String>) {
        self.input = input;
    }

    pub(crate) fn can_execute(&self, jobs: &[Job]) -> bool {
        let completed_uuids = jobs
            .iter()
            .filter(|job| job.status.is_succeeded())
            .map(|j| j.job_id)
            .collect::<Vec<_>>();

        self.dependencies
            .iter()
            .all(|dep| completed_uuids.contains(dep))
    }

    pub(crate) async fn execute(&mut self) -> Result<JobStatus> {
        let (tx, rx) = flume::bounded(1);

        let id = self.get_id();
        let job_type = self.job_type.clone();
        let started_at = Instant::now();
        self.set_status(&JobStatus::InProgress { started_at });

        // Add fixed_input before other elements
        let fixed_input = self.fixed_input.clone();
        let input = self.input.clone();

        std::thread::spawn(move || {
            //TODO: execute something
            let res = job_type.execute(&fixed_input, &input);

            match res {
                Ok(output) => {
                    tx.send((
                        JobStatus::Succeeded {
                            msg: output.clone(),
                            duration: Instant::now().duration_since(started_at),
                        },
                        output,
                    ))
                    .unwrap();
                    trace!("Job {:?} finished the execution", id);
                }
                Err(e) => {
                    tx.send((
                        JobStatus::Failed {
                            msg: e.to_string(),
                            duration: Instant::now().duration_since(started_at),
                        },
                        e.to_string(),
                    ))
                    .unwrap();
                    trace!("Job {:?} finished the execution", id);
                }
            }
        });

        let (status, output) = rx.recv_async().await?;

        trace!("Received \"job finished\" response from the thread");

        self.set_status(&status);
        self.set_output(&output);

        Ok(status)
    }
}

// Getters
impl Job {
    pub fn get_id(&self) -> Uuid {
        self.job_id
    }

    pub fn get_status(&self) -> JobStatus {
        self.status.clone()
    }
}

#[test]
pub fn test_job_execution() {
    let mut job1 = Job::new("Test job", JobType::Noop);

    let job_res = smol::block_on(job1.execute());

    assert!(job_res.is_ok(), "Job res is Error!");

    let job_res = job_res.unwrap();

    assert!(job_res.is_succeeded());
}
