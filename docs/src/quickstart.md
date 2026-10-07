# Quick start

```bash
# Clone a project and all its submodules in one step
git ent clone https://github.com/org/project.git

# Initialize / update submodules in an existing checkout
git ent update

# See all changes across the entire tree
git ent status

# Stage files - paths from any submodule work
git ent add lib/core/src/main.rs app/config.yaml

# Stage everything (like git add -A)
git ent add -A

# Commit everywhere that has staged changes
git ent commit -m "update config and core logic"

# Push all repos that have unpushed commits
git ent push

# Squash all commits on the current branch (across every submodule)
# relative to `main` - depth-first, GitHub-style.
git ent squash main
```
