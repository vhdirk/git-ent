# `git-ent rebase`

Rebase all branches recursively onto the given branch.

```bash
git-ent rebase <branch>
```

Processes repos depth-first. If a conflict occurs, git-ent stops and
prints the `cd` + `git rebase --continue` instructions for the
conflicting repo so you can resolve it manually. Updated submodule
pointers are automatically committed in parent repos.
