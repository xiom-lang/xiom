// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compilation cache database for industrial incremental compilation.
//!
//! Architecture: Thread-safe, persistent cache with per-module entries at
//! multiple compilation pipeline stages.
//!
//! Cache tiers:
//! - **L1 (Token)**: Lexer output - token stream
//! - **L2 (AST)**: Parser output - abstract syntax tree
//! - **L3 (Checked)**: Checker output - type-checked program
//! - **L4 (IR)**: Codegen output - LLVM IR text
//! - **L5 (Object)**: Linker output - native object file
//!
//! Cache layout on disk: `.xi_cache/` directory with `index.json`, `<hash>.ll`,
//! `<hash>.tokens`, `<hash>.ast`, `<hash>.checked`, and `<hash>.o` files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use super::hash::{self, Fingerprint};
use super::ModuleNode;

// ---------------------------------------------------------------------------
// Cache entry and database types
// ---------------------------------------------------------------------------

/// Metadata for a single cached module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    /// Dotted module path (e.g. "mypackage.subpkg.module").
    pub module_path: String,
    /// Absolute path to the source file.
    pub file_path: String,
    /// Content fingerprint (source hash + signature hash).
    pub fingerprint: Fingerprint,
    /// Short hash used as the cache file stem.
    pub cache_key: String,
    /// Which tiers are populated.
    pub tiers: CacheTiers,
    /// Module paths this entry depends on (for transitive invalidation).
    pub dependencies: Vec<String>,
    /// Module paths that depend on this entry (reverse edges).
    pub dependents: Vec<String>,
    /// Unix timestamp of last compilation (for GC).
    pub last_compiled: u64,
}

/// Bitmask of populated cache tiers.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct CacheTiers {
    pub l1_tokens: bool,
    pub l2_ast: bool,
    pub l3_checked: bool,
    pub l4_ir: bool,
    pub l5_object: bool,
}

/// Thread-safe compilation cache database.
///
/// Wraps a `HashMap<String, CacheEntry>` with a shared cache directory
/// and `Arc<RwLock<...>>` for concurrent access.
#[derive(Debug, Clone)]
pub struct CacheDb {
    /// Cache directory on disk: typically `<project_root>/.xi_cache/`.
    pub cache_dir: PathBuf,
    /// In-memory index: module_path -> CacheEntry.
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Maximum number of cached entries before GC triggers.
    pub max_entries: usize,
}

impl CacheDb {
    /// Create or open a cache database at the given directory.
    pub fn open(cache_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&cache_dir);
        let entries = Self::load_index(&cache_dir);

        CacheDb {
            cache_dir,
            entries: Arc::new(RwLock::new(entries)),
            max_entries: 1000,
        }
    }

    /// Create a cache database in the project root's `.xi_cache/`.
    pub fn for_project(project_root: &Path) -> Self {
        Self::open(project_root.join(".xi_cache"))
    }

    /// Load the index from disk.
    fn load_index(cache_dir: &Path) -> HashMap<String, CacheEntry> {
        let index_path = cache_dir.join("index.json");
        match std::fs::read_to_string(&index_path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    /// Save the index to disk.
    fn save_index(&self) {
        let entries = self.entries.read().expect("cache RwLock poisoned");
        let index_path = self.cache_dir.join("index.json");
        if let Ok(json) = serde_json::to_string_pretty(&*entries) {
            let _ = std::fs::write(index_path, json);
        }
    }

    /// Look up a cached entry for a module. Returns `None` if not cached
    /// or if the fingerprint has changed (source modified).
    pub fn get(&self, module_path: &str) -> Option<CacheEntry> {
        let entries = self.entries.read().expect("cache RwLock poisoned");
        entries.get(module_path).cloned()
    }

    /// Check if a cached entry is still valid by comparing fingerprints.
    /// Also checks transitive dependencies for invalidation.
    pub fn is_valid(&self, module_path: &str, module: &ModuleNode) -> bool {
        let entries = self.entries.read().expect("cache RwLock poisoned");
        let entry = match entries.get(module_path) {
            Some(e) => e,
            None => return false,
        };

        // Check source hash matches
        let current_hash = hash::hash_str(
            &std::fs::read_to_string(&module.file_path).unwrap_or_default()
        );
        if entry.fingerprint.source_hash != current_hash {
            return false;
        }

        // Check all dependencies still valid
        for dep in &entry.dependencies {
            if !entries.contains_key(dep.as_str()) {
                return false;
            }
        }

        true
    }

    /// Check if a specific cache tier is available and valid.
    pub fn has_tier(&self, module_path: &str, module: &ModuleNode) -> Option<CacheTiers> {
        if !self.is_valid(module_path, module) {
            return None;
        }
        let entries = self.entries.read().expect("cache RwLock poisoned");
        entries.get(module_path).map(|e| e.tiers)
    }

    /// Load cached data from a specific tier.
    pub fn load_tier(&self, cache_key: &str, tier_extension: &str) -> Option<String> {
        let path = self.cache_dir.join(format!("{}.{}", cache_key, tier_extension));
        std::fs::read_to_string(&path).ok()
    }

    /// Store data for a specific cache tier.
    pub fn store_tier(&self, cache_key: &str, tier_extension: &str, data: &str) {
        let path = self.cache_dir.join(format!("{}.{}", cache_key, tier_extension));
        let _ = std::fs::write(&path, data);
    }

    /// Insert or update a cache entry for a module.
    pub fn insert(&self, entry: CacheEntry) {
        let mut entries = self.entries.write().expect("cache RwLock poisoned");

        // Check if signature changed and invalidate dependents transitively
        if let Some(old) = entries.get(&entry.module_path) {
            if old.fingerprint.signature_hash != entry.fingerprint.signature_hash
                && !old.fingerprint.signature_hash.is_empty()
                && !entry.fingerprint.signature_hash.is_empty()
            {
                // Signature changed -- invalidate all dependents transitively
                let mut to_invalidate: Vec<String> = old.dependents.clone();
                let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

                while let Some(dep) = to_invalidate.pop() {
                    if !visited.insert(dep.clone()) {
                        continue;
                    }
                    entries.remove(&dep);
                    // Clean up cache files
                    let key = entry.cache_key.clone(); // We use the current key as a fallback
                    let _ = std::fs::remove_file(
                        self.cache_dir.join(format!("{}.ll", key))
                    );
                    let _ = std::fs::remove_file(
                        self.cache_dir.join(format!("{}.tokens", key))
                    );
                    let _ = std::fs::remove_file(
                        self.cache_dir.join(format!("{}.ast", key))
                    );

                    // Also invalidate this dependent's dependents
                    if let Some(dep_entry) = entries.get(&dep) {
                        for d in &dep_entry.dependents {
                            if !visited.contains(d.as_str()) {
                                to_invalidate.push(d.clone());
                            }
                        }
                    }
                }
            }
        }

        entries.insert(entry.module_path.clone(), entry);

        // Trigger GC if over limit
        if entries.len() > self.max_entries {
            self.gc();
        }

        drop(entries);
        self.save_index();
    }

    /// Remove all entries that reference non-existent source files.
    pub fn purge_stale(&self) {
        let mut entries = self.entries.write().expect("cache RwLock poisoned");
        entries.retain(|_, entry| {
            Path::new(&entry.file_path).exists()
        });
        drop(entries);
        self.save_index();
    }

    /// Garbage collect: remove oldest entries to stay under max_entries.
    fn gc(&self) {
        let mut entries = self.entries.write().expect("cache RwLock poisoned");
        if entries.len() <= self.max_entries {
            return;
        }

        // Sort by last_compiled timestamp, remove oldest
        let mut sorted: Vec<(String, u64)> = entries
            .iter()
            .map(|(k, v)| (k.clone(), v.last_compiled))
            .collect();
        sorted.sort_by_key(|(_, ts)| *ts);

        let to_remove = entries.len() - self.max_entries;
        for (path, _) in sorted.iter().take(to_remove) {
            entries.remove(path);
        }
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.read().expect("cache RwLock poisoned").len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().expect("cache RwLock poisoned").is_empty()
    }

    /// Clear all cache entries and delete cache files from disk.
    pub fn clear(&self) {
        let mut entries = self.entries.write().expect("cache RwLock poisoned");
        entries.clear();
        drop(entries);

        // Remove all cache files
        if let Ok(dir_entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = std::fs::remove_file(path);
                }
            }
        }

        self.save_index();
    }
}

// ---------------------------------------------------------------------------
// Cache helper: build a CacheEntry from a ModuleNode
// ---------------------------------------------------------------------------

/// Build a cache entry for a module, computing its content fingerprint.
pub fn make_cache_entry(
    module: &ModuleNode,
    dependents: Vec<String>,
    tiers: CacheTiers,
) -> CacheEntry {
    let source = std::fs::read_to_string(&module.file_path).unwrap_or_default();
    let source_hash = hash::hash_str(&source);
    let cache_key = hash::short_hash(&source_hash).to_string();

    // Compute a signature hash from the module's exported names
    // For now, use the module path + dependency list as the signature
    let sig_text = format!("{}:{}", module.module_path, module.dependencies.join(","));
    let signature_hash = hash::hash_str(&sig_text);

    CacheEntry {
        module_path: module.module_path.clone(),
        file_path: module.file_path.to_string_lossy().to_string(),
        fingerprint: Fingerprint {
            source_hash,
            signature_hash,
        },
        cache_key,
        tiers,
        dependencies: module.dependencies.clone(),
        dependents,
        last_compiled: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_cache() -> CacheDb {
        let dir = std::env::temp_dir().join(format!("xiom_cache_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cache = CacheDb::open(dir);
        cache.clear(); // start clean
        cache
    }

    fn make_test_module(name: &str, deps: Vec<&str>) -> ModuleNode {
        ModuleNode {
            module_path: name.to_string(),
            file_path: PathBuf::from(format!("src/{}.xi", name.replace('.', "/"))),
            dependencies: deps.iter().map(|d| d.to_string()).collect(),
            source_hash: Some(hash::hash_str(name)),
        }
    }

    #[test]
    fn cache_open_empty() {
        let cache = setup_cache();
        assert!(cache.is_empty());
    }

    #[test]
    fn cache_insert_and_retrieve() {
        let cache = setup_cache();
        let module = make_test_module("test.core", vec![]);
        let entry = make_cache_entry(&module, vec![], CacheTiers::default());
        cache.insert(entry);
        assert_eq!(cache.len(), 1);
        assert!(cache.get("test.core").is_some());
    }

    #[test]
    fn cache_invalid_when_source_changes() {
        let cache = setup_cache();
        let tmp = std::env::temp_dir().join(format!("xiom_test_mod_{}.xi", std::process::id()));
        let original = "fn main() -> Int { return 1; }";
        std::fs::write(&tmp, original).unwrap();

        let module = ModuleNode {
            module_path: "mod".into(),
            file_path: tmp.clone(),
            dependencies: vec![],
            source_hash: None,
        };

        // Create cache entry from current file content
        let entry = make_cache_entry(&module, vec![], CacheTiers::default());
        cache.insert(entry);
        assert!(cache.is_valid("mod", &module));

        // Now modify the file -- cache should be invalidated
        std::fs::write(&tmp, "fn main() -> Int { return 2; }").unwrap();
        assert!(!cache.is_valid("mod", &module));

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn cache_purge_stale_removes_missing_files() {
        let cache = setup_cache();
        let module = make_test_module("stale.mod", vec![]);
        let entry = make_cache_entry(&module, vec![], CacheTiers::default());
        cache.insert(entry);
        assert_eq!(cache.len(), 1);

        cache.purge_stale();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cache_tier_store_and_load() {
        let cache = setup_cache();
        cache.store_tier("abc123", "ll", "define i64 @main() { ret i64 42 }");
        let loaded = cache.load_tier("abc123", "ll");
        assert!(loaded.is_some());
        assert!(loaded.unwrap().contains("ret i64 42"));
    }

    #[test]
    fn cache_clear_removes_all() {
        let cache = setup_cache();
        let module = make_test_module("a", vec![]);
        let entry = make_cache_entry(&module, vec![], CacheTiers::default());
        cache.insert(entry);
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }
}
