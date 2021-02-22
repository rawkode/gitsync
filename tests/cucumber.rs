use async_trait::async_trait;
use cucumber::World;
use gitsync::errors;
use std::path::PathBuf;
use std::{convert::Infallible, fs::File};
use tempdir::TempDir;

pub struct CucumberState {
    test_dir: PathBuf,
    clone_dir: PathBuf,
    repo_url: String,
    sync_error: Option<errors::GitSyncError>,
    created_files: Vec<String>,
}

#[async_trait(?Send)]
impl World for CucumberState {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        let path = TempDir::new("gitsync-test").unwrap().into_path();

        Ok(Self {
            test_dir: path,
            repo_url: String::from(""),
            clone_dir: PathBuf::new(),
            sync_error: None,
            created_files: vec![],
        })
    }
}

mod bootstrap_steps {
    use crate::CucumberState;
    use cucumber::gherkin::Step;
    use cucumber::Steps;
    use gitsync::errors;
    use std::time::Duration;
    use std::{path::PathBuf, rc::Rc};

    pub fn steps() -> Steps<crate::CucumberState> {
        let mut builder: Steps<crate::CucumberState> = Steps::new();

        builder
            .given_regex(
                r#"^I have no directory called "(\S+)"$"#,
                i_have_no_directory,
            )
            .given_regex(r#"^I have a directory called "(\S+)"$"#, i_have_a_directory)
            .given_regex(
                r#"I have a Git repository in a directory called "(\S+)"$"#,
                i_have_a_git_repository,
            )
            .given_regex(
                r#"it has a remote called "(\S+)" that points to "([^"]+)"$"#,
                it_has_remote,
            )
            .given_regex(r#"it has no remote called "(\S+)"$"#, do_nothing_regex)
            .given_regex(r#"it contains a file called "(\S+)"#, create_file)
            .when_regex(
                r#"I bootstrap the "([^"]+)" repository$"#,
                bootstrap_git_repository,
            )
            .then_regex(
                r#"the repository is cloned into the "(\S+)" directory$"#,
                repository_is_cloned,
            )
            .then("the directory is left untouched", directory_left_untouched)
            .then("the bootstrap completes", bootstrap_is_ok)
            .then("the bootstrap errors", bootstrap_errors)
            .then_regex(
                r#"the bootstrap errors because "(.*)"$"#,
                bootstrap_errors_because,
            );

        builder
    }

    fn do_nothing_regex(
        world: CucumberState,
        _matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        world
    }

    fn bootstrap_is_ok(world: CucumberState, _step: Rc<Step>) -> CucumberState {
        assert_eq!(true, world.sync_error.is_none());

        world
    }

    fn bootstrap_errors(world: CucumberState, _step: Rc<Step>) -> CucumberState {
        assert_eq!(true, world.sync_error.is_some());

        world
    }

    fn bootstrap_errors_because(
        world: CucumberState,
        matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        assert_eq!(true, world.sync_error.is_some());

        let w = world.sync_error.as_ref().clone().unwrap();

        match matches.get(1).unwrap().as_str() {
            "local dir isn't git repository" => {
                assert!(matches!(w, errors::GitSyncError::Git2Error { .. }))
            }
            "incorrect remote" => {
                assert!(matches!(w, errors::GitSyncError::IncorrectGitRemotes { .. }))
            }
            _ => assert_eq!(true, false),
        };

        world
    }

    fn i_have_no_directory(
        mut world: CucumberState,
        matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let path: PathBuf = PathBuf::from(&world.test_dir).join(matches.get(1).unwrap());

        assert_eq!(false, path.exists());

        world.clone_dir = path;

        world
    }

    fn i_have_a_directory(
        mut world: CucumberState,
        matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let path: PathBuf = PathBuf::from(&world.test_dir).join(matches.get(1).unwrap());

        let _ = std::fs::create_dir(path.clone());

        assert_eq!(true, path.exists());

        world.clone_dir = path;

        world
    }

    fn create_file(
        mut world: CucumberState,
        matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let filename = matches.get(1).unwrap();

        let _ = std::fs::File::create(world.clone_dir.join(filename)).unwrap();
        assert_eq!(true, world.clone_dir.join(filename).is_file());

        world.created_files.push(filename.clone());

        world
    }

    fn i_have_a_git_repository(
        mut world: CucumberState,
        matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let clone_dir: PathBuf = world.test_dir.clone().join(matches.get(1).unwrap());

        let output = std::process::Command::new("git")
            .arg("init")
            .arg(&clone_dir)
            .status()
            .expect("Failed to initialise test repository");

        assert_eq!(true, output.success());

        world.clone_dir = clone_dir.clone();

        world
    }

    fn it_has_remote(world: CucumberState, matches: Vec<String>, _step: Rc<Step>) -> CucumberState {
        let name: &String = matches.get(1).unwrap();
        let url: &String = matches.get(2).unwrap();

        let output = std::process::Command::new("git")
            .current_dir(&world.clone_dir)
            .arg("remote")
            .arg("add")
            .arg(&name)
            .arg(&url)
            .status()
            .expect("Failed to add remote to test repository");

        assert_eq!(true, output.success());

        world
    }

    fn bootstrap_git_repository(
        mut world: CucumberState,
        matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let repo_url = matches.get(1).unwrap().clone();

        let gitsync = gitsync::GitSync {
            repo: repo_url.clone(),
            dir: world.clone_dir.clone(),
            sync_every: Duration::from_secs(30),
            username: None,
            private_key: None,
            passphrase: None,
        };

        world.repo_url = repo_url;

        let sync_error = match gitsync.bootstrap() {
            Ok(_) => None,
            Err(e) => Some(e),
        };

        world.sync_error = sync_error;

        world
    }

    fn repository_is_cloned(
        world: CucumberState,
        _matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let repo_url: &str = world.repo_url.as_str();
        let clone_dir: PathBuf = world.clone_dir.clone();

        let output = std::process::Command::new("git")
            .current_dir(PathBuf::from(clone_dir))
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .output()
            .expect("Failed to get Url for origin remote");

        assert_eq!(
            format!("{}\n", repo_url).as_bytes(),
            output.stdout.as_slice()
        );

        world
    }

    fn directory_left_untouched(world: CucumberState, _step: Rc<Step>) -> CucumberState {
        assert_eq!(true, world.clone_dir.is_dir());

        world.created_files.iter().for_each(|f| {
            assert_eq!(true, world.clone_dir.join(f).is_file());
        });

        world
    }
}

fn main() {
    let runner = cucumber::Cucumber::<CucumberState>::new()
        .features(&["./features"])
        .steps(bootstrap_steps::steps());

    futures::executor::block_on(runner.run());
}
