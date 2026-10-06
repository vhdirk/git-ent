# Design

git-nest is built around a single abstraction - **`RepoTree`** - which
models the top-level repo and all its (nested) submodules as a
depth-first traversable tree. Every command creates a `RepoTree` and
operates over it.

```text
RepoTree
├── root                  - the top-level repo
├── submodules            - depth-first list of submodule repos
├── all()                 - submodules + root (deepest first, root last)
├── resolve_file(path)    - route any path to its owning repo
├── list_status(&repo)    - staged / unstaged / untracked entries
└── stage_submodule_pointers(&repo) - auto-stage dirty refs
```

## Key behaviours

- **Depth-first ordering** - submodules are always processed before
  their parents. Commits propagate upward correctly and pushes never
  push a parent that references a submodule commit the remote hasn't
  received yet.
- **Automatic submodule pointer staging** - when `git-nest commit` detects
  that a submodule HEAD has moved, it stages the updated pointer in the
  parent before committing, keeping everything in sync with a single
  message.
- **Smart file routing** - `git-nest add` and `git-nest restore` accept paths
  relative to the project root and automatically figure out which repo
  each file belongs to, picking the **deepest** matching repo.

## Module layout

| Module                         | Purpose                                       |
|--------------------------------|-----------------------------------------------|
| [`git_nest::cli`]              | `clap`-derived CLI surface and dispatcher.    |
| [`git_nest::commands`]         | One submodule per subcommand.                 |
| [`git_nest::repo_tree`]        | `RepoTree`, status models, helpers.           |
| [`git_nest::error`]            | `GitNestError` via `thiserror`.               |
| [`git_nest::git`]              | Thin wrapper around shelling out to `git`.    |

[`git_nest::cli`]: ../doc/git-nest/cli/index.html
[`git_nest::commands`]: ../doc/git-nest/commands/index.html
[`git_nest::repo_tree`]: ../doc/git-nest/repo_tree/index.html
[`git_nest::error`]: ../doc/git-nest/error/index.html
[`git_nest::git`]: ../doc/git-nest/git/index.html
