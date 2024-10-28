use waterflow::job::Job;
use waterflow::job_type::JobType;
use waterflow::pipeline::Pipeline;
use waterflow::pipeline_tree::PipelineTree;

#[test]
pub fn test_print_pipeline_dep_tree() {
    let mut pipeline = Pipeline::new();

    let file_name = "tests/wasm_example/pkg/wasm_example_bg.wasm";

    let job1 = Job::new(
        "First hello world",
        JobType::new_bash("echo -n 'Hello World! #1'"),
    );
    let job2 = Job::new(
        "Second hello world",
        JobType::new_bash("echo -n 'Hello World! #2'"),
    );

    let mut job3 = Job::new("Reverse join", JobType::new_wasm("reverse_join", file_name));
    job3.add_dependency(job1.get_id());
    job3.add_dependency(job2.get_id());

    let mut job4 = Job::new("Normal join", JobType::new_wasm("normal_join", file_name));
    job4.add_dependency(job1.get_id());
    job4.add_dependency(job2.get_id());

    pipeline.add_jobs(vec![job1, job2, job3, job4]);

    dbg!(PipelineTree::new(&pipeline));
}

#[test]
#[ignore = "For this test to pass, you need to have the `jq` available on the system and a working network connection"]
pub fn test_io() {
    tracing_subscriber::fmt::init();
    let mut pipeline = Pipeline::new();

    let job1 = Job::new(
        "Fetch data",
        JobType::new_web_request(
            "https://jsonplaceholder.typicode.com/todos/1",
            waterflow::job_type::WebRequestType::Get,
        ),
    );
    let mut job2 = Job::new(
        "Extract data",
        JobType::new_bash("echo '{INPUT}' | jq .title"),
    );

    let mut job3 = Job::new("Print data", JobType::new_bash("echo -n \"{INPUT}\""));
    job2.add_dependency(job1.get_id());
    job3.add_dependency(job2.get_id());

    pipeline.add_jobs(vec![job1, job2, job3]);

    smol::block_on(async { pipeline.execute().await })
        .expect("Something went wrong while trying to run the pipeline!");

    dbg!(&pipeline);
    // dbg!(PipelineTree::new(&pipeline));
}
