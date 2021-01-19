use async_trait::async_trait;
use cucumber::World;
use std::convert::Infallible;
use std::path::PathBuf;
use tempdir::TempDir;

pub struct CucumberState {
    test_dir: PathBuf,
    clone_dir: PathBuf,
    repo_url: String,
}

#[async_trait(?Send)]
impl World for CucumberState {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            test_dir: TempDir::new("gitsync-test-").unwrap().into_path(),
            repo_url: String::from(""),
            clone_dir: PathBuf::new(),
        })
    }
}

mod example_steps {
    use crate::CucumberState;
    use cucumber::gherkin::Step;
    use cucumber::Steps;
    use std::time::Duration;
    use std::{path::PathBuf, rc::Rc};

    pub fn steps() -> Steps<crate::CucumberState> {
        let mut builder: Steps<crate::CucumberState> = Steps::new();

        builder
            .given_regex(
                r#"^I have no directory called "(\S+)"$"#,
                i_have_no_directory,
            )
            .when_regex(r#"I sync the "([^"]+)" repository$"#, sync_git_repository)
            .then_regex(
                r#"the repository is cloned into the "(\S+)" directory$"#,
                repository_is_cloned,
            )
            .then("the bootstrapping completes", do_nothing);

        builder
    }

    fn do_nothing(world: CucumberState, _step: Rc<Step>) -> CucumberState {
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

    fn sync_git_repository(
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

        gitsync
            .clone_repository()
            .expect("Failed to clone repository");

        world.repo_url = repo_url;

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
}

fn main() {
    let runner = cucumber::Cucumber::<CucumberState>::new()
        .features(&["./features"])
        .steps(example_steps::steps());

    futures::executor::block_on(runner.run());
}
