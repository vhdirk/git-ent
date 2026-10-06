# `git-nest restore`

Restore working tree files or unstage changes, recursively.

```bash
# Discard unstaged changes in specific files
git-nest restore <file> [<file> ...]

# Discard all unstaged changes across the entire tree
git-nest restore

# Unstage specific files
git-nest restore -S <file> [<file> ...]

# Unstage everything
git-nest restore -S
```

| Flag             | Description                                               |
|------------------|-----------------------------------------------------------|
| `-S`, `--staged` | Unstage files instead of discarding working-tree changes. |

Without `--staged`, restores working-tree files to their last committed
state (like `git restore`). With `--staged`, moves files from the index
back to the working tree (like `git restore --staged`).
