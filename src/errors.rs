use std::{fmt, path::PathBuf};

pub enum GitSyncError {
    IncorrectGitRemotes {
        dir: PathBuf,
        expected: String,
        actual: String,
    },
    WorkTreeNotClean,
    FastForwardMergeNotPossible,
    Git2Error {
        error: git2::Error,
    },
    GenericError {
        error: std::io::Error,
    },
}

impl fmt::Debug for GitSyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitSyncError::IncorrectGitRemotes {
                dir,
                expected,
                actual,
            } => {
                write!(f, "A directory already exists at {} and is a Git repository, but we expected the remote to be {} and got {}.",
                dir.to_str().unwrap(), expected, actual)
            }

            GitSyncError::FastForwardMergeNotPossible => {
                write!(f, "Can't fast-forward merge")
            }

            GitSyncError::WorkTreeNotClean => {
                write!(f, "The worktree isn't clean. Refusing to sync")
            }

            GitSyncError::Git2Error { error } => {
                write!(f, "There was an error reported by git2-rs: {}", error)
            }

            GitSyncError::GenericError { error } => {
                write!(f, "There was an IO error: {}", error)
            }
        }
    }
}

impl From<git2::Error> for GitSyncError {
    fn from(error: git2::Error) -> Self {
        GitSyncError::Git2Error { error }
    }
}
