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

    world.latest_commit_hash = output.stdout;

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
fn no_remote_changes(world: &mut World) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    world.current_commit_hash = output.stdout;
}

#[when("I sync")]
fn sync(world: &mut World) {
    let gitsync = gitsync::GitSync {
        repo: world.repo_url.clone(),
        dir: world.clone_dir.clone(),
        branch: world.branch.clone(),
        ..Default::default()
    };

    match gitsync.sync() {
        Ok(outcome) => {
            world.sync_outcome = Some(outcome);
            world.sync_error = None;
        }
        Err(error) => {
            world.sync_outcome = None;
            world.sync_error = Some(error);
        }
    }
}

#[when(regex = r#"I sync branch "(\S+)""#)]
fn sync_branch(world: &mut World, branch: String) {
    world.branch = Some(branch);
    sync(world);
}

#[then("there is no change")]
fn there_is_no_change(world: &mut World) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");

    assert_eq!(world.current_commit_hash, output.stdout);
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
    assert_eq!(world.latest_commit_hash, output.stdout);
    world.current_commit_hash = output.stdout;
}

#[then("the sync reports changes")]
fn sync_reports_changes(world: &mut World) {
    let outcome = world.sync_outcome.as_ref().expect("sync outcome");
    assert!(outcome.changed);
    assert_eq!(
        Some(trim_hash(&world.current_commit_hash)),
        outcome.previous.map(|oid| oid.to_string())
    );
    assert_eq!(
        trim_hash(&world.latest_commit_hash),
        outcome.current.to_string()
    );
}

#[then("the sync reports no changes")]
fn sync_reports_no_changes(world: &mut World) {
    let outcome = world.sync_outcome.as_ref().expect("sync outcome");
    assert!(!outcome.changed);
    assert_eq!(Some(outcome.current), outcome.previous);
    assert_eq!(
        trim_hash(&world.current_commit_hash),
        outcome.current.to_string()
    );
}

#[then("head_oid matches HEAD")]
fn head_oid_matches_head(world: &mut World) {
    let gitsync = gitsync::GitSync {
        repo: world.repo_url.clone(),
        dir: world.clone_dir.clone(),
        branch: world.branch.clone(),
        ..Default::default()
    };

    let head_oid = gitsync
        .head_oid()
        .expect("head oid can be read")
        .expect("head exists");
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");
    assert!(output.status.success());

    assert_eq!(trim_hash(&output.stdout), head_oid.to_string());
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

    world.current_commit_hash = output.stdout;

    // Modify the file, we don't need to commit
    std::fs::write(world.clone_dir.join("file"), "123")
        .expect("Failed to write file to repository");
}

#[given(regex = r#"there are remote changes on branch "(\S+)""#)]
fn remote_changes_on_branch(world: &mut World, branch: String) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get current commit hash");
    assert!(output.status.success());
    world.current_commit_hash = output.stdout;

    let output = std::process::Command::new("git")
        .current_dir(&world.source_dir)
        .args(vec!["checkout", &branch])
        .output()
        .expect("Failed to checkout branch");
    assert!(output.status.success());

    std::fs::write(world.source_dir.join("branch-file"), "12")
        .expect("Failed to write branch file");

    let output = std::process::Command::new("git")
        .current_dir(&world.source_dir)
        .args(vec!["add", "branch-file"])
        .output()
        .expect("Failed to add branch file");
    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&world.source_dir)
        .args(vec!["commit", "-m", "branch update"])
        .output()
        .expect("Failed to commit branch update");
    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&world.source_dir)
        .args(vec!["push", "origin", &branch])
        .output()
        .expect("Failed to push branch update");
    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&world.source_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("Failed to get latest commit hash");
    assert!(output.status.success());
    world.latest_commit_hash = output.stdout;
}

fn trim_hash(hash: &[u8]) -> String {
    String::from_utf8_lossy(hash).trim().to_owned()
}
