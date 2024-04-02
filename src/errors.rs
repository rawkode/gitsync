use std::{fmt, path::PathBuf};

pub enum GitSyncError {
    IncorrectGitRemotes {
        dir: PathBuf,
        expected: String,
        actual: String,
    },
    WorkTreeNotClean,
    FastForwardMergeNotPossible,

    GixOpenError {
        error: gix::open::Error,
    },
    GixParseError {
        error: gix::url::parse::Error,
    },
    GixCloneError {
        error: gix::clone::Error,
    },
    GixCloneFetchError {
        error: gix::clone::fetch::Error,
    },
    GixCloneCheckoutError {
        error: gix::clone::checkout::main_worktree::Error,
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

            GitSyncError::GixOpenError { error } => {
                write!(
                    f,
                    "There was an error opening the Git repository: {}",
                    error
                )
            }

            GitSyncError::GixParseError { error } => {
                write!(f, "There was an error parsing the URL: {}", error)
            }

            GitSyncError::GixCloneError { error } => {
                write!(f, "There was an error cloning the repository: {}", error)
            }

            GitSyncError::GixCloneFetchError { error } => {
                write!(f, "There was an error fetching the repository: {}", error)
            }

            GitSyncError::GixCloneCheckoutError { error } => {
                write!(f, "There was an error checking out the repository: {}", error)
            }

            GitSyncError::GenericError { error } => {
                write!(f, "There was an IO error: {}", error)
            }
        }
    }
}
