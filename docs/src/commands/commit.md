# `git-nest commit`

Commit across all (sub)modules that have staged changes.

```bash
git-nest commit -m "message"

# Skip pre-commit and commit-msg hooks
git-nest commit -m "message" --no-verify

# No -m --> opens $EDITOR with a git-style template
git-nest commit
```

| Flag              | Description                           |
|-------------------|---------------------------------------|
| `-m`, `--message` | Commit message.                       |
| `--no-verify`     | Skip pre-commit and commit-msg hooks. |

Commits are created **depth-first**: the deepest submodules are
committed first. After each submodule commit, its updated pointer is
automatically staged in the parent repo so a single message covers the
entire tree.
