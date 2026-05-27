use std::{error::Error, fmt, path::PathBuf};

#[derive(Debug)]
pub enum GitSyncError {
    IncorrectGitRemotes {
        dir: PathBuf,
        expected: String,
        actual: String,
    },
    CurrentBranchUnknown {
        dir: PathBuf,
    },
    WorkTreeNotClean,
    FastForwardMergeNotPossible,
    GixError {
        error: Box<dyn Error + Send + Sync>,
    },
    GitCommandError {
        command: String,
        stderr: String,
    },
    GenericError {
        error: std::io::Error,
    },
}

impl fmt::Display for GitSyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitSyncError::IncorrectGitRemotes {
                dir,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "A directory already exists at {} and is a Git repository, but we expected the remote to be {} and got {}.",
                    dir.display(),
                    expected,
                    actual
                )
            }

            GitSyncError::CurrentBranchUnknown { dir } => {
                write!(
                    f,
                    "The repository at {} is not on a named branch. Set GitSync::branch before syncing.",
                    dir.display()
                )
            }

            GitSyncError::FastForwardMergeNotPossible => {
                write!(f, "Can't fast-forward merge")
            }

            GitSyncError::WorkTreeNotClean => {
                write!(f, "The worktree isn't clean. Refusing to sync")
            }

            GitSyncError::GixError { error } => {
                write!(f, "There was an error reported by gix: {}", error)
            }

            GitSyncError::GitCommandError { command, stderr } => {
                write!(f, "Git command `{}` failed: {}", command, stderr)
            }

            GitSyncError::GenericError { error } => {
                write!(f, "There was an IO error: {}", error)
            }
        }
    }
}

impl Error for GitSyncError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GitSyncError::GixError { error } => Some(error.as_ref()),
            GitSyncError::GenericError { error } => Some(error),
            GitSyncError::GitCommandError { .. } => None,
            _ => None,
        }
    }
}

impl GitSyncError {
    pub fn from_gix(error: impl Error + Send + Sync + 'static) -> Self {
        GitSyncError::GixError {
            error: Box::new(error),
        }
    }
}
