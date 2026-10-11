<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# 01 -- Incremental compilation and the query system: rustc vs XIOM

Study for XIOM Stage 6 (real incremental engine: content/signature fingerprints
replacing module_path+dep-names, L1 parse/L2 check cache tiers, cold/warm run
latency budgets, self-build wall time). rustc source: `E:\repos\rust` at commit
`7211ea594cbf137ef6118b1752b959dea7cb9088` (2026-10-11).

**Layout note:** this rustc commit has NO `compiler/rustc_query_system` crate.
The query executor lives in `compiler/rustc_query_impl/src/execution.rs`; the
shared query types are in `compiler/rustc_middle/src/query/`; the dep graph is
`compiler/rustc_middle/src/dep_graph/`; persistence is
`compiler/rustc_incremental/src/persist/`. Do not hunt for the historical crate
split.

## 1. Planning input

- [S6] A real incremental engine is a **tracked read-graph**, not a per-file
  mtime table: rustc records every dependency edge during query execution
  (`compiler/rustc_middle/src/dep_graph/edges.rs`) and recomputes the *same*
  graph shape on every run so edges can be compared session to session. XIOM
  has no edge set today; the dedup'd, ordered read list per task is the
  smallest viable unit to replicate, and it is cheap (inline buffer of 16,
  pooled recorder beyond that).
- [S6] **Node identity is (kind, stable hash of the key)** -- never a dotted
  name or path (`compiler/rustc_middle/src/dep_graph/dep_node.rs:99-133`).
  XIOM's `signature_hash` today is `hash(module_path + deps.join(","))`
  (`crates/xiom-graph/src/cache.rs:294-297`); swapping in a structural
  interface hash (exported signatures in canonical order) is the single
  highest-value change and the direct Stage 6 instruction.
- [S6] Green marking must be **recursive and self-healing**: a previous node is
  green when every previous dependency is green, and any unknown dep is either
  recursed into or *forced* (recomputed) to learn its color
  (`compiler/rustc_middle/src/dep_graph/graph.rs:943-1035`). A one-level
  "cache entry exists" check (XIOM today) cannot express this, so XIOM
  currently invalidates whole files or nothing.
- [S6] **Value fingerprints decide red vs green after a recompute**: if a node
  is re-executed and hashes to its previous value, it stays green and its
  dependents are not dirtied (`graph.rs:791-835`). This is what makes
  benign-edit rebuilds (comment/format-only) cheap; XIOM has no result hash
  to compare, so any root change is a full recompute.
- [S6] **Promoted roots / promotion**: on a hit, rustc allocates fresh
  current-session nodes that import the previous session's edges
  (`graph.rs:837-854`), and before writing the new cache it *promotes* every
  green disk value into memory so it is re-serialized as part of the new cache
  (`graph.rs:1062-1091`, called from `rustc_incremental/src/persist/save.rs:76`).
  Without this, an entry not touched in a session is lost; XIOM's save path
  today writes only when a full compile runs (`crates/xiom/src/lib.rs:2877-2932`).
- [S6] **Cache tiers are per-query opt-ins**, not a global setting:
  `cache_on_disk`, `eval_always` (never tracked/reused: the escape hatch for
  untracked global state), `no_hash`, `no_force`
  (`compiler/rustc_middle/src/query/modifiers.rs:24-93`). XIOM's `CacheTiers`
  bitmask is the right shape but only `l4_ir` is ever set; each tier needs a
  documented eligibility rule before it is populated.
- [S6] **Eviction/GC is a session-generation protocol**: private
  `-working` dirs, hardlink-or-copy from the newest finalized dir, publish by
  rename to `s-{timestamp}-{SVH}`, delete older finalized dirs and stale
  private dirs (`compiler/rustc_incremental/src/persist/fs.rs:14-78,42-51`).
  XIOM's `$HOME/.xiom` files are flat with a 100 MiB mtime FIFO for scripts
  (`crates/xiom/src/jit.rs:177-208`); there is no generation boundary to make
  "cache is consistent with source X" a checkable statement.
- [S7] **Self-build wall time** is exactly the workload where per-module
  reuse compounds: rustc's codegen reuse is per CGU work product, gated by a
  green `CompileCodegenUnit` node, with the symbol set re-hashed even when
  green (`compiler/rustc_codegen_ssa/src/base.rs:1256-1298`,
  `graph.rs:1104-1134`). The XIOM compiler's own crate graph (80+ crates)
  is a bigger instance of the stdlib-graph problem; the same object-level
  reuse that took hello warm 8.7s -> 0.97s (rtcache) applies per module.
- [S8-9] Before cross-crate/registry package graphs (post-split), identity
  must be session-independent and serializable: rustc's whole design point is
  that `DepNode`s survive sessions without rebasing NodeIds/DefIds
  (`dep_node.rs:1-48`), and the dep-graph load is gated by a single
  `dep_tracking_hash` of the command line (`rustc_session/src/options.rs:285`,
  `persist/load.rs:93-107`). Package-level caching across installs will need
  the same property for package identity.

## 2. How rustc does it

### 2.1 Node identity and fingerprints

- `DepNode { kind: DepKind, key_fingerprint: PackedFingerprint }`; the key
  fingerprint is a stable hash of the query key and is session-independent by
  construction (`compiler/rustc_middle/src/dep_graph/dep_node.rs:99-133`).
  Module docs list the advantages: no rebasing, `Copy`, mmap-able, nodes for
  deleted items still constructible (`dep_node.rs:1-48`).
- `Fingerprint` is 128-bit (`u64, u64`):
  `compiler/rustc_data_structures/src/fingerprint.rs:13`;
  `to_smaller_hash()` at `:46`; `PackedFingerprint` newtype at `:203`.
- Key shapes are typed and partially recoverable: `KeyFingerprintStyle`
  distinguishes `Unit` (no params), `DefPathHash`, `Opaque`; query keys
  implement `QueryKey: Copy + Debug + Eq + Hash + StableHash`
  (`compiler/rustc_middle/src/query/keys.rs:28-57`).
- `DepNode::construct` hashes the key with a `StableHasher`
  (`dep_node.rs:128-133`); `from_def_path_hash` is the common fast path
  (`:138-145`).
- `WorkProductId` is itself a stable hash of the CGU name
  (`dep_node.rs:237-257`).

### 2.2 The red/green algorithm

- `DepGraph::try_mark_green` (`graph.rs:909-939`): a node not present in the
  previous graph returns `None`; colors are memoized in a `DepNodeColorMap`
  (`Green(idx)/Red/Unknown`, `:127-131`); `Unknown` triggers a recursive walk.
- `try_mark_previous_green` (`graph.rs:943-1035`): iterate the previous edge
  list; green dep -> keep and continue; red dep -> fail; unknown dep ->
  recurse (`:957-989`); recursion failure falls back to
  `try_force_from_dep_node` (`:991-994`); forcing must set a color, and if the
  force produced errors the walk gives up (`:1005-1015`) because a failed
  session must not persist an inconsistent graph.
- Success promotes the node into the current graph with imported edges:
  `promote_node_and_deps_to_current` (`graph.rs:837-854`, call at
  `:1028-1029`). Multiple threads may race here; both the encode and the color
  map tolerate the same write twice.
- Re-executed nodes are colored by **value fingerprint comparison**
  (`alloc_and_color_node`, `graph.rs:791-835`): same value as the previous
  session -> green; different -> red; `no_hash` queries -> always red (`:809-815`).
  The value hash is computed from a `StableHashState` in
  `hash_result_and_alloc_node` (`:486-502`).
- Forever-red and zero-dep-anon singletons occupy fixed indices 0 and 1
  (`graph.rs:188-210`); anon node ids mix a per-session seed so ids cannot
  collide across sessions (`:1197-1207`, seed built at `:1215-1224`).
- Reads are recorded per task into a dedup'd structure: inline buffer of 16
  (`SMALL_READS_MAX`, `edges.rs:9`), linear dedup (`:30-53`), pooled recorder
  with an epoch filter beyond that (`:65-137`); `read_index` inserts into the
  current task's read set (`graph.rs:507-547`).

### 2.3 In-memory query cache and execution engine

- `QueryVTable` carries the provider, the in-memory cache, the disk-load and
  value-hash function pointers, and modifiers
  (`compiler/rustc_middle/src/query/system.rs:26-84`). A query caches on disk
  only when its kind says so and the key is local if `separate_provide_extern`
  (`system.rs:87-89`).
- Cache implementations are specialized per key: sharded hash map (default),
  `SingleCache` for `()`, `DefIdCache` with a dense `VecCache` for local ids
  and a hash map for foreign ids (`compiler/rustc_middle/src/query/caches.rs:17-191`).
- `try_execute_query` (`compiler/rustc_query_impl/src/execution.rs:277-360`):
  check in-memory cache under the shard lock; otherwise single-flight per key
  with an active-job guard that poisons the key on panic (`:87-159`); parallel
  waiters park on a latch (`:178-217`) with cycle/deadlock detection.
- Incremental path (`execute_job_incr`, `execution.rs:469-515`): unless
  `eval_always`, try green first; a green node loads its value from disk when
  available, else re-invokes the provider with dependency recording *ignored*
  (deps are already in place) and verifies the ICH
  (`load_from_disk_or_invoke_provider_green`, `:517-586`; miss path `:554-562`;
  verify `:565-583`).
- Forcing recovers the key from the key fingerprint via
  `Key::try_recover_key`; not every key is recoverable, and failure just makes
  red/green marking fall back to recomputation
  (`force_query_dep_node`, `execution.rs:667-687`; vtable wiring at
  `compiler/rustc_query_impl/src/dep_kind_vtables.rs:99-130`).
- `ensure_ok` queries can skip execution entirely on green
  (`ensure_can_skip_execution`, `execution.rs:594-626`).

### 2.4 Persistent cache: dep graph, query values, work products

- **Dep graph serialization** is streaming and compact: nodes are written as
  they are allocated and not kept in memory; fixed-size bit-packed headers
  (DepKind + edge count + edge byte-width) and varint edge lists; the node and
  edge counts live in the last 16 bytes; the reverse index (DepNode -> index)
  is built lazily per DepKind (`compiler/rustc_middle/src/dep_graph/serialized.rs:1-40,97-150`).
  `SerializedDepGraph` holds nodes, per-node value fingerprints, edge headers,
  edge data, a lazy reverse index, and `session_count` (`serialized.rs:97-128`).
- **Query values** go to a separate mmap'd file with a footer index
  (`OnDiskCache`, `compiler/rustc_middle/src/query/on_disk_cache.rs:51-97`;
  `Footer` `:99-114`; `new` `:150-182`). On save, `exec_cache_promotions` loads
  every green disk-cached value back into memory first so it is re-serialized
  (`graph.rs:1062-1091`; `persist/save.rs:63-85`), then the mmap is dropped and
  the file re-written (`on_disk_cache.rs:199-207`).
- **Save is refused when compilation produced errors** (`persist/save.rs:33`);
  dep-graph and query-cache writes happen in `par_join`
  (`save.rs:45-88`); the graph is renamed from its staging path
  (`load.rs:219-228` staging writer).
- **Load is gated** by a command-line `dep_tracking_hash`; a mismatch discards
  everything (`persist/load.rs:45,93-107`). Work products are reconciled at
  load: if any file of a work product is missing, the whole load is
  `DataOutOfDate` (`load.rs:49-80`). Corrupt files warn and fall back to an
  empty cache (`load.rs:121-147`).
- **Verification**: on load, a deterministic rotating 1/32 subset of values
  (`hash % 32 == session_count % 32`) is re-hashed and compared, always with
  `-Zincremental-verify-ich`; mismatch panics with "Found unstable
  fingerprints" (`compiler/rustc_query_impl/src/incremental.rs:82-103`;
  `compiler/rustc_middle/src/verify_ich.rs:14,82`). Query-key hash uniqueness
  is asserted in debug/verify mode (`incremental.rs:46-80`).
- **Side effects** (diagnostics, used features) are stored as dep nodes with
  their own serialized payloads and replayed when the owning node is forced
  green (`QuerySideEffect`, `graph.rs:43-55`; `record_diagnostic` `:552-563`;
  `force_side_effect` `:749-789`; encode `:735-747`). This is how cached clean
  results still re-emit their warnings.

### 2.5 Session directory lifecycle and garbage collection

- Protocol documented in `persist/fs.rs:14-78`: allocate a private
  `s-{timestamp}-{random}-working` dir; find the newest finalized dir and take
  a shared lock; write privately, hardlinking unchanged files from the old
  session; on success rename to `s-{timestamp}-{SVH}`; then delete the old
  finalized dir. Lock files make crashed sessions reclaimable; dirs younger
  than a few seconds are not collected (`:66-78`).
- GC policy: delete private dirs not locked, and all finalized dirs per crate
  except the most recent (`fs.rs:42-51`; implementation `:559-771`,
  `delete_old` `:773-788`). GC runs on every incremental start
  (`load.rs:194-201`) and after finalize (`fs.rs:356`).
- `finalize_session_directory` locks in the publish step: rename to the crate
  SVH (`fs.rs:274-356`).

### 2.6 Work products and per-CGU codegen reuse

- A `WorkProduct` is a named set of files (object, bitcode, dwo) keyed by
  `WorkProductId`; docs explain the extra hash beyond dirty edges: the symbol
  set of a CGU is re-hashed next session and compared, because new
  monomorphizations can appear even when nothing the CGU read changed
  (`graph.rs:1104-1134`).
- `determine_cgu_reuse` (`compiler/rustc_codegen_ssa/src/base.rs:1256-1298`):
  no previous work product -> compile; otherwise `try_mark_green` on the
  `CompileCodegenUnit` dep node decides pre-/post-LTO object reuse.
- Work products are moved into the session dir with `link_or_copy`
  (hardlink first): `persist/work_product.rs:20-51`; the index is saved and
  debug-asserted to still exist (`persist/save.rs:93-123`).
- Incremental builds force per-module CGU granularity and mark generic fns
  volatile, trading optimization for reuse (`compiler/rustc_monomorphize/src/partitioning.rs:201-276,423-470`).

### 2.7 Driver wiring

- `setup_dep_graph` (load previous graph + work products, start streaming the
  new graph) and `load_query_result_cache` are called during early setup
  (`compiler/rustc_interface/src/passes.rs:958-979`).
- After codegen: `save_work_product_index`, drop the dep graph,
  `finalize_session_directory` (`compiler/rustc_interface/src/queries.rs:114-128`).
- `-Z` knobs worth knowing: `incremental-info`, `assert-incr-state`,
  `incremental-verify-ich`, `disable-incr-comp-backend-caching`
  (`persist/load.rs:151-173`, `base.rs:1258`).

## 3. How XIOM does it today

All paths verified against this worktree; no guessing.

### 3.1 CacheDb: five tiers designed, one used

- `crates/xiom-graph/src/cache.rs:9-17` documents L1 tokens, L2 AST, L3
  checked, L4 IR, L5 object; `CacheTiers` bitmask at `:51-59`. Grep across
  `crates/` finds `l1_tokens`/`l2_ast`/`l3_checked`/`l5_object` only in the
  struct definition; both write sites set `l4_ir` only
  (`crates/xiom/src/lib.rs:2902-2903,2918-2921`). The other four tiers are
  designed-but-unused.
- `CacheEntry` stores `module_path`, `file_path`, `Fingerprint{source_hash,
  signature_hash}`, `cache_key`, dep/dependent name lists, `last_compiled`
  (`cache.rs:30-49`). The index is JSON (`index.json`, `:94-109`); per-tier
  payloads are separate files `{key}.{ext}` (`:154-164`).

### 3.2 Fingerprints and invalidation

- `Fingerprint` = SHA-256 hex strings; `differs_from` requires both signature
  hashes non-empty (`crates/xiom-graph/src/hash.rs:44-77`).
- `make_cache_entry` computes `signature_hash = hash(module_path + ":" +
  dependencies.join(","))` (`cache.rs:294-297`): it fingerprints the *import
  list*, not the exported interface. A body-only edit in a dependency does not
  change it; an import reordering does.
- `CacheDb::is_valid` compares the module's own source hash and checks that
  dependency *names* exist in the map (`cache.rs:120-143`) -- it does not
  compare dependency fingerprints, and nothing walks transitively.
- `CacheDb::insert` invalidates dependents when both old and new signature
  hashes are non-empty and differ (`cache.rs:170-207`), but the file cleanup
  inside the walk deletes tier files using the **new** entry's `cache_key`
  (comment "We use the current key as a fallback", `:186-189`), so stale
  payload files of the invalidated dependents are not actually removed.
- `max_entries = 1000` with timestamp-based GC (`cache.rs:71-73,230-248`);
  never sized for a stdlib-scale graph.

### 3.3 Driver wiring and scope

- `incremental_check` (`crates/xiom/src/lib.rs:2835-2873`) rebuilds the whole
  project graph (`build_project_graph`, manifest walk + header parse) just to
  map the file to a module key, then compares the root file's source hash and
  `l4_ir` flag and returns the cached IR string. The doc comment claims
  "All dependencies are still valid" (`:2828-2832`), but no dependency check
  runs -- only the root file hash is compared.
- The call site checks only `effective_sources[0]` (`lib.rs:396-405`) and the
  save site also saves only source 0 (`:753-756`, `incremental_save`
  `:2877-2932`, which rebuilds the graph a second time at `:2894`).
- `incremental` is off by default (`CompileConfig::incremental` at
  `lib.rs:83,155`), opt-in via `--incremental` (`crates/xiom/src/main.rs:1841`)
  or `[compiler] incremental` in the manifest (`main.rs:2186-2187`). The
  `--cache` binary/script cache is a different, default-on mechanism
  (`main.rs:2673`).
- When a hit happens, the driver returns IR text directly and skips
  lex/parse/check/codegen entirely (`lib.rs:399-404`).

### 3.4 The caches that do work today

- **Script/binary cache** (`crates/xiom/src/jit.rs`): key = `xiom-cache-v3`
  identity (pkg version + debug + target, `:94-103`) + effective opt level +
  SHA-256 of the full script source (`:115-122`); `--run --cache` and the
  `--jit --cache` lane serve the cached binary (`lib.rs:782-807`,
  `main.rs:1669-1780`); 100 MiB eviction by mtime (`jit.rs:177-208`).
- **Runtime object cache** (`crates/xiom/src/rtcache.rs`, Stage 6 landed):
  64-bit FNV-1a key over clang path + version + compile flags + content hash
  of every `stdlib/runtime/*.c` (`:14-16,74-99`); links cached `.o` files,
  falls back to the historical single-invocation compile on any failure.
  Measured hello warm 8.7s -> 0.97s (STAGE6_PERF_PLAN.md:224-228).
- **Catalog index-header cache** (`crates/xiom-check/src/catalog.rs`):
  per-file `(mtime_ns, size) -> parsed header` rows in `$HOME/.xiom/catidx.txt`;
  identity = `xiom-catalog-index v1|<pkg>|<OS>-<ARCH>|<ptr bits>`
  (`:340-350`); load `:352-386`; save with >60k prune-to-touched `:388-420`;
  warm index 0.70s -> 0.32s with hits=2397/2397 (STAGE6_PERF_PLAN.md:233-238).
  This cache holds headers only, not tokens/ASTs.
- **Catalog-body check cache (m252)** (`crates/xiom-check/src/checkcache.rs`):
  positive-only; key = dotted module name (or `#<hash>`); value = 64-bit
  module content hash + tree digest + `catalog_resolved_calls` replay pairs
  (`:46-60,107-152`). Identity `xiom-check-body-cache v1|<pkg>|<OS>-<ARCH>|<bits>`
  (`:31-44`). The tree digest is FNV-1a over sorted `(path, mtime, size)` of
  every indexed file (`catalog.rs:428-448`) -- **any** indexed change
  invalidates **every** entry (comment at `crates/xiom-check/src/lib.rs:618-622`
  calls it "a conservative superset of the import closure changed
  condition"). Eligibility guards: clean-only, no global bare-slot fallback,
  no name overlap with the user program (`lib.rs:623-682,776-783`). Measured
  effect: 31 bodies checked -> 17 checked + 14 replayed; check 1.051s ->
  0.911s warm (STAGE6_PERF_PLAN.md:554-558).
- **LSP parse cache** (`crates/xiom-lsp/src/backend.rs:103-108`): per-uri
  parsed `Program` keyed by a 64-bit `DefaultHasher` of the text
  (`:145-149`); only a changed document re-parses. The hash is
  non-cryptographic and the key has no parser/compiler identity component.
- **Where the time goes** (lz4 smoke, true cold, release): wall 2.94s;
  index 0.576s; check 1.148s (catalog-bodies 0.393s, capture 0.118s);
  borrow 1.235s; codegen 1.327s; clang+link ~1.6s; arena box 7-8s is the
  same profile inflated (STAGE6_PERF_PLAN.md:472-486,344-357).

### 3.5 Summary of the gap

XIOM has strong whole-artifact caches (script binary, runtime objects,
headers, clean bodies) and a designed-but-dead tier table. It lacks: a
structural interface fingerprint, a tracked dependency edge set, recursive
green marking, promotion of untouched entries, a session generation, side
effect replay for anything except resolved calls, and any ICH-style
verification of hits.

## 4. Gap table

Key: P1 = first slice (unblocks Stage 6 budgets), P2 = next, P3 = later.
Effort: S/M/L.

| pattern | rustc ref | XIOM state | gap | stage | effort | priority |
|---|---|---|---|---|---|---|
| Node identity (kind + stable key hash) | dep_node.rs:99-133 | module_path+deps string hash (cache.rs:294-297) | key is name-derived, not structural | S6 | M | P1 |
| Structural interface fingerprint (exports) | keys.rs:28-57; dep_node.rs:128-133 | signature_hash covers import list only | body edits in deps invisible | S6 | M | P1 |
| Recursive green marking + forcing | graph.rs:909-1035 | one-level `is_valid`, no recursion, no force | imprecise invalidation | S6 | L | P1 |
| Value fingerprint after recompute | graph.rs:791-835 | none | any root change = recompute | S6 | M | P1 |
| Promotion of green nodes + disk values | graph.rs:837-854,1062-1091 | entries only written on full compile | untouched warm entries lost | S6 | M | P1 |
| Dependency edge set with dedup | edges.rs:9-137 | dep name lists only | no per-task reads; no edge compare | S6 | M | P2 |
| Per-query disk eligibility (modifiers) | modifiers.rs:24-93; system.rs:87-89 | tier bitmask dead except l4 | L1/L2/L3/L5 never populated | S6 | M | P1 |
| Cmdline/toolchain hash gates load | load.rs:45,93-107; options.rs:285 | pkg/OS/arch ids in catidx + checkcache | flags/opt-level absent from those keys | S6 | S | P1 |
| Per-module cache scope in driver | passes.rs:958-979; execution.rs | only sources[0]; graph rebuilt twice | multi-file projects uncached | S6 | S | P1 |
| Work product reuse (objects) + symbol recheck | base.rs:1256-1298; graph.rs:1104-1134 | rtcache for C runtime only | program/stdlib objects never reused | S6/S7 | L | P1 |
| Rotating ICH verification of hits | incremental.rs:82-103; verify_ich.rs:82 | parity locks in tests only | no runtime self-check of hits | S6/S7 | S | P2 |
| Session-generation GC | fs.rs:14-78,42-51 | flat $HOME/.xiom files + 100MiB FIFO | no consistent cache generation | S6 | M | P2 |
| Side-effect replay (diagnostics) | graph.rs:43-55,552-563,749-789 | resolved-call replay only | cached warnings/errors story incomplete | S6 | M | P2 |
| eval_always escape hatch | modifiers.rs:46-57,82-86 | none | untracked-state bodies unsafe to cache | S6 | S | P2 |
| Item-level keys / dense local cache | caches.rs:135-191 | whole-module keys only | no body/item granularity | S6 | L | P2 |
| Query cycles + single-flight | execution.rs:277-360 | n/a (batch); LSP mutex | LSP concurrency could improve | S7 | L | P3 |
| Compact serialization + lazy reverse index | serialized.rs:1-40,97-150 | JSON index + text payloads | load/store cost and size | S6 | M | P3 |
| Cache key entropy for LSP | (rustc: stable 128-bit hashes) | 64-bit DefaultHasher (backend.rs:145-149) | collision and stale-parser risk | S6 | S | P1 |
| Errors block persistence | save.rs:33; graph.rs:1005-1015 | checkcache positive-only | keep invariant when tiers grow | S6/S7 | S | P1 |

## 5. Optimization and performance opportunities

Tied to the recorded numbers (STAGE6_PERF_PLAN.md) and Stage 6 KPIs (arena
compile under 2s; `run -e` under 1s warm / 3s cold; scripting warm
10-20ms; self-build wall time).

1. **Unlock partial-edit warmth (biggest honest lever).** Today warm reuse
   requires either the identical root file (`incremental_check`) or the
   identical script text (jit cache). Per-module structural fingerprints plus
   a dependency edge set turn a one-module edit into "recheck that body,
   re-check dependents whose interface hash changed, reuse the rest". The m252
   numbers (0.14-0.3s off check per warm run) are the floor; the ceiling is
   the whole catalog-body window (0.39-0.62s) once the guard exclusion is
   removed by recording, per bare name, the resolved binding fingerprint
   (already the documented hit-rate follow-up).
2. **Promotion prevents warm decay.** Rustc re-serializes every green value so
   caches age well. XIOM writes `.xi_cache` entries only during full compiles
   and only for source 0; a dev loop that alternates files thrashes. Store all
   modules on every compile (cheap JSON + hash) and treat the cache as a
   session snapshot, not a per-file side effect.
3. **Object-level reuse is the structural cold lever.** The rtcache pattern
   (key = toolchain + flags + content hashes; link cached objects) took hello
   from 8.7s to 0.97s warm. rustc's CGU work-product reuse plus the
   symbol-set re-hash (graph.rs:1104-1134) is the correct model for the
   planned precompiled-stdlib slice: reuse per module/partition, re-verify the
   reachable symbol set each session, never trust only "nothing changed".
   This is the only path from the ~1.6s clang+link floor (perf-relevant even
   when check is 0).
4. **Check tiers at the body level are already safe-ish; extend with ICH.** A
   rotating 1/32 re-verification of served cache entries, deterministic by
   cache-key hash (rustc: `incremental.rs:82-103`), is a cheap way to keep
   raising hit rates without weakening the parity locks. On mismatch: warn in
   dev, fail the lock in CI, delete the entry.
5. **Kill the double graph build.** `build_project_graph` runs in both
   `incremental_check` and `incremental_save`; on a warm compile that is pure
   overhead before any cache decision. Compute once per invocation and reuse.
6. **Session-directory semantics for `$HOME/.xiom`.** Replace mtime-FIFO with
   content-addressed entries plus "keep the newest consistent generation"
   GC. This also gives LSP a natural "cache matches source snapshot X"
   boundary and makes corruption a clean fallback rather than mixed state.
7. **Self-build wall time** benefits from the same machinery at larger scale:
   per-crate fingerprints (crate source closure + interface) and per-crate
   object reuse map directly onto rustc's `--emit` reuse; the current
   all-or-nothing recheck of the `xiom` crate graph is the dominant cost for
   both the component compiler and the selfhost build.

## 6. Adoption proposals

1. **xiom-graph/src/hash.rs + cache.rs -- structural interface hash.** Add
   `interface_hash(module_signatures: &str) -> String` and compute
   `signature_hash` from the module's exported declarations (name, generic
   params, parameter/return types, visibility) in canonical sorted order;
   include it in every entry, not just the root. Mirrors rustc's
   key-fingerprint rule (dep_node.rs:99-133).
2. **xiom-graph/src/cache.rs -- fix invalidation.** In `CacheDb::insert`,
   delete invalidated dependents' tier files with the *old* entry's
   `cache_key` (cache.rs:186-189), walk dependents transitively with a visited
   set (there is a local `visited` but file cleanup uses the wrong key), and
   make `incremental_check` require the full validity walk instead of the
   root hash compare (xiom/src/lib.rs:2850-2863).
3. **xiom/src/lib.rs -- per-module, once-per-invocation driver wiring.**
   Thread the module list from `expand_sources_with_graph` through
   check/save; compute `build_project_graph` once; attempt `incremental_check`
   per module (check stage) and save L4 per module (codegen stage), keeping
   the current whole-program fallback.
4. **xiom-check/src/checkcache.rs + catalog.rs -- replace the tree digest with
   module + closure fingerprints.** Key = (module content hash, import-closure
   interface-hash set, compiler identity); an edit to one module then
   invalidates only its dependents, raising the m252 hit rate without touching
   the parity/IR locks. Keep positive-only + guard rules verbatim.
5. **xiom-check/src/catalog.rs -- keep catidx as metadata-only, but add a
   parser-identity component to the LSP key.** Wire the planned L2 tier by
   serializing a per-module token/AST blob beside the header, keyed by
   (header hash, parser identity), loaded before lex/parse in the driver
   (xiom/src/lib.rs:407-500 parse stage). This is lower priority than check
   tiers (parse is 0.003s cold) but pays in LSP and self-build.
6. **xiom/src/jit.rs + xiom-lsp/src/backend.rs -- one key helper.** Expose the
   jit cache identity/key scheme (jit.rs:94-122) through `xiom-graph::hash`
   and use it for the LSP parsed-document cache (replace the 64-bit
   `DefaultHasher`, backend.rs:145-149) and future tier blobs; reserve
   SHA-256 for correctness-critical keys.
7. **NEW xiom/src/objcache.rs (or extend rtcache.rs) -- per-module object
   cache.** Key = compiler identity + clang identity + flags + module content
   hash + interface hash; store `.ll`/`.o` per module; reuse at link time when
   every input matches. This is the precompiled-stdlib/separate-compilation
   slice the plan already prioritizes (STAGE6_PERF_PLAN.md:327-329), and the
   rtcache implementation is the template.
8. **Eviction -- session generations.** Move `$HOME/.xiom` caches to
   generation subdirectories (`gen-<timestamp>-<id>/`) published by rename and
   keep only the newest consistent generation, rustc-style
   (persist/fs.rs:14-78); keep the last N for LSP. For `.xi_cache/`, keep
   `last_compiled` GC but size it for stdlib graphs and make it a background
   sweep, not an in-line one.
9. **Verification -- rotating replay audit.** Extend the m252 lock suite with
   a warm-vs-cold parity re-run of a deterministic 1/32 subset of cached
   bodies (hash of cache key % 32 == run counter % 32), asserting
   byte-identical diagnostics and resolved-call tables, plus entry removal on
   mismatch. rustc reference: incremental.rs:82-103, verify_ich.rs:82.
10. **Side effects -- tagged replay records.** When the check cache grows
    beyond resolved calls (warnings, feature use), store each as a tagged
    record alongside the verdict (rustc `QuerySideEffect`, graph.rs:43-55) so
    a replay re-emits exactly what a live check would have.

## 7. Open-item relevance

- **REL-20261010-1854** (unused fn -> 5.6x bloat + malformed TLS accesses;
  COMPILER_BUGS 2026-10-10): rustc emits only reachable, instantiated mono
  items and its CGU reuse re-verifies the symbol set even when the dep node is
  green (graph.rs:1104-1134; base.rs:1256-1298). Any XIOM cache tier at or
  above codegen MUST treat the reachability set as a tracked input. If
  per-module IR/objects are cached without that, this exact bug class hides
  behind a cache hit and the green artifact (341 KB) and the crashing artifact
  (1.9 MB) cannot be distinguished by source hash: the unused helper changes
  the emitted set, not any cached module's source. Land the reachability fix
  (RUSTC-REFERENCE section 7) before introducing object-level reuse.
- **REL-20261010-1900** (W004 cascades after unresolved-module recovery):
  rustc never persists a session that produced errors (persist/save.rs:33)
  and abandons green marking when a forced dependency produced errors
  (graph.rs:1005-1015). XIOM's m252 cache is positive-only for the same
  reason; when L3 expands, keep "errors => no cache write" and "cached
  verdicts are clean-only" as invariants, ideally enforced in the cache
  module itself rather than at each call site. ErrorGuaranteed-driven cascade
  suppression also keeps the clean/error verdict itself stable, which is what
  a positive-only cache depends on.
- **REL-20261010-1904** (fn-ptr casts returning garbage for Float64): the
  direct link is ABI derivation from types (RUSTC-REFERENCE section 4), but
  the incremental-relevant rule is the same: identities and encodings must be
  derived from type structure, never printed names. XIOM's
  `signature_hash = hash(module_path + deps)` is the name-derived pattern that
  rustc's `DepNode`/`StableHash` design avoids; fixing 1904 and the fingerprint
  scheme are both instances of "type ground truth, not names".

## 8. Sources

XIOM (this worktree; read, not modified):
- docs/STAGE6_PERF_PLAN.md (all sections; measurements and m252 notes)
- docs/COMPILER_READINESS_PLAN.md (Stage 5 LSP entry, Stage 6, Stage 7)
- docs/ROADMAP.md (Stage 6/7 entries, 7B cache feature; searched)
- docs/RUSTC-REFERENCE.md (read from the main tree; absent in this worktree)
- crates/xiom-graph/src/cache.rs, hash.rs, lib.rs
- crates/xiom/src/lib.rs (incremental regions 396-405, 753-756, 2780-2932;
  binary-cache call site 780-807), jit.rs, rtcache.rs, main.rs
- crates/xiom-check/src/checkcache.rs, catalog.rs (index cache + tree digest),
  lib.rs (flush_catalog_bodies guards, 198-784)
- crates/xiom-lsp/src/backend.rs (parse cache)

rustc (`E:\repos\rust`, commit 7211ea594cbf137ef6118b1752b959dea7cb9088):
- compiler/rustc_middle/src/dep_graph/dep_node.rs, graph.rs, edges.rs,
  serialized.rs, mod.rs
- compiler/rustc_middle/src/query/system.rs, caches.rs, keys.rs, modifiers.rs,
  on_disk_cache.rs
- compiler/rustc_middle/src/verify_ich.rs
- compiler/rustc_query_impl/src/execution.rs, incremental.rs,
  dep_kind_vtables.rs
- compiler/rustc_incremental/src/persist/mod.rs, load.rs, save.rs, fs.rs,
  work_product.rs, clean.rs (rustc_clean assertion harness)
- compiler/rustc_codegen_ssa/src/base.rs (determine_cgu_reuse)
- compiler/rustc_monomorphize/src/partitioning.rs (incremental partitioning;
  searched)
- compiler/rustc_interface/src/passes.rs, queries.rs (wiring; searched)
- compiler/rustc_session/src/options.rs (dep_tracking_hash)
- compiler/rustc_data_structures/src/fingerprint.rs
