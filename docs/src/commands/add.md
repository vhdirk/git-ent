# `git-nest add`

Add files to the git index. Files can be from any submodule.

```bash
# Stage specific files
git-nest add <file> [<file> ...]

# Stage all changes (modified, deleted, untracked) across every repo
git-nest add -A

# Stage tracked-file changes only (modified, deleted) - skip untracked
git-nest add -u
```

| Flag             | Description                                                   |
|------------------|---------------------------------------------------------------|
| `-A`, `--all`    | Stage all changes in every repo (like `git add -A`).          |
| `-u`, `--update` | Stage tracked-file changes in every repo (like `git add -u`). |

Paths are the same as those shown by `git-nest status` - you don't need to
`cd` into a submodule first. git-nest automatically routes each file to the
**deepest** repo containing it. `-A` and `-u` are mutually exclusive.
