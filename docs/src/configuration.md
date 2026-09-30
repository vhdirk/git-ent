# Configuration

sgit reads configuration from two locations:

1. **Global** - `$XDG_CONFIG_HOME/sgit/sgit.toml` (typically
   `~/.config/sgit/sgit.toml`). Configures the `git` executable, global
   exclusions, and global aliases.
2. **Repo-local** - `.sgit.toml` at the root of each individual repo.
   Exclusions are relative to that repo root and can target nested
   submodules (for example `mid/leaf`).

## Git Executable

The path to the `git` executable can **only** be configured globally in
`$XDG_CONFIG_HOME/sgit/sgit.toml`. It cannot be set per repository:

```toml
# ~/.config/sgit/sgit.toml
git = "/usr/bin/git"  # Optional; defaults to "git" on PATH
```

## Where config files can be placed

- Global config: `$XDG_CONFIG_HOME/sgit/sgit.toml` (usually `~/.config/sgit/sgit.toml`)
- Root project config: `<project-root>/.sgit.toml`
- Submodule config: `<submodule>/.sgit.toml`

All three locations are valid. For exclusions, each repo's own `.sgit.toml`
governs its own subtree using paths relative to that repo root.

## Format

```toml
exclude = ["vendor/heavy-sdk", "third_party/legacy"]
```

Any submodule path listed under `exclude` is skipped for **all** sgit
operations - status, add, commit, push, switch, etc. Paths are
relative to the repo that contains the config file and use forward
slashes on all platforms.

## Aliases

Aliases use an `[alias]` table, similar to `.gitconfig` aliases.

```toml
[alias]
pushmr = "push -o merge_request.create -o merge_request.remove_source_branch -o merge_request.merge_when_pipeline_succeeds"
co = "checkout"
st = "status"
```

Examples:

```bash
sgit pushmr
sgit co main
sgit st
```

Alias resolution order:

1. Global alias config
2. Root project `.sgit.toml` (overrides global)

Aliases from submodule `.sgit.toml` files are ignored.

## Scope

| Config file location        | Governs                                                      |
|-----------------------------|--------------------------------------------------------------|
| `<project-root>/.sgit.toml` | Entire root subtree (supports nested paths like `mid/leaf`). |
| `<submodule>/.sgit.toml`    | That submodule subtree, relative to the submodule root.      |

Exclusions are applied at discovery time. Excluded submodules (and
their own children) never appear in `RepoTree::all()`, so no sgit
command will touch them.

## Example - mono-repo with vendor dependencies

```text
my-project/
  .sgit.toml          <- excludes vendor/*
  app/                <- submodule, included
  lib/                <- submodule, included
  vendor/
    some-lib/         <- submodule, excluded via root .sgit.toml
```

Root `.sgit.toml`:

```toml
exclude = ["vendor/some-lib"]
```

Nested-path example from a root repo:

```toml
exclude = ["mid/leaf"]
```

## Notes

- A missing `.sgit.toml` is silently ignored (no exclusions apply).
- A file with a parse error emits a warning to stderr and falls back to
  no exclusions; it does not abort the current operation.
- Exclusions are path-literal matches - no glob support yet.
