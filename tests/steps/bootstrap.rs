use cucumber_rust::{given, then, when};
use gitsync::errors;
use std::path::PathBuf;

use crate::MyWorld;

#[given(regex = r#"it has no remote called "(\S+)"$"#)]
fn ensure_no_git_remote_called(world: &mut MyWorld, remote_name: String) {
    std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("remote")
        .arg("remove")
        .arg(&remote_name)
        .status()
        .expect(format!("Failed to ensure no remote was called {}", remote_name).as_str());
}

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

#[given(regex = r#"it has a remote called "(\S+)" that points to "([^"]+)"#)]
fn it_has_remote(world: &mut MyWorld, name: String, url: String) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("remote")
        .arg("set-url")
        .arg("--add")
        .arg(&name)
        .arg(&url)
        .status()
        .expect("Failed to add remote to test repository");

    assert_eq!(true, output.success());
}

#[given(regex = r#"it has a correctly configured remote called "(\S+)""#)]
fn it_has_remote_correctly_configured(world: &mut MyWorld, name: String) {
    let output = std::process::Command::new("git")
        .current_dir(&world.clone_dir)
        .arg("remote")
        .arg("set-url")
        .arg("--add")
        .arg(&name)
        .arg(world.bare_dir.to_str().unwrap())
        .status()
        .expect("Failed to add remote to test repository");

    assert_eq!(true, output.success());
}

#[when("I bootstrap")]
fn bootstrap_git_repository(world: &mut MyWorld) {
    // When we "bootstrap" with no args, this uses the background
    // context that sets up a local bare repository
    world.repo_url = String::from(world.bare_dir.to_str().unwrap());

    let gitsync = gitsync::GitSync {
        repo: String::from(world.bare_dir.clone().to_str().unwrap()),
        dir: world.clone_dir.clone(),
        ..Default::default()
    };

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
            assert!(matches!(
                w,
                errors::GitSyncError::IncorrectGitRemotes { .. }
            ))
        }
        _ => assert_eq!(true, false),
    };
}
