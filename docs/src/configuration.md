# Configuration

git-ent reads two config files per operation, merging them in order:

1. **Global** - `$XDG_CONFIG_HOME/git-ent/git-ent.toml` (typically
   `~/.config/git-ent/git-ent.toml`). Exclusions here apply to every repo on
   the machine.
2. **Repo-local** - `.git-ent.toml` at the root of each individual repo.
  Exclusions are relative to that repo root and can target nested
  submodules (for example `mid/leaf`).

Both files use the same format. If the same path appears in both, it is
deduplicated.

## Where config files can be placed

- Global config: `$XDG_CONFIG_HOME/git-ent/git-ent.toml` (usually `~/.config/git-ent/git-ent.toml`)
- Root project config: `<project-root>/.git-ent.toml`
- Submodule config: `<submodule>/.git-ent.toml`

All three locations are valid. For exclusions, each repo's own `.git-ent.toml`
governs its own subtree using paths relative to that repo root.

## Format

```toml
exclude = ["vendor/heavy-sdk", "third_party/legacy"]
```

Any submodule path listed under `exclude` is skipped for **all** git-ent
operations - status, add, commit, push, switch, etc. Paths are
relative to the repo that contains the config file and use forward
slashes on all platforms.

## Scope

| Config file location            | Governs                                                      |
|---------------------------------|--------------------------------------------------------------|
| `<project-root>/.git-ent.toml` | Entire root subtree (supports nested paths like `mid/leaf`). |
| `<submodule>/.git-ent.toml`    | That submodule subtree, relative to the submodule root.      |

Exclusions are applied at discovery time. Excluded submodules (and
their own children) never appear in `RepoTree::all()`, so no git-ent
command will touch them.

## Example - mono-repo with vendor dependencies

```text
my-project/
  .git-ent.toml          <- excludes vendor/*
  app/                <- submodule, included
  lib/                <- submodule, included
  vendor/
    some-lib/         <- submodule, excluded via root .git-ent.toml
```

Root `.git-ent.toml`:

```toml
exclude = ["vendor/some-lib"]
```

Nested-path example from a root repo:

```toml
exclude = ["mid/leaf"]
```

## Notes

- A missing `.git-ent.toml` is silently ignored (no exclusions apply).
- A file with a parse error emits a warning to stderr and falls back to
  no exclusions; it does not abort the current operation.
- Exclusions are path-literal matches - no glob support yet.
