# Quick start

```bash
# Clone a project and all its submodules in one step
sgit clone https://github.com/org/project.git

# Initialize / update submodules in an existing checkout
sgit update

# See all changes across the entire tree
sgit status

# Stage files - paths from any submodule work
sgit add lib/core/src/main.rs app/config.yaml

# Stage everything (like git add -A)
sgit add -A

# Commit everywhere that has staged changes
sgit commit -m "update config and core logic"

# Push all repos that have unpushed commits
sgit push

# Squash all commits on the current branch (across every submodule)
# relative to `main` - depth-first, GitHub-style.
sgit squash main
```
