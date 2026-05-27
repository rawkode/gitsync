use cucumber::World as _;
use gitsync::errors;
use std::path::PathBuf;

mod steps;

#[derive(cucumber::World, Debug, Default)]
pub struct World {
    test_dir: PathBuf,
    bare_dir: PathBuf,
    source_dir: PathBuf,
    clone_dir: PathBuf,
    repo_url: String,
    branch: Option<String>,
    latest_commit_hash: Vec<u8>,
    current_commit_hash: Vec<u8>,
    sync_error: Option<errors::GitSyncError>,
    sync_outcome: Option<gitsync::SyncOutcome>,
    created_files: Vec<String>,
}

#[tokio::main]
async fn main() {
    World::run("./features").await;
}
