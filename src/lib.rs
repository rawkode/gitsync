use git2::build::RepoBuilder;
use git2::Repository;
use git2::{Cred, RemoteCallbacks};
use std::path::{Path, PathBuf};
use std::time::Duration;

// #[cfg(feature = "actix")]
// pub mod actix;
pub mod errors;

// When running tests, we can just use println instead of logger
#[cfg(not(test))]
use log::info;

#[cfg(test)]
use std::println as info;

#[derive(Clone, Debug)]
pub struct GitSync {
    pub repo: String,
    pub dir: PathBuf,
    pub sync_every: Duration,
    pub username: Option<String>,
    pub passphrase: Option<String>,
    pub private_key: Option<String>,
}

impl GitSync {
    pub fn bootstrap(self) -> Result<(), errors::GitSyncError> {
        if true == self.does_clone_exist()? {
            return Ok(());
        }

        self.clone_repository()?;

        return Ok(());
    }

    pub fn sync(self) -> Result<(), errors::GitSyncError> {
        return Ok(());
    }

    fn clone_repository(self) -> Result<(), errors::GitSyncError> {
        info!("Attempting to clone {} to {:?}", self.repo, self.dir,);

        let mut fetch_options = git2::FetchOptions::new();
        let mut builder = RepoBuilder::new();

        if self.private_key.is_some() {
            let mut callbacks = RemoteCallbacks::new();

            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                let username = match &self.username {
                    Some(u) => u,
                    _ => username_from_url.unwrap(),
                };

                Cred::ssh_key(
                    username,
                    None,
                    std::path::Path::new(&self.private_key.clone().unwrap()),
                    self.passphrase.as_deref(),
                )
            });

            fetch_options.remote_callbacks(callbacks);
        }

        builder.fetch_options(fetch_options);

        builder.clone(self.repo.as_str(), Path::new(&self.dir))?;

        Ok(())
    }

    fn does_clone_exist(&self) -> Result<bool, errors::GitSyncError> {
        // No local directory exists, so we can happily clone when required.
        if false == Path::new(&self.dir).exists() {
            return Ok(false);
        };

        // OK. If a directory exists, we need to check if it's a Git repository
        // and if the remotes match what we expect.
        let repository = Repository::open(&self.dir)?;
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

        let remote_url = match remote.url() {
            None => {
                return Err(errors::GitSyncError::IncorrectGitRemotes {
                    dir: self.dir.clone(),
                    actual: String::from(""),
                    expected: self.repo.clone(),
                })
            }
            Some(url) => url,
        };

        if remote_url.ne(self.repo.as_str()) {
            return Err(errors::GitSyncError::IncorrectGitRemotes {
                dir: self.dir.clone(),
                actual: String::from(""),
                expected: self.repo.clone(),
            });
        }

        Ok(true)
    }
}
