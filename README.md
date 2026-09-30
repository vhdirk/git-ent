# sgit (/ʃɪt/)

sgit, short for **subgit**, is a tool to manage projects with (nested) git submodules.

The CLI mirrors familiar git subcommands but operates **recursively** across every submodule in the tree - so a single `sgit commit -m "fix"`
creates a commit in every repo that has staged changes, then automatically updates the submodule pointers in parent repos.

## Installation

```bash
cargo install --path .
```

Requires a Rust toolchain (1.80+) and `git` on your `PATH`.
A NixOS flake is provided that packages `sgit` with `git` wrapped into its runtime environment.

## Quick start

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

# Squash a feature branch's commits (depth-first) onto main
sgit squash main
```

## Commands

### `sgit clone`

Clone a repository recursively (including all submodules).

```bash
sgit clone <url> [dest]
```

Equivalent to `git clone --recurse-submodules`.

### `sgit update`

Initialize and update all submodules recursively (`git submodule update --init --recursive`).

### `sgit status`

Show a single consolidated status view across the entire tree. File paths are shown relative to the top-level project root.

### `sgit branch`

List or create branches across the top-level project and every submodule.

```bash
sgit branch           # list
sgit branch -c feat   # create 'feat' everywhere
```

### `sgit switch`

Switch branches (or create them) recursively across every repo.

```bash
sgit switch <branch>                  # switch to an existing branch
sgit switch -c <new-branch>           # create and switch
sgit switch -c <new-branch> <start>   # create from a start point
sgit switch -C <branch> [<start>]     # force-create (reset if exists)
sgit switch --detach <commit-ish>     # detach HEAD everywhere
```

Repos where the target branch does not exist emit a warning and stay on their current branch.

### `sgit checkout`

Legacy branch-switching and file-restore command.

```bash
sgit checkout <branch>            # switch branch everywhere
sgit checkout -b <branch>         # create and switch
sgit checkout <branch> -- <file>  # restore files from a branch
```

For new work prefer `sgit switch` - it has a cleaner interface and matches modern git conventions.

### `sgit add`

Add files to the git index - from any submodule. Paths are the same as those shown by `sgit status`.

```bash
sgit add <file>...     # stage specific files (routed to the deepest owning repo)
sgit add -A            # like git add -A, everywhere
sgit add -u            # tracked changes only, everywhere
```

### `sgit commit`

Commit across all (sub)modules that have staged changes.

```bash
sgit commit -m "message"
sgit commit -m "message" --no-verify   # skip pre-commit / commit-msg hooks
sgit commit                            # opens $EDITOR with a git-style template
```

Commits are depth-first; submodule pointer updates are auto-staged in parent repos.

### `sgit push`

Push to remote across all repos that have commits to push.

```bash
sgit push
sgit push -o merge_request.create      # push options are repeatable
```

Repos with no remote, a detached HEAD, or no unpushed commits are silently skipped. For a merge-request shortcut, define a `pushmr` alias
in config (see the alias section below).

### `sgit restore`

Restore working-tree files or unstage changes, recursively.

```bash
sgit restore <file>...         # discard unstaged changes in specific files
sgit restore                   # discard all unstaged changes in the tree
sgit restore -S <file>...      # unstage specific files
sgit restore -S                # unstage everything
```

### `sgit reset`

```bash
sgit reset                     # mixed reset (unstage all)
sgit reset --hard              # discard all changes
sgit reset --hard HEAD~1       # ...to a specific ref
```

### `sgit merge <branch>`

Merge `<branch>` recursively across all submodules, depth-first. Auto-commits updated submodule pointers in parents. Conflicts abort with
instructions for the failing repo.

### `sgit rebase <branch>`

Rebase all branches recursively onto `<branch>`. Same semantics as `merge` for submodule pointers and conflicts.

### `sgit squash <branch>`

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
sgit squash main
sgit squash main -m "feat: squashed merge of MR 42"
```

## Configuration

sgit reads two config files per operation:

- **Global**: `~/.config/sgit/sgit.toml` (XDG: `$XDG_CONFIG_HOME/sgit/sgit.toml`).
  Exclusions apply to every repo on the machine.
- **Repo-local**: `.sgit.toml` at the root of any repo (root or submodule).
  Exclusions are relative to that repo root and may target nested submodules (for example `mid/leaf`).

Both files use the same format and are merged; duplicates are deduplicated.

```toml
# .sgit.toml  or  ~/.config/sgit/sgit.toml
exclude = ["vendor/heavy-sdk", "third_party/legacy"]
```

Each entry in `exclude` is matched as a full path relative to the repo that owns the config file, so nested paths are supported.

### Command aliases

You can define command aliases in config files using `[alias]`.

```toml
[alias]
pushmr = "push -o merge_request.create -o merge_request.remove_source_branch -o merge_request.merge_when_pipeline_succeeds"
co = "checkout"
st = "status"
```

Usage:

```bash
sgit pushmr
sgit co main
sgit st
```

Alias config locations and scope:

- **Global** aliases in `~/.config/sgit/sgit.toml` (or `$XDG_CONFIG_HOME/sgit/sgit.toml`) apply everywhere.
- **Root repo** aliases in `<project-root>/.sgit.toml` override global aliases.
- **Submodule** `.sgit.toml` aliases are ignored.

Exclusions apply at discovery time and affect **all** sgit operations.
A missing or unparseable file does not abort the current command.

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
