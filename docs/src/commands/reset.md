# `git-ent reset`

Reset HEAD across all repos recursively.

```bash
# Unstage all staged changes (mixed reset)
git-ent reset

# Hard reset to HEAD - discard all changes
git-ent reset --hard

# Hard reset to a specific ref
git-ent reset --hard HEAD~1
git-ent reset --hard <commit-sha>
git-ent reset --hard origin/main
```

| Flag / Arg | Description                                                                                   |
|------------|-----------------------------------------------------------------------------------------------|
| `[ref]`    | Target ref to reset to (default: `HEAD`). Accepts `HEAD~N`, a commit SHA, a branch name, etc. |
| `--hard`   | Discard all changes and reset the working tree.                                               |

Without `--hard`, performs a mixed reset (unstages everything but keeps
working-tree changes). With `--hard`, resets the index and working tree
to match the target ref.
