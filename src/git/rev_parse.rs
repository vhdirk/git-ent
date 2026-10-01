use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Output;

use crate::error::Result;
use crate::git::{ParseOutput, ToArgs};

#[derive(Default)]
pub struct ParseOpt {
    pub keep_dashdash: bool,
    pub stop_at_non_option: bool,
    pub stuck_long: bool,
    /// The usage/option spec, written to stdin.
    pub spec: String,
    /// Arguments after the `--` separator.
    pub args: Vec<String>,
}

#[derive(Default)]
pub struct Verify {
    /// `None` only makes sense together with `default`.
    pub rev: Option<String>,
    pub default: Option<String>,
    pub abbreviation: Abbreviation,
    pub quiet: bool, // -q: only meaningful here
    pub name_style: NameStyle,
    pub object_format: Option<ObjectFormat>,
}

#[derive(Default)]
pub enum Abbreviation {
    #[default]
    Full, // plain --verify
    Short(Option<u8>), // --short[=<length>], minimum 4
}

#[derive(Default)]
pub struct Parse {
    pub filter: Option<Filter>,
    pub sq: bool, // --sq: single quoted line
    pub default: Option<String>,
    pub prefix: Option<String>,
    pub not: bool, // --not
    pub name_style: NameStyle,
    pub object_format: Option<ObjectFormat>,
    /// Order matters: --exclude, --path-format etc. affect later items.
    pub items: Vec<Item>,
}

pub enum Filter {
    RevsOnly,
    NoRevs,
    Flags,
    NoFlags,
}

#[derive(Default)]
pub enum NameStyle {
    #[default]
    Default,
    Symbolic,                      // --symbolic
    SymbolicFullName,              // --symbolic-full-name
    AbbrevRef(Option<AbbrevMode>), // --abbrev-ref[=strict|loose]
}

pub enum AbbrevMode {
    Strict,
    Loose,
}
pub enum ObjectFormat {
    Sha1,
    Sha256,
    Storage,
}

pub enum Item {
    Rev(String),
    Refs(RefSelector),
    Disambiguate(String),   // --disambiguate=<prefix>
    Since(String),          // --since / --after  -> --max-age
    Until(String),          // --until / --before -> --min-age
    PathFormat(PathFormat), // affects the following items
    Query(RepoQuery),
    LocalEnvVars,
}

#[derive(Default)]
pub struct RefSelector {
    pub kind: RefKind,
    /// Accumulate up to the next selector.
    pub excludes: Vec<String>,
    /// Only applies to `All` and `Glob`.
    pub exclude_hidden: Option<HiddenRefs>,
}

#[derive(Default)]
pub enum RefKind {
    #[default]
    All,
    Branches(Option<String>),
    Tags(Option<String>),
    Remotes(Option<String>),
    Glob(String),
}

pub enum HiddenRefs {
    Fetch,
    Receive,
    UploadPack,
}
pub enum PathFormat {
    Absolute,
    Relative,
}

pub enum RepoQuery {
    // Affected by --path-format
    GitDir,
    GitCommonDir,
    ResolveGitDir(PathBuf),
    GitPath(PathBuf),
    ShowToplevel,
    ShowSuperprojectWorkingTree,
    SharedIndexPath,
    // Unaffected by --path-format
    AbsoluteGitDir,
    IsInsideGitDir,
    IsInsideWorkTree,
    IsBareRepository,
    IsShallowRepository,
    ShowCdup,
    ShowPrefix,
    ShowObjectFormat(Option<ObjectFormatKind>),
    ShowRefFormat,
}

pub enum ObjectFormatKind {
    Storage,
    Input,
    Output,
    Compat,
}

#[derive(Debug, Clone)]
pub enum RevParseOutput {
    /// Shell snippets or single text blocks (`--parseopt`, `--sq-quote`)
    Text(String),
    /// Multiple lines of hashes or references
    Lines(Vec<String>),
    /// Filesystem paths (`--show-toplevel`, `--git-dir`, etc.)
    Path(PathBuf),
    /// Boolean repository checks (`--is-inside-work-tree`, etc.)
    Bool(bool),
}

pub enum RevParseCmd {
    /// `--parseopt`: option normalizer for shell scripts (spec goes on stdin).
    ParseOpt(ParseOpt),
    /// `--sq-quote`: shell-quote the arguments, nothing else.
    SqQuote { args: Vec<String> },
    /// `--verify` / `--short`: exactly one revision to a single object name.
    Verify(Verify),
    /// Default mode: ordered mix of revisions, ref selectors and repo queries.
    Parse(Parse),
}

impl RevParseCmd {
    /// `--parseopt` and `--sq-quote` work outside a repository.
    pub fn requires_repo(&self) -> bool {
        !matches!(self, Self::ParseOpt(_) | Self::SqQuote { .. })
    }
}

impl ToArgs for NameStyle {
    fn to_args(&self, args: &mut Vec<OsString>) {
        match self {
            NameStyle::Default => {}
            NameStyle::Symbolic => args.push("--symbolic".into()),
            NameStyle::SymbolicFullName => args.push("--symbolic-full-name".into()),
            NameStyle::AbbrevRef(None) => args.push("--abbrev-ref".into()),
            NameStyle::AbbrevRef(Some(AbbrevMode::Strict)) => {
                args.push("--abbrev-ref=strict".into())
            }
            NameStyle::AbbrevRef(Some(AbbrevMode::Loose)) => args.push("--abbrev-ref=loose".into()),
        }
    }
}

impl ToArgs for Option<ObjectFormat> {
    fn to_args(&self, args: &mut Vec<OsString>) {
        if let Some(f) = self {
            args.push(
                format!(
                    "--output-object-format={}",
                    match f {
                        ObjectFormat::Sha1 => "sha1",
                        ObjectFormat::Sha256 => "sha256",
                        ObjectFormat::Storage => "storage",
                    }
                )
                .into(),
            );
        }
    }
}

impl ToArgs for Item {
    fn to_args(&self, args: &mut Vec<OsString>) {
        match self {
            Item::Rev(r) => args.push(r.into()),
            Item::Disambiguate(p) => args.push(format!("--disambiguate={p}").into()),
            Item::Since(d) => args.push(format!("--since={d}").into()),
            Item::Until(d) => args.push(format!("--until={d}").into()),
            Item::LocalEnvVars => args.push("--local-env-vars".into()),
            Item::PathFormat(f) => args.push(
                format!(
                    "--path-format={}",
                    match f {
                        PathFormat::Absolute => "absolute",
                        PathFormat::Relative => "relative",
                    }
                )
                .into(),
            ),
            Item::Refs(sel) => {
                // Excludes must precede the selector they apply to.
                for ex in &sel.excludes {
                    args.push(format!("--exclude={ex}").into());
                }
                if let Some(h) = &sel.exclude_hidden {
                    args.push(
                        format!(
                            "--exclude-hidden={}",
                            match h {
                                HiddenRefs::Fetch => "fetch",
                                HiddenRefs::Receive => "receive",
                                HiddenRefs::UploadPack => "uploadpack",
                            }
                        )
                        .into(),
                    );
                }
                let with = |name: &str, pat: &Option<String>| match pat {
                    Some(p) => format!("--{name}={p}"),
                    None => format!("--{name}"),
                };
                args.push(
                    match &sel.kind {
                        RefKind::All => "--all".to_string(),
                        RefKind::Branches(p) => with("branches", p),
                        RefKind::Tags(p) => with("tags", p),
                        RefKind::Remotes(p) => with("remotes", p),
                        RefKind::Glob(p) => format!("--glob={p}"),
                    }
                    .into(),
                );
            }
            Item::Query(q) => match q {
                RepoQuery::GitDir => args.push("--git-dir".into()),
                RepoQuery::GitCommonDir => args.push("--git-common-dir".into()),
                RepoQuery::ResolveGitDir(p) => args.extend(["--resolve-git-dir".into(), p.into()]),
                RepoQuery::GitPath(p) => args.extend(["--git-path".into(), p.into()]),
                RepoQuery::ShowToplevel => args.push("--show-toplevel".into()),
                RepoQuery::ShowSuperprojectWorkingTree => {
                    args.push("--show-superproject-working-tree".into())
                }
                RepoQuery::SharedIndexPath => args.push("--shared-index-path".into()),
                RepoQuery::AbsoluteGitDir => args.push("--absolute-git-dir".into()),
                RepoQuery::IsInsideGitDir => args.push("--is-inside-git-dir".into()),
                RepoQuery::IsInsideWorkTree => args.push("--is-inside-work-tree".into()),
                RepoQuery::IsBareRepository => args.push("--is-bare-repository".into()),
                RepoQuery::IsShallowRepository => args.push("--is-shallow-repository".into()),
                RepoQuery::ShowCdup => args.push("--show-cdup".into()),
                RepoQuery::ShowPrefix => args.push("--show-prefix".into()),
                RepoQuery::ShowRefFormat => args.push("--show-ref-format".into()),
                RepoQuery::ShowObjectFormat(k) => args.push(
                    match k {
                        None => "--show-object-format".to_string(),
                        Some(k) => format!(
                            "--show-object-format={}",
                            match k {
                                ObjectFormatKind::Storage => "storage",
                                ObjectFormatKind::Input => "input",
                                ObjectFormatKind::Output => "output",
                                ObjectFormatKind::Compat => "compat",
                            }
                        ),
                    }
                    .into(),
                ),
            },
        }
    }
}

impl ToArgs for RevParseCmd {
    fn to_args(&self, args: &mut Vec<OsString>) {
        args.push("rev-parse".into());
        match self {
            RevParseCmd::SqQuote { args: sq_args } => {
                args.push("--sq-quote".into());
                args.extend(sq_args.iter().map(OsString::from));
            }
            RevParseCmd::ParseOpt(o) => {
                args.push("--parseopt".into());
                if o.keep_dashdash {
                    args.push("--keep-dashdash".into());
                }
                if o.stop_at_non_option {
                    args.push("--stop-at-non-option".into());
                }
                if o.stuck_long {
                    args.push("--stuck-long".into());
                }
                args.push("--".into());
                args.extend(o.args.iter().map(OsString::from));
            }
            RevParseCmd::Verify(v) => {
                match v.abbreviation {
                    Abbreviation::Full => args.push("--verify".into()),
                    Abbreviation::Short(None) => args.push("--short".into()),
                    Abbreviation::Short(Some(n)) => args.push(format!("--short={n}").into()),
                }
                if v.quiet {
                    args.push("--quiet".into());
                }
                if let Some(d) = &v.default {
                    args.extend(["--default".into(), d.into()]);
                }
                v.name_style.to_args(args);
                v.object_format.to_args(args);

                if let Some(rev) = &v.rev {
                    // Guards against an untrusted name being read as an option.
                    args.push("--end-of-options".into());
                    args.push(rev.into());
                }
            }
            RevParseCmd::Parse(p) => {
                args.extend(p.filter.as_ref().map(|f| {
                    OsString::from(match f {
                        Filter::RevsOnly => "--revs-only",
                        Filter::NoRevs => "--no-revs",
                        Filter::Flags => "--flags",
                        Filter::NoFlags => "--no-flags",
                    })
                }));
                if p.sq {
                    args.push("--sq".into());
                }
                if p.not {
                    args.push("--not".into());
                }
                if let Some(d) = &p.default {
                    args.extend(["--default".into(), d.into()]);
                }
                if let Some(x) = &p.prefix {
                    args.extend(["--prefix".into(), x.into()]);
                }

                p.name_style.to_args(args);
                p.object_format.to_args(args);
                for item in &p.items {
                    item.to_args(args);
                }
            }
        }
        // No need to return anything as we're modifying the args vector directly
    }
}

impl ParseOutput for RevParseCmd {
    type Output = RevParseOutput;

    fn allow_failure(&self) -> bool {
        matches!(self, Self::Verify(_))
    }

    fn parse_output(&self, output: &Output) -> Result<Self::Output> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();

        match self {
            RevParseCmd::SqQuote { .. } | RevParseCmd::ParseOpt(_) => {
                Ok(RevParseOutput::Text(stdout.to_string()))
            }
            RevParseCmd::Verify(_) => {
                // Verify returns a single hash or ref name
                Ok(RevParseOutput::Text(trimmed.to_string()))
            }
            RevParseCmd::Parse(p) => {
                // Check if any repo queries dictate a specific output type
                if let Some(Item::Query(q)) = p.items.first() {
                    match q {
                        RepoQuery::IsInsideWorkTree
                        | RepoQuery::IsInsideGitDir
                        | RepoQuery::IsBareRepository
                        | RepoQuery::IsShallowRepository => {
                            let val = trimmed.eq_ignore_ascii_case("true");
                            return Ok(RevParseOutput::Bool(val));
                        }
                        RepoQuery::GitDir
                        | RepoQuery::GitCommonDir
                        | RepoQuery::ResolveGitDir(_)
                        | RepoQuery::GitPath(_)
                        | RepoQuery::ShowToplevel
                        | RepoQuery::ShowSuperprojectWorkingTree
                        | RepoQuery::SharedIndexPath
                        | RepoQuery::AbsoluteGitDir
                        | RepoQuery::ShowCdup
                        | RepoQuery::ShowPrefix => {
                            return Ok(RevParseOutput::Path(PathBuf::from(trimmed)));
                        }
                        _ => {}
                    }
                }

                // Default fallback for general parsing / listing items: split into lines
                let lines = trimmed
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| l.to_string())
                    .collect();

                Ok(RevParseOutput::Lines(lines))
            }
        }
    }
}
