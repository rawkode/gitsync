use cucumber::given;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::World;

#[given("I have a remote Git repository available")]
fn i_have_a_remote_git_repository(world: &mut World) {
    let path = TempDir::new().unwrap().into_path().canonicalize().unwrap();

    world.test_dir = path;
    world.bare_dir = PathBuf::from(&world.test_dir).join("bare");
    world.clone_dir = PathBuf::from(&world.test_dir).join("clone");

    // --initial-branch=main doesn't work on older Gits
    let output = std::process::Command::new("git")
        .args(vec!["init", "--bare"])
        .arg(&world.bare_dir)
        .status()
        .expect("Failed to initialise bare repository");

    assert!(output.success());

    // We want this Git repository to have at-least one commit
    let path = world.bare_dir.parent().unwrap().join("initial-commits");

    let output = std::process::Command::new("git")
        .args(vec![
            "clone",
            world.bare_dir.to_str().unwrap(),
            path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to clone bare repository");

    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&path)
        .args(vec!["config", "user.name", "Example Author"])
        .output()
        .expect("Failed to add first file");

    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&path)
        .args(vec!["config", "user.email", "example@example.com"])
        .output()
        .expect("Failed to add first file");

    assert!(output.status.success());

    std::fs::write(path.join("file"), "1").expect("Failed to write file to repository");

    let output = std::process::Command::new("git")
        .current_dir(&path)
        .args(vec!["add", "file"])
        .output()
        .expect("Failed to add first file");

    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&path)
        .args(vec!["commit", "-m", "1"])
        .output()
        .expect("Failed to commit first file");

    assert!(output.status.success());

    std::fs::write(path.join("file"), "12").expect("Failed to write file to repository");

    let output = std::process::Command::new("git")
        .current_dir(&path)
        .args(vec!["add", "file"])
        .output()
        .expect("Failed to add second file");
    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&path)
        .args(vec!["commit", "-m", "2"])
        .output()
        .expect("Failed to commit second file");
    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .current_dir(&path)
        .args(vec!["push", "-u", "origin", "master"])
        .output()
        .expect("Failed to push changes");
    assert!(output.status.success());

    world.repo_url = String::from(world.bare_dir.to_str().unwrap());
}

#[given(regex = r#"I have a Git repository in a directory called "(\S+)""#)]
fn i_have_a_git_repository(world: &mut World, directory: String) {
    let clone_dir: PathBuf = world.test_dir.clone().join(directory);

    let output = std::process::Command::new("git")
        .arg("clone")
        .arg(&world.bare_dir)
        .arg(&clone_dir)
        .status()
        .expect("Failed to initialise test repository");

    assert!(output.success());

    world.clone_dir = clone_dir;
}
