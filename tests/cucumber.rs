use cucumber_rust::{async_trait, World, WorldInit};
use gitsync::errors;
use std::convert::Infallible;
use std::path::PathBuf;
use tempdir::TempDir;

mod steps;

#[derive(WorldInit)]
pub struct MyWorld {
    test_dir: PathBuf,
    clone_dir: PathBuf,
    repo_url: String,
    sync_error: Option<errors::GitSyncError>,
    created_files: Vec<String>,
}

#[async_trait(?Send)]
impl World for MyWorld {
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

#[tokio::main]
async fn main() {
    let runner = MyWorld::init(&["./features"]);
    runner.run_and_exit().await;
}
