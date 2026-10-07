# `git-ent merge`

Merge a branch recursively across all submodules.

```bash
git-ent merge <branch>
```

Processes repos depth-first. Repos where the target branch doesn't
exist or HEAD is detached are skipped. If a conflict occurs, git-ent stops
and prints the `cd` + `git merge --continue` instructions for the
conflicting repo. Updated submodule pointers are automatically committed
in parent repos.
