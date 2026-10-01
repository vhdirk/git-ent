use std::ffi::OsString;
use std::process::Output;

use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};

/// How to handle commits that are or become empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyAction {
    Drop,
    Keep,
    Ask,
    Stop,
}

/// Action to take on whitespace errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceAction {
    Apply,
    Warn,
    NoWarn,
    Error,
    ErrorAll,
    Fix,
}

/// Mode for rebasing merges (`--rebase-merges[=<mode>]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseMergesMode {
    RebaseCousins,
    NoRebaseCousins,
}

/// Format submode for `--show-current-patch[=(diff|raw)]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowCurrentPatchSubmode {
    Diff,
    Raw,
}

/// Options for starting a rebase operation.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Start {
    /// Upstream branch or commit to compare against.
    pub upstream: Option<String>,
    /// Working branch to rebase; defaults to `HEAD` if not set.
    pub branch: Option<String>,
    /// Starting point at which to create the new commits (`--onto <newbase>`).
    pub onto: Option<String>,
    /// Use the merge base of upstream and branch as the base (`--keep-base`).
    pub keep_base: bool,
    /// Rebase all reachable commits up to the root (`--root`).
    pub root: bool,
    /// Let the user edit the list of commits to rebase (`-i`, `--interactive`).
    pub interactive: bool,
    /// Command to append as `exec <cmd>` after each commit line (`-x`, `--exec`).
    pub exec: Vec<String>,
    /// Automatically modify todo list for `squash!` and `fixup!` (`--autosquash` / `--no-autosquash`).
    pub auto_squash: Option<bool>,
    /// Automatically stash and pop before/after rebase (`--autostash` / `--no-autostash`).
    pub auto_stash: Option<bool>,
    /// How to handle commits that become empty (`--empty={drop,keep,ask,stop}`).
    pub empty: Option<EmptyAction>,
    /// Cherry-pick all commits, even if unchanged (`-f`, `--force-rebase`, `--no-ff`).
    pub force_rebase: bool,
    /// Use `merge-base --fork-point` to refine upstream (`--fork-point` / `--no-fork-point`).
    pub fork_point: Option<bool>,
    /// Ignore whitespace changes when resolving conflicts (`--ignore-whitespace`).
    pub ignore_whitespace: bool,
    /// Detect and handle whitespace errors (`--whitespace=<action>`).
    pub whitespace: Option<WhitespaceAction>,
    /// Make committer date match author date (`--committer-date-is-author-date`).
    pub committer_date_is_author_date: bool,
    /// Ignore author date and use current date (`--reset-author-date`).
    pub reset_author_date: bool,
    /// Add Signed-off-by trailer to each commit (`--signoff`).
    pub signoff: bool,
    /// Add custom trailer(s) (`--trailer <trailer>`).
    pub trailers: Vec<String>,
    /// GPG-sign commits (`-S`, `--gpg-sign[=<keyid>]`, `--no-gpg-sign`).
    pub gpg_sign: Option<Option<String>>,
    /// Use the given merge strategy (`-s`, `--strategy=<strategy>`).
    pub strategy: Option<String>,
    /// Pass option to the merge strategy (`-X`, `--strategy-option=<option>`).
    pub strategy_options: Vec<String>,
    /// Update index with reused conflict resolution (`--rerere-autoupdate` / `--no-rerere-autoupdate`).
    pub rerere_autoupdate: Option<bool>,
    /// Reapply all clean cherry-picks (`--reapply-cherry-picks` / `--no-reapply-cherry-picks`).
    pub reapply_cherry_picks: Option<bool>,
    /// Rebase merge commits instead of flattening (`-r`, `--rebase-merges[=<mode>]`).
    pub rebase_merges: Option<Option<RebaseMergesMode>>,
    /// Update branches that point to commits being rebased (`--update-refs` / `--no-update-refs`).
    pub update_refs: Option<bool>,
    /// Automatically reschedule failed `exec` commands (`--reschedule-failed-exec`).
    pub reschedule_failed_exec: Option<bool>,
    /// Suppress informative output (`-q`, `--quiet`).
    pub quiet: bool,
    /// Display diffstat of changes (`-v`, `--verbose`).
    pub verbose: bool,
    /// Show or suppress diffstat (`--stat` / `-n`, `--no-stat`).
    pub stat: Option<bool>,
    /// Bypass the `pre-rebase` hook (`--no-verify`).
    pub no_verify: bool,
    /// Allow the `pre-rebase` hook to run (`--verify`).
    pub verify: bool,
    /// Use apply strategies to rebase (`--apply`).
    pub apply: bool,
    /// Use merging strategies to rebase (`-m`, `--merge`).
    pub merge: bool,
    /// Context lines passed to `git apply` (`-C <n>`).
    pub context_lines: Option<usize>,
}

impl Start {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if let Some(onto) = &self.onto {
            args.push("--onto".into());
            args.push(onto.into());
        }
        if self.keep_base {
            args.push("--keep-base".into());
        }
        if self.root {
            args.push("--root".into());
        }
        if self.interactive {
            args.push("--interactive".into());
        }
        for cmd in &self.exec {
            args.push("--exec".into());
            args.push(cmd.into());
        }
        match self.auto_squash {
            Some(true) => args.push("--autosquash".into()),
            Some(false) => args.push("--no-autosquash".into()),
            None => {}
        }
        match self.auto_stash {
            Some(true) => args.push("--autostash".into()),
            Some(false) => args.push("--no-autostash".into()),
            None => {}
        }
        if let Some(empty) = self.empty {
            let val = match empty {
                EmptyAction::Drop => "drop",
                EmptyAction::Keep => "keep",
                EmptyAction::Ask => "ask",
                EmptyAction::Stop => "stop",
            };
            args.push(format!("--empty={val}").into());
        }
        if self.force_rebase {
            args.push("--force-rebase".into());
        }
        match self.fork_point {
            Some(true) => args.push("--fork-point".into()),
            Some(false) => args.push("--no-fork-point".into()),
            None => {}
        }
        if self.ignore_whitespace {
            args.push("--ignore-whitespace".into());
        }
        if let Some(ws) = self.whitespace {
            let val = match ws {
                WhitespaceAction::Apply => "apply",
                WhitespaceAction::Warn => "warn",
                WhitespaceAction::NoWarn => "nowarn",
                WhitespaceAction::Error => "error",
                WhitespaceAction::ErrorAll => "error-all",
                WhitespaceAction::Fix => "fix",
            };
            args.push(format!("--whitespace={val}").into());
        }
        if self.committer_date_is_author_date {
            args.push("--committer-date-is-author-date".into());
        }
        if self.reset_author_date {
            args.push("--reset-author-date".into());
        }
        if self.signoff {
            args.push("--signoff".into());
        }
        for t in &self.trailers {
            args.push("--trailer".into());
            args.push(t.into());
        }
        if let Some(gpg) = &self.gpg_sign {
            match gpg {
                Some(keyid) => args.push(format!("--gpg-sign={keyid}").into()),
                None => args.push("--gpg-sign".into()),
            }
        }
        if let Some(s) = &self.strategy {
            args.push("--strategy".into());
            args.push(s.into());
        }
        for opt in &self.strategy_options {
            args.push("--strategy-option".into());
            args.push(opt.into());
        }
        match self.rerere_autoupdate {
            Some(true) => args.push("--rerere-autoupdate".into()),
            Some(false) => args.push("--no-rerere-autoupdate".into()),
            None => {}
        }
        match self.reapply_cherry_picks {
            Some(true) => args.push("--reapply-cherry-picks".into()),
            Some(false) => args.push("--no-reapply-cherry-picks".into()),
            None => {}
        }
        if let Some(merges) = &self.rebase_merges {
            match merges {
                Some(RebaseMergesMode::RebaseCousins) => {
                    args.push("--rebase-merges=rebase-cousins".into())
                }
                Some(RebaseMergesMode::NoRebaseCousins) => {
                    args.push("--rebase-merges=no-rebase-cousins".into())
                }
                None => args.push("--rebase-merges".into()),
            }
        }
        match self.update_refs {
            Some(true) => args.push("--update-refs".into()),
            Some(false) => args.push("--no-update-refs".into()),
            None => {}
        }
        match self.reschedule_failed_exec {
            Some(true) => args.push("--reschedule-failed-exec".into()),
            Some(false) => args.push("--no-reschedule-failed-exec".into()),
            None => {}
        }
        if self.quiet {
            args.push("--quiet".into());
        }
        if self.verbose {
            args.push("--verbose".into());
        }
        match self.stat {
            Some(true) => args.push("--stat".into()),
            Some(false) => args.push("--no-stat".into()),
            None => {}
        }
        if self.no_verify {
            args.push("--no-verify".into());
        }
        if self.verify {
            args.push("--verify".into());
        }
        if self.apply {
            args.push("--apply".into());
        }
        if self.merge {
            args.push("--merge".into());
        }
        if let Some(c) = self.context_lines {
            args.push(format!("-C{c}").into());
        }

        if let Some(upstream) = &self.upstream {
            args.push(upstream.into());
        }
        if let Some(branch) = &self.branch {
            args.push(branch.into());
        }
    }
}

/// Options for `--show-current-patch[=(diff|raw)]`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ShowCurrentPatch {
    pub submode: Option<ShowCurrentPatchSubmode>,
}

impl ShowCurrentPatch {
    fn to_args(&self, args: &mut Vec<OsString>) {
        match self.submode {
            None => args.push("--show-current-patch".into()),
            Some(ShowCurrentPatchSubmode::Diff) => args.push("--show-current-patch=diff".into()),
            Some(ShowCurrentPatchSubmode::Raw) => args.push("--show-current-patch=raw".into()),
        }
    }
}

/// Main `git rebase` command enum.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum RebaseCmd {
    /// Start or execute a rebase operation.
    Start(Start),
    /// Restart the rebasing process after resolving a merge conflict (`--continue`).
    Continue,
    /// Abort the rebase operation and reset HEAD to the original branch (`--abort`).
    Abort,
    /// Restart the rebasing process by skipping the current patch (`--skip`).
    Skip,
    /// Abort the rebase operation but HEAD is not reset back to the original branch (`--quit`).
    Quit,
    /// Edit the todo list during an interactive rebase (`--edit-todo`).
    EditTodo,
    /// Show the current patch in an interactive rebase or during conflict resolution (`--show-current-patch`).
    ShowCurrentPatch(ShowCurrentPatch),
}

impl RebaseCmd {
    /// Create a standard rebase command onto the specified upstream branch or ref.
    pub fn new(upstream: impl Into<String>) -> Self {
        Self::Start(Start {
            upstream: Some(upstream.into()),
            ..Default::default()
        })
    }

    /// Create a rebase command with explicit `--onto <newbase>`.
    pub fn onto(newbase: impl Into<String>) -> Self {
        Self::Start(Start {
            onto: Some(newbase.into()),
            ..Default::default()
        })
    }

    /// Continue a rebase in progress after resolving conflicts (`--continue`).
    pub fn continue_() -> Self {
        Self::Continue
    }

    /// Abort a rebase in progress and reset HEAD (`--abort`).
    pub fn abort() -> Self {
        Self::Abort
    }

    /// Skip the current patch and continue (`--skip`).
    pub fn skip() -> Self {
        Self::Skip
    }

    /// Quit a rebase in progress without resetting HEAD (`--quit`).
    pub fn quit() -> Self {
        Self::Quit
    }

    /// Show the current patch being applied (`--show-current-patch`).
    pub fn show_current_patch() -> Self {
        Self::ShowCurrentPatch(ShowCurrentPatch::default())
    }
}

impl From<Start> for RebaseCmd {
    fn from(start: Start) -> Self {
        Self::Start(start)
    }
}

impl ToArgs for RebaseCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("rebase".into());

        match self {
            Self::Start(opts) => opts.to_args(args),
            Self::Continue => args.push("--continue".into()),
            Self::Abort => args.push("--abort".into()),
            Self::Skip => args.push("--skip".into()),
            Self::Quit => args.push("--quit".into()),
            Self::EditTodo => args.push("--edit-todo".into()),
            Self::ShowCurrentPatch(opts) => opts.to_args(args),
        }
    }
}

/// Output representation of a successful `git rebase` command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RebaseOutput {
    /// Rebase completed successfully.
    Success { message: String },
    /// Current branch was already up to date.
    UpToDate { message: String },
    /// Output from `--show-current-patch`.
    Patch(String),
    /// Action completed (`--abort`, `--continue`, `--skip`, `--quit`, `--edit-todo`).
    Done,
}

impl RebaseOutput {
    /// True if the output indicates that the branch was already up to date.
    pub fn is_up_to_date(&self) -> bool {
        matches!(self, Self::UpToDate { .. })
    }

    /// True if the rebase succeeded (either newly rebased or up to date).
    pub fn is_successful(&self) -> bool {
        matches!(
            self,
            Self::Success { .. } | Self::UpToDate { .. } | Self::Done
        )
    }
}

impl ParseOutput for RebaseCmd {
    type Output = RebaseOutput;

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}").trim().to_string();

        match self {
            Self::ShowCurrentPatch(_) => Ok(RebaseOutput::Patch(stdout.to_string())),
            Self::Continue | Self::Abort | Self::Skip | Self::Quit | Self::EditTodo => {
                Ok(RebaseOutput::Done)
            }
            Self::Start(_) => {
                let lower = combined.to_ascii_lowercase();
                if lower.contains("is up to date") || lower.contains("up-to-date") {
                    Ok(RebaseOutput::UpToDate { message: combined })
                } else {
                    Ok(RebaseOutput::Success { message: combined })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_args_vec(cmd: &RebaseCmd) -> Vec<String> {
        let mut args = Vec::new();
        cmd.to_args(&mut args);
        args.into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn rebase_new_upstream_args() {
        let cmd = RebaseCmd::new("main");
        let args = to_args_vec(&cmd);
        assert_eq!(args, vec!["rebase", "main"]);
    }

    #[test]
    fn rebase_onto_args() {
        let cmd = RebaseCmd::onto("main");
        let args = to_args_vec(&cmd);
        assert_eq!(args, vec!["rebase", "--onto", "main"]);
    }

    #[test]
    fn rebase_start_comprehensive_args() {
        let cmd = RebaseCmd::Start(Start {
            upstream: Some("origin/main".into()),
            branch: Some("feature".into()),
            onto: Some("master".into()),
            keep_base: true,
            root: true,
            interactive: true,
            exec: vec!["cargo check".into(), "cargo test".into()],
            auto_squash: Some(true),
            auto_stash: Some(false),
            empty: Some(EmptyAction::Drop),
            force_rebase: true,
            fork_point: Some(false),
            ignore_whitespace: true,
            whitespace: Some(WhitespaceAction::Fix),
            committer_date_is_author_date: true,
            reset_author_date: true,
            signoff: true,
            trailers: vec!["Reviewed-by: Dev".into()],
            gpg_sign: Some(Some("0x1234".into())),
            strategy: Some("ort".into()),
            strategy_options: vec!["ours".into()],
            rerere_autoupdate: Some(true),
            reapply_cherry_picks: Some(true),
            rebase_merges: Some(Some(RebaseMergesMode::RebaseCousins)),
            update_refs: Some(true),
            reschedule_failed_exec: Some(true),
            quiet: false,
            verbose: true,
            stat: Some(true),
            no_verify: true,
            verify: false,
            apply: true,
            merge: false,
            context_lines: Some(3),
        });

        let args = to_args_vec(&cmd);
        assert!(args.contains(&"--onto".to_string()));
        assert!(args.contains(&"master".to_string()));
        assert!(args.contains(&"--keep-base".to_string()));
        assert!(args.contains(&"--root".to_string()));
        assert!(args.contains(&"--interactive".to_string()));
        assert!(args.contains(&"--autosquash".to_string()));
        assert!(args.contains(&"--no-autostash".to_string()));
        assert!(args.contains(&"--empty=drop".to_string()));
        assert!(args.contains(&"--force-rebase".to_string()));
        assert!(args.contains(&"--no-fork-point".to_string()));
        assert!(args.contains(&"--ignore-whitespace".to_string()));
        assert!(args.contains(&"--whitespace=fix".to_string()));
        assert!(args.contains(&"--committer-date-is-author-date".to_string()));
        assert!(args.contains(&"--reset-author-date".to_string()));
        assert!(args.contains(&"--signoff".to_string()));
        assert!(args.contains(&"--trailer".to_string()));
        assert!(args.contains(&"Reviewed-by: Dev".to_string()));
        assert!(args.contains(&"--gpg-sign=0x1234".to_string()));
        assert!(args.contains(&"--strategy".to_string()));
        assert!(args.contains(&"ort".to_string()));
        assert!(args.contains(&"--strategy-option".to_string()));
        assert!(args.contains(&"ours".to_string()));
        assert!(args.contains(&"--rerere-autoupdate".to_string()));
        assert!(args.contains(&"--reapply-cherry-picks".to_string()));
        assert!(args.contains(&"--rebase-merges=rebase-cousins".to_string()));
        assert!(args.contains(&"--update-refs".to_string()));
        assert!(args.contains(&"--reschedule-failed-exec".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--stat".to_string()));
        assert!(args.contains(&"--no-verify".to_string()));
        assert!(args.contains(&"--apply".to_string()));
        assert!(args.contains(&"-C3".to_string()));
        assert_eq!(args.iter().rev().nth(1), Some(&"origin/main".to_string()));
        assert_eq!(args.iter().rev().next(), Some(&"feature".to_string()));
    }

    #[test]
    fn rebase_action_modes_args() {
        assert_eq!(
            to_args_vec(&RebaseCmd::Continue),
            vec!["rebase", "--continue"]
        );
        assert_eq!(to_args_vec(&RebaseCmd::Abort), vec!["rebase", "--abort"]);
        assert_eq!(to_args_vec(&RebaseCmd::Skip), vec!["rebase", "--skip"]);
        assert_eq!(to_args_vec(&RebaseCmd::Quit), vec!["rebase", "--quit"]);
        assert_eq!(
            to_args_vec(&RebaseCmd::EditTodo),
            vec!["rebase", "--edit-todo"]
        );
        assert_eq!(
            to_args_vec(&RebaseCmd::ShowCurrentPatch(ShowCurrentPatch::default())),
            vec!["rebase", "--show-current-patch"]
        );
        assert_eq!(
            to_args_vec(&RebaseCmd::ShowCurrentPatch(ShowCurrentPatch {
                submode: Some(ShowCurrentPatchSubmode::Diff),
            })),
            vec!["rebase", "--show-current-patch=diff"]
        );
        assert_eq!(
            to_args_vec(&RebaseCmd::ShowCurrentPatch(ShowCurrentPatch {
                submode: Some(ShowCurrentPatchSubmode::Raw),
            })),
            vec!["rebase", "--show-current-patch=raw"]
        );
    }
}
