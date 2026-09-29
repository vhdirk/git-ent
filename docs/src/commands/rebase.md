# `sgit rebase`

Rebase all branches recursively onto the given branch.

```bash
sgit rebase <branch>
```

Processes repos depth-first. If a conflict occurs, sgit stops and
prints the `cd` + `git rebase --continue` instructions for the
conflicting repo so you can resolve it manually. Updated submodule
pointers are automatically committed in parent repos.
