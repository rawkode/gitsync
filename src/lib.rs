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
        let remote = repository.find_remote("origin")?;

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempdir::TempDir;

//     #[test]
//     fn it_can_check_if_clone_exists() -> Result<(), std::io::Error> {
//         // Create a temporary directory for our test to clone the repository
//         let clone_dir = TempDir::new("test-clone-public-repo-exists")?;

//         let git_sync = GitSync {
//             repo: String::from("https://gitlab.com/rawkode/gitsync"),
//             sync_every: Duration::from_secs(30),
//             dir: clone_dir.into_path().join("clone-dir"),
//             username: None,
//             private_key: None,
//             passphrase: None,
//         };

//         let clone = git_sync.clone();

//         assert_eq!(
//             false,
//             clone.does_clone_exist(),
//             "testing we get a false when its a fresh clone"
//         );

//         let result = git_sync.clone_repository();
//         match result {
//             Err(e) => panic!("Failed to clone repository: {}", e),
//             Ok(_) => (),
//         }

//         assert_eq!(
//             true,
//             clone.does_clone_exist(),
//             "testing we get a true when the clone exists"
//         );

//         Ok(())
//     }

//     #[test]
//     fn it_can_clone_a_public_repository() -> Result<(), std::io::Error> {
//         // Create a temporary directory for our test to clone the repository
//         let clone_dir = TempDir::new("test-clone-public-repo")?;

//         let git_sync = GitSync {
//             repo: String::from("https://gitlab.com/rawkode/gitsync"),
//             sync_every: Duration::from_secs(30),
//             dir: clone_dir.into_path(),
//             username: None,
//             private_key: None,
//             passphrase: None,
//         };

//         let result = git_sync.clone_repository();

//         assert_eq!(result.is_ok(), true);

//         Ok(())
//     }
// }
