# `git-nest switch`

Recursive equivalent of `git switch`. Switches branches (or creates
them) across the top-level repo and every submodule.

```bash
# Switch to an existing branch everywhere
git-nest switch <branch>

# Create a new branch from HEAD and switch to it everywhere
git-nest switch -c <new-branch>

# Create from a specific start point
git-nest switch -c <new-branch> <start-point>

# Force-create (reset branch pointer if it already exists)
git-nest switch -C <branch> [<start-point>]

# Detach HEAD at a commit-ish everywhere
git-nest switch --detach <commit-ish>
```

| Flag                   | Description                                                        |
|------------------------|--------------------------------------------------------------------|
| `-c`, `--create`       | Create a new branch and switch to it.                              |
| `-C`, `--force-create` | Create or hard-reset the branch pointer, then switch.              |
| `--detach`             | Detach HEAD at the given commit-ish in every repo.                 |

Repos in which the target branch does not exist emit a warning and are
left on their current branch. Repos that fail to switch emit an error
and processing continues with the remaining repos.
