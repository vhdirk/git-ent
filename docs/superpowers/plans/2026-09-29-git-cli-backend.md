# Git CLI Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace production `git2`/libgit2 repository operations with the Git executable while preserving `sgit`'s recursive orchestration and documented command behavior.

**Architecture:** Add one argument-safe process boundary in `src/git.rs`; have command implementations invoke Git in each `Repo.workdir`; then migrate `RepoTree` discovery, status, and submodule handling to machine-readable Git output and remove its `Repository` field. Keep recursive selection, path routing, and order in Rust, and use Git's native operation semantics inside each repository.

**Tech Stack:** Rust 2021, `std::process::Command`, `OsString`, `thiserror`, Cargo integration tests using temporary real Git repositories, Nix packaging.

**Spec:** `docs/superpowers/specs/2026-09-29-git-cli-backend-design.md`

## Global Constraints

- Pass Git arguments as distinct OS strings; never invoke a shell or interpolate command strings.
- The installed `git` executable is a runtime dependency and missing Git must produce a clear actionable error.
- Keep recursive repository discovery, traversal order, exclusions, and deepest-owner path routing in `RepoTree`.
- Use stable machine-readable Git output for parsed state; do not parse human-formatted output.
- Preserve command-specific recursive behavior for clone, update, push, hooks, conflict handling, and submodule pointers.
- Do not add another Git implementation such as `gix`.

## Review Focus

- **Paths/arguments containing whitespace, leading dashes, and shell metacharacters:** pin safe handling in the process, add, and restore tests.
- **Nested, uninitialized, and excluded submodules:** retain initialized-tree discovery and parent-first clone/update behavior; test nested and excluded cases.
- **Hooks and edited messages:** prove Git runs hooks once, hook failure prevents that repository's commit, and `--no-verify` bypasses the hooks.
- **Child-first operations and parent gitlinks:** prove commits, squash, and pushes preserve their required ordering and parent pointer updates.
- **Missing branches, conflicts, detached HEAD, and remote failures:** pin expected skip/abort/error behavior with focused command tests and repository context in errors.

---

### Task 1: Add the Git Process Boundary

**Files:**
- Modify: `src/git.rs`
- Modify: `src/error.rs`
- Create: `tests/test_git.rs`
- Test helpers: `tests/common/mod.rs` only if focused fixtures are needed

**Interfaces:**
- Add `run_git(cwd: &Path, args: &[OsString]) -> Result<Output>`; it succeeds only for exit status zero and reports command failure with working-directory context and useful stderr.
- Add an internal `run_git_with(executable: &OsStr, cwd: &Path, args: &[OsString]) -> Result<Output>` so spawn failures can be tested without mutating process-wide `PATH`.
- Add `run_git_allow_failure(cwd: &Path, args: &[OsString]) -> Result<Output>` for commands where a non-zero status is an expected query result; callers must inspect the returned status.
- Add `SgitError` variants for executable launch failure and non-zero Git exit. Do not include remote URLs or arbitrary argument contents in displayed errors.

- [x] **Step 1: Write failing process-layer tests** in `tests/test_git.rs` for successful invocation in a specified cwd, distinct argument preservation with spaces/shell metacharacters, non-zero exit including repository context/stderr, and an unavailable executable.
- [x] **Step 2: Run `cargo test --test test_git`;** confirm the new tests fail because the runner and error variants do not exist.
- [x] **Step 3: Implement the process boundary** in `src/git.rs` using `Command::new`, `.args`, and `.current_dir`; centralize exit-status conversion in `src/error.rs`. Preserve `Output` for callers that need machine-readable stdout.
- [x] **Step 4: Run `cargo test --test test_git` and `cargo check --all-targets`;** both must pass before callers are migrated.

### Task 2: Migrate Local Index and Branch Commands

**Files:**
- Modify: `src/commands/add.rs`
- Modify: `src/commands/restore.rs`
- Modify: `src/commands/reset.rs`
- Modify: `src/commands/branch.rs`
- Modify: `src/commands/checkout.rs`
- Modify: `src/commands/switch.rs`
- Tests: `tests/test_add.rs`, `tests/test_restore.rs`, `tests/test_reset.rs`, `tests/test_branch.rs`, `tests/test_checkout.rs`, `tests/test_switch.rs`

**Interfaces:**
- Each command invokes the Task 1 runner with `Repo.workdir` as cwd; it does not access `Repo.repo` after this task.
- Keep existing public `run(...)` signatures and `RepoTree` traversal/path-routing behavior.

- [x] **Step 1: Add focused regression tests** for a path with spaces and a leading dash in add/restore; retain the existing `-A`/`-u`, unstage, branch-idempotency, checkout path-routing, switch force-create, and reset tests as behavior assertions.
- [x] **Step 2: Run** `cargo test --test test_add --test test_restore --test test_reset --test test_branch --test test_checkout --test test_switch`; confirm newly added cases fail or expose current behavior that the Git invocation must preserve.
- [x] **Step 3: Replace direct index, branch, checkout, and reset mutations** with Git CLI arguments. Preserve `--` path separators, per-repository iteration, command-specific warnings, checkout `-b` idempotency, switch `-C` force-create behavior, and paths routed to the deepest owning repository.
- [x] **Step 4: Run the same six test targets and `cargo check --all-targets`;** all must pass. Do not remove `Repo.repo` yet because remaining command groups and `RepoTree` still use it.

### Task 3: Migrate Repository Discovery, Status, and Pointer Staging

**Files:**
- Modify: `src/repo_tree.rs`
- Modify: `src/config.rs`
- Modify: `tests/test_repo_tree.rs`
- Modify: `tests/test_status.rs`
- Modify: `tests/test_cli.rs` or add a focused config test where root-config discovery is exercised

**Interfaces:**
- Migrate `discover`, `collect_submodules`, `branch_name`, `list_status`, `has_changes`, and pointer staging to Git CLI operations.
- Keep `repo: git2::Repository` temporarily for command groups not yet migrated; populate it by opening each discovered worktree. Remove this compatibility field only in Task 7 after all command call sites have moved to Git.
- Preserve `label()`, `display_label()`, `branch_name()`, `list_status()`, `has_changes()`, `RepoTree::all()`, and file-routing interfaces unless a small signature adjustment is needed to return `crate::Result` for Git failures.
- Provide a path-parameterized root-config discovery helper, `load_root_config_from(cwd: &Path) -> SgitConfig`, with `load_root_config()` delegating to the current directory.

- [x] **Step 1: Add tests** for nested initialized traversal, uninitialized child omission, root and nested `.sgit.toml` exclusions, detached branch reporting, status paths containing whitespace, and changed submodule-pointer detection/staging. Extend `tests/test_repo_tree.rs` and `tests/test_status.rs`.
- [x] **Step 2: Run** `cargo test --test test_repo_tree --test test_status`; confirm the new assertions fail against the existing Git2-backed implementation where applicable.
- [x] **Step 3: Implement Git-backed discovery and status.** Find the worktree with `git rev-parse --show-toplevel`; enumerate `.gitmodules` paths using `git config -z --file ... --get-regexp` and parse NUL-terminated records; recurse only into initialized worktrees while applying the existing config exclusion chain. Determine initialization by asking Git to resolve the submodule worktree, not by assuming `.git` is a directory. Parse `git status --porcelain=v2 -z` for staged, unstaged, untracked, rename, and submodule state. Use `git symbolic-ref --quiet --short HEAD` for attached branch names. Stage changed gitlinks with `git add -- <submodule-path>`. Open the discovered repositories with `Repository::open` only as a temporary bridge for command modules not migrated yet.
- [x] **Step 4: Migrate `load_root_config`** from `git2::Repository::discover` to repeated `git rev-parse --show-toplevel` lookups while walking parent directories; test both a repository root and a nested repository path.
- [x] **Step 5: Remove libgit2 discovery/status calls from `src/repo_tree.rs` and `src/config.rs`, retaining only the temporary `Repo.repo` bridge;** run `cargo test --test test_repo_tree --test test_status` and `cargo check --all-targets`.

### Task 4: Migrate Commit and Native Hook Handling

**Files:**
- Modify: `src/commands/commit.rs`
- Modify: `tests/test_commit.rs`
- Modify: `src/git.rs` only if the runner needs a message argument helper

**Interfaces:**
- Keep `commit::run(message: Option<&str>, no_verify: bool) -> Result<()>`.
- `commit.rs` uses only `Repo.workdir`, `RepoTree::list_status`, `RepoTree::stage_submodule_pointers`, and the shared Git runner; remove manual hook and direct commit construction helpers.

- [x] **Step 1: Add/adjust tests** to assert one execution each for `pre-commit`, `prepare-commit-msg`, and `commit-msg`, preservation of a `commit-msg` edited message, hook failure preventing the failing repository's commit, `--no-verify` bypass of `pre-commit` and `commit-msg`, and child-before-parent commit plus pointer staging. Use existing tests `commit_staged_changes`, `commits_in_submodule_and_parent`, `auto_stages_submodule_pointer`, `no_verify_bypasses_hooks`, `pre_commit_hook_rejection_prevents_commit`, and `commit_msg_hook_can_edit_message_from_configured_hooks_path` as the baseline; add a `prepare-commit-msg` test to pin behavior absent from the current manual hook runner.
- [x] **Step 2: Run `cargo test --test test_commit`;** verify the hook-count and ordering assertions discriminate against the current manual-hook flow where appropriate.
- [x] **Step 3: Invoke `git commit` per repository** only when staged changes exist; pass explicit messages as a distinct `-m` argument and pass `--no-verify` through. For no-message commits, preserve the existing `sgit` message-template/editor workflow, then pass the edited message to Git. Do not invoke hook binaries manually.
- [x] **Step 4: Preserve child-first traversal and parent gitlink staging**, and propagate hook/commit failures with repository context while retaining existing overall error behavior.
- [x] **Step 5: Run `cargo test --test test_commit --test test_repo_tree`;** all commit and pointer-staging cases must pass.

### Task 5: Migrate Merge, Rebase, and Squash

**Files:**
- Modify: `src/commands/merge.rs`
- Modify: `src/commands/rebase.rs`
- Modify: `src/commands/squash.rs`
- Tests: `tests/test_merge.rs`, `tests/test_rebase.rs`, `tests/test_squash.rs`

**Interfaces:**
- Keep existing `run(...)` entry points and recursive ordering.
- Squash continues to use the migrated `reset::reset_to()` path; all branch/revision checks use Git machine-readable queries through `src/git.rs`.

- [x] **Step 1: Extend focused tests** for merge conflict cleanup/error hints, rebase conflict abort and repository state, squash merge-base behavior, generated/custom messages, and submodule-only parent pointer commits. Retain tests `merge_conflict_exits`, `rebase_onto_branch`, `squash_collapses_multiple_commits_into_one`, `squash_custom_message`, `squash_default_message_lists_subjects`, and `squash_submodule_only_creates_parent_pointer_commit`.
- [x] **Step 2: Run** `cargo test --test test_merge --test test_rebase --test test_squash`; confirm the new conflict-state assertions fail before implementation.
- [x] **Step 3: Replace libgit2 history operations** with `git merge`, `git rebase`, `git merge-base`, `git rev-list`, `git reset --soft`, and `git commit` as appropriate. Use explicit options to preserve current fast-forward, conflict, cleanup, and message behavior; inspect exit status rather than parsing human output.
- [x] **Step 4: Keep child-before-parent ordering** and the existing rule that missing branches are skipped/reported as tested. Ensure a failed conflict operation leaves the worktree in the intended state and prints the correct per-repository resolution hint.
- [x] **Step 5: Run the same three test targets and `cargo check --all-targets`.**

### Task 6: Migrate Clone, Update, and Push

**Files:**
- Modify: `src/commands/clone.rs`
- Modify: `src/commands/update.rs`
- Modify: `src/commands/push.rs`
- Tests: `tests/test_clone.rs`, `tests/test_update.rs`, `tests/test_push.rs`

**Interfaces:**
- Keep existing command `run(...)` entry points and CLI option behavior.
- Remove `push::remote_callbacks()` after clone/update/push no longer depend on libgit2 callbacks; Git uses configured transports, SSH agents, and credential helpers.

- [x] **Step 1: Add network-operation integration tests** using local bare remotes for nested clone/update initialization, excluded submodules where the current behavior applies, recursive child-before-parent push, push options, and a failing remote with repository context.
- [x] **Step 2: Run** `cargo test --test test_clone --test test_update --test test_push`; verify new tests expose currently untested behavior.
- [x] **Step 3: Replace clone with `git clone`** while preserving inferred destination and recursive submodule initialization. Replace update with `git submodule update --init --recursive` or equivalent per-repository traversal where required to retain exclusions. Keep initialization parent-first because uninitialized children are not in `RepoTree` yet.
- [x] **Step 4: Replace push with Git CLI operations** while retaining deepest-first order, first-remote selection, detached/no-remote/up-to-date skips, ahead checks, push options, upstream setup, and selected user-facing notices. Use status codes and machine-readable queries, not formatted push output as state.
- [x] **Step 5: Run** `cargo test --test test_clone --test test_update --test test_push` and the focused recursive commit/squash tests to verify shared ordering and pointer behavior.

### Task 7: Remove libgit2 and Update Packaging/Documentation

**Files:**
- Modify: `Cargo.toml`, `Cargo.lock`
- Modify: `src/git.rs`, `src/error.rs`, `src/config.rs`, `src/repo_tree.rs`, and any remaining command modules
- Modify: `nix/package.nix`, `flake.nix`
- Modify: `README.md`, `docs/src/installation.md`, and affected command documentation under `docs/src/commands/`

**Interfaces:**
- No production module imports or exposes `git2` types.
- Nix-installed `sgit` has Git on its runtime `PATH`; development shells retain Git and no longer require libgit2 solely for `sgit`.

- [x] **Step 1: Search for remaining references** with `rg -n 'git2|Repository::|\.repo\.' src Cargo.toml nix flake.nix`; categorize any matches that are comments/docs versus live dependency use.
- [x] **Step 2: Remove the temporary `Repo.repo` bridge and all remaining libgit2 helpers, error variants, dependency declarations, and build inputs.** Remove `git2` from `Cargo.toml`, regenerate `Cargo.lock`, remove `libgit2.dev`, `openssl.dev`, and now-unused `pkg-config` inputs where no other build requirement remains. Add `makeWrapper` to the Nix package build inputs and wrap `$out/bin/sgit` with Git's `bin` directory prepended to `PATH`, so Git is present when the packaged program runs rather than only during its build.
- [x] **Step 3: Update installation and command docs** to state Git is required at runtime and remove libgit2/C-compiler setup requirements only where they no longer apply. Clarify hook and credential behavior now comes from Git.
- [x] **Step 4: Run `cargo fmt --all -- --check`, `cargo test --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`, and `git diff --check`.** Run `nix build .#sgit`, then invoke the packaged program with `nix run .#sgit -- status` from a test checkout to verify its wrapper can resolve Git at runtime.
- [x] **Step 5: Verify `rg -n 'git2|libgit2|libgit2-sys' src Cargo.toml Cargo.lock nix flake.nix README.md docs/src` finds no active dependency or inaccurate setup documentation.**

## Final Acceptance Gate

- All seven tasks pass their focused checks before proceeding to the next task.
- `cargo test --all-targets`, formatting, lint, and available Nix package validation pass at the end.
- Production code has no libgit2 dependency; every Git invocation uses the shared argument-safe process layer.
- Existing documented recursive behavior, ordering, path routing, hook semantics, conflict handling, and remote workflow are covered by integration tests.
- Git runtime requirements and intentional behavior differences are documented.
