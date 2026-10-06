# Quick start

```bash
# Clone a project and all its submodules in one step
git nest clone https://github.com/org/project.git

# Initialize / update submodules in an existing checkout
git nest update

# See all changes across the entire tree
git nest status

# Stage files - paths from any submodule work
git nest add lib/core/src/main.rs app/config.yaml

# Stage everything (like git add -A)
git nest add -A

# Commit everywhere that has staged changes
git nest commit -m "update config and core logic"

# Push all repos that have unpushed commits
git nest push

# Squash all commits on the current branch (across every submodule)
# relative to `main` - depth-first, GitHub-style.
git nest squash main
```
