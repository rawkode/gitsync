use errors::GitSyncError;
use gix::bstr::ByteSlice;
use gix::remote::Direction;
use gix::{ObjectId, Repository};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;

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

    fn ensure_worktree_is_clean(&self) -> Result<(), errors::GitSyncError> {
        let repository = gix::open(self.dir.clone()).map_err(GitSyncError::from_gix)?;

        if repository.is_dirty().map_err(GitSyncError::from_gix)? {
            return Err(GitSyncError::WorkTreeNotClean);
        }

        let mut statuses = repository
            .status(gix::progress::Discard)
            .map_err(GitSyncError::from_gix)?
            .untracked_files(gix::status::UntrackedFiles::Files)
            .into_index_worktree_iter(Vec::<gix::bstr::BString>::new())
            .map_err(GitSyncError::from_gix)?;

        if let Some(status) = statuses.next() {
            status.map_err(GitSyncError::from_gix)?;
            return Err(GitSyncError::WorkTreeNotClean);
        }

        Ok(())
    }

    fn sync_branch(&self, repository: &Repository) -> Result<String, errors::GitSyncError> {
        if let Some(branch) = &self.branch {
            return Ok(branch.clone());
        }

        let head_name = repository
            .head_name()
            .map_err(GitSyncError::from_gix)?
            .ok_or_else(|| GitSyncError::CurrentBranchUnknown {
                dir: self.dir.clone(),
            })?;

        Ok(head_name.shorten().to_str_lossy().into_owned())
    }

    pub fn sync(&self) -> Result<(), errors::GitSyncError> {
        self.ensure_worktree_is_clean()?;
        let repository = gix::open(&self.dir).map_err(GitSyncError::from_gix)?;
        let branch = self.sync_branch(&repository)?;
        let branch_reference = format!("refs/heads/{}", branch);

        self.fetch(&repository)?;

        let mut local_reference = repository
            .find_reference(branch_reference.as_str())
            .map_err(GitSyncError::from_gix)?;
        let local_id = local_reference
            .peel_to_id()
            .map_err(GitSyncError::from_gix)?
            .detach();

        let remote_reference = format!("refs/remotes/origin/{}", branch);
        let mut remote_reference = repository
            .find_reference(remote_reference.as_str())
            .map_err(GitSyncError::from_gix)?;
        let remote_id = remote_reference
            .peel_to_id()
            .map_err(GitSyncError::from_gix)?
            .detach();

        if local_id == remote_id {
            return Ok(());
        }

        if !self.is_ancestor(&repository, local_id, remote_id)? {
            return Err(GitSyncError::FastForwardMergeNotPossible);
        }

        local_reference
            .set_target_id(
                remote_id,
                format!("fast-forward {} to {}", branch_reference, remote_id),
            )
            .map_err(GitSyncError::from_gix)?;

        self.git(&["checkout", "--force", branch.as_str()])?;
        self.git(&["reset", "--hard", remote_id.to_string().as_str()])?;

        Ok(())
    }

    fn fetch(&self, repository: &Repository) -> Result<(), errors::GitSyncError> {
        let remote = repository
            .find_remote("origin")
            .map_err(GitSyncError::from_gix)?;
        let interrupt = AtomicBool::new(false);
        remote
            .connect(Direction::Fetch)
            .map_err(GitSyncError::from_gix)?
            .prepare_fetch(gix::progress::Discard, Default::default())
            .map_err(GitSyncError::from_gix)?
            .receive(gix::progress::Discard, &interrupt)
            .map_err(GitSyncError::from_gix)?;

        Ok(())
    }

    fn is_ancestor(
        &self,
        repository: &Repository,
        ancestor: ObjectId,
        descendant: ObjectId,
    ) -> Result<bool, errors::GitSyncError> {
        let mut seen = HashSet::new();
        let mut commits = vec![descendant];

        while let Some(commit_id) = commits.pop() {
            if commit_id == ancestor {
                return Ok(true);
            }

            if !seen.insert(commit_id) {
                continue;
            }

            let commit = repository
                .find_commit(commit_id)
                .map_err(GitSyncError::from_gix)?;
            commits.extend(commit.parent_ids().map(gix::Id::detach));
        }

        Ok(false)
    }

    fn clone_repository(&self) -> Result<(), errors::GitSyncError> {
        info!("Attempting to clone {} to {:?}", self.repo, self.dir,);

        let mut prepare =
            gix::prepare_clone(self.repo.as_str(), &self.dir).map_err(GitSyncError::from_gix)?;

        if let Some(branch) = self.branch.as_deref() {
            prepare = prepare
                .with_ref_name(Some(branch))
                .map_err(GitSyncError::from_gix)?;
        }

        let interrupt = AtomicBool::new(false);
        let mut checkout = prepare
            .fetch_then_checkout(gix::progress::Discard, &interrupt)
            .map_err(GitSyncError::from_gix)?
            .0;
        checkout
            .main_worktree(gix::progress::Discard, &interrupt)
            .map_err(GitSyncError::from_gix)?;
        self.git(&["remote", "set-url", "origin", self.repo.as_str()])?;

        Ok(())
    }

    fn does_clone_exist(&self) -> Result<bool, errors::GitSyncError> {
        // No local directory exists, so we can happily clone when required.
        if !Path::new(&self.dir).exists() {
            return Ok(false);
        };

        // OK. If a directory exists, we need to check if it's a Git repository
        // and if the remotes match what we expect.
        let repository = gix::open(&self.dir).map_err(GitSyncError::from_gix)?;
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

        let remote_url = remote
            .url(Direction::Fetch)
            .ok_or_else(|| errors::GitSyncError::IncorrectGitRemotes {
                dir: self.dir.clone(),
                actual: String::from(""),
                expected: self.repo.clone(),
            })?
            .to_bstring()
            .to_str_lossy()
            .into_owned();

        if remote_url.ne(self.repo.as_str()) {
            return Err(errors::GitSyncError::IncorrectGitRemotes {
                dir: self.dir.clone(),
                actual: remote_url,
                expected: self.repo.clone(),
            });
        }

        Ok(true)
    }

    fn git(&self, args: &[&str]) -> Result<(), errors::GitSyncError> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .args(args)
            .output()
            .map_err(|error| GitSyncError::GenericError { error })?;

        if output.status.success() {
            return Ok(());
        }

        Err(GitSyncError::GitCommandError {
            command: format!("git -C {} {}", self.dir.display(), args.join(" ")),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
