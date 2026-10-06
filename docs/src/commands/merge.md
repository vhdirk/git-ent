# `git-nest merge`

Merge a branch recursively across all submodules.

```bash
git-nest merge <branch>
```

Processes repos depth-first. Repos where the target branch doesn't
exist or HEAD is detached are skipped. If a conflict occurs, git-nest stops
and prints the `cd` + `git merge --continue` instructions for the
conflicting repo. Updated submodule pointers are automatically committed
in parent repos.
