use git2::build::RepoBuilder;
use git2::Repository;
use git2::{Cred, Error, RemoteCallbacks};
use std::path::Path;
use std::time::Duration;

#[cfg(feature = "actix")]
pub mod actix;

#[cfg(not(test))]
use log::info;

#[derive(Clone, Debug)]
pub struct GitSync {
    pub repo: String,
    pub dir: String,
    pub sync_every: Duration,
    pub username: Option<String>,
    pub passphrase: Option<String>,
    pub private_key: Option<String>,
}

impl GitSync {
    pub fn clone_repository(self) -> Result<Repository, Error> {
        info!(
            "Attempting to clone {} every {} to {}",
            self.repo,
            self.sync_every.as_secs(),
            self.dir,
        );

        let mut fetch_options = git2::FetchOptions::new();
        let mut builder = RepoBuilder::new();

        if self.private_key.is_some() {
            let mut callbacks = RemoteCallbacks::new();

            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                let username = match &self.username {
                    Some(u) => u.as_str(),
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

        return builder.clone(self.repo.as_str(), Path::new(&self.dir));
    }

    fn open_repository(&self) -> Repository {
        // Is this directory a Git repository?
        return match Repository::open(&self.dir) {
            Ok(repository) => repository,
            Err(_) => {
                panic!(
                    "Directory, {}, exists and is not a Git repository",
                    &self.dir
                );
            }
        };
    }

    pub fn does_clone_exist(&self) -> bool {
        // If the directory doesn't exist, we're good.
        if false == Path::new(&self.dir).exists() {
            return false;
        };

        let repository: Repository = self.open_repository();

        // Check if it has a remote
        let remote = match repository.find_remote("origin") {
            Ok(remote) => remote,
            Err(_) => {
                panic!(
                    "Directory, {}, exists and is a Git repository; but not one managed by Gitsync",
                    &self.dir
                );
            }
        };

        // Check if the remote matches what we expect
        match remote.url() {
            None => panic!("Not the correct remote URL"),
            Some(url) => {
                if url.eq(&self.repo) {
                    return true;
                }
            }
        }

        panic!(
            "There is a Git repository at {}, but its remote does not match {}",
            self.dir, self.repo
        );
    }
}

#[cfg(test)]
use std::{println as info, println as error};

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn it_can_check_if_clone_exists() -> Result<(), std::io::Error> {
        // Create a temporary directory for our test to clone the repository
        let clone_dir = TempDir::new("test-clone-public-repo-exists")?;

        let git_sync = GitSync {
            repo: String::from("https://gitlab.com/rawkode/gitsync"),
            sync_every: Duration::from_secs(30),
            dir: clone_dir
                .into_path()
                .join("clone")
                .into_os_string()
                .into_string()
                .unwrap(),
            username: None,
            private_key: None,
            passphrase: None,
        };

        let clone = git_sync.clone();

        assert_eq!(
            false,
            clone.does_clone_exist(),
            "testing we get a false when its a fresh clone"
        );

        let result = git_sync.clone_repository();
        match result {
            Err(e) => panic!("Failed to clone repository: {}", e),
            Ok(_) => (),
        }

        assert_eq!(
            true,
            clone.does_clone_exist(),
            "testing we get a true when the clone exists"
        );

        Ok(())
    }

    #[test]
    fn it_can_clone_a_public_repository() -> Result<(), std::io::Error> {
        // Create a temporary directory for our test to clone the repository
        let clone_dir = TempDir::new("test-clone-public-repo")?;

        let git_sync = GitSync {
            repo: String::from("https://gitlab.com/rawkode/gitsync"),
            sync_every: Duration::from_secs(30),
            dir: clone_dir
                .into_path()
                .into_os_string()
                .into_string()
                .unwrap(),
            username: None,
            private_key: None,
            passphrase: None,
        };

        let result = git_sync.clone_repository();

        assert_eq!(result.is_ok(), true);

        Ok(())
    }
}
