# Design

sgit is built around a single abstraction - **`RepoTree`** - which
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
- **Automatic submodule pointer staging** - when `sgit commit` detects
  that a submodule HEAD has moved, it stages the updated pointer in the
  parent before committing, keeping everything in sync with a single
  message.
- **Smart file routing** - `sgit add` and `sgit restore` accept paths
  relative to the project root and automatically figure out which repo
  each file belongs to, picking the **deepest** matching repo.

## Module layout

| Module                     | Purpose                                       |
|----------------------------|-----------------------------------------------|
| [`sgit::cli`]              | `clap`-derived CLI surface and dispatcher.    |
| [`sgit::commands`]         | One submodule per subcommand.                 |
| [`sgit::repo_tree`]        | `RepoTree`, status models, helpers.           |
| [`sgit::error`]            | `SgitError` via `thiserror`.                  |
| [`sgit::git`]              | Thin wrapper around shelling out to `git`.    |

[`sgit::cli`]: ../doc/sgit/cli/index.html
[`sgit::commands`]: ../doc/sgit/commands/index.html
[`sgit::repo_tree`]: ../doc/sgit/repo_tree/index.html
[`sgit::error`]: ../doc/sgit/error/index.html
[`sgit::git`]: ../doc/sgit/git/index.html
