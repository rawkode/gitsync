use cucumber::{given, then, when};

use crate::World;

#[given("there are remote changes")]
fn remote_changes(world: &mut World) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    world.latest_commit_hash = String::from_utf8(output.stdout).unwrap();

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

    world.current_commit_hash = String::from_utf8(output.stdout).unwrap();
}

#[given("there are no remote changes")]
fn no_remote_changes(world: &mut World) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    world.current_commit_hash = String::from_utf8(output.stdout).unwrap();
}

#[when("I sync")]
fn sync(world: &mut World) {
    let gitsync = gitsync::GitSync {
        repo: world.repo_url.clone(),
        dir: world.clone_dir.clone(),
        ..Default::default()
    };

    let sync_error = match gitsync.sync() {
        Ok(_) => None,
        Err(e) => Some(e),
    };

    world.sync_error = sync_error;
}

#[then("there is no change")]
fn there_is_no_change(world: &mut World) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    assert_eq!(world.current_commit_hash, String::from_utf8(output.stdout).unwrap());
}

#[then("there are changes")]
fn there_are_changes(world: &mut World) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    assert_ne!(world.latest_commit_hash, world.current_commit_hash);
    assert_eq!(world.latest_commit_hash, String::from_utf8(output.stdout).unwrap());
}

#[then("the sync completes")]
fn the_sync_completes(world: &mut World) {
    println!("Bare Repository {:?}", world.bare_dir);
    println!("Clone Repository {:?}", world.clone_dir);
    println!("{:?}", world.sync_error);
    assert!(world.sync_error.is_none());
}

#[then("the sync errors")]
fn the_sync_errors(world: &mut World) {
    println!("Clone is {:?}", world.bare_dir);
    assert!(world.sync_error.is_some());
}

#[given("there are local changes")]
fn there_is_local_changes(world: &mut World) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    world.current_commit_hash = String::from_utf8(output.stdout).unwrap();

    // Modify the file, we don't need to commit
    std::fs::write(world.clone_dir.join("file"), "123")
        .expect("Failed to write file to repository");
}
