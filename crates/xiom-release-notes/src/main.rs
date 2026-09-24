// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Release-notes (`What's new`) converter and validator.
//!
//! Implements the website contract schema v1 (`xiom-lang/website`,
//! `docs/release-notes-schema.md`): authoring happens in
//! `release-notes/<tag>.md`, the converter merges the stdlib fragment at
//! `<stdlib>/release-notes/<tag>.md` (highlights default to `kind: stdlib`),
//! validates HARD, and writes `release-notes/<tag>.json`.
//!
//! The release workflow runs `verify` before creating the tag/release: it
//! regenerates the document from the markdown sources and requires the
//! committed JSON to be byte-identical, so the tag-pinned raw fallback the
//! website reads is always exactly what the validator approved.
//!
//! No dependencies: the tool runs in the release job with nothing but the
//! toolchain, and the audited-only supply chain stays untouched.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const KINDS: [&str; 6] = ["language", "compiler", "stdlib", "tooling", "fix", "security"];
const MAX_HIGHLIGHTS: usize = 6;
const MAX_SUMMARY: usize = 240;
const MAX_TITLE: usize = 60;
const MAX_TEXT: usize = 320;
const MAX_ITEM: usize = 240;
const MAX_DOCS: usize = 6;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("xiom-release-notes: error: {e}");
            ExitCode::from(1)
        }
    }
}

fn usage() -> String {
    "usage:\n  \
     xiom-release-notes convert --tag <vX.Y.Z> [--stdlib <dir>] [--notes-dir <dir>] [--out <path>]\n  \
     xiom-release-notes verify  --tag <vX.Y.Z> [--stdlib <dir>] [--notes-dir <dir>]\n\
     \n\
     convert  reads <notes-dir>/<tag>.md (+ <stdlib>/release-notes/<tag>.md when present),\n         \
     validates it, and writes <notes-dir>/<tag>.json\n\
     verify   regenerates in memory and fails unless the committed JSON matches byte for byte"
        .to_string()
}

fn run(args: &[String]) -> Result<String, String> {
    let Some(sub) = args.first().map(|s| s.as_str()) else {
        return Err(usage());
    };
    if sub == "-h" || sub == "--help" || sub == "help" {
        return Ok(usage());
    }
    let mut tag: Option<String> = None;
    let mut stdlib: Option<PathBuf> = None;
    let mut notes_dir = PathBuf::from("release-notes");
    let mut out: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tag" => {
                i += 1;
                tag = Some(args.get(i).ok_or("--tag needs a value")?.clone());
            }
            "--stdlib" => {
                i += 1;
                stdlib = Some(PathBuf::from(args.get(i).ok_or("--stdlib needs a value")?));
            }
            "--notes-dir" => {
                i += 1;
                notes_dir = PathBuf::from(args.get(i).ok_or("--notes-dir needs a value")?);
            }
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(args.get(i).ok_or("--out needs a value")?));
            }
            other => return Err(format!("unknown argument '{other}'\n{}", usage())),
        }
        i += 1;
    }
    let tag = tag.ok_or_else(|| format!("--tag is required\n{}", usage()))?;
    match sub {
        "convert" => {
            let notes = build_notes(&tag, stdlib.as_deref(), &notes_dir)?;
            let json = to_json(&notes);
            let path = out.unwrap_or_else(|| notes_dir.join(format!("{tag}.json")));
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            fs::write(&path, &json).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
            Ok(format!(
                "wrote {} ({} highlights, {} breaking, {} docs)",
                path.display(),
                notes.highlights.len(),
                notes.breaking.len(),
                notes.docs.len()
            ))
        }
        "verify" => {
            let notes = build_notes(&tag, stdlib.as_deref(), &notes_dir)?;
            let expected = to_json(&notes);
            let path = notes_dir.join(format!("{tag}.json"));
            let committed = fs::read_to_string(&path).map_err(|e| {
                format!(
                    "cannot read {}: {e}\n  commit the release-notes JSON BEFORE creating the tag (see release-notes/README.md)",
                    path.display()
                )
            })?;
            if committed != expected {
                return Err(format!(
                    "{} is out of sync with {}.md ({} bytes committed, {} expected)\n  first difference: {}\n  re-run: xiom-release-notes convert --tag {tag}",
                    path.display(),
                    tag,
                    committed.len(),
                    expected.len(),
                    first_difference(&committed, &expected)
                ));
            }
            Ok(format!(
                "{} is valid schema-1 notes and in sync with {}.md ({} highlights)",
                path.display(),
                tag,
                notes.highlights.len()
            ))
        }
        other => Err(format!("unknown subcommand '{other}'\n{}", usage())),
    }
}

fn first_difference(a: &str, b: &str) -> String {
    for (i, (ca, cb)) in a.chars().zip(b.chars()).enumerate() {
        if ca != cb {
            let line = a[..i.min(a.len())].matches('\n').count() + 1;
            return format!("line {line} (char {i}): committed {ca:?}, expected {cb:?}");
        }
    }
    if a.len() != b.len() {
        return format!("trailing length difference after {} chars", a.chars().count());
    }
    "documents are identical".to_string()
}

#[derive(Default)]
struct Md {
    summary: Option<String>,
    highlights: Vec<Highlight>,
    breaking: Vec<String>,
    known: Vec<String>,
    docs: Vec<(String, String)>,
    saw_breaking: bool,
}

#[derive(Clone, Debug)]
struct Highlight {
    kind: String,
    title: String,
    text: String,
}

#[derive(Debug)]
struct Notes {
    tag: String,
    summary: String,
    highlights: Vec<Highlight>,
    breaking: Vec<String>,
    known: Vec<String>,
    docs: Vec<(String, String)>,
}

fn validate_tag(tag: &str) -> Result<(), String> {
    let rest = tag.strip_prefix('v').ok_or_else(|| format!("tag '{tag}' must start with 'v'"))?;
    let (core, _suffix) = match rest.split_once('-') {
        Some((c, s)) => (c, Some(s)),
        None => (rest, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit())) {
        return Err(format!("tag '{tag}' must look like vX.Y.Z (suffix allowed)"));
    }
    Ok(())
}

fn build_notes(tag: &str, stdlib: Option<&Path>, notes_dir: &Path) -> Result<Notes, String> {
    validate_tag(tag)?;
    let compiler_md = notes_dir.join(format!("{tag}.md"));
    let text = fs::read_to_string(&compiler_md)
        .map_err(|e| format!("cannot read {}: {e}", compiler_md.display()))?;
    let md = parse_md(&text, tag, None).map_err(|e| format!("{}: {e}", compiler_md.display()))?;

    let mut highlights = md.highlights;
    if let Some(dir) = stdlib {
        let fragment = dir.join("release-notes").join(format!("{tag}.md"));
        if fragment.is_file() {
            let ftext = fs::read_to_string(&fragment)
                .map_err(|e| format!("cannot read {}: {e}", fragment.display()))?;
            let frag = parse_md(&ftext, tag, Some("stdlib"))
                .map_err(|e| format!("{}: {e}", fragment.display()))?;
            // The release summary comes from the compiler document; the
            // fragment contributes highlights only (contract: its highlights
            // become kind "stdlib" unless the fragment sets its own kind).
            highlights.extend(frag.highlights);
        }
    }

    let summary = md
        .summary
        .ok_or_else(|| "missing '## Summary' section".to_string())?;
    check_range(&summary, 1, MAX_SUMMARY, "summary")?;
    check_plain(&summary, "summary")?;
    check_no_internal_ids(&summary, "summary")?;
    if contains_version_like(&summary) {
        return Err(format!("summary must not contain a version number: {summary:?}"));
    }
    check_single_sentence(&summary)?;

    if highlights.is_empty() {
        return Err("at least one highlight is required".to_string());
    }
    if highlights.len() > MAX_HIGHLIGHTS {
        return Err(format!(
            "too many highlights ({}) -- including the stdlib fragment the schema allows at most {MAX_HIGHLIGHTS}",
            highlights.len()
        ));
    }
    for h in &highlights {
        if !KINDS.contains(&h.kind.as_str()) {
            return Err(format!(
                "highlight '{}' has kind '{}' -- expected one of {}",
                h.title,
                h.kind,
                KINDS.join(", ")
            ));
        }
        check_range(&h.title, 1, MAX_TITLE, &format!("highlight title '{}'", h.title))?;
        check_range(&h.text, 1, MAX_TEXT, &format!("highlight text for '{}'", h.title))?;
        check_plain(&h.title, "highlight title")?;
        check_plain(&h.text, "highlight text")?;
        check_no_internal_ids(&h.title, "highlight title")?;
        check_no_internal_ids(&h.text, "highlight text")?;
    }

    if !md.saw_breaking {
        return Err(
            "missing '## Breaking changes' section (write '- None.' when there is nothing)".to_string(),
        );
    }
    if md.breaking.is_empty() {
        return Err("'## Breaking changes' must list at least one item or '- None.'".to_string());
    }
    let breaking = normalize_none(md.breaking, "breaking changes")?;
    for b in &breaking {
        check_range(b, 1, MAX_ITEM, "breaking item")?;
        check_plain(b, "breaking item")?;
        check_no_internal_ids(b, "breaking item")?;
    }
    let known = normalize_none(md.known, "known issues")?;
    for k in &known {
        check_range(k, 1, MAX_ITEM, "known issue")?;
        check_plain(k, "known issue")?;
        check_no_internal_ids(k, "known issue")?;
    }
    if md.docs.len() > MAX_DOCS {
        return Err(format!("at most {MAX_DOCS} docs entries are allowed"));
    }
    for (title, url) in &md.docs {
        check_range(title, 1, MAX_TITLE, "docs title")?;
        check_plain(title, "docs title")?;
        check_no_internal_ids(title, "docs title")?;
        if !url.starts_with("https://") {
            return Err(format!("docs URL must start with https:// (got {url:?})"));
        }
    }

    Ok(Notes {
        tag: tag.to_string(),
        summary,
        highlights,
        breaking,
        known,
        docs: md.docs,
    })
}

fn normalize_none(items: Vec<String>, what: &str) -> Result<Vec<String>, String> {
    if items.len() == 1 && items[0].eq_ignore_ascii_case("none.") {
        return Ok(Vec::new());
    }
    if items.iter().any(|i| i.eq_ignore_ascii_case("none.")) {
        return Err(format!("{what}: 'None.' cannot be combined with other items"));
    }
    Ok(items)
}

fn parse_md(text: &str, tag: &str, default_kind: Option<&str>) -> Result<Md, String> {
    #[derive(PartialEq)]
    enum Sec {
        None,
        Summary,
        Highlights,
        Breaking,
        Known,
        Docs,
    }
    let mut sec = Sec::None;
    let mut md = Md::default();
    let mut current: Option<Highlight> = None;
    let mut seen_any = false;

    for (idx, raw) in text.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = raw.trim_end_matches('\r').trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("## ") {
            if let Some(h) = current.take() {
                md.highlights.push(finalize_highlight(h, default_kind)?);
            }
            sec = match name.trim() {
                "Summary" => {
                    if md.summary.is_some() {
                        return Err(format!("line {lineno}: duplicate '## Summary' section"));
                    }
                    Sec::Summary
                }
                "Highlights" => Sec::Highlights,
                "Breaking changes" => {
                    md.saw_breaking = true;
                    Sec::Breaking
                }
                "Known issues" => Sec::Known,
                "Docs" => Sec::Docs,
                other => {
                    return Err(format!(
                        "line {lineno}: unknown section '## {other}' (expected Summary, Highlights, Breaking changes, Known issues, Docs)"
                    ))
                }
            };
            seen_any = true;
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("### ") {
            if sec != Sec::Highlights {
                return Err(format!("line {lineno}: '###' highlights must appear under '## Highlights'"));
            }
            if let Some(h) = current.take() {
                md.highlights.push(finalize_highlight(h, default_kind)?);
            }
            current = Some(Highlight {
                kind: String::new(),
                title: title.trim().to_string(),
                text: String::new(),
            });
            seen_any = true;
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("# ") {
            if !seen_any {
                let title = title.trim();
                if !title.is_empty() && !title.contains(tag) {
                    return Err(format!("line {lineno}: title '# {title}' does not match the tag {tag}"));
                }
                seen_any = true;
                continue;
            }
            return Err(format!("line {lineno}: unexpected top-level heading inside a section"));
        }
        seen_any = true;
        match sec {
            Sec::None => return Err(format!("line {lineno}: content outside any '##' section")),
            Sec::Summary => {
                let s = md.summary.get_or_insert_with(String::new);
                if !s.is_empty() {
                    s.push(' ');
                }
                s.push_str(trimmed);
            }
            Sec::Highlights => {
                let h = current.as_mut().ok_or_else(|| {
                    format!("line {lineno}: text before the first '###' highlight")
                })?;
                if let Some(k) = trimmed.strip_prefix("kind:") {
                    if !h.text.is_empty() || !h.kind.is_empty() {
                        return Err(format!(
                            "line {lineno}: the 'kind:' line must be the first line of the highlight body"
                        ));
                    }
                    h.kind = k.trim().to_string();
                } else {
                    if !h.text.is_empty() {
                        h.text.push(' ');
                    }
                    h.text.push_str(trimmed);
                }
            }
            Sec::Breaking | Sec::Known => {
                let Some(item) = trimmed.strip_prefix("- ") else {
                    return Err(format!("line {lineno}: expected a '- ' list item"));
                };
                let item = item.trim().to_string();
                if item.is_empty() {
                    return Err(format!("line {lineno}: empty list item"));
                }
                if sec == Sec::Breaking {
                    md.breaking.push(item);
                } else {
                    md.known.push(item);
                }
            }
            Sec::Docs => {
                let Some(item) = trimmed.strip_prefix("- ") else {
                    return Err(format!("line {lineno}: expected a '- Title | https://...' entry"));
                };
                let (title, url) = item
                    .split_once('|')
                    .ok_or_else(|| format!("line {lineno}: docs entries are 'Title | https://...'"))?;
                let (title, url) = (title.trim().to_string(), url.trim().to_string());
                if title.is_empty() || url.is_empty() {
                    return Err(format!("line {lineno}: docs entries need both a title and a URL"));
                }
                md.docs.push((title, url));
            }
        }
    }
    if let Some(h) = current.take() {
        md.highlights.push(finalize_highlight(h, default_kind)?);
    }
    Ok(md)
}

fn finalize_highlight(h: Highlight, default_kind: Option<&str>) -> Result<Highlight, String> {
    let title = h.title.trim().to_string();
    if title.is_empty() {
        return Err("highlight is missing a title".to_string());
    }
    let kind = if h.kind.trim().is_empty() {
        default_kind.map(|k| k.to_string()).ok_or_else(|| {
            format!("highlight '{title}' is missing its 'kind:' line")
        })?
    } else {
        h.kind.trim().to_string()
    };
    let text = h.text.trim().to_string();
    if text.is_empty() {
        return Err(format!("highlight '{title}' has no text"));
    }
    Ok(Highlight { kind, title, text })
}

fn check_range(s: &str, min: usize, max: usize, what: &str) -> Result<(), String> {
    let n = s.chars().count();
    if n < min {
        return Err(format!("{what} is empty"));
    }
    if n > max {
        return Err(format!("{what} is {n} characters (max {max})"));
    }
    Ok(())
}

fn check_plain(s: &str, what: &str) -> Result<(), String> {
    if !s.is_ascii() {
        return Err(format!("{what} must be plain ASCII: {s:?}"));
    }
    if s.chars().any(|c| c.is_control()) {
        return Err(format!("{what} must not contain control characters"));
    }
    for bad in ["<", ">", "&", "`", "**", "__", "](", "[]("] {
        if s.contains(bad) {
            return Err(format!("{what} must be plain text (found markdown/HTML '{bad}')"));
        }
    }
    if s.starts_with('#') || s.starts_with("- ") {
        return Err(format!("{what} must be plain text (found markdown syntax)"));
    }
    Ok(())
}

fn check_no_internal_ids(s: &str, what: &str) -> Result<(), String> {
    for token in s.split(|c: char| !c.is_ascii_alphanumeric()) {
        if token.is_empty() {
            continue;
        }
        let lower = token.to_ascii_lowercase();
        let hex_like = token.len() >= 7
            && token.chars().all(|c| c.is_ascii_hexdigit())
            && token.chars().any(|c| c.is_ascii_digit())
            && token.chars().any(|c| c.is_ascii_alphabetic());
        if hex_like {
            return Err(format!("{what} looks like a commit hash ('{token}') -- release notes must not contain internal identifiers"));
        }
        if token.len() >= 3
            && (lower.starts_with('r') || lower.starts_with('m'))
            && token[1..].chars().all(|c| c.is_ascii_digit())
        {
            return Err(format!("{what} looks like an internal task id ('{token}') -- describe the change for users instead"));
        }
    }
    Ok(())
}

fn contains_version_like(s: &str) -> bool {
    s.split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ':' || c == '(' || c == ')')
        .any(|tok| {
            let t = tok.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.');
            let t = t.strip_prefix('v').unwrap_or(t);
            let parts: Vec<&str> = t.split('.').collect();
            parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        })
}

fn check_single_sentence(s: &str) -> Result<(), String> {
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if matches!(c, '.' | '!' | '?') {
            if i + 1 < chars.len() {
                let next = chars[i + 1];
                if next.is_whitespace() {
                    return Err(format!("summary must be one sentence: {s:?}"));
                }
            }
        }
    }
    match chars.last() {
        Some('.') | Some('!') | Some('?') => Ok(()),
        _ => Err(format!("summary must end with a period: {s:?}")),
    }
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn push_str_array(out: &mut String, name: &str, items: &[String]) {
    if items.is_empty() {
        out.push_str(&format!("  \"{name}\": [],\n"));
        return;
    }
    out.push_str(&format!("  \"{name}\": [\n"));
    for (i, item) in items.iter().enumerate() {
        let comma = if i + 1 < items.len() { "," } else { "" };
        out.push_str(&format!("    \"{}\"{comma}\n", esc(item)));
    }
    out.push_str("  ],\n");
}

fn to_json(n: &Notes) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"schema\": 1,\n");
    s.push_str(&format!("  \"tag\": \"{}\",\n", esc(&n.tag)));
    s.push_str(&format!("  \"summary\": \"{}\",\n", esc(&n.summary)));
    s.push_str("  \"highlights\": [\n");
    for (i, h) in n.highlights.iter().enumerate() {
        s.push_str("    {\n");
        s.push_str(&format!("      \"kind\": \"{}\",\n", esc(&h.kind)));
        s.push_str(&format!("      \"title\": \"{}\",\n", esc(&h.title)));
        s.push_str(&format!("      \"text\": \"{}\"\n", esc(&h.text)));
        let comma = if i + 1 < n.highlights.len() { "," } else { "" };
        s.push_str(&format!("    }}{comma}\n"));
    }
    s.push_str("  ],\n");
    push_str_array(&mut s, "breaking", &n.breaking);
    push_str_array(&mut s, "known_issues", &n.known);
    if n.docs.is_empty() {
        s.push_str("  \"docs\": [],\n");
    } else {
        s.push_str("  \"docs\": [\n");
        for (i, (title, url)) in n.docs.iter().enumerate() {
            let comma = if i + 1 < n.docs.len() { "," } else { "" };
            s.push_str("    {\n");
            s.push_str(&format!("      \"title\": \"{}\",\n", esc(title)));
            s.push_str(&format!("      \"url\": \"{}\"\n", esc(url)));
            s.push_str(&format!("    }}{comma}\n"));
        }
        s.push_str("  ],\n");
    }
    s.push_str(&format!(
        "  \"full_changelog\": \"https://github.com/xiom-lang/xiom/releases/tag/{}\"\n",
        esc(&n.tag)
    ));
    s.push_str("}\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compiler_md() -> String {
        "# v0.62.0\n\n\
         ## Summary\n\
         A smoother first run and stronger collection semantics.\n\n\
         ## Highlights\n\n\
         ### Collections iterate like you expect\n\
         kind: compiler\n\
         Loops over vectors and arrays now visit every element.\n\n\
         ### Field access returns the value you stored\n\
         kind: fix\n\
         Optional and result payloads keep their real type.\n\n\
         ## Breaking changes\n\
         - None.\n\n\
         ## Docs\n\
         - Contracts guide | https://docs.xiom-lang.org/latest/contracts/\n"
            .to_string()
    }

    fn fragment() -> String {
        "# v0.62.0\n\n\
         ## Summary\n\
         Standard library work (ignored; the release summary wins).\n\n\
         ## Highlights\n\n\
         ### More modules carry contracts\n\
         kind: stdlib\n\
         New clauses across collections and text.\n\n\
         ### Sorting by key keeps order\n\
         Key-based sorting is correct across types.\n"
            .to_string()
    }

    fn write_tree(root: &Path, compiler: &str, frag: Option<&str>) -> (PathBuf, PathBuf) {
        let notes = root.join("release-notes");
        fs::create_dir_all(&notes).unwrap();
        fs::write(notes.join("v0.62.0.md"), compiler).unwrap();
        let stdlib = root.join("stdlib");
        if let Some(f) = frag {
            fs::create_dir_all(stdlib.join("release-notes")).unwrap();
            fs::write(stdlib.join("release-notes").join("v0.62.0.md"), f).unwrap();
        }
        (notes, stdlib)
    }

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiom_relnotes_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn converts_and_merges_the_fragment() {
        let root = tmp("merge");
        let (notes, stdlib) = write_tree(&root, &compiler_md(), Some(&fragment()));
        let n = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap();
        assert_eq!(n.highlights.len(), 4);
        assert_eq!(n.highlights[0].kind, "compiler");
        assert_eq!(n.highlights[2].kind, "stdlib");
        assert_eq!(n.highlights[3].kind, "stdlib", "fragment default kind");
        assert!(n.breaking.is_empty(), "None. means no breaking changes");
        assert_eq!(n.docs.len(), 1);
        let json = to_json(&n);
        assert!(json.contains("\"breaking\": []"));
        assert!(json.contains("\"schema\": 1"));
        assert!(json.contains("\"full_changelog\": \"https://github.com/xiom-lang/xiom/releases/tag/v0.62.0\""));
        // deterministic
        assert_eq!(json, to_json(&build_notes("v0.62.0", Some(&stdlib), &notes).unwrap()));
    }

    #[test]
    fn verify_detects_drift() {
        let root = tmp("verify");
        let (notes, stdlib) = write_tree(&root, &compiler_md(), Some(&fragment()));
        let n = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap();
        fs::write(notes.join("v0.62.0.json"), to_json(&n)).unwrap();
        assert!(run(&["verify".into(), "--tag".into(), "v0.62.0".into(), "--stdlib".into(), stdlib.display().to_string(), "--notes-dir".into(), notes.display().to_string()]).is_ok());
        // Drift: change a title in the markdown.
        let edited = compiler_md().replace("Collections iterate like you expect", "Collections iterate how you expect");
        fs::write(notes.join("v0.62.0.md"), edited).unwrap();
        let err = run(&["verify".into(), "--tag".into(), "v0.62.0".into(), "--stdlib".into(), stdlib.display().to_string(), "--notes-dir".into(), notes.display().to_string()]).unwrap_err();
        assert!(err.contains("out of sync"), "{err}");
    }

    #[test]
    fn rejects_missing_breaking_section() {
        let root = tmp("nobreaking");
        let md = compiler_md().replace("## Breaking changes\n- None.\n\n", "");
        let (notes, stdlib) = write_tree(&root, &md, None);
        let err = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap_err();
        assert!(err.contains("Breaking changes"), "{err}");
    }

    #[test]
    fn rejects_limits_and_kinds() {
        let root = tmp("limits");
        // 7 highlights (compiler 2 + fragment 5)
        let frag = "# v0.62.0\n\n## Highlights\n\n".to_string()
            + &(1..=5)
                .map(|i| format!("### Item {i}\nkind: stdlib\nText {i}.\n\n"))
                .collect::<String>();
        let (notes, stdlib) = write_tree(&root, &compiler_md(), Some(&frag));
        let err = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap_err();
        assert!(err.contains("too many highlights"), "{err}");

        // invalid kind
        let md = compiler_md().replace("kind: fix", "kind: refactor");
        let (notes2, stdlib2) = write_tree(&root.join("k"), &md, None);
        let err = build_notes("v0.62.0", Some(&stdlib2), &notes2).unwrap_err();
        assert!(err.contains("kind 'refactor'"), "{err}");

        // over-long title
        let long = "T".repeat(61);
        let md = compiler_md().replace("Collections iterate like you expect", &long);
        let (notes3, stdlib3) = write_tree(&root.join("t"), &md, None);
        let err = build_notes("v0.62.0", Some(&stdlib3), &notes3).unwrap_err();
        assert!(err.contains("max 60"), "{err}");
    }

    #[test]
    fn rejects_impure_content() {
        let root = tmp("content");
        for (bad, needle) in [
            ("A <b>bold</b> claim.", "plain text"),
            ("Use `code` here.", "plain text"),
            ("Fixed deadbeef1 in the parser.", "commit hash"),
            ("Fixed R66 for good.", "task id"),
        ] {
            let md = compiler_md().replace("Loops over vectors and arrays now visit every element.", bad);
            let (notes, stdlib) = write_tree(&root.join(needle.replace(' ', "_")), &md, None);
            let err = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap_err();
            assert!(err.contains(needle), "want {needle}, got {err}");
        }
        // non-ASCII
        let md = compiler_md().replace("Loops over vectors and arrays now visit every element.", "Caf\u{e9} semantics.");
        let (notes, stdlib) = write_tree(&root.join("ascii"), &md, None);
        let err = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap_err();
        assert!(err.contains("ASCII"), "{err}");
        // version in summary
        let md = compiler_md().replace("A smoother first run and stronger collection semantics.", "Upgrade to v0.62.0 today.");
        let (notes, stdlib) = write_tree(&root.join("ver"), &md, None);
        let err = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap_err();
        assert!(err.contains("version number"), "{err}");
        // http docs url
        let md = compiler_md().replace("https://docs.xiom-lang.org/latest/contracts/", "http://docs.xiom-lang.org/latest/contracts/");
        let (notes, stdlib) = write_tree(&root.join("url"), &md, None);
        let err = build_notes("v0.62.0", Some(&stdlib), &notes).unwrap_err();
        assert!(err.contains("https://"), "{err}");
    }

    #[test]
    fn tags_are_checked() {
        assert!(validate_tag("v0.62.0").is_ok());
        assert!(validate_tag("v1.2.3-rc1").is_ok());
        assert!(validate_tag("0.62.0").is_err());
        assert!(validate_tag("v0.62").is_err());
    }
}
