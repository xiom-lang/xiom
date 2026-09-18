// XIOM Compiler CLI -- clap surface (Stage 5)
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// The driver historically scanned `env::args()` ad hoc: booleans via
// `args.iter().any(|a| a == "--flag")`, values via
// `parse_flag_value(&args, "--flag")`, optional `--flag=value` spellings via
// `starts_with`. That surface is now DEFINED here with clap: one
// authoritative table of every flag, its arity and short aliases.
//
// Compatibility contract (byte-compatible CLI): `parse` runs clap in
// lenient mode (`ignore_errors`) so unknown arguments keep being ignored and
// missing values for optional flags keep behaving as before, and it returns
// the ORIGINAL vector unchanged -- every legacy scan in `main.rs` sees
// exactly the argv it saw before. This is migration step 1 of 2; step 2
// (replacing the scan reads with the parsed matches) can proceed
// flag-by-flag without touching the surface definition.

use clap::{Arg, ArgAction, Command};

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
    "registry",
    "verify-output",
    "link-path",
    "c-source",
    "timeout",
    "max-depth",
    "max-memory-mb",
    "target",
    "test-dir",
    "bench-file",
];

/// Flags that may appear with or without a value (`--sandbox`,
/// `--sandbox=strict`, `--graph`, `--graph=mermaid`, `--sandbox-report[=X]`).
const OPTIONAL_VALUE_FLAGS: &[&str] = &["sandbox", "sandbox-report", "graph"];

/// Boolean flags. This list is the union of every `--flag` the driver reads,
/// plus dispatch words that must parse (`--clean`, `--doctor`, `--test`,
/// `--doc`) and the `--`-prefixed flags forwarded to external tooling in
/// generated contexts (`--locked`, `--frozen`, `--link`, `--batch`,
/// `--depth`, `--connect-timeout`).
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
    "depth",
    "doc",
    "doctor",
    "dump-contracts",
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
    "keep-debug-checks",
    "lazy",
    "link",
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
        cmd = cmd.arg(Arg::new(*name).long(*name).action(ArgAction::SetTrue));
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

/// Parse `argv` with clap for validation/metadata, then return it UNCHANGED
/// (see the module contract). Errors are intentionally swallowed by
/// `ignore_errors`: the historical parser silently ignored unknown flags, and
/// the CLI surface must stay byte-compatible.
pub fn parse(argv: Vec<String>) -> Vec<String> {
    let _ = command().try_get_matches_from(&argv);
    argv
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
        assert_eq!(parse(input.clone()), input);
    }
}
