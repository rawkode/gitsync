use async_trait::async_trait;
use cucumber::World;
use gitsync::errors;
use std::convert::Infallible;
use std::path::PathBuf;
use tempdir::TempDir;

mod steps;
use steps::bootstrap;

pub struct CucumberState {
    test_dir: PathBuf,
    clone_dir: PathBuf,
    repo_url: String,
    sync_error: Option<errors::GitSyncError>,
    created_files: Vec<String>,
}

#[async_trait(?Send)]
impl World for CucumberState {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        let path = TempDir::new("gitsync-test").unwrap().into_path();

        Ok(Self {
            test_dir: path,
            repo_url: String::from(""),
            clone_dir: PathBuf::new(),
            sync_error: None,
            created_files: vec![],
        })
    }
}

fn main() {
    let runner = cucumber::Cucumber::<CucumberState>::new()
        .features(&["./features"])
        .steps(bootstrap::steps());

    futures::executor::block_on(runner.run());
}
