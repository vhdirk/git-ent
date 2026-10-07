# `git-ent restore`

Restore working tree files or unstage changes, recursively.

```bash
# Discard unstaged changes in specific files
git-ent restore <file> [<file> ...]

# Discard all unstaged changes across the entire tree
git-ent restore

# Unstage specific files
git-ent restore -S <file> [<file> ...]

# Unstage everything
git-ent restore -S
```

| Flag             | Description                                               |
|------------------|-----------------------------------------------------------|
| `-S`, `--staged` | Unstage files instead of discarding working-tree changes. |

Without `--staged`, restores working-tree files to their last committed
state (like `git restore`). With `--staged`, moves files from the index
back to the working tree (like `git restore --staged`).
