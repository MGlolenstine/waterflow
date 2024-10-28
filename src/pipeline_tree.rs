use std::collections::BTreeMap;

use uuid::Uuid;

use crate::pipeline::Pipeline;

type Deps = Vec<PipelineTree>;
type AllJobs = BTreeMap<Uuid, (String, Vec<Uuid>)>;
type DepCache = BTreeMap<Uuid, PipelineTree>;

#[derive(Clone, Debug)]
pub struct PipelineTree {
    pub name: String,
    pub dependency_of: Deps,
}

impl PipelineTree {
    pub fn new(pipeline: &Pipeline) -> Self {
        Self::generate_with_dependants(pipeline)
    }

    fn get_all_jobs_with_deps(pipeline: &Pipeline) -> AllJobs {
        pipeline
            .jobs
            .iter()
            .map(|job| (job.get_id(), (job.name.clone(), job.dependencies.clone())))
            .collect()
    }

    fn get_deps_for_uuid(job_id: &Uuid, all_jobs: &AllJobs, dep_cache: &mut DepCache) -> Deps {
        all_jobs
            .iter()
            .filter(|(_, (_, deps))| deps.contains(job_id))
            .map(|(id, (name, _))| {
                if let Some(cached) = dep_cache.get(id) {
                    cached.clone()
                } else {
                    let new_leaf = PipelineTree {
                        name: name.to_string(),
                        dependency_of: PipelineTree::get_deps_for_uuid(id, all_jobs, dep_cache),
                    };

                    dep_cache.insert(*id, new_leaf.clone());
                    new_leaf
                }
            })
            .collect::<Vec<_>>()
    }

    fn generate_with_dependants(pipeline: &Pipeline) -> Self {
        let all_jobs = Self::get_all_jobs_with_deps(pipeline);

        let root_jobs = all_jobs
            .iter()
            .filter(|(_, (_, deps))| deps.is_empty())
            .collect::<Vec<_>>();

        let mut dep_cache: DepCache = BTreeMap::new();

        let root_jobs = root_jobs
            .into_iter()
            .map(|(job_id, (name, _))| PipelineTree {
                name: name.to_string(),
                dependency_of: PipelineTree::get_deps_for_uuid(job_id, &all_jobs, &mut dep_cache),
            })
            .collect::<Vec<_>>();

        PipelineTree {
            name: "ROOT".to_string(),
            dependency_of: root_jobs,
        }
    }
}
