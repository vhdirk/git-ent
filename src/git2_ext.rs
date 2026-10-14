use git2;

pub trait RepositoryExt {
    fn find_superproject(&self) -> Result<Option<git2::Repository>, git2::Error>;

    fn is_submodule(&self) -> Result<bool, git2::Error>;
}

impl RepositoryExt for git2::Repository {
    fn find_superproject(&self) -> Result<Option<git2::Repository>, git2::Error> {
        // Bare repositories have no workdir, so they cannot be submodule checkouts
        let Some(workdir) = self.workdir() else {
            return Ok(None);
        };

        // Canonicalize to avoid symlink and trailing-slash mismatches
        let Ok(workdir) = workdir.canonicalize() else {
            return Ok(None);
        };

        let Some(parent_dir) = workdir.parent() else {
            return Ok(None);
        };

        // Find the nearest enclosing repository, if any
        let parent = match git2::Repository::discover(parent_dir) {
            Ok(r) => r,
            Err(e) if e.code() == git2::ErrorCode::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        let Some(parent_workdir) = parent.workdir().and_then(|p| p.canonicalize().ok()) else {
            return Ok(None);
        };

        let Ok(relative) = workdir.strip_prefix(&parent_workdir) else {
            return Ok(None);
        };

        // Check whether the enclosing repository registers this path as a submodule
        match parent
            .find_submodule(&relative.to_string_lossy())
            .map(|_| true)
        {
            Ok(_) => Ok(Some(parent)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn is_submodule(&self) -> Result<bool, git2::Error> {
        let Some(workdir) = self.workdir() else {
            return Ok(false);
        };

        // A submodule has a .git file in its workdir and is not a worktree itself.
        Ok(workdir.join(".git").is_file() && !self.is_worktree())
    }
}
