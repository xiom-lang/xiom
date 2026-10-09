// XIOM -- persistent catalog-body check cache (Stage 6 item 1, m252)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stage 6 (STAGE6_PERF_PLAN item 1, m252): persistent POSITIVE-ONLY cache
//! for catalog-body type checks.
//!
//! `flush_catalog_bodies` re-checks every loaded catalog module body on every
//! compile (the whole-stdlib re-check is the recorded cold-lane cost). The
//! checker's body pass produces diagnostics + resolved-call tables; codegen
//! monomorphisation is independent (xiom-codegen `generic_instantiations`),
//! so a body that checked CLEAN under the same checker, module content and
//! source tree can be skipped and its resolved-call delta replayed.
//!
//! Cache layout: the first line is an identity guard (compiler version, OS,
//! arch, pointer width); then one line per module:
//!   `<key>\t<module_content_hash>\t<tree_digest>\t<k1 v1 k2 v2 ...>`
//! `<key>` is the module dotted name (or `#<hash>` for anonymous modules);
//! the trailing field is the `catalog_resolved_calls` delta (keys and dotted
//! target paths never contain whitespace).
//!
//! Positive-only: only bodies with ZERO new diagnostics are recorded, and a
//! body whose resolution fell back to a non-isolated global table (or that
//! references a name the user program declares) is never recorded -- see the
//! guards in `flush_catalog_bodies`. Any failure (unreadable/corrupt file,
//! mismatched identity/hash/digest) is a miss and falls back to a live check.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

const IDENTITY_PREFIX: &str = "xiom-check-body-cache v1";

/// Identity line guarding the cache file against compiler upgrades and
/// platform changes (same scheme as the catidx index cache).
pub fn identity() -> String {
    format!(
        "{}|{}|{}-{}|{}",
        IDENTITY_PREFIX,
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        usize::BITS
    )
}

#[derive(Debug, Clone)]
struct Entry {
    module_hash: u64,
    tree_digest: String,
    replay: Vec<(String, String)>,
}

/// Persistent positive-only catalog-body check cache.
pub struct BodyCheckCache {
    path: PathBuf,
    entries: HashMap<String, Entry>,
    dirty: bool,
    pub hits: usize,
    pub misses: usize,
}

impl BodyCheckCache {
    /// Load the cache file; a missing file or stale identity yields an empty
    /// cache (every body is checked live and re-recorded).
    pub fn load(path: PathBuf) -> Self {
        let mut cache = Self {
            path,
            entries: HashMap::new(),
            dirty: false,
            hits: 0,
            misses: 0,
        };
        if let Ok(data) = std::fs::read_to_string(&cache.path) {
            let mut lines = data.lines();
            if lines.next() == Some(identity().as_str()) {
                for line in lines {
                    let mut parts = line.splitn(4, '\t');
                    let (Some(key), Some(hash), Some(digest), Some(replay)) =
                        (parts.next(), parts.next(), parts.next(), parts.next())
                    else {
                        continue;
                    };
                    let Ok(module_hash) = u64::from_str_radix(hash, 16) else {
                        continue;
                    };
                    let mut pairs: Vec<(String, String)> = Vec::new();
                    let mut it = replay.split_whitespace();
                    while let (Some(k), Some(v)) = (it.next(), it.next()) {
                        pairs.push((k.to_string(), v.to_string()));
                    }
                    cache.entries.insert(
                        key.to_string(),
                        Entry {
                            module_hash,
                            tree_digest: digest.to_string(),
                            replay: pairs,
                        },
                    );
                }
            }
        }
        cache
    }

    /// Look up a cached clean verdict. A hit requires the module content hash
    /// AND the indexed-tree digest to match the recorded ones.
    pub fn lookup(
        &mut self,
        key: &str,
        module_hash: u64,
        tree_digest: &str,
    ) -> Option<Vec<(String, String)>> {
        match self.entries.get(key) {
            Some(e) if e.module_hash == module_hash && e.tree_digest == tree_digest => {
                self.hits += 1;
                Some(e.replay.clone())
            }
            Some(e) => {
                if std::env::var_os("XIOM_TIMINGS").is_some() {
                    eprintln!(
                        "[timings]   body-cache {key} mismatch: hash {:016x} vs {module_hash:016x}, tree {} vs {tree_digest}",
                        e.module_hash, e.tree_digest
                    );
                }
                self.misses += 1;
                None
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    /// Record a clean body verdict plus its resolved-call delta.
    pub fn record(
        &mut self,
        key: &str,
        module_hash: u64,
        tree_digest: &str,
        replay: Vec<(String, String)>,
    ) {
        self.entries.insert(
            key.to_string(),
            Entry {
                module_hash,
                tree_digest: tree_digest.to_string(),
                replay,
            },
        );
        self.dirty = true;
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }

    /// Persist when entries changed (temp + rename, like the catidx cache).
    pub fn save(&mut self) {
        if !self.dirty {
            return;
        }
        let mut out = String::new();
        out.push_str(&identity());
        out.push('\n');
        for (key, e) in &self.entries {
            out.push_str(key);
            out.push('\t');
            out.push_str(&format!("{:016x}", e.module_hash));
            out.push('\t');
            out.push_str(&e.tree_digest);
            out.push('\t');
            for (i, (k, v)) in e.replay.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                out.push_str(k);
                out.push(' ');
                out.push_str(v);
            }
            out.push('\n');
        }
        let tmp = Path::new(&self.path).with_extension(format!("tmp-{}", std::process::id()));
        if std::fs::write(&tmp, out.as_bytes()).is_ok() {
            let _ = std::fs::rename(&tmp, &self.path);
        }
        self.dirty = false;
    }
}

/// Default persistent path: `$HOME/.xiom/checkbodies.txt` (temp fallback),
/// mirroring the jit script cache and the catidx index cache.
pub fn default_body_cache_path() -> PathBuf {
    for key in ["HOME", "USERPROFILE"] {
        if let Ok(dir) = std::env::var(key) {
            if !dir.is_empty() {
                let candidate = PathBuf::from(dir).join(".xiom");
                if std::fs::create_dir_all(&candidate).is_ok() {
                    return candidate.join("checkbodies.txt");
                }
            }
        }
    }
    std::env::temp_dir().join("xiom-checkbodies.txt")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "xiom-checkcache-{name}-{}-{nanos:x}.txt",
            std::process::id()
        ))
    }

    #[test]
    fn cache_roundtrip_identity_and_invalidation() {
        let path = temp_path("roundtrip");
        {
            let mut c = BodyCheckCache::load(path.clone());
            assert!(c.lookup("m.one", 1, "d1").is_none(), "empty cache misses");
            c.record(
                "m.one",
                1,
                "d1",
                vec![
                    ("fn#1:2".to_string(), "m.two.f".to_string()),
                    ("fn#3:4".to_string(), "m.two.g".to_string()),
                ],
            );
            c.save();
        }
        {
            let mut c = BodyCheckCache::load(path.clone());
            let replay = c.lookup("m.one", 1, "d1").expect("hit after save/load");
            assert_eq!(
                replay,
                vec![
                    ("fn#1:2".to_string(), "m.two.f".to_string()),
                    ("fn#3:4".to_string(), "m.two.g".to_string()),
                ]
            );
            assert!(c.lookup("m.one", 2, "d1").is_none(), "content change misses");
            assert!(c.lookup("m.one", 1, "d2").is_none(), "tree change misses");
            assert_eq!(c.stats(), (1, 2));
        }
        // A stale identity header invalidates everything.
        std::fs::write(&path, "xiom-check-body-cache v0|old\n").expect("write stale cache");
        let mut c = BodyCheckCache::load(path.clone());
        assert!(c.lookup("m.one", 1, "d1").is_none(), "stale identity misses");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn cache_corrupt_lines_are_ignored() {
        let path = temp_path("corrupt");
        let mut body = String::new();
        body.push_str(&identity());
        body.push('\n');
        body.push_str("garbage line without tabs\n");
        body.push_str("m.ok\tzz\tnothex\t\n");
        body.push_str("m.good\t0000000000000001\td1\tk v\n");
        std::fs::write(&path, body).expect("write corrupt cache");
        let mut c = BodyCheckCache::load(path.clone());
        let replay = c.lookup("m.good", 1, "d1").expect("valid line survives");
        assert_eq!(replay, vec![("k".to_string(), "v".to_string())]);
        assert!(c.lookup("m.ok", 1, "d1").is_none());
        let _ = std::fs::remove_file(&path);
    }
}
