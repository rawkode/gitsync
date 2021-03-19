use cucumber_rust::{given, then, when};
use std::time::Duration;

use crate::MyWorld;

#[given("there are remote changes")]
fn remote_changes(world: &mut MyWorld) {
    std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .args(vec!["reset", "--hard", "HEAD^1"])
        .output()
        .expect("Failed to revert a commit");

    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    world.current_commit_hash = output.stdout;
}

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

    let sync_error = match gitsync.sync() {
        Ok(_) => None,
        Err(e) => Some(e),
    };

    world.sync_error = sync_error;
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

#[then("there are changes")]
fn there_are_changes(world: &mut MyWorld) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    assert_ne!(world.current_commit_hash, output.stdout);
}

#[then("the sync completes")]
fn the_sync_completes(world: &mut MyWorld) {
    println!("Bare Repository {:?}", world.bare_dir);
    println!("Clone Repository {:?}", world.clone_dir);
    println!("{:?}", world.sync_error);
    assert!(world.sync_error.is_none());
}

#[then("the sync errors")]
fn the_sync_errors(world: &mut MyWorld) {
    println!("Clone is {:?}", world.bare_dir);
    assert_eq!(true, world.sync_error.is_some());
}

#[given("there are local changes")]
fn there_is_local_changes(world: &mut MyWorld) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    world.current_commit_hash = output.stdout;

    // Modify the file, we don't need to commit
    std::fs::write(world.clone_dir.join("file"), "123")
        .expect("Failed to write file to repository");
}
