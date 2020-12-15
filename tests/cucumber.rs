use async_trait::async_trait;
use cucumber::World;
use gitsync;
use std::convert::Infallible;
use std::path::PathBuf;
use tempdir::TempDir;

pub struct CucumberState {
    sync: Option<gitsync::GitSync>,
    dir: PathBuf,
}

#[async_trait(?Send)]
impl World for CucumberState {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            dir: TempDir::new("test").unwrap().into_path(),
            sync: None,
        })
    }
}

mod example_steps {
    use crate::CucumberState;
    use cucumber::gherkin::Step;
    use cucumber::Steps;
    use std::rc::Rc;
    use std::time::Duration;

    pub fn steps() -> Steps<crate::CucumberState> {
        let mut builder: Steps<crate::CucumberState> = Steps::new();

        // builder
        //     .given("we have GitSync configured to clone itself", configure_self)
        //     .given("the local directory does not exist", do_nothing)
        //     .when("I sync a Git repository", sync_repository)
        //     .then("the repository is cloned", |world, step| world);

        builder
    }

    // fn do_nothing(world: CucumberState, _step: Rc<Step>) -> CucumberState {
    //     world
    // }

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
