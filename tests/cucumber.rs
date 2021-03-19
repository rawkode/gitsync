use cucumber_rust::{async_trait, World, WorldInit};
use gitsync::errors;
use std::convert::Infallible;
use std::path::PathBuf;
use tempdir::TempDir;

mod steps;

#[derive(WorldInit)]
pub struct MyWorld {
    test_dir: PathBuf,
    bare_dir: PathBuf,
    clone_dir: PathBuf,
    repo_url: String,
    latest_commit_hash: Vec<u8>,
    current_commit_hash: Vec<u8>,
    sync_error: Option<errors::GitSyncError>,
    created_files: Vec<String>,
}

#[async_trait(?Send)]
impl World for MyWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        let path = TempDir::new("gitsync-test").unwrap().into_path();

        Ok(Self {
            test_dir: path.clone(),
            repo_url: String::from(""),
            bare_dir: path.clone().join("bare"),
            clone_dir: path.clone().join("clone"),
            latest_commit_hash: vec![],
            current_commit_hash: vec![],
            sync_error: None,
            created_files: vec![],
        })
    }
}

#[tokio::main]
async fn main() {
    MyWorld::init(&["./features"])
        .enable_capture(true)
        .cli()
        .run_and_exit()
        .await;
}
