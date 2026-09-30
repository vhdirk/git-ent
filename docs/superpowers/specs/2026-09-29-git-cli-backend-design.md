# Git CLI Backend Design

**Status:** Approved design direction; awaiting written-spec review.

## Goal

Replace `git2`/libgit2 as `sgit`'s repository-operation backend with the installed Git executable, while retaining `sgit`'s Rust CLI and recursive orchestration of nested repositories.

## Context

`sgit` recursively applies Git-like commands across a repository and its submodules. The current implementation stores `git2::Repository` in `Repo` and uses libgit2 across repository discovery, status, staging, history operations, and remote operations. This requires `sgit` to implement or manually reproduce parts of Git's CLI behaviour, including commit hook handling.

The executable is already used by integration-test helpers to create real temporary repositories. Moving production repository operations to Git aligns command semantics with users' Git installation, at the cost of requiring Git at runtime.

## Chosen Architecture

A small Rust process layer will invoke Git with explicit argument arrays and a repository working directory. It will not interpolate commands through a shell. The layer will provide consistent process-launch errors, exit-status handling, and access to stdout and stderr. Git's standard environment, configuration, credential helpers, and hooks remain available unless an individual command deliberately needs different arguments or environment.

`RepoTree` remains the owner of recursive repository discovery, traversal order, path-to-repository routing, exclusions, and multi-repository orchestration. Repository handles become path/topology-oriented rather than holding `git2::Repository` instances. Operations requiring structured results use Git's machine-readable output formats; human-formatted output is not parsed as a data contract.

Commands execute once per selected repository through the shared process layer. Operations are not routed through a shell, and paths and user arguments remain distinct process arguments. The installed `git` executable becomes an explicit runtime dependency; missing Git must produce a clear actionable error.

## Behavioural Requirements

1. Preserve the current recursive selection rules, deepest-first ordering where required, exclusions, and deepest-owning-repository routing for paths.
2. Preserve `sgit`'s recursive workflow while allowing Git itself to own each repository operation's hooks, signing, configuration, credential helpers, and native conflict behaviour.
3. Commit child repositories before parents and stage changed submodule pointers in the parent before committing it. Remove manual hook execution so hooks run once through `git commit`; `--no-verify` is passed through with Git's native meaning.
4. Preserve command-specific recursive behaviour for clone, update, and push, including submodule initialization/update, upstream setup, and current push eligibility/reporting rules unless a difference is required to delegate behaviour correctly to Git.
5. Preserve relevant user-facing output where practical. For structured state, use stable machine-readable Git output and parse paths safely, including paths containing whitespace or unusual characters.
6. Propagate repository failures with repository context. A failed child operation must not be reported as success for the overall command; continue-or-stop behaviour across repositories must match existing `sgit` behaviour unless a change is explicitly identified and tested.
7. Do not add a second Git implementation such as `gix` as part of this migration.

## Scope

In scope:
- Shared Git process invocation and typed error conversion.
- Removal of `git2::Repository` ownership from repository handles.
- Migration of repository discovery, configuration-root discovery, status, submodule-pointer staging, and all current commands to Git CLI operations.
- Removal of `git2` dependencies and obsolete libgit2 helpers after all production call sites are migrated.
- Documentation of the runtime Git requirement and any behavior differences found during migration.

Out of scope:
- Replacing Rust CLI parsing, configuration parsing, recursive orchestration, or user-facing command names.
- Adding new Git commands or changing `sgit`'s product scope.
- Guaranteeing byte-for-byte identical output to every historical `sgit` version where that output was not documented; changes must be deliberate and covered by tests.
- Implementing Git object operations directly in Rust.

## Failure Handling

The process layer distinguishes inability to launch Git from a Git command's non-zero exit status. Errors include the repository path and preserve useful stderr context without treating arbitrary stderr output alone as failure. Git output that is parsed must be requested in a stable machine-readable format. Subprocess invocation must not invoke a shell, even when paths, remotes, or user-supplied arguments contain spaces or shell metacharacters.

Remote operations retain Git's normal credential-helper and transport behavior. Authentication and per-repository failures must be surfaced with enough context to identify the repository and operation.

## Migration and Verification

Migrate incrementally behind the process layer. First establish invocation and repository-topology tests; then migrate local inspection and staging, local mutations, history/conflict operations, and finally network operations. Each stage must pass focused integration tests against temporary real Git repositories before the next stage proceeds. The final gate is formatting, the complete Rust test suite, and documentation checks available in the repository.

The existing `RepoTree`, status, add, restore, commit, clone, update, and push integration tests are primary regression coverage. Add focused tests for missing Git, non-zero exit with repository context, argument/path safety, hook execution exactly once, nested repository order, conflict outcomes, and remote failures as their owning components migrate.

## Acceptance Criteria

- Production code no longer depends on `git2` or libgit2.
- All repository operations are performed through the shared argument-safe Git process layer.
- Recursive traversal, exclusions, path routing, and parent/child ordering are covered by integration tests and retain their expected behaviour.
- Git-native commit hooks run once; hook failure blocks the commit; `--no-verify` follows native Git behaviour.
- Clone, update, and push preserve the existing recursive contract and surface repository-specific failures.
- Git's runtime requirement and any intentional user-visible differences are documented.
- Formatting and all repository tests pass.
