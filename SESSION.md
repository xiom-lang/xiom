# XIOM Session Handoff — 2026-08-07 15:55

## ⚠️ CRITICAL CONSTRAINTS
1. **NEVER commit/modify `xiom-benchmark-chaos/`** — owner works in a PARALLEL session (it has uncommitted changes). It now COPIES `stdlib/` directly into its build (stdlib-pin was deleted per user). Stage only: `crates/`, `stdlib/`, `docs/`, `tests/`, `examples/`, `selfhost/` (except `_diff_*`), `packages/`.
2. **Parallel session rebuilds `target/debug/xiom.exe` frequently** — verify failures by recompiling the specific smoke manually before assuming regression.
3. **Selfhost tests are IGNORED** (user directive): `test_selfhost_bootstrap_v050` + all `full_diff_tests.rs` — `#[ignore]`. Do not touch.
4. **Models**: user confirmed ALL agents/subagents run DeepSeek V4 FLASH (`deepseek-v4-flash/deepseek-v4-flash`), verified live via subagent self-report. NEVER use the pro model for agent work. `subagent_variant_overrides`: flash → high.
5. **Monorepo split is DEFERRED** (user: "leave it for now"). No shims/deprecation needed pre-public. stdlib WILL become its own repo later — `stdlib/xiom/test.xi` (+ test harness) moves to compiler-side on split (backlog).

## CURRENT BRANCH
`feat/architect` — HEAD `fbbfd712`

## TEST SUITE STATE (verified 2026-08-07)
| Suite | Result |
|-------|--------|
| checker | 156/156 |
| parser | 96/96 |
| feature-reg | 510/510 |
| integration | 128/128 |
| stdlib-compile | 40/40 |
| stdlib-exec | 57/57 (incl. 16 Tier-2 module smokes) |
| API-freeze gate | 2/2 (snapshots ~2,328 pub sigs) |
| E2E gates | 23/23 (m21_borrow, m33_b14, eco_algo, safety, m35, wasm) |
| E2E full | 2,230/2,231 — `e2e_spawn_capture` FLAKY (pre-existing, verified at HEAD too: detached spawn threads race CRT stdio on exit; fix = runtime join-on-exit, NOT stdlib) |

## STDLIB STATE: 60 modules, 2,328 pub fns (was 51/~1,300)

### Completed & committed (chronological)
1. `c3887591` — Tier 1: deleted 8 orphan modules (b64, hex, random, runner, types, pbkdf, ed25519, demo); API-freeze test gate added (snapshots every pub fn signature; additive-only enforcement).
2. `913d60fc` — Tier 2: 16 NEW modules: sort (10 algos), search (8), bits (24), geom (58: Vec2/3/4, Quat, Mat2/3/4, AABB/Sphere/Ray), complex (23), bigint (28), chacha (11), poly1305 (9), ecc (19), rsa (12), des (6), utf8 (11), platform (14), debug (10), misc (11: glob/levenshtein/semver/soundex/natural), process (8) + 16 CI smokes + **compiler fix** (`resolve_module_call` leaf-first for dotted receivers — fixed crypto.rsa_encrypt vs rsa.rsa_encrypt collision) + runtime `xiom_getpid`.
3. `ef144ae5` — Tier 3: +600 fns across 30 modules via 5 parallel agents: string 45 (Unicode-aware), char 42 (full categories), fmt 28 (table/wrap/hexdump), regex 31, time 66 (**Date + ISO8601 + Zeller**), io 53, os 61, env 29, path 28, collections 127, iter 44, num 73 (primes/modular/base-conv), cmp 22, array 29, hash 23 (**fnv/murmur3/xxhash32 verified vs test vectors**), compress 36 (LZ77/Huffman), encoding 35 (base32/percent), serialize 41 (JSON pretty/minify/path + varint), rand 36 (Xorshift64 + uuid_v4/v7), stats 31 (covariance/regression/histogram), core 92, error 11, reflect 24, test 30, bench 17, log 38, sync 66 (Semaphore/Barrier/CountDownLatch/atomics), async 29, thread 22, net 29 (IPv4/URL parse).
4. `3cdcf57e` — Phase 4: hardware popcnt/clz/ctz (runtime C with SWAR fallback), SIMD `mem_copy/set/compare` in mem.xi; SHA-NI/AES-NI/SSE2 asm already in place (crypto_x86_64.asm, mem_x86_64.asm, context_switch.asm).
5. `f9ffb8c0` — removed `stdlib-pin/` (user: benchmark-chaos now copies stdlib/ directly).
6. `fbbfd712` — **production crypto/ECC/OS layer** (agent work, verified green): real FIPS 46-3 DES + 3DES (des.xi rewritten, FIPS vector 0x85E813540F0AB405), crypto.xi +860 lines (real BLAKE3, HKDF-SHA256 RFC 5869, ChaCha20-Poly1305 AEAD RFC 8439, SHA-224/384, X25519), ecc.xi +118 (production secp256k1/Ed25519/X25519 delegating to runtime C), xiom_runtime.c +1,192 lines (256-bit field arithmetic, Ed25519 sign/verify RFC 8032, hostname, process spawn/kill/wait/running, os_version), collections.xi +347 (comparator sorts etc).

## PLANS (docs/STDLIB_EXTENSION.md)
- **§10 IMPLEMENTATION STATUS**: all original phases (0-4) complete; 5 known compiler bugs documented (Result[Vec].value corruption, chained .method on qualified Str returns, Bool→Int cast, is Ok+.value broken, Option/Result match-arm mixing).
- **NEW §"CROSS-REFERENCE & PRODUCTION PLAN v2"** (after the raw ~2,387-entry list): every entry categorized HAVE/STDLIB/PACKAGE + folder-grouping audit + package README assignments (§9 table) + execution order (§10).
- **Folder modules VERIFIED working**: `use xiom.foo.bar` → `stdlib/xiom/foo/bar.xi` (catalog strategy a, tested exit 42). Refactor plan: num (2,612 lines) → num/{checked,convert,int128,fraction,base,float}; crypto → crypto/{...}; geom → geom/{vec,mat,quat,collision}; collections → collect/{tree,heap,cache,hash,queue,graph}; net → net/{http,url,dns,proto}; os → os/{fs,ioctl,proc,term,mmap}; io → io/fs; ffi → ffi/dl; fmt → format/{number,text,dump}. Flat contract files stay as aggregates (freeze gate).
- **Numeric tower**: concrete per-width functions (generic operators corrupt Float64 — compiler bug; `impl` blocks parsed but IGNORED by codegen — interface dispatch on generics is a hardening backlog item). num.xi already has i64/u32/etc checked/sat/Int128/Fraction per the plan.

## NEXT SESSION — TODO (in priority order)
1. **Package README phase** (user's explicit ask): for EVERY package folder in `packages/` (68 real + ~257 stub placeholders) write/update `README.md` with: name + one-line scope + the libs inventory from the STDLIB_EXTENSION.md §9 table (names // one-line description ONLY, no full specs). New packages to create folders for: xiom-charts, xiom-diagrams, xiom-geo, xiom-metadata, xiom-audio-meta, xiom-finance, xiom-stats-tests, xiom-stats-ml, xiom-ml, xiom-timeseries, xiom-geom3d, xiom-imaging, xiom-pki, xiom-logging, xiom-apple, xiom-windows, xiom-packet, xiom-wireless, xiom-cloud, xiom-messaging, xiom-streaming, xiom-discovery, xiom-legacy-proto, xiom-c-binding, xiom-text-markup, xiom-typography, xiom-aviation (+ mqtt, amqp, nats, pulsar, memcached, mysql, mongo, elastic, odbc, socks, tor, i2p, curl, git2, ssh2, xml2, expat, yaml, jansson). Stdlib = zero deps; packages may depend on stdlib + wrap C; DON'T pre-plan package internals beyond README names.
2. **Folder refactor** of the 4 files >1,000 lines (num, crypto, geom, collections) per the audit — keep flat files as aggregates, add folder modules for new code. Verify freeze gate after.
3. **Fill stdlib gaps** (all ⬜ STDLIB entries in the v2 plan): hashing adds (city/highway/spooky/metro/jenkins/t1ha/farm/superfast/crc64), collections adds (avl/rbtree/fheap/skiplist/trie/bloom/cuckoo/graph/unionfind/kdtree/queues), string/text adds (damerau/jaro/metaphone/ngram/cosine/lcs/Unicode normalize/bidi → text/ folder), conversion adds (base58/62/ascii85/punycode/idna/shell-escape/strptime), net adds (http headers/url/dns records/ntp/jsonrpc/sse), os adds (fs/stat/perm/termios/mmap/exec variants/dynamic loading), rand adds (MT19937/PCG/ChaCha).
4. **Compiler hardening backlog** (stdlib surfaces the bugs; fix in compiler): (a) interface `impl` dispatch on generic params (`x.add(x)` fails — parser handles ImplDecl, checker+codegen ignore it); (b) generic operator monomorphization corrupts Float64 (`add2[T](a+b)` garbage for floats); (c) Result[Vec].value accessor; (d) chained .method on qualified Str returns; (e) Bool→Int cast; (f) stdlib/xiom/test.xi moves to compiler repo on monorepo split.
5. **Verify**: cargo test full (freeze + stdlib + feature-reg + integration + checker + parser + E2E gates) after each step.

## KNOWN COMPILER BUGS (stdlib works around; fix in compiler later)
1. `Result[Vec[T], _]` payload corrupted when MANY modules with Vec[UInt8] fns combine — `.value` returns garbage; `match { Ok(v) }` works in small programs. Use match.
2. `.method()` chained on module-qualified Str-returning calls emits inttoptr i64→i8* of a ptr — bind to var first.
3. `is Ok` + `.value` on Result[Vec] broken — use match.
4. Bool→Int cast unsupported — use if/else.
5. Match arms must match type: Option→Some/None, Result→Ok/Err (mixing → out-of-bounds GEP).
6. Generic operators corrupt Float64 in monomorphized bodies (numeric tower must be concrete per-width).
7. Interface `impl` blocks are parsed but IGNORED by checker+codegen (traits are declare-only until hardening).
8. CG02 DEBUG prints in crates/xiom-codegen/src/types.rs (pre-existing, always-fire in global_const_init — types.rs not in allowed commit list; remove if noise matters).

## IMPORTANT SEMANTIC FINDINGS (still valid)
- `&T` params carry the ADDRESS as i64 (NOT the value). `*r` must inttoptr+load.
- `&mut T` / `*T` are real pointers (i64*).
- `&Vec[T]`/`&Slice[T]` params are the Vec STRUCT by value (%struct.Vec).
- `&[N]T` fixed-array params are the Vec DATA pointer (i64*).
- Injected stdlib free fns are leaf-qualified (`array.contains`); bare internal calls resolve via `MonoContext::bare_fn_aliases` (keep-first).
- User program free fns SHADOW same-leaf stdlib fns at injection time.
- `use xiom.foo.bar` resolves `stdlib/xiom/foo/bar.xi` (folder modules VERIFIED).
- All 13 numeric primitives exist (Int8/16/32/64, UInt8/16/32/64, Float32/64) with working concrete arithmetic.
- `[T: Ord]` generics work ONLY for algorithms using `.compare`/`.eq` (builtin inline scalar dispatch).

## AGENT WORKFLOW NOTES (proven in this session)
- Launch `general` subagents with: exact file paths, VERIFY-MANDATORY (compile+run smoke with known test vectors, exit 0), syntax rules (no for loops, while only, if/elif/else, Some/None vs Ok/Err, no Bool→Int casts, bind-qualified-Str-before-method, struct literals with semicolons, const arrays `const _X: [64]Int = [...]` works, write WITHOUT BOM).
- Agents write files even when the session reports "aborted"/"connection reset" — ALWAYS verify with cargo test after fan-out before committing.
- fn-typed params (fn(&T) -> Bool) sometimes crash at runtime in free functions — mirror iter.xi's working pattern; test first.
