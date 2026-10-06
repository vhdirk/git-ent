# git nest

git nest is a tool to manage projects with (nested) git submodules.

The CLI mirrors familiar git subcommands but operates **recursively** across every submodule in the tree - so a single `git nest commit -m "fix"`
creates a commit in every repo that has staged changes, then automatically updates the submodule pointers in parent repos.

## Installation

```bash
cargo install --path .
```

Requires a Rust toolchain (1.80+), a C compiler, libgit2, libopenssl and `pkg-config` (needed by `libgit2-sys`).
A NixOS flake is provided that pins all of this.

## Quick start

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

# Squash a feature branch's commits (depth-first) onto main
git nest squash main
```

## Commands

### `git nest clone`

Clone a repository recursively (including all submodules).

```bash
git nest clone <url> [dest]
```

Equivalent to `git clone --recurse-submodules`.

### `git nest update`

Initialize and update all submodules recursively (`git submodule update --init --recursive`).

### `git nest status`

Show a single consolidated status view across the entire tree. File paths are shown relative to the top-level project root.

### `git nest branch`

List or create branches across the top-level project and every submodule.

```bash
git nest branch           # list
git nest branch -c feat   # create 'feat' everywhere
```

### `git nest switch`

Switch branches (or create them) recursively across every repo.

```bash
git nest switch <branch>                  # switch to an existing branch
git nest switch -c <new-branch>           # create and switch
git nest switch -c <new-branch> <start>   # create from a start point
git nest switch -C <branch> [<start>]     # force-create (reset if exists)
git nest switch --detach <commit-ish>     # detach HEAD everywhere
```

Repos where the target branch does not exist emit a warning and stay on their current branch.

### `git nest checkout`

Legacy branch-switching and file-restore command.

```bash
git nest checkout <branch>            # switch branch everywhere
git nest checkout -b <branch>         # create and switch
git nest checkout <branch> -- <file>  # restore files from a branch
```

For new work prefer `git nest switch` - it has a cleaner interface and matches modern git conventions.

### `git nest add`

Add files to the git index - from any submodule. Paths are the same as those shown by `git nest status`.

```bash
git nest add <file>...     # stage specific files (routed to the deepest owning repo)
git nest add -A            # like git add -A, everywhere
git nest add -u            # tracked changes only, everywhere
```

### `git nest commit`

Commit across all (sub)modules that have staged changes.

```bash
git nest commit -m "message"
git nest commit -m "message" --no-verify   # skip pre-commit / commit-msg hooks
git nest commit                            # opens $EDITOR with a git-style template
```

Commits are depth-first; submodule pointer updates are auto-staged in parent repos.

### `git nest push`

Push to remote across all repos that have commits to push.

```bash
git nest push
git nest push -o merge_request.create      # push options are repeatable
```

Repos with no remote, a detached HEAD, or no unpushed commits are silently skipped. For a merge-request shortcut, define a `pushmr` alias
in config (see the alias section below).

### `git nest restore`

Restore working-tree files or unstage changes, recursively.

```bash
git nest restore <file>...         # discard unstaged changes in specific files
git nest restore                   # discard all unstaged changes in the tree
git nest restore -S <file>...      # unstage specific files
git nest restore -S                # unstage everything
```

### `git nest reset`

```bash
git nest reset                     # mixed reset (unstage all)
git nest reset --hard              # discard all changes
git nest reset --hard HEAD~1       # ...to a specific ref
```

### `git nest merge <branch>`

Merge `<branch>` recursively across all submodules, depth-first. Auto-commits updated submodule pointers in parents. Conflicts abort with
instructions for the failing repo.

### `git nest rebase <branch>`

Rebase all branches recursively onto `<branch>`. Same semantics as `merge` for submodule pointers and conflicts.

### `git nest squash <branch>`

Squash all commits on the current branch back to `merge-base(HEAD, <branch>)`, recursively depth-first across submodules. This is the
depth-first equivalent of GitHub/GitLab's "squash and merge":

1. Walk depth-first, squash each submodule first.
2. For each repo, find `merge-base(HEAD, <branch>)` and
   `git reset --soft` to it, so everything since then is staged.
3. Stage any submodule-pointer entries that moved due to earlier
   squashes in children.
4. Create a single commit (`-m <msg>` or an auto-generated summary).

A parent with no commits of its own to squash but a dirty submodule pointer still gets a single pointer-update commit, preserving the
tree invariant.

```bash
git nest squash main
git nest squash main -m "feat: squashed merge of MR 42"
```

## Configuration

git nest reads two config files per operation:

- **Global**: `~/.config/git-nest/git-nest.toml` (XDG: `$XDG_CONFIG_HOME/git-nest/git-nest.toml`).
  Exclusions apply to every repo on the machine.
- **Repo-local**: `.git-nest.toml` at the root of any repo (root or submodule).
  Exclusions are relative to that repo root and may target nested submodules (for example `mid/leaf`).

Both files use the same format and are merged; duplicates are deduplicated.

```toml
# .git-nest.toml  or  ~/.config/git-nest/git-nest.toml
exclude = ["vendor/heavy-sdk", "third_party/legacy"]
```

Each entry in `exclude` is matched as a full path relative to the repo that owns the config file, so nested paths are supported.

## Design

See the full docs in [`docs/`](./docs/) or build with `mdbook serve docs`.
The core abstraction is **`RepoTree`**:

```text
RepoTree
├── root                  - the top-level repo
├── submodules            - depth-first list of submodule repos
├── all()                 - submodules + root (deepest first, root last)
├── resolve_file(path)    - route any path to its owning repo
└── stage_submodule_pointers(&repo)   - auto-stage dirty refs
```

## Development

```bash
# Build
cargo build

# Run tests (integration tests spin up real git repos in tempdirs)
cargo test

# API docs
cargo doc --no-deps --open

# User docs (mdbook)
mdbook serve docs
```
