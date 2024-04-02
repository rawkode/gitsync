use errors::GitSyncError;
use gix::clone::PrepareCheckout;
use std::path::{Path, PathBuf};

pub mod errors;

// When running tests, we can just use println instead of logger
#[cfg(not(test))]
use log::info;

#[cfg(test)]
use std::println as info;

#[derive(Clone, Debug, Default)]
pub struct GitSync {
    pub repo: String,
    pub dir: PathBuf,

    pub branch: Option<String>,
    pub username: Option<String>,
    pub passphrase: Option<String>,
    pub private_key: Option<String>,
}

impl GitSync {
    pub fn bootstrap(&self) -> Result<(), errors::GitSyncError> {
        if self.does_clone_exist()? {
            return Ok(());
        }

        self.clone_repository()?;

        Ok(())
    }

    fn check_worktree_is_clean(&self) -> bool {
        true
    }

    pub fn sync(&self) -> Result<(), errors::GitSyncError> {
        if !self.check_worktree_is_clean() {
            return Err(GitSyncError::WorkTreeNotClean);
        }

        self.clone_repository()
    }

    fn clone_repository(&self) -> Result<(), errors::GitSyncError> {
        info!("Attempting to clone {} to {:?}", self.repo, self.dir,);

        unsafe {
            match gix::interrupt::init_handler(1, || {}) {
                Ok(_) => (),
                Err(error) => {
                    return Err(GitSyncError::GenericError { error });
                }
            };
        }

        if !self.dir.exists() {
            match std::fs::create_dir_all(&self.dir) {
                Ok(_) => {}
                Err(error) => {
                    return Err(GitSyncError::GenericError { error });
                }
            }
        }

        let url = match gix::url::parse(self.repo.as_str().into()) {
            Ok(url) => url,
            Err(error) => {
                return Err(GitSyncError::GixParseError { error });
            }
        };

        let mut prepare_clone = match gix::prepare_clone(url, &self.dir) {
            Ok(prepared_fetch) => prepared_fetch,
            Err(error) => {
                return Err(GitSyncError::GixCloneError { error });
            }
        };

        let (mut prepare_checkout, _) = match prepare_clone
            .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        {
            Ok(checkout) => checkout,
            Err(error) => {
                return Err(GitSyncError::GixCloneFetchError { error });
            }
        };

        match prepare_checkout
            .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        {
            Ok(repository) => repository,
            Err(error) => {
                return Err(GitSyncError::GixCloneCheckoutError { error });
            }
        };

        Ok(())
    }

    fn does_clone_exist(&self) -> Result<bool, errors::GitSyncError> {
        // No local directory exists, so we can happily clone when required.
        if !Path::new(&self.dir).exists() {
            return Ok(false);
        };

        // OK. If a directory exists, we need to check if it's a Git repository
        // and if the remotes match what we expect.
        let repository = match gix::open(&self.dir) {
            Ok(repository) => repository,
            Err(gixerror) => return Err(errors::GitSyncError::GixOpenError { error: gixerror }),
        };

        let remote = match repository.find_remote("origin") {
            Ok(remote) => remote,
            Err(_) => {
                return Err(errors::GitSyncError::IncorrectGitRemotes {
                    dir: self.dir.clone(),
                    actual: String::from("No origin remote"),
                    expected: self.repo.clone(),
                })
            }
        };

        let remote_url = match remote.url(gix::remote::Direction::Fetch) {
            None => {
                return Err(errors::GitSyncError::IncorrectGitRemotes {
                    dir: self.dir.clone(),
                    actual: String::from(""),
                    expected: self.repo.clone(),
                })
            }
            Some(url) => url.to_string(),
        };

        if remote_url.ne(&self.repo) {
            return Err(errors::GitSyncError::IncorrectGitRemotes {
                dir: self.dir.clone(),
                actual: String::from(""),
                expected: self.repo.clone(),
            });
        }

        Ok(true)
    }
}
