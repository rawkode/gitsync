use cucumber_rust::{given, then, when};
use std::time::Duration;

use crate::MyWorld;

#[given("there are no remote changes")]
fn no_remote_changes(world: &mut MyWorld) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    world.current_commit_hash = output.stdout;
}

#[when("I sync")]
fn sync(world: &mut MyWorld) {
    let gitsync = gitsync::GitSync {
        repo: world.repo_url.clone(),
        dir: world.clone_dir.clone(),
        sync_every: Duration::from_secs(30),
        username: None,
        private_key: None,
        passphrase: None,
    };

    gitsync.sync().expect("Failed to GitSync");
}

#[then("there is no change")]
fn there_is_no_change(world: &mut MyWorld) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    assert_eq!(world.current_commit_hash, output.stdout);
}
