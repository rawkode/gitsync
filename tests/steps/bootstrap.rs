use cucumber_rust::{given, then, when};
use gitsync::errors;
use std::{path::PathBuf, time::Duration};

use crate::MyWorld;

#[given(regex = r#"it has no remote called "(\S+)"$"#)]
fn do_nothing_regex(_: &mut MyWorld, _remote_name: String) {}

#[given(regex = r#"I have no directory called "(\S+)"$"#)]
fn i_have_no_directory(world: &mut MyWorld, directory: String) {
    let path: PathBuf = PathBuf::from(&world.test_dir).join(directory);

    assert_eq!(false, path.exists());

    world.clone_dir = path;
}

#[given(regex = r#"I have a directory called "(\S+)"$"#)]
fn i_have_a_directory(world: &mut MyWorld, directory: String) {
    let path: PathBuf = PathBuf::from(&world.test_dir).join(directory);

    let _ = std::fs::create_dir(path.clone());

    assert_eq!(true, path.exists());

    world.clone_dir = path;
}

#[given(regex = r#"it contains a file called "(\S+)""#)]
fn create_file(world: &mut MyWorld, filename: String) {
    let _ = std::fs::File::create(world.clone_dir.join(&filename)).unwrap();
    assert_eq!(true, world.clone_dir.join(&filename).is_file());

    world.created_files.push(filename.clone());
}

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

#[given(regex = r#"it has a remote called "(\S+)" that points to "([^"]+)"#)]
fn it_has_remote(world: &mut MyWorld, name: String, url: String) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("remote")
        .arg("add")
        .arg(&name)
        .arg(&url)
        .status()
        .expect("Failed to add remote to test repository");

    assert_eq!(true, output.success());
}

#[when(regex = r#"I bootstrap the "([^"]+)"#)]
fn bootstrap_git_repository(world: &mut MyWorld, repo: String) {
    let gitsync = gitsync::GitSync {
        repo: repo.clone(),
        dir: world.clone_dir.clone(),
        sync_every: Duration::from_secs(30),
        username: None,
        private_key: None,
        passphrase: None,
    };

    world.repo_url = repo;

    let sync_error = match gitsync.bootstrap() {
        Ok(_) => None,
        Err(e) => Some(e),
    };

    world.sync_error = sync_error;
}

#[then(regex = r#"the repository is cloned"#)]
fn repository_is_cloned(world: &mut MyWorld) {
    let repo_url: &str = world.repo_url.as_str();

    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .expect("Failed to get Url for origin remote");

    assert_eq!(
        format!("{}\n", repo_url).as_bytes(),
        output.stdout.as_slice()
    );
}

#[then("the directory is left untouched")]
fn directory_left_untouched(world: &mut MyWorld) {
    assert_eq!(true, world.clone_dir.is_dir());

    world.created_files.iter().for_each(|f| {
        assert_eq!(true, world.clone_dir.join(f).is_file());
    });
}

#[then("the bootstrap completes")]
fn bootstrap_is_ok(world: &mut MyWorld) {
    assert_eq!(true, world.sync_error.is_none());
}

#[then("the bootstrap errors")]
fn bootstrap_errors(world: &mut MyWorld) {
    assert_eq!(true, world.sync_error.is_some());
}

#[then(regex = r#"the bootstrap errors because "(.*)"$"#)]
fn bootstrap_errors_because(world: &mut MyWorld, error: String) {
    assert_eq!(true, world.sync_error.is_some());

    let w = world.sync_error.as_ref().clone().unwrap();

    match error.as_ref() {
        "local dir isn't git repository" => {
            assert!(matches!(w, errors::GitSyncError::Git2Error { .. }))
        }
        "incorrect remote" => {
            assert!(matches!(w, errors::GitSyncError::IncorrectGitRemotes { .. }))
        }
        _ => assert_eq!(true, false),
    };
}
