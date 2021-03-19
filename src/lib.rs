use errors::GitSyncError;
use git2::build::RepoBuilder;
use git2::{Cred, RemoteCallbacks};
use git2::{Repository, StatusOptions};
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
    pub branch: String,
    pub dir: PathBuf,
    pub sync_every: Duration,
    pub username: Option<String>,
    pub passphrase: Option<String>,
    pub private_key: Option<String>,
}

impl GitSync {
    pub fn bootstrap(&self) -> Result<(), errors::GitSyncError> {
        if true == self.does_clone_exist()? {
            return Ok(());
        }

        self.clone_repository()?;

        return Ok(());
    }

    fn check_worktree_is_clean(&self) -> bool {
        let mut opts = StatusOptions::new();
        opts.include_ignored(false);
        opts.include_untracked(true);

        let repository = match Repository::open(self.dir.clone()) {
            Ok(repository) => repository,
            Err(_) => {
                return false;
            }
        };

        let clean = match repository.statuses(Some(&mut opts)) {
            Ok(status) => status.is_empty(),
            Err(_) => false,
        };

        clean
    }

    pub fn sync(&self) -> Result<(), errors::GitSyncError> {
        if !self.check_worktree_is_clean() {
            return Err(GitSyncError::WorkTreeNotClean);
        }

        let mut fetch_options = git2::FetchOptions::new();

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

        let repository: Repository = Repository::open(&self.dir)?;
        let mut remote = repository.find_remote("origin")?;
        remote.fetch(&[&self.branch], Some(&mut fetch_options), None)?;

        let fetch_head = repository.find_reference("FETCH_HEAD")?;
        let fetch_commit = repository.reference_to_annotated_commit(&&fetch_head)?;
        let analysis = repository.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_up_to_date() {
            return Ok(());
        }

        // We only support fast forward merges for now
        if !analysis.0.is_fast_forward() {
            return Err(GitSyncError::FastForwardMergeNotPossible);
        }

        let refname = format!("refs/heads/{}", &self.branch);

        match repository.find_reference(&refname) {
            Ok(mut r) => {
                // fast_forward(repo, &mut r, &fetch_commit)?;
                let name = match r.name() {
                    Some(s) => s.to_string(),
                    None => String::from_utf8_lossy(r.name_bytes()).to_string(),
                };

                let msg = format!(
                    "Fast-Forward: Setting {} to id: {}",
                    name,
                    fetch_commit.id()
                );

                r.set_target(fetch_commit.id(), &msg)?;
                repository.set_head(&name)?;
                repository.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        // For some reason the force is required to make the working directory actually get updated
                        // I suspect we should be adding some logic to handle dirty working directory states
                        // but this is just an example so maybe not.
                        .force(),
                ))?;
            }
            Err(_) => {
                return Err(GitSyncError::FastForwardMergeNotPossible);
            }
        };

        return Ok(());
    }

    fn clone_repository(&self) -> Result<(), errors::GitSyncError> {
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
