# `git-nest squash`

Squash all commits since the common ancestor with `<branch>`,
recursively depth-first across submodules.

```bash
# Default: message is built from squashed commit subjects
git-nest squash <branch>

# Custom message
git-nest squash <branch> -m "Squashed feature work"
```

| Flag              | Description                                                    |
|-------------------|----------------------------------------------------------------|
| `<branch>`        | The base branch whose **merge-base** with `HEAD` defines the squash range. |
| `-m`, `--message` | Commit message. Defaults to a summary of the squashed subjects. |

The semantics mirror GitHub/GitLab's **"squash and merge"**, applied at
every level of the (sub)module tree:

1. For each repo, walk depth-first (deepest submodules first).
2. Find `merge-base(HEAD, <branch>)`.
3. `git reset --soft` to that merge-base. All work since then is now
   staged; the working tree is untouched.
4. Because deeper submodules were squashed first, their HEAD has
   already moved. Their parent's submodule-pointer entry is now dirty
   - git-nest stages it before committing.
5. Create a single new commit. If `-m` was not given, the message is
   built from the subjects of the squashed commits (newest last, in
   original chronological order).

## Submodule-only squash

A repo with **no commits to squash** but a **dirty submodule pointer**
(because a nested submodule got squashed) still gets a single
pointer-update commit. This preserves the tree invariant that parent
refs always point at existing child commits.

## Example

```text
main branch tree                    squash onto main

root (3 commits)                    root (1 squash commit)
├── sub-a (2 commits)               ├── sub-a (1 squash commit)
└── sub-b (5 commits)               └── sub-b (1 squash commit)
```

After `git-nest squash main`, every repo has exactly one new commit
replacing the work that diverged from `main`, and the parent's
submodule pointers correctly record the newly-squashed children.

## Skipping rules

- HEAD is detached --> skip.
- `<branch>` doesn't exist in the repo --> skip (matches `git-nest merge`).
- Already on `<branch>` --> skip (nothing sensible to do).
- No commits to squash and no submodule pointer change --> skip.
