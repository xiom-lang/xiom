// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Version parsing, ordering and constraint matching.
//!
//! REL-20261010-1549-packages-7: moved here from `xiom-pkg::registry` so the
//! dependency source-root resolver can apply the same semver-ish semantics
//! to INSTALLED packages (`[dependencies]` range specs used to be matched
//! with a literal `starts_with`, so `">=0.3.0"` never resolved).

/// Parse a dotted version: numeric core segments plus an optional
/// pre-release/build tail kept verbatim ("1.2.3-rc1" -> ([1,2,3], "-rc1")).
pub fn parse_version(v: &str) -> (Vec<u64>, String) {
    let v = v.trim();
    let (core, tail) = match v.find(['-', '+']) {
        Some(i) => (&v[..i], &v[i..]),
        None => (v, ""),
    };
    let nums = core
        .split('.')
        .map(|s| s.trim().parse::<u64>().unwrap_or(0))
        .collect();
    (nums, tail.to_string())
}

/// Numeric-aware version ordering: missing segments are 0 and a release sorts
/// ABOVE its pre-releases (`1.0.0-rc1 < 1.0.0`).
pub fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (an, at) = parse_version(a);
    let (bn, bt) = parse_version(b);
    for i in 0..an.len().max(bn.len()) {
        let x = an.get(i).copied().unwrap_or(0);
        let y = bn.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    match (at.is_empty(), bt.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => at.cmp(&bt),
    }
}

/// True when `version` satisfies a comma-separated constraint list.
/// Operators: `=` (or bare), `>=`, `<=`, `>`, `<`, `^`, `~`; `*`/empty = any.
pub fn version_satisfies(version: &str, req: &str) -> bool {
    use std::cmp::Ordering;
    let req = req.trim();
    if req.is_empty() || req == "*" {
        return true;
    }
    for part in req.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let (op, rest) = if let Some(r) = p.strip_prefix(">=") {
            (">=", r)
        } else if let Some(r) = p.strip_prefix("<=") {
            ("<=", r)
        } else if let Some(r) = p.strip_prefix('>') {
            (">", r)
        } else if let Some(r) = p.strip_prefix('<') {
            ("<", r)
        } else if let Some(r) = p.strip_prefix('=') {
            ("=", r)
        } else if let Some(r) = p.strip_prefix('^') {
            ("^", r)
        } else if let Some(r) = p.strip_prefix('~') {
            ("~", r)
        } else {
            ("=", p)
        };
        let rest = rest.trim();
        let (vn, _) = parse_version(version);
        let (rn, _) = parse_version(rest);
        let ord = version_cmp(version, rest);
        let ok = match op {
            ">=" => ord != Ordering::Less,
            "<=" => ord != Ordering::Greater,
            ">" => ord == Ordering::Greater,
            "<" => ord == Ordering::Less,
            "=" => ord == Ordering::Equal,
            "^" => vn.first() == rn.first() && ord != Ordering::Less,
            "~" => {
                vn.first() == rn.first()
                    && vn.get(1).copied().unwrap_or(0) == rn.get(1).copied().unwrap_or(0)
                    && ord != Ordering::Less
            }
            _ => false,
        };
        if !ok {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn version_cmp_is_numeric_aware() {
        assert_eq!(version_cmp("1.2.3", "1.2.10"), Ordering::Less);
        assert_eq!(version_cmp("1.0", "1.0.0"), Ordering::Equal);
        assert_eq!(version_cmp("1.0.0-rc1", "1.0.0"), Ordering::Less);
        assert_eq!(version_cmp("2.0.0", "1.9.9"), Ordering::Greater);
    }

    #[test]
    fn version_satisfies_operators() {
        assert!(version_satisfies("0.9.9", ">=0.5.0,<1.0.0"));
        assert!(!version_satisfies("1.0.0", ">=0.5.0,<1.0.0"));
        assert!(version_satisfies("1.9.0", "^1.2.0"));
        assert!(!version_satisfies("2.0.0", "^1.2.0"));
        assert!(version_satisfies("1.2.9", "~1.2.0"));
        assert!(!version_satisfies("1.3.0", "~1.2.0"));
        assert!(version_satisfies("1.2.3", "1.2.3"));
        assert!(version_satisfies("9.9.9", "*"));
    }
}
