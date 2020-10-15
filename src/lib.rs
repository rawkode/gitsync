// use actix::prelude::*;
use git2::build::RepoBuilder;
use git2::Repository;
use git2::{Cred, Error, RemoteCallbacks};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(not(test))]
use log::{info, warn};

#[derive(Clone, Debug)]
pub struct GitSync {
    pub repo: String,
    pub dir: String,
    pub sync_every: Duration,
    pub username: Option<String>,
    pub passphrase: Option<String>,
    pub private_key: Option<String>,
}

// impl Actor for GitSync {
//     type Context = Context<Self>;

//     fn started(&mut self, ctx: &mut Context<Self>) {
//         info!("Starting ...");

//         // Clone Git repository
//         self.clone_repository();

//         ctx.run_interval(self.sync_every, move |act, ctx| {
//             info!("Syncing ...");

//             // Prepare callbacks.
//             let mut callbacks = RemoteCallbacks::new();
//             callbacks.credentials(|_url, username_from_url, _allowed_types| {
//                 Cred::ssh_key(
//                     username_from_url.unwrap(),
//                     None,
//                     std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
//                     None,
//                 )
//             });

//             // Prepare fetch options.
//             let mut fo = git2::FetchOptions::new();
//             fo.remote_callbacks(callbacks);

//             let repository: Repository = match Repository::open("repository") {
//                 Ok(repository) => repository,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("Failed to open repository");
//                 }
//             };

//             let mut remote = match repository.find_remote("origin") {
//                 Ok(remote) => remote,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("Failed to get 'origin' remote from repository");
//                 }
//             };

//             let old_obj_id = match repository
//                 .refname_to_id(&format!("refs/remotes/{}/{}", "origin", "master"))
//             {
//                 Ok(r) => r,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("help");
//                 }
//             };

//             if let Err(e) = remote.fetch(&["master"], Some(&mut fo), None) {
//                 ctx.stop();
//                 panic!("Failed to fetch latest changes for repository: {}", e);
//             };

//             let new_obj_id = match repository
//                 .refname_to_id(&format!("refs/remotes/{}/{}", "origin", "master"))
//             {
//                 Ok(r) => r,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("help");
//                 }
//             };

//             if new_obj_id == old_obj_id {
//                 info!("No updates to repository");
//                 return;
//             }

//             let obj = match repository.find_object(new_obj_id, None) {
//                 Ok(r) => r,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("help 2");
//                 }
//             };

//             repository.reset(&obj, ResetType::Hard, None);

//             let paths = fs::read_dir("repository").unwrap();

//             for path in paths {
//                 println!("Name: {}", path.unwrap().path().display())
//             }

//             // info!("Sending RELOAD to NodeGroupController");
//             // act.cluster_controller.do_send(ReloadCommand);

//             info!("Synced!");
//         });
//     }
// }

impl GitSync {
    fn clone_repository(self) -> Result<Repository, Error> {
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

    fn does_clone_exist(self) -> bool {
        // If the directory doesn't exist, we're good.
        if false == Path::new(&self.dir).exists() {
            return false;
        };

        // Is this directory a Git repository?
        let repository: Repository = match Repository::open(&self.dir) {
            Ok(repository) => repository,
            Err(_) => {
                panic!(
                    "Directory, {}, exists and is not a Git repository",
                    &self.dir
                );
            }
        };

        // Check if it has a remote
        // I don't know how to clone and rename the origin yet
        let mut remote = match repository.find_remote("origin") {
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

        panic!("Urls don't match");
    }

    //     fn clone_repository(&mut self) {
    //         info!("Cloning Repository {}", self.git_url);

    //         // Prepare callbacks.

    //         // Prepare fetch options.
    //         let mut fo = git2::FetchOptions::new();
    //         fo.remote_callbacks(callbacks);

    //         // Prepare builder.
    //         let mut builder = git2::build::RepoBuilder::new();
    //         builder.fetch_options(fo);

    //         // Clone the project.
    //         match builder.clone(self.git_url.as_str(), Path::new("repository")) {
    //             Ok(_) => info!("Cloned"),
    //             Err(_) => error!("Already cloned"),
    //         }
    //     }
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
        let mut clone_dir = TempDir::new("test-clone-public-repo-exists")?;

        let git_sync = GitSync {
            repo: String::from("https://gitlab.com/rawkode/dotfiles"),
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

        let assert1 = git_sync.clone();
        let assert2 = git_sync.clone();

        assert_eq!(
            false,
            assert2.does_clone_exist(),
            "testing we get a false when its a fresh clone"
        );

        let result = git_sync.clone_repository();
        match result {
            Err(e) => panic!("Weird failure"),
            Ok(r) => (),
        }

        assert_eq!(
            true,
            assert1.does_clone_exist(),
            "testing we get a true when the clone exists"
        );

        Ok(())
    }

    #[test]
    fn it_can_clone_a_public_repository() -> Result<(), std::io::Error> {
        // Create a temporary directory for our test to clone the repository
        let mut clone_dir = TempDir::new("test-clone-public-repo")?;

        let git_sync = GitSync {
            repo: String::from("https://gitlab.com/rawkode/dotfiles"),
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
