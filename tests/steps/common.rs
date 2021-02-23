use cucumber_rust::given;
use std::path::PathBuf;

use crate::MyWorld;

#[given(regex = r#"I have a Git repository in a directory called "(\S+)"#)]
fn i_have_a_git_repository(world: &mut MyWorld, directory: String) {
    let clone_dir: PathBuf = world.test_dir.clone().join(directory);

    let output = std::process::Command::new("git")
        .arg("init")
        .arg(&clone_dir)
        .status()
        .expect("Failed to initialise test repository");

    assert_eq!(true, output.success());

    world.clone_dir = clone_dir.clone();
}
