# `sgit status`

Show status recursively across all submodules.

```bash
sgit status
```

Collects staged, modified, and untracked files from every repo in the
tree and displays them in a single consolidated view grouped by
category, similar to `git status`. File paths are shown relative to the
top-level project root, even for files inside deeply nested submodules.
