use async_trait::async_trait;
use cucumber::World;
use gitsync;
use std::convert::Infallible;
use std::path::PathBuf;
use tempdir::TempDir;

pub struct CucumberState {
    sync: Option<gitsync::GitSync>,
    dir: PathBuf,
    repo_url: String,
    clone_dir: String,
}

#[async_trait(?Send)]
impl World for CucumberState {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            dir: TempDir::new("test").unwrap().into_path(),
            sync: None,
            repo_url: String::from(""),
            clone_dir: String::from(""),
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
        let path: &String = matches.first().unwrap();

        assert_eq!(false, PathBuf::from(&world.dir).join(&path).exists());

        world.clone_dir = path.clone();

        world
    }

    fn sync_git_repository(
        mut world: CucumberState,
        matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let repo_url: &String = matches.first().unwrap();

        let gitsync = gitsync::GitSync {
            repo: repo_url.clone(),
            dir: world.clone_dir.clone(),
            sync_every: Duration::from_secs(30),
            username: None,
            private_key: None,
            passphrase: None,
        };

        gitsync.clone_repository();

        world.repo_url = repo_url.clone();

        world
    }

    fn repository_is_cloned(
        world: CucumberState,
        _matches: Vec<String>,
        _step: Rc<Step>,
    ) -> CucumberState {
        let repo_url: &String = &world.repo_url;
        let clone_dir: &String = &world.clone_dir;

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

    // fn configure_self(world: CucumberState, _step: Rc<Step>) -> CucumberState {
    //     world.sync = Some(gitsync::GitSync {
    //         repo: String::from("https://gitlab.com/rawkode/gitsync"),
    //         dir: String::from(world.dir.to_str().unwrap()),
    //         sync_every: Duration::from_secs(5),
    //         username: None,
    //         private_key: None,
    //         passphrase: None,
    //     });

    //     world
    // }

    // fn sync_repository(world: CucumberState, step: Rc<Step>) -> CucumberState {
    //     let sync = world.sync;
    //     let result = sync.unwrap().clone_repository();

    //     world
    // }
}

fn main() {
    let runner = cucumber::Cucumber::<CucumberState>::new()
        .features(&["./features"])
        .steps(example_steps::steps());

    futures::executor::block_on(runner.run());
}
