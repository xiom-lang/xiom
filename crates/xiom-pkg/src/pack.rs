// XIOM -- Package Manager (publish pack guard)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// REL-20261010-1548-registry-2: `xiom pkg publish` refuses build/artifact
// entries by default. The exact refusal list is the publishing guide's
// (PUBLISHING.md section 0): directories target/, build/, dist/, out/,
// .git/, .toolchain/, .xiom_ai_cache/, node_modules/ and files *.exe,
// *.o, *.obj, *.a, *.lib, *.dll, *.so, *.dylib, *.wasm, *.pdb,
// .DS_Store, Thumbs.db. `.xiomignore` (gitignore syntax) extends the
// list and can suppress a default-refused entry; entries explicitly
// declared in package.xi (the `modules:` list, plus package.xi itself) are
// kept; `--force` overrides refusals for intentional non-ignored cases.
// Module collection (the pack walk) uses the same
// filter -- there is one collector, so a module cannot be dropped while
// its stale neighbor rides along.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Directory NAMES refused at any depth (the guide's list, verbatim).
pub const REFUSED_DIRS: &[&str] = &[
    "target",
    "build",
    "dist",
    "out",
    ".git",
    ".toolchain",
    ".xiom_ai_cache",
    "node_modules",
];

/// File NAMES refused at any depth.
pub const REFUSED_FILES: &[&str] = &[".DS_Store", "Thumbs.db"];

/// File EXTENSIONS refused at any depth (case-insensitive).
pub const REFUSED_EXTS: &[&str] = &[
    "exe", "o", "obj", "a", "lib", "dll", "so", "dylib", "wasm", "pdb",
];

/// What happens to one entry during the pack walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Pack it.
    Keep,
    /// `.xiomignore` matched: leave it out (its subtree too, for dirs).
    Skip,
    /// A default build/artifact refusal: fail the pack unless --force.
    Refuse,
}

/// True when the entry matches the default refusal list.
pub fn default_refused(rel: &str, is_dir: bool) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    if is_dir {
        return REFUSED_DIRS.contains(&name);
    }
    if REFUSED_FILES.contains(&name) {
        return true;
    }
    match name.rsplit_once('.') {
        Some((_, ext)) => REFUSED_EXTS.iter().any(|r| ext.eq_ignore_ascii_case(r)),
        None => false,
    }
}

// ============================================================================
// .xiomignore -- gitignore-syntax subset
// ============================================================================

#[derive(Debug, Clone)]
struct Rule {
    negated: bool,
    dir_only: bool,
    anchored: bool,
    pattern: String,
}

/// A parsed `.xiomignore`: gitignore syntax (blank lines and `#` comments
/// skipped, `!` negation, trailing `/` for directories, leading `/` anchors,
/// `*`/`?` within a segment, `**` across segments). Last matching rule wins,
/// so `!` can re-include a path a broad pattern excluded.
#[derive(Debug, Clone, Default)]
pub struct XiomIgnore {
    rules: Vec<Rule>,
}

impl XiomIgnore {
    pub fn parse(text: &str) -> Self {
        let mut rules = Vec::new();
        for raw in text.lines() {
            let line = raw.trim_end();
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let (negated, rest) = match line.strip_prefix('!') {
                Some(r) => (true, r),
                None => (false, line),
            };
            let (dir_only, rest) = match rest.strip_suffix('/') {
                Some(r) => (true, r),
                None => (false, rest),
            };
            let (anchored, rest) = match rest.strip_prefix('/') {
                Some(r) => (true, r),
                None => (false, rest),
            };
            let pattern = rest.trim().to_string();
            if pattern.is_empty() {
                continue;
            }
            rules.push(Rule { negated, dir_only, anchored, pattern });
        }
        Self { rules }
    }

    /// gitignore-style match: the LAST matching rule decides; a `!` rule
    /// re-includes. Directory-only rules never match files.
    pub fn matches(&self, rel: &str, is_dir: bool) -> bool {
        let mut hit = false;
        for rule in &self.rules {
            if rule.dir_only && !is_dir {
                continue;
            }
            let matched = if rule.anchored || rule.pattern.contains('/') {
                glob_path_match(&rule.pattern, rel)
            } else {
                // No slash: the basename (at any depth) must match.
                rel.split('/').any(|seg| glob_segment_match(&rule.pattern, seg))
            };
            if matched {
                hit = !rule.negated;
            }
        }
        hit
    }
}

/// `*` and `?` within one path segment (never crossing '/').
fn glob_segment_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut mark = 0usize;
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            pi += 1;
            mark = ti;
        } else if let Some(sp) = star {
            pi = sp + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Full-path glob where `**` matches zero or more whole segments.
fn glob_path_match(pattern: &str, path: &str) -> bool {
    let ps: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let ts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    fn rec(ps: &[&str], ts: &[&str]) -> bool {
        match ps.first() {
            None => ts.is_empty(),
            Some(&"**") => {
                rec(&ps[1..], ts) || (!ts.is_empty() && rec(ps, &ts[1..]))
            }
            Some(&seg) => match ts.first() {
                None => false,
                Some(&text) => glob_segment_match(seg, text) && rec(&ps[1..], &ts[1..]),
            },
        }
    }
    rec(&ps, &ts)
}

// ============================================================================
// PackPolicy -- the per-publish decision maker
// ============================================================================

/// The publish policy: the refusal list, `.xiomignore`, the declared-module
/// keep set, and the `--force` override.
#[derive(Debug, Clone)]
pub struct PackPolicy {
    pub force: bool,
    keep: BTreeSet<String>,
    ignore: XiomIgnore,
}

impl PackPolicy {
    /// Load the policy for a package directory: `.xiomignore` when present,
    /// the `modules:` list as declared keeps, and `package.xi` itself.
    pub fn for_package(root: &Path, modules: &[String], force: bool) -> Self {
        let ignore = fs::read_to_string(root.join(".xiomignore"))
            .map(|text| XiomIgnore::parse(&text))
            .unwrap_or_default();
        let mut keep = BTreeSet::new();
        keep.insert("package.xi".to_string());
        for module in modules {
            let normalized = module.trim().replace('\\', "/");
            let normalized = normalized.trim_start_matches("./").trim_start_matches('/');
            if !normalized.is_empty() {
                keep.insert(normalized.to_string());
            }
        }
        Self { force, keep, ignore }
    }

    pub fn is_forced(&self) -> bool {
        self.force
    }

    fn keeps_descendant(&self, rel: &str) -> bool {
        let prefix = format!("{rel}/");
        self.keep.iter().any(|k| k.starts_with(&prefix))
    }

    /// Decide one entry's fate. Declared entries win; `.xiomignore` is next
    /// (it suppresses a default-refused entry too, which is the documented
    /// escape and works under `--force`); the default refusal list is last
    /// (with `--force` overriding only non-ignored entries).
    pub fn verdict(&self, rel: &str, is_dir: bool) -> Verdict {
        if self.keep.contains(rel) || (is_dir && self.keeps_descendant(rel)) {
            return Verdict::Keep;
        }
        if self.ignore.matches(rel, is_dir) {
            return Verdict::Skip;
        }
        if default_refused(rel, is_dir) {
            return if self.force { Verdict::Keep } else { Verdict::Refuse };
        }
        Verdict::Keep
    }
}

/// Human message when the pack is refused: the offending entries (capped)
/// and the three escapes (clean checkout, .xiomignore, --force).
pub fn refusal_message(refused: &[String]) -> String {
    const SHOW: usize = 12;
    let mut msg = format!(
        "refusing to pack {} build/artifact entr{}:\n",
        refused.len(),
        if refused.len() == 1 { "y" } else { "ies" }
    );
    for entry in refused.iter().take(SHOW) {
        msg.push_str("  ");
        msg.push_str(entry);
        msg.push('\n');
    }
    if refused.len() > SHOW {
        msg.push_str(&format!("  ... and {} more\n", refused.len() - SHOW));
    }
    msg.push_str(
        "Publish from a clean checkout, add them to .xiomignore, declare intentional \
         entries in package.xi (modules:), or pass --force.",
    );
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(force: bool) -> PackPolicy {
        PackPolicy { force, keep: BTreeSet::new(), ignore: XiomIgnore::default() }
    }

    // REL-20261010-1548-registry-2: the guide's exact list is refused.
    #[test]
    fn m306_pack_guard_refuses_build_artifact_entries() {
        for dir in REFUSED_DIRS {
            assert_eq!(policy(false).verdict(dir, true), Verdict::Refuse, "{dir}");
            assert_eq!(policy(false).verdict(&format!("nested/src/{dir}"), true), Verdict::Refuse);
        }
        for file in REFUSED_FILES {
            assert_eq!(policy(false).verdict(file, false), Verdict::Refuse, "{file}");
        }
        for ext in REFUSED_EXTS {
            assert_eq!(policy(false).verdict(&format!("src/blob.{ext}"), false), Verdict::Refuse);
            assert_eq!(
                policy(false).verdict(&format!("src/BLOB.{}", ext.to_uppercase()), false),
                Verdict::Refuse
            );
        }
        // Ordinary sources stay.
        assert_eq!(policy(false).verdict("src/main.xi", false), Verdict::Keep);
        assert_eq!(policy(false).verdict("src", true), Verdict::Keep);
        assert_eq!(policy(false).verdict("README.md", false), Verdict::Keep);
    }

    // --force overrides the default refusals (and only those).
    #[test]
    fn m306_pack_guard_force_overrides_refusals() {
        let forced = policy(true);
        assert_eq!(forced.verdict("target", true), Verdict::Keep);
        assert_eq!(forced.verdict("dist/app.exe", false), Verdict::Keep);
        assert!(forced.is_forced());
    }

    // .xiomignore extends the list with gitignore syntax.
    #[test]
    fn m306_pack_guard_honors_xiomignore() {
        let ignore = XiomIgnore::parse(
            "# comment\n\n*.log\nbuild-output/\n/docs-local\ntmp**\nsrc/*.gen.xi\n!keep.log\n",
        );
        assert!(ignore.matches("app.log", false));
        assert!(ignore.matches("nested/deep/app.log", false));
        assert!(!ignore.matches("keep.log", false), "negation re-includes");
        assert!(ignore.matches("build-output", true), "dir-only matches a dir");
        assert!(!ignore.matches("build-output", false), "dir-only never matches a file");
        assert!(ignore.matches("docs-local", true), "anchored at the package root");
        assert!(!ignore.matches("nested/docs-local", true), "anchored does not float");
        assert!(ignore.matches("src/generated.gen.xi", false), "path pattern");
        assert!(!ignore.matches("src/other.xi", false));

        let mut p = policy(false);
        p.ignore = ignore;
        assert_eq!(p.verdict("notes.log", false), Verdict::Skip);
        assert_eq!(p.verdict("build-output", true), Verdict::Skip);
        assert_eq!(p.verdict("keep.log", false), Verdict::Keep);
        assert_eq!(p.verdict("target", true), Verdict::Refuse, "defaults still fire");
    }

    // REL-20261014-1504-registry: .xiomignore is the documented escape for a
    // default-refused entry. It must win over the refusal list, including
    // under --force (which must not resurrect an ignored entry).
    #[test]
    fn m306_pack_guard_xiomignore_precedes_defaults() {
        let ignore = XiomIgnore::parse("target\ndist/\n*.exe\n");
        let mut p = policy(false);
        p.ignore = ignore.clone();
        assert_eq!(p.verdict("target", true), Verdict::Skip, "no-slash basename form");
        assert_eq!(p.verdict("nested/target", true), Verdict::Skip, "basename at any depth");
        assert_eq!(p.verdict("dist", true), Verdict::Skip, "dir-only form");
        assert_eq!(p.verdict("app.exe", false), Verdict::Skip, "extension form, LF only");
        assert_eq!(p.verdict("src/main.xi", false), Verdict::Keep, "ordinary source stays");

        let mut forced = policy(true);
        forced.ignore = ignore;
        assert_eq!(forced.verdict("target", true), Verdict::Skip, "ignore wins under --force");
        assert_eq!(forced.verdict("app.exe", false), Verdict::Skip, "ignore wins under --force (file)");
        assert_eq!(
            forced.verdict("build", true),
            Verdict::Keep,
            "--force still keeps a non-ignored refusal"
        );
    }

    // Declared entries in package.xi are kept, including inside a refused dir.
    #[test]
    fn m306_pack_guard_keeps_declared_entries() {
        let mut p = policy(false);
        p.keep.insert("package.xi".into());
        p.keep.insert("dist/extra.xi".into());
        assert_eq!(p.verdict("package.xi", false), Verdict::Keep);
        assert_eq!(p.verdict("dist/extra.xi", false), Verdict::Keep, "declared escape");
        assert_eq!(p.verdict("dist", true), Verdict::Keep, "walked for its declared child");
        assert_eq!(p.verdict("dist/other.exe", false), Verdict::Refuse, "undeclared stays refused");
        assert_eq!(p.verdict("target", true), Verdict::Refuse, "unrelated dir unaffected");
    }

    #[test]
    fn m306_pack_guard_glob_matching() {
        assert!(glob_segment_match("*.xi", "main.xi"));
        assert!(!glob_segment_match("*.xi", "main.xyz"));
        assert!(glob_segment_match("a?c", "abc"));
        assert!(glob_segment_match("*", "anything"));
        assert!(glob_path_match("src/*.xi", "src/main.xi"));
        assert!(!glob_path_match("src/*.xi", "src/deep/main.xi"), "* crosses no slash");
        assert!(glob_path_match("**/*.xi", "src/deep/main.xi"));
        assert!(glob_path_match("src/**/gen.xi", "src/gen.xi"), "** matches zero segments");
        assert!(glob_path_match("src/**/gen.xi", "src/a/b/gen.xi"));
        assert!(!glob_path_match("src/**/gen.xi", "other/a/gen.xi"));
    }

    #[test]
    fn m306_pack_guard_refusal_message_lists_escapes() {
        let msg = refusal_message(&["target/".to_string(), "app.exe".to_string()]);
        assert!(msg.contains("refusing to pack 2 build/artifact entries"), "{msg}");
        assert!(msg.contains("target/"), "{msg}");
        assert!(msg.contains(".xiomignore"), "{msg}");
        assert!(msg.contains("--force"), "{msg}");

        let many: Vec<String> = (0..20).map(|i| format!("f{i}.exe")).collect();
        let msg = refusal_message(&many);
        assert!(msg.contains("... and 8 more"), "{msg}");
    }
}
