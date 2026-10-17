// XIOM Compiler CLI -- clap surface (Stage 5)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The driver historically scanned `env::args()` ad hoc: booleans via
// `args.iter().any(|a| a == "--flag")`, values via
// `parse_flag_value(&args, "--flag")`, optional `--flag=value` spellings via
// `starts_with`. This module is now the parser of record: `parse` runs clap
// over argv and returns a `Cli` whose matches drive the flag reads in
// `main.rs`.
//
// Compatibility contract (byte-compatible CLI):
// - clap runs with `ignore_errors`, so unknown arguments keep being silently
//   ignored exactly as before;
// - help and version auto-flags are disabled -- `print_usage()` and the
//   `--version` arm own that output;
// - `Cli` derefs to the ORIGINAL argv, so the legacy scans that remain
//   (command words like `install`/`build`, `run`'s private mini-language,
//   and the `--flag=`-form-sensitive sandbox/graph checks) see the exact
//   vector they saw before;
// - `flag`/`value`/`values`/`present` read the clap matches; `raw_has`
//   covers the checks that intentionally distinguish `--flag=value` from
//   `--flag value`.
//
// m359 (REL-20261016-1128-engine class 4): `ignore_errors(true)` returns a
// PARTIAL match set after an argument conflict -- and those matches carry no
// SetTrue defaults. Reading a flag off them panicked inside clap
// (`arg 'emit-ir''s 'ArgAction' should be one of SetTrue/SetFalse`) for
// `xiom <file> --timeout 900 --timeout 900`. The accessors below never read
// default-less matches unguarded, and `duplicated_value_flag` detects the
// offending raw-argv shape so the driver can report a clean usage error.

use clap::{Arg, ArgAction, ArgMatches, Command};

/// Flags that take a required value (`--flag value` or `--flag=value`).
/// `--output` is the long form of the driver's `-o`.
const VALUE_FLAGS: &[&str] = &[
    "explain",
    "opt-level",
    "sanitize",
    "ai-model",
    "ai-timeout",
    "jobs",
    "count",
    // m214: --icon <path.ico> (Windows exe resource embedding).
    "icon",
    // m258: --cxx-standard <std> (C++ --c-source standard, e.g. 17).
    "cxx-standard",
    "registry",
    "verify-output",
    "timeout",
    "max-depth",
    "max-memory-mb",
    "target",
    "test-dir",
    "bench-file",
];

/// Value flags that may be repeated (`--link a --link b`).
const APPEND_VALUE_FLAGS: &[&str] = &["link", "link-path", "c-source"];

/// Flags that may appear with or without a value (`--sandbox`,
/// `--sandbox=strict`, `--graph`, `--graph=mermaid`, `--sandbox-report[=X]`).
const OPTIONAL_VALUE_FLAGS: &[&str] = &["sandbox", "sandbox-report", "graph"];

/// Boolean flags: the union of every `--flag` the driver reads, plus
/// dispatch words that must parse (`--clean`, `--doctor`, `--test`, `--doc`)
/// and `--`-prefixed flags appearing in forwarding/generated contexts
/// (`--locked`, `--frozen`, `--batch`, `--depth`, `--connect-timeout`).
const FLAG_FLAGS: &[&str] = &[
    "ai",
    "ai-batch",
    "ai-dry-run",
    "ai-local",
    "ai-silent",
    "ai-strict",
    "batch",
    "cache",
    "check",
    "clean",
    "connect-timeout",
    "debug",
    "deep",
    "depth",
    "doc",
    "doctor",
    "dump-contracts",
    "dump-ast",
    "dump-check",
    "dump-tokens",
    "emit-ir",
    "emit-tokens",
    "enable-unsafe-direct",
    "force",
    "frozen",
    "help",
    "help-ai",
    "hot-reload",
    "hot-reload-contracts",
    "incremental",
    "jit",
    "json",
    "keep-debug-checks",
    "lazy",
    "locked",
    "lto",
    "no-cache",
    "no-contracts",
    "overflow-checks",
    "parallel",
    "parallel-codegen",
    "release",
    "run",
    "runtime-contracts",
    "scaffold",
    "sequential",
    "shared",
    "stack-protector",
    "standalone",
    "static",
    "strict",
    "strict-exhaustive",
    "test",
    "verify",
    "version",
    "watch",
];

/// The complete driver command. Help/version output stays owned by
/// `print_usage()` / the `--version` arm in `main.rs`, so clap's automatic
/// flags are disabled.
pub fn command() -> Command {
    let mut cmd = Command::new("xiom")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .ignore_errors(true);
    for name in FLAG_FLAGS {
        let mut arg = Arg::new(*name).long(*name).action(ArgAction::SetTrue);
        // Website/docs spelling: `--strict-mode` is an accepted alias of the
        // historical `--strict` flag (same id, same behavior).
        if *name == "strict" {
            arg = arg.alias("strict-mode");
        }
        cmd = cmd.arg(arg);
    }
    for name in OPTIONAL_VALUE_FLAGS {
        cmd = cmd.arg(
            Arg::new(*name)
                .long(*name)
                .num_args(0..=1)
                .value_name("VALUE"),
        );
    }
    for name in VALUE_FLAGS {
        cmd = cmd.arg(
            Arg::new(*name)
                .long(*name)
                .num_args(1)
                .value_name("VALUE"),
        );
    }
    for name in APPEND_VALUE_FLAGS {
        cmd = cmd.arg(
            Arg::new(*name)
                .long(*name)
                .num_args(1)
                .action(ArgAction::Append)
                .value_name("VALUE"),
        );
    }
    cmd = cmd
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .num_args(1)
                .value_name("PATH"),
        )
        .arg(Arg::new("debug-symbols").short('g').action(ArgAction::SetTrue))
        .arg(Arg::new("sources").num_args(0..).value_name("SOURCE"));
    cmd
}

/// Parsed driver invocation. Derefs to the original argv for the legacy
/// scans (command words, `run`'s mini-language, exact-form checks).
pub struct Cli {
    raw: Vec<String>,
    matches: ArgMatches,
}

impl std::ops::Deref for Cli {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.raw
    }
}

impl Cli {
    /// `true` when a boolean flag was given.
    ///
    /// m359: `try_get_one` instead of `get_flag` -- a partial (default-less)
    /// match set must read as `false`, never panic.
    pub fn flag(&self, name: &str) -> bool {
        self.matches
            .try_get_one::<bool>(name)
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false)
    }

    /// First value of a single-value flag, `--flag value` or `--flag=value`.
    ///
    /// m359: `try_get_one` keeps a definition/access mismatch or a missing
    /// partial-match entry from panicking; both read as "not given".
    pub fn value(&self, name: &str) -> Option<String> {
        self.matches
            .try_get_one::<String>(name)
            .ok()
            .flatten()
            .cloned()
    }

    /// All values of a repeatable flag, in command-line order.
    pub fn values(&self, name: &str) -> Vec<String> {
        self.matches
            .try_get_many::<String>(name)
            .ok()
            .flatten()
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default()
    }

    /// `true` when the flag appeared on the command line, including
    /// optional-value flags given without a value.
    pub fn present(&self, name: &str) -> bool {
        self.matches.value_source(name).is_some()
    }

    /// Exact raw-token check, for the checks that intentionally distinguish
    /// `--flag=value` from `--flag value` (sandbox/graph/diagnostics forms).
    pub fn raw_has(&self, exact: &str) -> bool {
        self.raw.iter().any(|a| a == exact)
    }

    /// The original argv.
    pub fn raw(&self) -> &[String] {
        &self.raw
    }
}

/// Parse `argv` with clap and return the parsed view. Errors are
/// intentionally swallowed by `ignore_errors`: the historical parser
/// silently ignored unknown flags, and the CLI surface must stay
/// byte-compatible.
pub fn parse(argv: Vec<String>) -> Cli {
    let matches = command()
        .try_get_matches_from(&argv)
        .unwrap_or_else(|_| command().get_matches_from(["xiom"]));
    Cli { raw: argv, matches }
}

/// m359: the single-value flag id (the long spelling) duplicated after the
/// first positional, if any.
///
/// clap treats a repeated `Set` flag as an argument conflict. With
/// `ignore_errors(true)` that conflict is swallowed, but the returned matches
/// are partial and default-less, and every driver flag read used to panic
/// (engine repro: `xiom <file> --timeout 900 --timeout 900`, rc 101).
///
/// Detection is raw-argv based because the partial matches cannot name the
/// offending argument. Only occurrences AFTER the first positional are
/// reported: the pre-positional duplicate shape (`xiom --timeout 900
/// --timeout 900 <file>`) predates the clap surface and stayed accepted (the
/// repeated value is simply not read), so it keeps those semantics.
pub fn duplicated_value_flag(argv: &[String]) -> Option<String> {
    let mut positional_seen = false;
    let mut seen: Vec<&'static str> = Vec::new();
    let mut i = 1;
    while i < argv.len() {
        let tok = &argv[i];
        if tok == "--" {
            // Everything after `--` is a positional value, never a flag.
            break;
        }
        if let Some(rest) = tok.strip_prefix("--") {
            let (name, inline_value) = match rest.split_once('=') {
                Some((name, _)) => (name, true),
                None => (rest, false),
            };
            if name == "output" {
                if positional_seen && seen.contains(&"output") {
                    return Some("--output".to_string());
                }
                seen.push("output");
                i += if inline_value { 1 } else { 2 };
                continue;
            }
            if let Some(flag) = VALUE_FLAGS.iter().find(|flag| **flag == name) {
                if positional_seen && seen.contains(flag) {
                    return Some(format!("--{}", *flag));
                }
                seen.push(*flag);
                i += if inline_value { 1 } else { 2 };
                continue;
            }
            if OPTIONAL_VALUE_FLAGS.contains(&name) && !inline_value {
                // A bare optional-value flag consumes a following non-flag
                // token (clap `num_args(0..=1)`), so that token is NOT the
                // positional boundary.
                i += if i + 1 < argv.len() && !argv[i + 1].starts_with('-') {
                    2
                } else {
                    1
                };
                continue;
            }
            i += 1;
            continue;
        }
        if tok == "-o" {
            if positional_seen && seen.contains(&"output") {
                return Some("--output".to_string());
            }
            seen.push("output");
            i += 2;
            continue;
        }
        if tok.starts_with('-') && tok.len() > 1 {
            // Boolean, optional-value (short) or unknown flag: no source
            // boundary.
            i += 1;
            continue;
        }
        positional_seen = true;
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        std::iter::once("xiom")
            .chain(parts.iter().copied())
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn command_defines_the_full_surface() {
        let cmd = command();
        for name in VALUE_FLAGS
            .iter()
            .chain(APPEND_VALUE_FLAGS)
            .chain(OPTIONAL_VALUE_FLAGS)
            .chain(FLAG_FLAGS)
        {
            assert!(
                cmd.get_arguments().any(|a| a.get_id() == *name),
                "flag --{name} missing from the clap surface"
            );
        }
        assert!(cmd.get_arguments().any(|a| a.get_id() == "output"));
        assert!(cmd.get_arguments().any(|a| a.get_id() == "debug-symbols"));
        assert!(cmd.get_arguments().any(|a| a.get_id() == "sources"));
    }

    #[test]
    fn representative_invocations_parse() {
        let cases: &[&[&str]] = &[
            &["file.xi"],
            &["--emit-ir", "file.xi"],
            &["--dump-tokens", "file.xi"],
            &["--dump-ast", "file.xi"],
            &["--dump-check", "file.xi"],
            &["--check", "file.xi"],
            &["-o", "out.exe", "file.xi"],
            &["--sanitize=address", "-o", "out.exe", "file.xi"],
            &["--sanitize", "address", "file.xi"],
            &["--sandbox=strict", "file.xi"],
            &["--sandbox-report=json", "file.xi"],
            &["--sandbox-report", "report.json", "file.xi"],
            &["--graph=mermaid", "file.xi"],
            &["--target", "wasm", "file.xi"],
            &["--opt-level", "0", "--release", "file.xi"],
            &["--runtime-contracts", "--keep-debug-checks", "file.xi"],
            &["--enable-unsafe-direct", "file.xi"],
            &["run", "-e", "1 + 1"],
            &["run", "--watch", "file.xi"],
            &["build-runtime"],
            &["repl"],
            &["--explain", "T001"],
            &["--version"],
            &["--help"],
            // Unknown flags are tolerated by contract.
            &["--definitely-not-a-flag", "file.xi"],
        ];
        for case in cases {
            let parsed = command().try_get_matches_from(argv(case));
            assert!(parsed.is_ok(), "argv {case:?} must parse leniently: {parsed:?}");
        }
    }

    #[test]
    fn parse_returns_the_original_vector() {
        let input = argv(&["--emit-ir", "--sanitize=address", "file.xi"]);
        assert_eq!(parse(input.clone()).raw(), input.as_slice());
    }

    #[test]
    fn matches_drive_flag_value_reads() {
        let cli = parse(argv(&[
            "--emit-ir",
            "--sanitize=address",
            "--jobs",
            "4",
            "--link",
            "a",
            "--link",
            "b",
            "--icon",
            "app.ico",
            "--cxx-standard",
            "17",
            "file.xi",
        ]));
        assert!(cli.flag("emit-ir"));
        assert!(!cli.flag("release"));
        assert_eq!(cli.value("sanitize").as_deref(), Some("address"));
        assert_eq!(cli.value("jobs").as_deref(), Some("4"));
        assert_eq!(cli.value("icon").as_deref(), Some("app.ico"));
        assert_eq!(cli.value("cxx-standard").as_deref(), Some("17"));
        assert_eq!(cli.values("link"), vec!["a".to_string(), "b".to_string()]);
        assert!(!cli.present("sandbox"));
        assert!(cli.raw_has("--sanitize=address"));
        // The raw view keeps the legacy scans working.
        assert!(cli.iter().any(|a| a == "file.xi"));
    }

    /// Docs spelling `--strict-mode` must select the same flag as `--strict`.
    #[test]
    fn strict_mode_alias_sets_strict() {
        assert!(parse(argv(&["--strict-mode", "f.xi"])).flag("strict"));
        assert!(parse(argv(&["--strict", "f.xi"])).flag("strict"));
    }

    #[test]
    fn optional_value_flags_report_presence() {
        assert!(parse(argv(&["--sandbox", "f.xi"])).present("sandbox"));
        assert!(parse(argv(&["--sandbox=strict", "f.xi"])).present("sandbox"));
        assert!(parse(argv(&["--graph", "f.xi"])).present("graph"));
        assert!(!parse(argv(&["f.xi"])).present("sandbox"));
    }

    /// m359: clap `ignore_errors` partial matches have no SetTrue defaults.
    /// Every accessor must read them as "not given" instead of panicking.
    #[test]
    fn defaultless_matches_read_as_defaults() {
        let raw = argv(&["f.xi", "--timeout", "900", "--timeout", "900"]);
        let partial = command()
            .try_get_matches_from(&raw)
            .expect("ignore_errors swallows the conflict and returns partial matches");
        assert!(
            !partial.contains_id("emit-ir"),
            "the repro must reach the default-less match shape"
        );
        let cli = Cli {
            raw: raw.clone(),
            matches: partial,
        };
        assert!(!cli.flag("emit-ir"));
        assert!(!cli.flag("does-not-exist"));
        assert_eq!(cli.value("timeout"), None);
        assert!(cli.values("link").is_empty());
        assert!(!cli.present("sandbox"));
    }

    /// m359: duplicates AFTER the first positional are the panicking shape and
    /// must be reported; duplicates BEFORE it keep the legacy accepted shape.
    #[test]
    fn duplicate_value_flags_are_detected_only_after_a_positional() {
        // Engine repro shapes.
        assert_eq!(
            duplicated_value_flag(&argv(&["f.xi", "--timeout", "900", "--timeout", "900"])),
            Some("--timeout".to_string())
        );
        assert_eq!(
            duplicated_value_flag(&argv(&["f.xi", "--jobs", "4", "--jobs", "4"])),
            Some("--jobs".to_string())
        );
        // `=` spelling and mixed spellings.
        assert_eq!(
            duplicated_value_flag(&argv(&["f.xi", "--timeout=900", "--timeout", "900"])),
            Some("--timeout".to_string())
        );
        // Legacy pre-positional duplicates stay accepted.
        assert_eq!(
            duplicated_value_flag(&argv(&["--timeout", "900", "--timeout", "900", "f.xi"])),
            None
        );
        assert_eq!(
            duplicated_value_flag(&argv(&["--jobs", "4", "--jobs", "4", "f.xi"])),
            None
        );
        // Append-style flags are repeatable by design.
        assert_eq!(
            duplicated_value_flag(&argv(&["f.xi", "--link", "a", "--link", "b"])),
            None
        );
        // `--` ends flag parsing.
        assert_eq!(
            duplicated_value_flag(&argv(&["f.xi", "--", "--timeout", "9", "--timeout", "9"])),
            None
        );
        // `-o` is the short spelling of --output.
        assert_eq!(
            duplicated_value_flag(&argv(&["f.xi", "-o", "a", "-o", "b"])),
            Some("--output".to_string())
        );
        // An optional-value flag consumes the next non-flag token, so that
        // token is NOT the positional boundary.
        assert_eq!(
            duplicated_value_flag(&argv(&[
                "--sandbox",
                "strict",
                "--timeout",
                "9",
                "--timeout",
                "9"
            ])),
            None
        );
        confirm_pre_positional_conflict_shape();
    }

    /// Cross-check the premise of the "legacy accepted" assertions above: the
    /// pre-positional duplicate really does reach clap as a swallowed conflict
    /// (not as a clean parse), yet it predates the clap surface, so it must
    /// keep compiling instead of turning into a usage error.
    fn confirm_pre_positional_conflict_shape() {
        let strict = command().ignore_errors(false).try_get_matches_from(argv(&[
            "--timeout", "900", "--timeout", "900", "f.xi",
        ]));
        assert!(strict.is_err(), "clap must see the duplicate conflict");
        let ignored = command().try_get_matches_from(argv(&[
            "--timeout", "900", "--timeout", "900", "f.xi",
        ]));
        assert!(ignored.is_ok(), "ignore_errors must keep it accepted");
    }
}
