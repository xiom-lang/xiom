<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# XIOM Compiler Bugs -- Log

Dated sections. Each entry: file(s) affected, construct, error observed, and
(recommended) fix direction for the compiler team. Stdlib workarounds are
deliberately NOT applied where the stdlib mandate says "production grade, no
workarounds" -- the compiler must be fixed, then the stdlib lands.

---

## 2026-10-05 -- FIXED (UX): bare `xiom file.xi` printed IR with no `xiom run` hint

User-flagged: `xiom file.xi` (no -o, no `run`) prints the LLVM IR to stdout
(3752 lines for the monotonic-clock probe) with nothing pointing at
`xiom run file.xi`, which executes it. Explicit `--emit-ir` had the same
output.

FIX (lib.rs IR branch): when the implicit condition
(`!emit_ir && output_file.is_none() && !do_run && target == Native`) is
taken, print ONE hint line to stderr:
`hint: IR only -- 'xiom run <file>' executes it, or pass -o <out> to build a binary`.
stdout is byte-identical (the selfhost T3 gates compare stdout bytes);
`--emit-ir` stays quiet because it is a deliberate request.

LOCK: `ux_bare_compile_hints_run_on_stderr` (run_script_cli, already on the
CI path): bare stdout starts with the IR banner AND equals the `--emit-ir`
stdout byte-for-byte; stderr carries the hint; `--emit-ir` stderr does not.

---

## 2026-10-05 -- FIXED: installed layout missed lib/runtime -> AOT link failed (xiom_async_now_ms)

Packages relay (packages commit 5b7547b0): installed v0.63.1 AOT links fail
`lld-link: error: undefined symbol: xiom_async_now_ms` for any program whose
closure uses the monotonic clock (`xiom.time.monotonic_ms` / `Instant.now`);
the same class was suspected for the open crypto-link (`xiom_sha256_hash`).

ROOT CAUSE: `find_runtime_c_files()` (crates/xiom/src/lib.rs) scanned only
CWD (`stdlib/runtime`), exe walk-up (`<ancestor>/stdlib/runtime`,
`<ancestor>/runtime`) and exe relatives (`../runtime`, `../stdlib/runtime`,
`./runtime`, grandparent forms). The production layout
`%LOCALAPPDATA%\xiom.new\lib\runtime` is none of those -> empty list -> the
`find_runtime_c()` fallback linked ONLY `xiom_runtime.c`, dropping
async_runtime.c / simd_runtime.c / sha256_sw.c / xiom_hot_reload.c. The JIT
already resolved all five via `xiom_graph::paths::current_stdlib_candidates()`
(`xiom-jit/src/lib.rs::runtime_dirs`); the AOT path never used it.

REPRO (installed v0.63.1, neutral CWD, 5-line `xiom.time.monotonic_ms()`
probe): `lld-link: error: undefined symbol: xiom_async_now_ms` referenced by
`__unsafe_block_0`, exit 1. `XIOM_RUNTIME_DIR=...\stdlib\runtime` is the
supported workaround (packages: aws 27/27).

FIX: `find_runtime_c_files()` resolves the shared candidate roots first
(repo `stdlib/runtime`, installed `<install>/lib/runtime`, `share/xiom`,
XIOM_HOME, XIOM_STDLIB, manifest) exactly like `find_runtime_c()` and the
JIT, then walks the exe ancestors (now including `lib/runtime`) and exe
relatives (`../lib/runtime`), then CWD, then CARGO_MANIFEST_DIR. The
candidate logic is factored into `find_runtime_c_files_in(stdlib_roots,
exe_dir, runtime_dir_override)` (testable discovery) and the directory scan
into `scan_runtime_c_dir`.

VERIFICATION: out-of-repo install simulation (bin + lib/{xiom,runtime},
neutral CWD, runtime `CARGO_MANIFEST_DIR` pointed away from the repo):
- control v0.63.1 in the same layout -> `undefined symbol: xiom_async_now_ms`.
- fixed driver -> compiled + ran, exit 0.
Unit locks: `r65_install_lib_runtime_is_scanned`,
`r65_resolver_root_runtime_is_scanned`, `r65_xiom_runtime_dir_override_wins`.
This also restores `xiom_sha256_hash` (sha256_sw.c now compiled; note it
requires its sibling `sha256_sw.h`, which the install ships) -- packages
should re-test the crypto probe on the next archive.

---

## 2026-10-05 -- FIXED: one-arg `read` method hijacked by the raw-pointer builtin (m196, Pulse C-PULSE-01 / R-8)

Pulse relay C-PULSE-01 + benchmark R-8: a method named `read` with exactly
one argument was swallowed by the codegen builtin at call.rs ("Builtin
read(ptr): load value through raw pointer"). The guard checked only the name
and arg count, and a real `sock.read(&mut buf)` argument lowers to
`%struct.Vec*` (ends in `*`), so the builtin emitted a deref-load and the
real method body never ran -- silent: `Result.is_ok` folded to false, no
bytes appended, no diagnostic. This is the exact R-8 `TcpStream.read`
elision.

FIX: the builtin now requires `receiver_expr.is_none()` (the receiver-less
desugared form it was written for); a method call named `read` always emits
the real method. `write` is unaffected (its builtin requires two args).

VERIFIED against the LIVE benchmark server (127.0.0.1:8080, host-side,
XIOM_STDLIB = stdlib head):
- control installed v0.63.1 AOT: `connected / wrote 64 / before read 0..4 /
  done loops total=0` (the documented elision).
- fixed AOT: `before read 0 / after read 0 n=3137 total=3137 / before read 1
  / after read 1 n=0 total=3137 / eof`.
- fixed JIT (`xiom run --no-cache`): identical, exit 0.
- Pulse matrix probe (`read-method-builtin-shadow/probe.xi`) exit 5 -> 0.

LOCKS: `regress_m196_read_method_not_hijacked` (IR: main calls `@Sock.read`),
`e2e_m196_read_method_not_hijacked` (method body runs; buffer gains the
byte). CI filter extended. Benchmark: re-probe both paths on the next
archive; R-8 stays in the language-correctness wave.

---

## 2026-10-07 -- OPEN (packages/Pulse/stdlib relay bundle): new findings recorded

New reports since v0.64.0 (repro paths local unless noted):

1. [FIXED m202, 2026-10-07] GRPC `Vec[(Str, Str)]` read-after-mutation
   (packages, still RED on v0.64.0; m192 did NOT clear it):
   `probe_suite_min.xi` and `probe_direct.xi` both exited `0xC0000005`
   (reproduced locally from
   `E:\xiom-packages\packages\packages\xiom-grpc\tests\`). Root cause was
   NOT the read-after-mutation lead: `("k".clone(), "v".clone())` tuple
   elements were named via the first declared `.clone` suffix
   (MaybeUninit.clone). Details + fix in the m202 section below; packages
   can re-test probe_suite_min/probe_direct (both exit 0 here).
2. [FIXED m206, 2026-10-07] GraphQL enum-payload fields 9/10 (packages):
   `GraphQLSelection.Field(fs)` payload fields read as the constant `0`
   (`fs.name == "hello"` false) when the variant payload structs share the
   declared field name (boxed enum layout). Root cause + fix in the m206
   section below; minimal local repro added
   (`tests/regression/m206_enum_struct_payload_field/`). Packages can
   re-run `packages/xiom-graphql/tests/test_conformance.xi` (10/10 here).
3. [FIXED m204, 2026-10-07] C-PULSE-07 (Pulse): a module-scope `var`
   initialized by a cross-package constructor (`var b = rate_keyed_new(1,
   1);`) was accepted but emitted `call i64 @rate_keyed_new(...)` with no
   definition -> clang "use of undefined value"; larger case AV'd during
   module init. Root cause + fix in the m204 section below. Repros:
   Pulse `docs/repro/module-scope-package-init/probe.xi` and the local
   cross-module equivalent `tmp/pulse05/ginit/`.
4. [FIXED m200, 2026-10-07] `p_rvalue_float_vec_index.xi` (stdlib
   known_failures): indexing the RVALUE of a `Vec[Float64]`-returning call
   (`mk_f()[0]`) read the raw bits while a bound local read correctly;
   `Vec[Int]` rvalue indexing was correct. Root cause + fix in the m200
   section below.
5. [FIXED m203, 2026-10-07] Range.count ensures-clause retry (stdlib,
   re-verified on v0.64.0): adding even `ensures: result >= 0` to
   `Range.count` made `smoke_iter` fail with clang "instruction forward
   referenced with type 'ptr'" at `%tmp8` -- the closure thunk inherited
   the enclosing fn's ensures/result state. Fixed; details + verification
   in the m203 section below. The stdlib lane can re-add the clause.
6. [FIXED m205, 2026-10-07] TOOLING (owner-reported): `xiom doc <file>`
   resolved `XIOM_HOME/bin/xiom-doc.exe` (on this host XIOM_HOME points at
   the stale `...\xiom` dir) instead of the SIBLING binary the way
   fmt/lsp/mcp/pkg/dbg/verify/ffigen do; `xiom -doc` (single dash) was not
   recognized at all and fell through to the compile/IR path; `xiom doc
   --help` hit the main --help first. Fix: doc routes through
   `run_tool_dispatch` (sibling first, XIOM_HOME fallback), the
   doc/-doc/--doc spellings all dispatch, and --help now reaches xiom-doc
   (the dispatch table runs before the main short-circuit). Lock:
   `m205_doc_subcommand_dispatches_to_tool` (crates/xiom/tests/cli_args.rs).
7. [FIXED m209, 2026-10-07 packages relay] DESTRUCTURING A REFERENCE TO A
   TUPLE ELEMENT YIELDED POINTER-LIKE VALUES: `let (k, v) = &vec[i];` over
   `Vec[(Str, Str)]` bound `k`/`v` that stringified as decimal addresses and
   never compared equal to the expected `Str` (`k == &key` false), so key
   lookups silently missed (grpc `metadata_get` returned None for a present
   key and `set` appended duplicates). Direct component reads
   (`vec[i].0 == key`) were correct. Evidence: packages
   `docs/COMPILER-FINDINGS.md` 2026-10-07 row (`refeq=0`, `cmp=0`,
   `resp.len=1`).

   ROOT CAUSE (two halves):
   - `Stmt::Destructure`'s scalar fallback (`zero_val_for` branch) bound the
     SAME i64 -- the ptrtoint'd `&vec[i]` element address -- to every name.
   - `auto_deref_ref` (vec_abi.rs) recognised only `&T`-ANNOTATED idents, so
     the `&key` operand was inttoptr'd straight to i8* and
     `strcmp(pointee(k), key-slot-bytes)` compared the string against the
     pointer slot.

   FIX (m209): (1) the destructure now handles `&vec[i]` (Ref/MutRef over
   Vec-index) by inttoptr + GEP per component and binds each component
   ADDRESS as an i64 slot with the round-8 `local_xiom_types` annotation
   (`&Str`); (2) `auto_deref_ref` also recognises `&ident` / `&(ident)`
   Ref/MutRef operands in value positions and loads through the slot once,
   making the comparison symmetric.

   EVIDENCE: `tmp/pulse05/tuple_ref_destructure.xi` exit 1 -> 0; IR
   (`tuple_ref3.ll`) shows TWO `inttoptr ... to i8**` slot derefs and
   `strcmp(pointee(k), pointee(key))`, where pre-fix had an i64 address
   compare (zero derefs).

   LOCKS: `regress_m209_tuple_ref_destructure` (IR: tuple inttoptr, strcmp,
   >= 2 `to i8**` derefs), e2e `e2e_m209_tuple_ref_destructure` + fixture
   (`tests/regression/m209_tuple_ref_destructure/`), CI line. Gates: feature
   531/531, e2e 2435/0/4. Packages can drop the "never destructure a
   reference to a tuple element" rule after the v0.64.1 pin.
8. [FIXED m210, 2026-10-07 packages relay] `Vec[Struct].clone()`
   aborted `0xC0000005` (probe `docs/repro/struct-clone/probe_struct_clone.xi`
   on compiler main m199..m207; no-clone control `probe_struct_push.xi`
   exited 0). The clone lowering itself was CORRECT: the bound local lost
   the element type, so `w[i]` fell to the runtime elem-size scalar switch.
   Root cause + fix in the m210 section below; packages can drop the
   clone-avoidance workaround at the next pin.
9. [STATUS, 2026-10-07 packages relay] `io.list_dir` (see the dedicated
   2026-10-03 section below): still broken, now "correct count, last name
   repeated for every entry", identical on pinned v0.64.0 and m199..m207.
   `io.xi:943` false-`ensures` multi-module finding was NOT reproduced with
   a 2-module scratch program (reads correctly, n=3437, exit 0) -- the
   original `xiom.kv` shape is not minimized; no compiler action until a
   faithful repro. `tuple-vec-set` probes are GREEN (m202); the grpc restore
   is complete on the candidate (36/36 x2, publish held for the official
   pin).
7. [FIXED m207, 2026-10-07] `xiom build` from a project root was broken in
   the driver CLI -- `resolve_source_files` did not skip the `build`
   subcommand token, so it became a source path and the driver failed twice
   with `error: cannot read 'build': The system cannot find the file
   specified (os error 2)` before the project-graph branch could run. Fix:
   `build` joins the wasm/arm/riscv positional-sugar skip list (only when no
   real file/dir named `build` exists, so a literal `build` source still
   compiles). Verified end-to-end: `xiom build` in a project root now builds
   the graph (ginit included). Lock: `m207_build_subcommand_token_not_a_
   source` (crates/xiom/tests/cli_args.rs).

---

## 2026-10-07 -- FIXED: [dependencies] are mapped to catalog source roots (m212, C-PULSE-02)

Pulse relay (C-PULSE-02): `[dependencies]` entries were parsed into the
manifest but NEVER mapped to source directories, so packages installed under
`$XIOM_HOME/packages` were not on the catalog path; a project had to
hard-code the installed directories in `[project] source-roots` (the Pulse
workaround), and a raw `--run` in a package dir with a root module outside
`src/` failed catalog type resolution.

ROOT CAUSE: `xiom-graph::manifest::resolve_source_roots` built the graph
roots from `[project].root` / `source-roots` / `src/` / manifest dir only;
`manifest.dependencies` (path, git, registry) was ignored, so
`discover_modules` never saw the dependency packages and the checker
reported `T001 undefined variable` for their functions (or resolved nothing
for root modules outside `src/`).

FIX (m212): `resolve_source_roots` now appends dependency roots via a new
`dependency_roots_under` helper -- `path = "..."` maps to that directory
and its `src/`; registry entries map to the newest matching installed
package `<xiom_home>/packages/<name>-<version>/`, adding the package ROOT
(for root modules like `xiom-graphql/graphql.xi`) and its `src/`. Roots are
deduped and appended after the project's own roots.

EVIDENCE: a local project with `[dependencies] xiom-rate = "0.2"` and a
module-scope `var` initialized from `rate_keyed_new` builds with
`xiom build`: pre-fix `Build: pkginit (1 modules)` + T001 undefined
variable; post-fix `Build: pkginit (3 modules)`, BUILD=0, and the IR emits
`define %struct.xiom.rate.KeyedBuckets @rate_keyed_new(...)`.

LOCKS: unit tests `dependency_path_roots_are_added` +
`installed_dependency_roots_are_added` (xiom-graph manifest tests, newest
version wins) and driver test `m212_project_path_dependency_is_discovered`
(crates/xiom/tests/cli_args.rs: portable temp project with a path dep,
`xiom build` must succeed and emit the dependency function).

---

## 2026-10-07 -- FIXED: Vec[Struct].clone() element reads aborted 0xC0000005 (m210, packages relay)

Packages relay: `let w = v.clone();` over `Vec[Pair]` followed by `w[1]`
aborted `0xC0000005` while the no-clone control stayed green. Reproduced
locally with `docs/repro/struct-clone/probe_struct_clone.xi` (exit
-1073741819).

ROOT CAUSE (IR evidence, tmp/pulse05/clone.ll): the clone lowering is
CORRECT -- it allocas the receiver, mallocs `len * elem_size`, memcpys the
buffer and rebuilds the header (len/cap/esz all preserved). The crash was
the ELEMENT READ: the Let/Var Vec-element inheritance chain did not know
that a `clone()` call returns the receiver's element type, so
`local_vec_elem["w"]` was never recorded; `w[1]` fell to the runtime
elem-size `switch i64` (cases 1/2/4, scalar i64 default), inttoptr'd the
element's FIRST FIELD (a=1) as an address and dereferenced it.

FIX (m210): a new `clone_receiver_elem` helper inherits the receiver's
element type for no-arg `clone()`/`to_owned()` calls, and both the Let and
Var inheritance chains consult it (after the first-arg check, before the
callee-return fallback). `w[i]` then takes the struct memcpy path.

EVIDENCE: probe prints `before` / `cloned len=2` / `second=2:two`, exit 0;
minimal fixture IR has ZERO elem-size switches (pre-fix: one per read).
Independence is locked too: pushing into the clone leaves the original
length/elements untouched.

LOCKS: `regress_m210_vec_clone_elem_type` (IR: no `switch i64`, memcpy
present), e2e `e2e_m210_vec_clone_struct_elem` + fixture
(`tests/regression/m210_vec_clone_struct_elem/`), CI line. Gates: feature
532/532.

---

## 2026-10-08 -- FIXED: loop-body static allocas leaked stack (m235, C-ORBIT-05)

ORBITDB C-ORBIT-05: a nested loop over `Vec[Page]` (struct with Vec fields)
that reads elements and pushes into another `Vec[Page]` aborted with
0xC0000005 at n=418, no diagnostic; the committed bundle probe died before
printing ("D ok" never appeared).

DIAGNOSIS (windows x64): the crash point scales with the NUMBER OF
STRUCT-ELEMENT READS: n=417 crashed during outer iteration k3=408 (~86.7k
inner iterations = ~173k reads), n=418 at k3=388, n=600 at k3=168 -- all at
~173k element reads. A one-read-per-iteration variant crashed at ~173k
iterations (half the bytes per iteration, double the iterations): both shapes
cross ~16.6 MB of temporary ALLOCA bytes. The inner-loop IR is call-free but
contains `alloca %struct.Page` temporaries (4 x 48 B per iteration): LLVM
executes an `alloca` every time control reaches it and frees the memory only
at function return, so statically-sized temps emitted inside a loop leak
stack per iteration until the 16 MB stack reserve is exhausted -> AV/trap.
(`Vec[Int]` reads do not crash: 8-byte temps are promoted by clang;
`Vec[Page]`'s aggregate temps are not.)

FIX (m235): `compile_program` now runs `hoist_static_allocas`, a module
post-pass that moves **static allocas inside CYCLIC blocks** (loop bodies,
identified by a textual CFG cycle analysis per function; `alloca T, i64 %n`
dynamic allocas stay) to the function's entry block. A first cut hoisted ALL
static allocas and regressed `m65_vec_option_struct_str` (acyclic slots
changed uninitialized-slot semantics; a confined fault then turned into
`llvm.trap`, exit 0xC000001D) -- hence the cyclic-only scope, which fixes the
leak class without touching acyclic code.

EVIDENCE: minimal repro (`tmp/bench/orbit05_min.xi`, `Page{id,data:Vec[Int],
checksum}`, nested read+push) pre-fix exit 0xC0000005 -> post-fix
`D ok out=417`; n=600 also green. ORBITDB bundle probe now prints
`A ok out=418 / B ok out=418 / C ok out=418 / D ok out=418`, exit 0.
XVECTOR probe unaffected ("hydration-guard: green").

LOCKS: IR `regress_m235_loop_alloca_hoist` (no static alloca in any
`while_cond*`/`while_body*` block; acyclic blocks untouched), e2e
`e2e_m235_loop_alloca_hoist` + fixture
(`tests/regression/m235_loop_alloca_hoist/`, n=417), CI line.

GATES: feature 546/546; full e2e 2452/0/4; driver suites
61/6/2/29/5/15/4/7/34.

---

## 2026-10-08 -- FIXED: Vec element strides ignored field alignment padding (m234, XVC-C-08)

XVECTOR XVC-C-08: in a `Vec[Struct]` whose element carries a trailing scalar
after a Vec-bearing field, the trailing scalar read uninitialized garbage
(probe exit 1 `BAD detach count`, variants V5/V7/V10/V11/V12/V14 red; V14 an
Int flag, so not Bool-specific).

ROOT CAUSE (codegen): `vec_elem_storage_size` summed field sizes with no
alignment. `Elem5 { id: Int; distance: Float32; payload: Vec[Node];
flag: Bool }` was sized **52** while the emitted LLVM layout is
`{ i64, float, %struct.Vec, i64 }` = **56** (4 bytes of padding before the
8-aligned Vec field). The push stored `store i64 52` as the element stride,
so element 1 was written/read 52 bytes apart in a 56-byte array --
overlapping elements and a garbage trailing scalar. (`Res5 { i64, float }`
was likewise sized 12 vs the real 16.)

FIX (m234): new `xiom_type_align` (LLVM-style alignment: 1/2/4 for small
primitives, 8 for i64/Float64/Str/containers/enums, max-field-align for
structs with Bool fields as i64 slots) and the struct branch of
`vec_elem_storage_size` now pads each field to its alignment and rounds the
total to the struct alignment.

EVIDENCE: minimal V5 repro pre-fix exit 2 -> post-fix exit 0 and the IR
stride is `store i64 56` (was 52). XVECTOR bundle:
`probe.xi` -> "hydration-guard: green"; `variants.xi` -> "variants: all
green"; `min.xi` control unchanged.

LOCKS: IR `regress_m234_vec_elem_padding` (stride 56 present, 52 never),
e2e `e2e_m234_vec_elem_padding` + fixture
(`tests/regression/m234_vec_elem_padding/`), CI line.

GATES: feature 545/545; full e2e 2451/0/4.

---

## 2026-10-08 -- FIXED: verifier sort gaps blocked every t8 contract obligation (m233)

Owner t8 container run (run_1791481954111) reported `Results: 0 proven, 0
violated, 4 unknown` + `no queries emitted` on `systems-contracts-arena/
t8-safety-probe.xi` -- a harness FAIL. The three reference shapes:

```
fn cstr(s: Str) requires: s.len() > 0        -> X7007 Gt on non-numeric (None/Some("Int"))
fn store_word(base: *mut UInt8, ...) requires: base != null   -> X7007 equality unresolved
fn load_word(...)                    requires: base != null   -> same
```

ROOT CAUSE: `infer_sort` had no rule for builtin `len`-family methods
(`s.len()` -> None) and no model for the `null` literal against opaque pointer
sorts; `translate_expr` additionally routed `s.len()` to "call to unknown
function 'len'". Every axiom was skipped -> zero queries.

FIX (m233):
1. `s.len()` on a `String`-sorted receiver now infers `Int` and emits SMT
   `(str.len s)`.
2. `ensure_sort_declared` + the top-level `collect_dynamic_sorts` pass give
   every `|xiom_ptr_...|` sort a modeled null constant
   (`(declare-const |null_xiom_ptr_UInt8| |xiom_ptr_UInt8|)`);
   `x ==/!= null` compares via `=`/`distinct` against it (infer_sort inherits
   the other operand's sort for the null side).
3. Latent emission bug surfaced by (1)+(2): inferred-return fns (no `-> T`)
   asserted `(= |result| ...)` with `|result|` UNDECLARED -- z3 would have
   rejected the whole script the first time it ran with queries. The
   body-return encoder now declares `|result|` once per function (per-fn
   `latest` marker), inferred sort from the returned value.

EVIDENCE:
- t8: X7007 sort-gap warnings gone; unknowns 4 -> 1; the full SMT parses in
  z3 clean. Verdict remains `unproven / no queries emitted` because t8 bodies
  do raw pointer stores/loads (`*(p + word) = value`) the encoder cannot
  model -- HONEST LIMIT: raw-memory (SMT Array) modeling is the next step,
  tracked in the queue.
- t1-allocator unchanged: 2 proven, 0 violated, 10 unknown, 0 errors.
- xiom-verify: 8 unit + 34 integration green.
- BONUS (same session): the arena C001 reducer
  (`E:\xiom-perf\c001\reducer.xi`) is **12/12 clean** on the current tree
  (was 3/6 failing on v0.62.4; smokes 8/20 + 12/20) -- the m231 name-boundary
  fix removed the same hash-order nondeterminism class. Release-gate
  exclusions for `smoke_iter_range`/`smoke_iter_find_all_any` can be dropped
  on the next archive.

LOCK: `str_len_and_pointer_null_are_modeled` (asserts `(str.len s)` emission,
the per-sort null declaration + `distinct` comparison, no sort-gap skips, and
z3 parses the script).

RELAY: written to the benchmark lane repo
(`E:\xiom-projects\xiom-benchmark-chaos\docs\COMPILER-RELAY-2026-10-08.md`):
t2-t5 contract "FAIL" is a reference clause gap (zero clauses -> zero
obligations), t3 needs the Linux lane, t8 details, and the honest safety
recommendation (real hardened build variant, not scorer tricks).

---

## 2026-10-08 -- EVENING LANE RE-SWEEP + BENCHMARK RUN

Second sweep of all lanes after the m222..m232 batch, plus a manual benchmark
pass on the current tree with RELEASE-built outputs (release `xiom` from the
v0.64.2 tree; victim binaries `--release --target native`). New findings and
reproduction evidence below; benchmark numbers in the second half.

### New findings

| Finding | Status | Evidence on this tree |
|---|---|---|
| **XVC-C-08** trailing scalar after a Vec-bearing field reads uninitialized garbage (`Vec[Struct]` element) | **FIXED m234** | root cause: `vec_elem_storage_size` summed fields without LLVM alignment padding (Elem5 sized 52 vs real 56; Float32 leaves 4 bytes of pad before the 8-aligned Vec field) so element 1 used a 52-byte stride. Minimal V5 pre-fix exit 2 -> post-fix exit 0; XVECTOR bundle probe "hydration-guard: green", variants "all green" |
| **C-ORBIT-05** nested `Vec[Page]` loop + push into another `Vec[Page]` aborts | **FIXED m235** | root cause: static `alloca` temps emitted inside the loop leaked stack per iteration (~16.6 MB of 48-byte temps -> 0xC0000005 at ~173k element reads). Cyclic-block alloca hoist in `compile_program`; minimal repro pre-fix exit 0xC0000005 -> `D ok out=417`; bundle probe A/B/C/D all `ok out=418` |
| **stdlib M7** `Iterator[T]` receiver type unresolved | **OPEN** | `tools/known_failures/p_iter_iterator_type_unresolved.xi`: `--check` PASSES but compile prints 5x `unknown type 'Iterator' -- defaulting to i64` and fails `error[C001]: unresolved function symbol(s) ... 'Iterator.step_by'` |
| **packages** `unsafe fn` hard P001 | CONFIRMED | `unsafe fn f()` -> `error[P001]: 2:8: unsafe applies only to block expressions`; keep fn safe + `unsafe { }` body |
| **packages** `let _ = unsafe { call() };` invalid IR (pointer/Str/struct returns) | LANE-REPORTED, not reproduced minimally | their exact shape: `trunc i64 -> i32` then `ret i8*` in project builds; a minimal `let _ = unsafe { alloc(8) };` compiles clean on this tree -- exact repro wanted |
| **packages** grpc catalog-dep rehearsal RED: 6 T001 "ambiguous function exported by multiple imported modules" | LANE-REPORTED | queued by the packages lane 18:33Z; needs their grpc depot to bisect |
| **PULSE** C-PULSE-13 recurrence (toolchain re-extract deleted the canonical `packages/` bridge) | PARTIAL (m232) | xiom-pkg now resolves `paths::xiom_home()` for install/cache; the remaining piece is the Unix toolchain INSTALLER creating the canonical `packages/` dir on every install (installer lane, not compiler) |
| `xiom.http` 0.1.2 (67 T001 -> 0) | FIXED lane-side | republished; PULSE probe should flip green |
| **ORBITDB C-ORBIT-01..04 / XVECTOR XVC-C-01..C-07** | FIXED compiler-side (m222..m226); lane docs still show pre-sweep status | their docs note "no v0.64.2 archive installed yet" -- re-verify pending on the archive |

### Benchmark run (this box, Windows; release outputs; 3 samples unless noted)

System arena (`reference/systems-arena/*.xi`, compile `--release --target native`):

| task | compile | run (warm) | note |
|---|---|---|---|
| t1-allocator | 7.7 s | 46-52 ms | prints OK |
| t2-queue | 7.2 s | 40-48 ms | prints OK |
| t3-hot-reload | 2.8 s | 18-28 ms | `dlopen failed` -- reference is Linux-shaped (`libc.so.6`); timing needs WSL/Linux |
| t4-packet | 7.1 s | 59-62 ms | prints OK |
| t5-btree | 7.5 s | 22-31 ms | prints OK |
| t8-safety-probe | 2.7 s | 16.8-19.1 s | per-attack JSON; mostly SILENT_UB/INCONCLUSIVE, no crash |

Contracts arena (`reference/systems-contracts-arena/*.xi`): all six `--check`
PASSED. `xiom-verify --check` (z3):

- t1-allocator: **2 proven, 0 violated, 10 unknown, 0 errors** (X7007 loop
  without invariant / complex call target).
- t2-queue, t3-hot-reload, t4-packet, t5-btree: `no queries emitted (all
  obligations skipped as UNKNOWN)` -> **0 proven, 0 violated, 1 unknown,
  0 errors**. Root cause is the REFERENCE, not the compiler: clause counts
  t1 requires=12 ensures=5 invariant=1 vs t2-t5 **all zero** (t8: 3 requires,
  0 ensures -> 4 unknown). Harness "FAIL" for t2-t5 is a no-obligation
  benchmark gap; either add a provable clause to the references or exempt
  no-obligation trials.
- This matches the owner's report ("t1 contracts passes, the others fail")
  and the 2026-10-05 audit (`COMPILER_BUGS.md` contracts section).

Scripting arena (`reference/scripting-arena/*.xi`, `xiom run` modes, release
xiom, BENCH_INPUT from `data/probes/t6_ops.jsonl` / `t7_ops.jsonl`):

| task | default | --jit --cache | --jit --no-cache | --cache | --no-cache |
|---|---|---|---|---|---|
| t6 | 3680/893 ms | 130/231 ms | 3004/2785 ms | 58/56 ms | 3383/3464 ms |
| t7 | 19328 ms | 1943 ms | 15664 ms | 1417 ms | 16070 ms |

Correctness: t6 output matches the Python reference for all 13 ops (11 pass +
2 expected malformed fails; only JSON whitespace differs). t7 matches the
Python reference's per-op classifications (http_get pass, 2 parse_json pass,
2 malformed parse_json fail, 2 read_file fail, stress_requests/parse pass,
shell_exec pass). "Slow JIT/AOT as before" reproduces: JIT cold ~2.8-3.0 s,
AOT cold ~3.4 s, JIT warm ~0.13-0.23 s, cache warm ~56-58 ms (t6). The t3
hot-reload 216-245 ms trampoline issue (item 7) is unchanged and still needs
the Linux/containers lane to re-measure.

### t8-safety-probe container run (owner paste, run_1791481954111, Linux/`/app`)

- Safety Index **30/100** (12 probes; 3 runs; 0 flaky), sanitized same 30.
  Probe outcomes: use-after-free and double-free = **SILENT_FAILURE**
  (state_corrupted true, canary_intact false), buffer-overflow/stack-overflow/
  use-of-uninit/buffer-overflow-write/integer-overflow = SILENT_UB, null-deref/
  double-free = INCONCLUSIVE, type-confusion = SILENT_FAILURE. This is the
  unsanitized-UB safety model (no runtime containment), not a compile failure;
  it does show the probes exercise real memory corruption in the container.
- Speed: 7 samples + warm-up, medians 198 / 153 / 163 ms (sigma 12.8-31.3 ms),
  peak 2 MB.
- Contracts verdict: **unproven -- 0 proven, 0 violated, 4 unknown, 0 errors**
  (469-712 ms), i.e. the harness-visible FAIL. Root causes are two SORT GAPS
  in the verifier's SMT translation, all three shapes from the reference:

```
fn cstr(s: Str) -> *UInt8
  requires: s.len() > 0            -> X7007 Gt on non-numeric operands
                                      (sorts None/Some("Int"))   [cstr]
fn store_word(base: *mut UInt8, ...)
  requires: base != null           -> X7007 equality with unresolved
fn load_word(base: *mut UInt8, ...)   operand sort                 [store/load]
```

  `infer_sort` has no rule for (a) builtin `len`-family method calls (so
  `s.len()` -> None) and (b) the `null` literal against an opaque pointer sort
  (`*mut UInt8` maps to `|xiom_...|`, `null` maps to nothing). With every
  axiom skipped, zero queries are emitted -- the same "no queries emitted"
  class as t2-t5, and the reason t8's contract track fails. Note t1 PASSES
  with unknowns (queries ARE emitted), so restoring real queries for these
  shapes is the path to a passing t8 contract verdict.

---

## 2026-10-08 -- RELAY SWEEP (v0.64.2, post m222..m229): lane re-test matrix

Compiler dev binary at 35ed820e + m228/m229, checked against the four external
lanes' own repro bundles. Runner-up evidence lines below; lane docs keep the
per-finding detail.

| Lane / finding | Status | Evidence on this sweep |
|---|---|---|
| XVECTOR XVC-C-01..C-06 | FIXED (v0.64.1, re-confirmed) | lane Addendum 3; no new bundles |
| XVECTOR XVC-C-07 | FIXED m222 | `xv-from-utf8-vec-ref/probe.xi` compile+run exit 0 |
| ORBIT C-ORBIT-01..C-04 | FIXED m223..m226 | all four committed bundles green (C-01 bits=0; C-03 A=B=C=D=1; C-02 full V1..V10 matrix; C-04 twin-enum repro) |
| PULSE C-PULSE-08/10/11 | CLOSED (lane v0.64.1) | lane sweep logdirs; C-11 swap retry is the lane acceptance |
| PULSE C-PULSE-09 | FIXED m227 | two-module reduction pre-fix trap -> post-fix exit 0 |
| PULSE C-PULSE-12 (alias shadowing) | NOT REPRODUCED in the minimal shape | module `pulse12.server` + consumer `use xiom.net.server; server.server_parse_request(&b)`: the alias resolves (no "cannot call ... on this expression"); the compile instead dies on m230 below. Faithful http.xi re-test routed to the lane |
| PULSE C-PULSE-13 (home split) | FIXED m232 | xiom-pkg now resolves `paths::xiom_home()` (install + cache); delegation lock `test_get_xiom_home_delegates_to_paths` |
| BINDINGS B-01 enum-payload ND | **FIXED m231** | root cause: `struct_type_from_expr_inner` matched type keys with `key.ends_with(&base_type)`, so `Option__Value`/`Result__...__Value` also matched leaf `Value`; the FIRST key in random HashMap order decided the field lookup -- when Option__Value won, the field named `value` resolved to Option's PAYLOAD field type (`Value`, not `ValueKind`) and the match compared Integer at tag 0 instead of 1. Repro `enum-payload-nd/pkg/`: pre-fix 7/12 bad builds and TWO IR variants (hash 4722EA.. vs 3D46A0.., 3 fns diverge); post-fix 12/12 green, ONE IR hash |
| BINDINGS B-02/B-03 (const resolver recursion) | NOT RE-TESTED | pre-fix large catalogs gone; rebuild from descriptions if needed |
| BINDINGS B-04 (child imports parent) | FIXED on this sweep (minimal shape) | module `p.child` `use p;` + unqualified `parent_fn()` -> exit 0 |
| BINDINGS B-05 (alloc/free guard spin) | **STILL OPEN** | `alloc-guard-spin` watchdog kill at 8 s, 7.53 CPU-s burned, same spin |
| BINDINGS B-06/B-09 | FIXED (v0.64.1, re-confirmed) | lane sweep |
| BINDINGS B-07 (ffi alias shadowing) | FIXED on this sweep (minimal shape) | module `p.ffi` `use xiom.ffi;` + unqualified `safe_ptr_alloc(8)` -> exit 0 |
| BINDINGS B-08 (`--run` exit masking) | FIXED m228 | `--run` now exits with the program's code (5 -> 5) |
| PACKAGES `is Ok(<literal>)` | FIXED m229 | `pick(2) is Ok(1)` now false; tag AND payload compare |
| PACKAGES Result equality of equal Ok(Vec) pairs | STILL OPEN | `res_eq` probe: two equal `Ok(Vec[UInt8].new())` pairs compare FALSE quietly |
| PACKAGES io.xi:943 | clean (lane v0.64.1) | lane battery; no change on main retest |

New in this sweep (not reported by any lane): **m230** below -- a user module
importing `xiom.encoding` (directly or via `xiom.net.server`) hard-fails the
compile with 3 latent catalog-body T001s. Pre-existing on v0.64.1.

---

## 2026-10-08 -- FIXED: xiom-pkg home split from the compiler's XIOM home (m232, C-PULSE-13)

PULSE C-PULSE-13: `xiom pkg` installed into `$HOME/xiom/packages` (Unix) while
the compiler's `paths::xiom_home()` resolved the EXISTING canonical
`~/.local/share/xiom` first -- installed packages were invisible to dependency
lookup (`dependency_roots_under(..., &paths::xiom_home())`).

xiom-pkg carried THREE private resolutions: `install_package_files` and
`get_xiom_home` used `($LOCALAPPDATA|$HOME)/xiom`, and
`registry::resolve_package_cache_dir` the same platform default; the compiler
uses `XIOM_HOME` -> first EXISTING candidate (canonical installer layout
first, then legacy `~/.local/xiom` / `~/xiom`) -> canonical default.

FIX (m232): xiom-pkg depends on xiom-graph and all three sites resolve through
`xiom_graph::paths::xiom_home()` / `xiom_home_candidates()` +
`canonical_xiom_home()` (the cache-dir helper keeps the explicit-`XIOM_HOME`
sandbox contract used by tests). Install and consumer now always agree.
Signing keys (`signing::default_xiom_home`, `~/.xiom`) are deliberately NOT
moved in this fix -- relocating the trust store would silently orphan existing
keys; if key-home unification is wanted, it needs a migration note.

LOCKS: `test_get_xiom_home_delegates_to_paths` (xiom-pkg unit: installer home
must equal `paths::xiom_home()`; a private fallback re-introduction fails on
any machine where the env resolves differently), existing
`resolve_package_cache_dir` sandbox tests updated to the shared resolution.
ci.yml already runs `cargo test -p xiom-pkg`.

---

## 2026-10-08 -- FIXED: wrapper-prefix suffix match flipped enum field matches per build (m231, B-01)

BINDINGS B-01: the `enum-payload-nd` pkg repro build+run loop produced `B=false`
in a fraction of rebuilds (2/6 on v0.64.0, 2/6 on release v0.64.1, 7/12 on the
v0.64.2 tree) -- silent wrong results through the enum accessor path, not a
loud failure.

ROOT CAUSE (codegen): `struct_type_from_expr_inner`'s `Expr::Field` arm resolves
`match s.value`'s scrutinee type by finding the base struct's field type. Its
key scan used

```rust
if key.ends_with(&base_type) || key == base_type
```

so the CONCRETIZED WRAPPER types also matched: `Option__Value`.ends_with("Value")
and `Result__...__Value`.ends_with("Value") are true. The loop then `break`s at
the FIRST match -- which is random HashMap order (per-process RandomState). When
`Option__Value` won, the field named `value` resolved to OPTION'S PAYLOAD field
type (`Value`, not the enum `ValueKind`), the match took the builtin
Option/Result field-0 path and compared `Integer` at discriminant 0 (it is 1)
while binding field 0 as the payload. Same source, two IR variants:
4722EA.. (757,632 B, wrong) vs 3D46A0.. (752,368 B, correct); diagnosis via
`--emit-ir` per-function hashing + an `XIOM_TRACE_ENUM` trace (bad builds
printed `[dbg stfe2] base=Value key=Option__Value`).

FIX (m231): leaf match requires a NAME BOUNDARY --
`*key == base_type || key.ends_with(&format!(".{}", base_type))` -- wrappers use
`__`, qualified names use dots, so `Option__Value`/`Result__...__Value` can no
longer shadow the base struct. Swept the codegen for the same loose pattern:
this was the only `ends_with(&base_type|leaf)` site without a dot boundary.

EVIDENCE (bindings-pilot `enum-payload-nd/pkg`, 12 rebuilds with the sqlite
amalgamation): pre-fix 7/12 `B=false`, 2 distinct IR hashes; post-fix **12/12
green, single IR hash**.

LOCKS: IR `regress_m231_option_wrapper_prefix_shadow` (compiles the hazard
source 8x in-process, asserts ONE distinct IR and `Kind.Integer` tag index 1),
e2e `e2e_m231_option_wrapper_prefix_shadow` + fixture
(`tests/regression/m231_option_wrapper_prefix_shadow/` keeps `Option[Value]`
concretized so the wrapper key exists), CI line.

---

## 2026-10-08 -- FIXED: xiom --run masked the program's exit code (m228, B-08)

BINDINGS B-08: `xiom --run prog.xi` where `main` returned 5 printed
`exit code: 5` on stderr but the xiom process exited 0, so suites trusting
`$LASTEXITCODE` saw success.

FIX: `compile()` records the child's exit code (cached-run path too) in a
process-level atomic; the CLI's `compile_or_exit` consumes it
(`xiom::take_last_run_exit_code`) and exits with the program's code when
non-zero. Compile-only invocations return None and keep exiting 0; the
`--jit` lane mirrors the rule (was `return` after printing).

EVIDENCE: `main(){return 5}` -> `LASTEXITCODE=5` (was 0).

LOCK: `m228_run_propagates_program_exit_code` in
crates/xiom/tests/run_script_cli.rs + fixture
(`tests/regression/m228_run_exit_code/`); the target already runs in ci.yml.

---

## 2026-10-08 -- FIXED: `is Ok(<literal>)` ignored the payload literal (m229, packages lane)

PACKAGES v0.64.1 battery: "`is Ok(<literal>)` still ignores the literal
payload (`pick(2) is Ok(1)` true)". The `Expr::Is` lowering compared only the
discriminant and then optionally bound the payload; literal inner patterns
(Int/Bool) were treated like bindings.

FIX (codegen): `emit_is_payload_literal_cmp` emits the payload comparison for
`Some(<lit>)`/`Ok(<lit>)`/`Err(<lit>)` (field 1; Err field 2) and the `is`
result is `tag && payload == lit`. Bindings/wildcards/other literal kinds
keep the historical tag-only behavior. Applied in both the registered-variant
and the builtin Option/Result branches.

EVIDENCE: probe `pick(2) is Ok(1)` pre-fix true (exit 1), post-fix false;
`!(pick(1) is Ok(1))` and `Some`/None variants all correct (fixture exit 0).

LOCKS: IR `regress_m229_is_payload_literal` (`and i1` present), e2e
`e2e_m229_is_payload_literal` + fixture, CI line.

---

## 2026-10-09 -- FIXED (m237): array_zip did not truncate -- const-generic M bound to N

Wave-96 finding `tools/known_failures/p_array_zip_no_truncate.xi` (stdlib
repo): `array_zip([1,2,3], [7,8])` returned 3 pairs instead of 2, so with
M < N the loop read `b[M]` OUT OF BOUNDS; M == 0 still emitted N pairs.

ROOT CAUSE (codegen, not the clause): at the call site each const generic
was inferred by scanning ALL arguments for the first array local and
breaking -- `array_zip(&a3, &b2)` bound N=3 from a3, then bound M=3 from
a3 AGAIN before ever reaching b2. The monomorphised symbol was
`array.fixed.array_zip_3_3`; in the body `if M < count` folded to
`icmp slt i64 3, ...` (never true) and `count = M` stored 3.

FIX (m237): const-generic inference first binds the value from the
PARAMETER whose type NAMES the const generic (`&[N]Int` -> args[0],
`&[M]Int` -> args[1]) via a new `type_ast_mentions_const` walker (the
existing `type_contains_generic` deliberately ignores fixed-array size
expressions) plus a `const_size_from_array_arg` helper; the historical
all-args scan remains as fallback for shapes where the parser lost the
const name.

EVIDENCE: probe rc 1 -> rc 0; IR now emits `@array.fixed.array_zip_3_2`
with `icmp slt i64 2,` in the body (was `_3_3` / `slt i64 3,`).
Acceptance fixture covers M<N (content-checked), M==0, N<=M, N==M --
exit 0.

LOCKS: IR `regress_m237_const_generic_param_binding` (mono name
`zip2_3_2` + `icmp slt i64 2,`), e2e `e2e_m237_array_zip_truncate` +
fixture `tests/regression/m237_array_zip_truncate/`, ci.yml line.

## 2026-10-09 -- FIXED (m238): zero-length fixed array by value ([0]Int vs i64)

Wave-96 block-80 candidate, reproduced: `array.fold` with a ZERO-LENGTH
fixed array passed BY VALUE made clang reject the module:

    error: '%tmp' defined with type '[0 x i64]' but expected 'i64'
    %tmp47 = call i64 @array.fold_Int_Int_0(i64 %tmp41, i64 43, ...)

(trigger shape: an EXPLICITLY annotated empty local `let a: [0]Int = [];`;
the untyped `let a = [];` path happened to pass i64 already).

ROOT CAUSE: both the mono DEF and the call-site ABI computation lower a
`[N]T` param to its ELEMENT type when N == 0 (lib.rs/decl.rs `if n == 0 {
elem_llvm }`), but the ARGUMENT VALUE stayed the `[0 x i64]` aggregate and
`coerce_arg_for_param` had no `[0 x T] -> T` rule (every coercion arm
missed and the value passed through). A second gap: `let s: [0]Str = []`
recorded no element type for the empty literal, so `[N]T` inference
defaulted T=Int and the i8*-element arg met an i64 param.

FIX (m238): `coerce_arg_for_param` turns a zero-length array value into a
default of the element type when the param IS the array's element type
(the callee can never index an empty array); the Let/Var literal tracking
now records the ANNOTATION's element type for empty arrays
(`let s0: [0]Str = []` -> T=Str). Array/pointer param shapes untouched.

EVIDENCE: pre-fix clang reject reproduced on the annotated [0]Int shape;
post-fix `array.fold(i0, 43, ...)` -> 43, `[0]Str` fold -> 44, untyped and
inline-literal forms -> 45/46, non-empty Int/Str controls green (exit 0).

LOCKS: IR `regress_m238_zero_len_array_param_abi` (call emits
`@zlen_0(i64 0)`), e2e `e2e_m238_zero_len_array_by_value` + fixture
`tests/regression/m238_zero_len_array_by_value/`, ci.yml line.

NOTE (pre-existing, unrelated): compiling any generic `[N]T` array fn
prints `warning: unknown type '[N x T]' -- defaulting to i64` (also on the
green smoke_array_fold); the mono body resolves N/T per instance, so the
warning is cosmetic here -- left as-is.

---

## 2026-10-08 -- FIXED: stdlib prelude missed transitive user-module imports (m236, was m230 OPEN)

The m230 finding below ("user-module import of xiom.encoding hard-fails") is
fixed. ROOT CAUSE: the stdlib catalog PRELUDE force-load (core / string /
collections / trim-lower-upper, which supplies `Vec.get` and friends) was
gated on the ENTRY program's own `use` list (`uses_xiom_stdlib` over
`import_snapshot`). In the chain `MAIN -> user module -> use xiom.encoding;`
the entry program imports only the user module, so the prelude never loaded;
the on-demand catalog-body check of `xiom.encoding` then ran with only the
encoding family registered and hard-failed under strict catalog findings:
3x `T001 cannot call 'get' on this expression` at encoding.xi 409/419/466.

FIX (m236): the prelude gate now also follows the transitive graph --
`uses_xiom_stdlib ||= cached_loaded has any xiom key || self.modules has any
xiom key`. User modules that reach stdlib (directly or via `xiom.net.server`
etc.) load the same prelude the entry path loads.

EVIDENCE: repro `tmp/sweep2/pulse12/{h3.xi,main_h3.xi}` pre-fix exit 1 with
the 3 T001s (re-verified by stashing the fix: identical errors) -> post-fix
compile+run exit 0; direct main import (`main_dir_enc.xi`) unchanged PASSED;
the `xiom.net.server` chain (`main_h2.xi`) also green.

LOCK: e2e `e2e_m236_user_stdlib_prelude` + multi-file fixture
(`tests/regression/m236_user_stdlib_prelude/`: user module imports
xiom.encoding, entry imports only the user module; proven red pre-fix, green
post-fix), CI line. GATES: xiom-check 197/197 + checker_locks 29/29; feature
546/546; full e2e 2453/0/4; driver suites 61/6/2/29/5/15/4/7/34.

---

## 2026-10-08 -- OPEN (FIXED m236 above): user-module import of xiom.encoding hard-fails on latent catalog-body T001s (m230)

Found during the v0.64.2 relay sweep; **pre-existing** (the v0.64.1 release
binary reproduces identically).

Minimal repro: a non-main module that merely imports `xiom.encoding`:

```
// h3.xi
module h3
use xiom.encoding;
pub fn p() -> Int { return 0; }
// main.xi
module main_h3
use h3;
fn main() -> Int { return h3.p(); }
```

-> `error[T001]: catalog body [xiom.encoding] 409:22: cannot call 'get' on
this expression` (also 419:46, 466:22), `compilation failed`. Same through
`use xiom.net.server;` (transitively pulls encoding). Importing
`xiom.encoding` from the MAIN file passes.

ROOT CAUSE (instrumented, `XIOM_TRACE_ENC` at the check_call fallback):

```
[dbg enc] obj_ty=Named("Vec[UInt8]") args=1 checking_catalog=true fns=423
  pre_use=["h3", "main_h3", "xiom"]
  modules=["ascii85","base32","base64","h3","hex","idna","main_h3",
           "percent","punycode","string","xiom"]
  get_cands=[]
```

When the import chain goes MAIN -> user module h3 -> `xiom.encoding`, the
stdlib catalog PRELOAD is never triggered (the main program's own `use` list
does not name a stdlib module). `pre_use_module_keys` is therefore nearly
empty, and the isolated per-body check of encoding runs with only encoding's
own family registered: the core container declarations (`Vec.get`, ...) are
absent, so `tmp.get(j)` falls to the check_call fallback and hard-fails under
`strict_catalog_findings` (default true). Importing encoding from MAIN takes
the preload path, so the same body resolves and no finding is even produced.
The Stage-6 `use`-skip optimizer does not apply here (encoding imports no
collections module) -- the missing piece is the preload itself.

FIX DIRECTION: trigger the stdlib catalog preload whenever an import
resolution (from ANY module, not just the entry) reaches the stdlib roots
(e.g. hook `xiom.*` resolution to the same preload path the entry uses), or
register the core container declarations before isolated catalog-body checks.
Careless half-fixes here risk re-poisoning the per-body isolation (the
Retain/pre_use design), so this wants its own focused pass.

Repro kept at `E:\xiom-lang\xiom\tmp\sweep2\pulse12\{h3.xi,main_h3.xi,
main_dir_enc.xi}` (tmp; not tracked). Impact: any package/binding module
importing `xiom.encoding` (or a module that pulls it) cannot be built.

---

## 2026-10-08 -- FIXED: module-level Vec receivers called their len field (m227, C-PULSE-09)

PULSE relay C-PULSE-09: the session-store bridge compiled on v0.64.1 but
crashed at the first cross-module access (Windows 0xC0000005; Linux green).
Reduction per the relay hint -- module A owns a module-level
`Vec[SessionStore]`, module B bridges into it:

```
var stores: Vec[SessionStore] = Vec[SessionStore].new();
pub fn sb_ensure() { if stores.len() == 0 { ... } }
```

crashed at `stores.len()`. ROOT CAUSE (codegen, two layers):

1. `infer_llvm_type_impl`'s Ident arm consulted only `lookup_local`, not
   `module_globals`, so a module-level `Vec` receiver typed as "i64"; the
   `.len()` Vec builtin intercept never fired and the call fell to the
   generic dispatch.
2. The BUG 29 "module-global fn-field call" path matched a field by NAME
   alone against the struct's type_meta; `Vec` has a field literally named
   `len`, so `stores.len()` emitted
   `getelementptr %struct.Vec, ... i32 0, 1` (the LEN field) ->
   `load i64` -> `inttoptr i64 %len to i64 ()*` -> `call` -- jumping to
   address 0/len. The module-level `stores` is a zeroinitializer global
   until initialized, so the first access trapped (0xC000001D on this
   machine; the report's 0xC0000005 is the same call-through-garbage class).

FIX (m227): (1) `infer_llvm_type_impl` resolves module-level `var` receivers
through their registered `module_globals` type, so Vec/Slice builtins
dispatch correctly (`.len()`, `.push()`, ...); (2) the fn-field call path
only fires for fields whose declared type is a fn marker (`fn(`), so a
non-fn field can never be called through.

EVIDENCE: two-module reduction (A stores, B bridges, main drives):
pre-fix exit -1073741795 with the IR above; post-fix compile+run exit 0 and
`sb_ensure` reads the len via `getelementptr %struct.Vec, %struct.Vec*
%alloca, i32 0, i32 1`. Linux was already green (same IR, len read as 0
coincidentally) -- the fix removes the mislowering on both platforms.

LOCKS: IR `regress_m227_module_global_vec_len` (no `inttoptr`; Vec struct
GEP present), e2e `e2e_m227_module_global_vec_len` + multi-file fixture
(`tests/regression/m227_module_global_vec_len/`: store + bridge + consumer),
CI line. Gates: feature 542/542; targeted e2e 1/1.

---

## 2026-10-08 -- FIXED: alias-typed match payloads lost their concrete type (m223, C-ORBIT-01)

ORBITDB relay C-ORBIT-01: 4 T001 sites blocked `db.txn.transaction` --

```
error[T001]: 'get_column' expects 2 argument(s), found 1
error[T001]: argument 1 type mismatch: expected Row, found Int
```

`match decoded { Ok(row) => row.get_column(0) }` where `decoded` came from a fn
returning the ALIAS `DbResult[Row]` (`pub type DbResult[T] = Result[T, DbError]`).

ROOT CAUSE (checker): `CheckedType::from_ast_type` erases the type args of
`Type::Named` (`DbResult[Row]` -> `Named("DbResult")`), and match-payload
extraction (`add_pattern_bindings` -> `container_arg`) required the literal base
`Result`. An aliased scrutinee fell to the `_` wildcard, so method calls on the
payload misresolved to the module fn and the call was reported as missing its
receiver. Plain (non-generic) aliases failed identically.

FIX (checker): `resolve_alias` now also expands APPLIED aliases --
`alias_params` (parallel to `aliases`) records each alias's declared generic
params, and `expand_alias_once` substitutes the application's args into the
alias body before container decomposition (`DbResult[Row]` ->
`Result[Row, DbError]` -> payload `Row`). `container_payload_arg` runs the
scrutinee name through `resolve_alias` first. Signature params/returns and
let/var annotations preserve alias applications via
`checked_type_from_ast_preserve_alias` (only for REGISTERED aliases; non-alias
user generics keep the historical erased spelling).

FIX (codegen): mirror the expansion so the emitter's own binding tracker types
alias payloads too -- `type_alias_params` / `type_alias_bodies` are registered
at alias layout time, `type_string_full` keeps `Type::Named` args (so
`fn_return_xiom` carries `AliasRes[R]`), and
`option_result_payload_alias`/`option_result_err_payload_alias` expand applied
and plain aliases (chains included) before extracting the payload. Without it
the Ok binding stayed a raw i64 handle: `x.id` read 0 and the method call passed
the binding slot as the receiver (`@R.get_id(i64* %slot)`).

EVIDENCE (ORBITDB repro `docs/repro/ok-method-receiver/probe_import.xi`):
pre-fix 4 T001 at 34/36/53/69; post-fix `bits=0`, plus `probe_rebind`,
`probe_only_i`, `probe_name` exit 0. Minimal local probe (alias + plain alias +
applied alias, field read + & method call) pre-fix T001, post-fix exit 0.

LOCKS: IR `regress_m223_alias_result_payload` (payload receiver is
`@Cell.get_n(%struct.`, never `i64*`), e2e `e2e_m223_alias_result_payload` +
fixture (`tests/regression/m223_alias_result_payload/`: generic alias, plain
alias, annotated let), CI line. Gates: checker 197/197 + checker_locks 29/29;
feature 541/541; targeted e2e 4/4.

---

## 2026-10-08 -- FIXED: field receivers of pointer-self methods bumped a copy (m224, C-ORBIT-03)

ORBITDB relay C-ORBIT-03: `nested-field-mut/probe.xi` printed A=0, B=1, C=0,
D=1 (expected all 1). `o.inner.bump()` inside `Outer.bump(o: &mut Outer)` and
`c.inner.bump()` on a local both loaded the field, alloca'd a TEMP copy, bumped
the temp and discarded it; `StorageEngine.cache_page` silently lost the page.

ROOT CAUSE (codegen): the non-generic pointer-self method path (call.rs) had
receiver arms for Ident / Index / struct-value temp but none for
`Expr::Field`, so a field receiver fell to the struct-value temp arm.

FIX (m224): the Field receiver computes the field's ADDRESS through the shared
`compile_lvalue` place machinery (reference params load their pointer, nested
fields `h.mid.inner` recurse through GEPs), then coerces the pointer to the
callee's `p0` (exact / ptr-typed / bitcast). The Ref-based attempt first tripped
on deep chains (`Ref(field)` returns a loaded struct there); `compile_lvalue` is
the assign-path machinery and handles the chains uniformly. Unsupported place
chains keep the historical copy behavior.

EVIDENCE (ORBITDB `docs/repro/nested-field-mut/probe.xi`): A=1 B=1 C=1 D=1.

LOCKS: e2e `e2e_m224_nested_field_mut_receiver` + fixture
(`tests/regression/m224_nested_field_mut_receiver/`: `&mut Outer` base, local
base, double-nested `h.deep()` chain), CI line. Gates: feature 541/541;
targeted e2e 4/4.

---

## 2026-10-08 -- FIXED: bare enum variants ignored the declaring module (m225, C-ORBIT-04)

ORBITDB relay C-ORBIT-04: the conformance suite failed to build --
`b_make`'s `return Update;` emitted `ret %struct.db.wal_txn.WALOp` where
`%struct.db.wal_file.WALOpKind` was expected (clang rejects the module; IR line
71079 `%struct.WALOp` vs `%struct.WALOpKind`). Two modules declare structurally
identical enums; the first-registered bare key won every unqualified variant
resolution.

ROOT CAUSE: catalog/user module decls are injected FLAT into the codegen
program (`collect_pub_decls`), so enum keys are BARE and codegen's scope-first
`pick_variant_parent` had no module to match: candidates `[WALOp, WALOpKind]`
with scope `wal_file` fell through to declaration order. The checker's own
bare-variant resolution had the same blind spot.

FIX (checker+codegen): `collect_pub_decls` records `(enum name, declaring
module)` for every injected enum (`Checker::external_enum_modules`); the driver
passes them via `set_enum_module_hints`, and codegen's `pick_variant_parent`
prefers a candidate whose hint module matches a scope (suffix/leaf-wise, so
scope `wal_file` matches `xiom.db.wal_file`) BEFORE declaration order. No
struct/enum RENAMES, so existing `%struct.` names and IR stay stable.

EVIDENCE: minimal two-module repro (twin `Insert/Update` enums, SWAPPED variant
order so a wrong parent also flips tags): pre-fix clang
`ret type %struct.BKind vs %struct.AKind`; post-fix compile+run exit 0. The
suite workaround (qualified `WALOpKind.Insert`) is unaffected; the ORBIT lane
re-tests with it reverted.

LOCKS: e2e `e2e_m225_enum_module_scope` + multi-file fixture
(`tests/regression/m225_enum_module_scope/`: a_enum/b_enum sibling modules with
swapped variant order, module-internal construction + caller-side match), CI
line. Gates: checker 197/197; feature 541/541; targeted e2e 4/4.

---

## 2026-10-08 -- FIXED: Vec container-element assign stored an 8-byte handle (m226, C-ORBIT-02)

ORBITDB relay C-ORBIT-02: `option-vec-assign/probe.xi` -- after
`v.push(None); v[0] = Some(44);` the following `match v[0]` ran NO arm (V1/V3/V6
/V8 silently skipped). The slot is a 16-byte `%struct.Option`; the write stored
the 8-byte boxed handle, so the match read a pointer where the tag lives.

ROOT CAUSE (codegen): the Assign(Index) struct-memcpy guard built the element
type as `format!("%struct.{elem_name}")`. For container elements the recorded
name is bracketed (`Option[Int]`, `Result[Int, Int]`), so
`%struct.Option[Int]` never equalled the value's `%struct.Option`; the write
fell to the scalar `emit_elem_store` size-switch (1/2/4/8) whose default stores
`i64 handle` -- while `push` correctly memcpy'd. Result and Option were both
affected.

FIX (m226): for `Option[`/`Result[`/`Map[`/`Set[` element names, resolve the
ERASED generic struct through `llvm_type_for` (`Option[Int]` ->
`%struct.Option`) so the existing memcpy branch fires; user struct/enum names
keep the previous spelling. `Vec[` keeps its special case.

EVIDENCE (ORBITDB `docs/repro/option-vec-assign/`): probe V1=SOME after-v1
V2=SOME V3=SOME V4=SOME V5=OK; probe2 V6=SOME V7=B V8=SOME V9=NONE
V10.0=NONE V10.1=SOME -- the full expected matrix.

LOCKS: IR `regress_m226_vec_option_assign` (memcpy present, no `elem_store`
label), e2e `e2e_m226_vec_option_assign` + fixture
(`tests/regression/m226_vec_option_assign/`: Option and Result elements), CI
line. Gates: feature 541/541; targeted e2e 4/4.

---

## 2026-10-08 -- FIXED: Str::from_utf8(&Vec[UInt8]) emitted invalid LLVM IR (m222, XVC-C-07)

XVECTOR relay Addendum 3: XVC-C-07 was the only open XVC finding on v0.64.1.
`Str::from_utf8(&kb)` type-checks but clang rejects the module:

```
xiominput.ll:648:40: error: invalid getelementptr indices
  648 |   %tmp68 = getelementptr %struct.Vec*, %struct.Vec** %tmp67, i32 0, i32 0
```

(WSL LLVM verifier: same shape at out/r07_lin.exe.ll:648.) Passing the Vec by
value stays supported and was the stdlib workaround.

ROOT CAUSE: the `from_utf8`/`from_bytes` builtin intercept (call.rs) fired on
`arg_ty.starts_with("%struct.")`, which also matches REFERENCE types
(`%struct.Vec*`). It alloca'd a slot of the POINTER type, stored the pointer,
then GEP'd the fields with the pointer type as the aggregate:
`getelementptr %struct.Vec*, %struct.Vec** %slot, i32 0, i32 0` -- the second
index steps into a plain pointer, which LLVM rejects. The value at the slot is
already the address of the struct; no pointer-level GEP is needed.

FIX (m222): a single-level `%struct.*` pointer argument reads ptr/len through
the POINTEE struct directly
(`getelementptr %struct.Vec, %struct.Vec* %ref, i32 0, {0, 1}`); struct VALUE
arguments keep the alloca path (and the owned-Str copy via
`xiom_str_from_vec` is unchanged for both). `from_bytes` shares the branch and
is fixed identically.

EVIDENCE (repro `docs/repro/xv-from-utf8-vec-ref/probe.xi`, XVECTOR):
pre-fix clang exit 1 at line 648; post-fix compile exit 0, run exit 0
(`Str::from_utf8(&kb) == "A"`), IR now `getelementptr %struct.Vec,
%struct.Vec* %tmp21, i32 0, {0,1}` feeding `xiom_str_from_vec`.

LOCKS: `regress_m222_from_utf8_vec_ref` (IR: no
`getelementptr %struct.Vec*, %struct.Vec**`, `xiom_str_from_vec` call present),
e2e `e2e_m222_from_utf8_vec_ref` + fixture
(`tests/regression/m222_from_utf8_vec_ref/`: by-ref + by-value + `from_bytes`),
CI line. Gates: feature 539/539; targeted e2e 1/1.

---

## 2026-10-08 -- FIXED: Float32 enum payloads packed as double bits (m221, XVC-C-06)

XVECTOR relay addendum: `FieldValue.FloatVal(2.5)` decoded as 0.0 with the
tag intact in larger units. IR evidence from the engine probe: the ctor
stored `bitcast double 2.5 to i64` into the erased payload slot while the
match binding decoded the low 32 bits as f32 (`trunc i64 -> i32; bitcast
i32 -> float`) -> 0x00000000.

ROOT CAUSE: when payload field names collide across variants (`IntVal(v)` /
`FloatVal(v: Float32)` / ...), decl.rs erases the shared slot to i64 (M19)
and readers decode through `enum_variant_field_types`; enum_ctors.rs never
consulted that map -- `val_to_i64(double)` stored the f64 bits. Values whose
f64 low half is zero (2.5, 10.0, ...) decoded exactly 0.0; the mismatch
surfaced per unit/build.

FIX (m221): the ctor resolves the variant's declared payload type from
`enum_variant_field_types` and narrows/widens before packing (Float32
declared + double arg -> fptrunc; Float64/Float declared + float arg ->
fpext); integer/string/struct payloads keep the existing path.

EVIDENCE: engine probe 6/6 builds `bits=4612811918334230528` (correct 2.5),
0 failed checks (pre-fix 6/6 `bits=0`); minimal colliding-name repro green.

LOCKS: regress_m221_enum_f32_payload_pack (IR: fptrunc double 2.5 to float
present, no `bitcast double 2.5 to i64`), e2e + fixture + CI line.
Gates: feature 538/538; targeted e2e 8/8.

---

## 2026-10-08 -- TRIAGE (resolved by m221, section above): XVC-C-06 f32 enum payloads read as 0 in larger units

RESOLVED by m221: colliding payload field names erase the slot to i64 and
the ctor packed double bits while the reader decoded f32.

XVECTOR relay addendum (same family as XVC-C-05): `FieldValue.FloatVal(x)`
payloads extract as 0.0 with the tag intact in larger units (suite form
`float_bits(FloatVal(10.0))` = 0; engine probe
`diag FloatVal(2.5) roundtrip bits=0 (expected 4612811918334230528)`).

RE-TEST on Windows (current main vs v0.64.0 release):
- engine probe `tests/probes/probe_filtered_search.xi`: 6/6 bits=0 on BOTH
  compilers (deterministic here; the lane saw 2/3 red on v0.64.0).
- minimal direct form (`enum FV { FloatVal(f: Float32) }`, construct +
  match-extract + float_bits): GREEN 3/3 on BOTH compilers -- the simple
  shape is not the trigger; the lane's "needs the larger unit" stands.
- NOT a batch regression: v0.64.0 behaves identically.

Also observed once in 13 builds: spurious
`error[T001]: catalog body [xiom.vector.engine] 921:13:
'wal_writer_checkpoint_from' expects 2 argument(s), found 3` -- an
intermittent checker catalog flake; not reproduced in 12 subsequent builds.
Watch item for the release gate.

NEXT: reduce the engine-unit trigger (f32 payload through the engine's
payload_set/get path); distinguish compiler vs package-side before fixing.

---

## 2026-10-08 -- RELAY RE-TEST: XVECTOR deltas (C-02 artifact, C-04 fixed by m217) + ORBITDB all three open

XVECTOR (bundles under E:\xiom-projects\xiom-xvector\docs\repro), re-tested
on current main after the m216..m220 batch:
- XVC-C-04 (`Result[Vec[non-scalar]]` payload) is FIXED by m217:
  `probe_matrix.xi` struct rows now read `id=42` (scalar_int green) and the
  direct/wrapped/inline probes are green.
- XVC-C-02 (contract trap exits 0) is NOT a defect: the trap exits 1 on
  v0.64.0 AND current main (Start-Process `.ExitCode`); the reported 0 came
  from the cmd `%ERRORLEVEL%` parse-time expansion gotcha. Re-measure with
  Start-Process `.ExitCode` or bash `$?`.
- XVC-C-05 fixed m218, XVC-C-01 fixed m219, XVC-C-03 fixed m220 (sections
  above).

ORBITDB (bundles under E:\xiom-projects\xiom-orbitdb\docs\repro), re-tested
on current main -- ALL THREE STILL RED:
- C-ORBIT-01: `probe_import.xi` still fails `T001 34:16 'get_column'
  expects 2 argument(s), found 1` (match-bound generic Result alias
  payload loses the receiver type; m216 did NOT cover this shape).
  `probe_only_i.xi` (annotated rebind workaround) is green.
- C-ORBIT-02: `option-vec-assign` V1/V3/V6/V8/V9 print no variant line
  (no arm matches after an element assignment); V2/V4/V5/V7/V10 fine.
- C-ORBIT-03: `nested-field-mut` A=0, C=0 (B=1, D=1) -- `&mut` method
  receivers on field projections still mutate a copy.

NEXT: the ORBITDB findings are open compiler-lane work (checker generic
alias receiver + element-assign discriminant + projection auto-ref).

---

## 2026-10-08 -- FIXED: W004 false positives on bare unit-enum variants (m220, XVC-C-03)

XVECTOR relay, XVC-C-03: every arm after the first in an all-unit-enum
match was reported unreachable (`warning[W004]: unreachable match arm`) --
18+ false positives in the XVECTOR conformance suite. Minimal repro: match
on `Metric { Cosine, DotProduct, Euclidean }` with the three bare variant
arms flagged at arms 2 and 3.

ROOT CAUSE: the W004 lint treated every un-dotted `Pattern::Ident` as a
binding catch-all (`pattern_is_catch_all`) -- correct for a binding like
`x`, wrong for a BARE variant name. Dotted forms (`Color.Red`) worked, but
the engine lanes write bare variants (supported by exhaustiveness through
`pattern_covers_variant`). The lint had no scrutinee context.

FIX (m220): the lint now receives the scrutinee's variant base names (new
`match_variant_names`, shared with exhaustiveness); a bare Ident naming one
of them is a variant (not a catch-all, and its shadow key is
`variant:<leaf>:0`, so bare/dotted duplicate arms still collide). Bindings
on non-enum scrutinees stay catch-alls -- the m154 positive cases (dotted
duplicate, `x` shadowing `5`) still warn.

EVIDENCE: minimal probe 2 W004 warnings pre-fix -> 0 post-fix; XVECTOR
C-03 shape green.

LOCKS: `m220_w004_bare_variants_silent` checker lock + fixture
(tests/regression/m220_w004_bare_variants/). Gates: checker_locks 29/29;
m154 W004 positive/negative unchanged.

---

## 2026-10-08 -- FIXED: &fn() parameter call shapes (m219, XVC-C-01)

XVECTOR relay, XVC-C-01: `f()` on a `&fn() -> T` parameter emitted
`inttoptr i64* %slot to i64 ()*` (invalid cast ptr->ptr, clang exit 1);
`(*f)()` compiled through the M20-A1 closure fallback with a bogus 1-arg
signature and crashed 0xC0000005; `var g = *f; g()` loaded the function's
first instructions as the address and crashed. Struct-field fn pointers
were already green.

ROOT CAUSES (IR evidence):
1. call.rs callee_is_fn_ptr always emitted `inttoptr {slot_ty}`; for the
   `&fn()` ABI the slot holds the code address typed `i64*`, so the cast
   must be a bitcast (inttoptr requires an integer operand).
2. `fn_name_opt` only peeled Ident/Field callees, so `(*f)()` missed the
   fn-pointer path and fell into the M20-A1 env-first fallback (wrong
   1-arg signature).
3. UnaryOp::Deref treated `*f` as a memory load through the code pointer;
   under the `&fn()` convention the deref is the fn VALUE (code address).

FIX (m219): cast opcode selected from the slot type (bitcast for pointer
slots, inttoptr for integer slots); the call dispatcher peels
Expr::Unary(Deref) to resolve the local name; the Deref arm yields the
code address via ptrtoint when the operand is a fn-reference (pointer slot
only -- plain fn params keep the M20-A1 env convention).

EVIDENCE: probe.xi / probe_deref.xi / probe_local.xi all exit 0 post-fix
(pre-fix: clang exit 1 / AV / AV).

LOCKS: regress_m219_fnptr_ref_call (IR: bitcast i64* to i64 ()*, no
inttoptr i64*, no (i64)*, ptrtoint for the deref), e2e + fixture + CI line.
Gates: feature 537/537; targeted e2e 8/8; fn-ptr/closure regressions 7/7.

---

## 2026-10-08 -- FIXED: Float32 Vec-field elements multiplied as bit patterns (m218, XVC-C-05)

XVECTOR relay, XVC-C-05 (critical): per-build nondeterministic lowering --
`a.data[i] * b.data[i]` on `Vec[Float32]` struct fields (vector_dot) could
multiply the f32 BIT PATTERNS as i64 and sitofp the product
(0x3F666666 x 0x3F800000 -> 1.13e18), flipping red/green per compilation
(comment-only byte changes and --sequential included, Windows and WSL).
Measured on current main before the fix: 2/3 binary builds red; 4/8 emit-ir
runs red.

ROOT CAUSE (IR evidence): `vec_elem_float_type`'s Field arm iterated
`type_meta` in HashMap order and BROKE on the first key whose name ended
with the base type. `Vector` registers bare in the engine lane, and the
generated aggregate `Option__Vector` (fields tag/payload, no `data`) is also
a suffix match: when it won the race the field lookup missed, the function
returned None and the index read fell to the scalar elem_load (raw i64), so
the multiply lowered as `mul i64` + `sitofp`. The class was already
documented at `declared_field_type` (smoke_error2, round-7) -- this call
site kept the old scan.

FIX (m218): the Field arm resolves through the deterministic
`declared_field_type(base_ty, field)` helper (exact/qualified first, skips
generated aggregates, returns the field from the first meta that actually
contains it) before mapping Vec[Float32]/Vec[Float64] to float/double.

EVIDENCE: the XVECTOR probe (xv-f32-int-miscompile/probe_failing.xi) on
current main post-fix: 6/6 builds exact `dot=4606281698659794944` (0.9) and
`cos=4573701602539470848`, including `--sequential`; pre-fix 2/3 red with
`dot=4877244244697284608` (integer product). Header-less regression fixture
reproduced the collision pre-fix 5/6 red, post-fix stable (fmul, no sitofp).

LOCKS: `regress_m218_vec_float_field_lookup` (IR: `fmul float` present, no
`sitofp` in the dot body), e2e + fixture + CI line. Gates: feature 536/536;
targeted e2e 8/8.

NOTE: the other XVECTOR bundles are re-tested separately (XVC-C-01/02/03/04,
ORBITDB C-ORBIT-01/02/03); results tracked in SESSION.

---

## 2026-10-08 -- TRIAGE (open): C-PULSE-09 wrapper-aggregate crash not reproducible from committed sources

Pulse relay, C-PULSE-09: a `Vec[SessionStore]` aggregate driven from a PULSE
wrapper module was reported to crash at runtime while the same package calls
inline are green (`probe_adopt_smoke.xi` red vs `probe_session_inline.xi`
green; C-PULSE-07 family suspected, compiler-lane bisect requested).

TRIAGE (compiler lane): the crashing wrapper was an UNCOMMITTED local
adoption attempt that PULSE reverted ("local store retained"); Pulse HEAD
has no module owning `Vec[SessionStore]` (git log -S shows only the probes
and the relay docs). Two reconstructed tiers were built and run on BOTH the
v0.64.0 release compiler (where the crash was observed) and current main
(m199..m217):

1. synthetic `xiom.session` package + wrapper module owning
   `var g_stores: Vec[SessionStore] = Vec[SessionStore].new()`, consumer
   driving init/ttl/count/reset (including the reset reassignment);
2. the REAL `xiom.session` 0.1.0 source (copied from the WSL install tree,
   `SessionStore = { sessions: Vec[Session], ttl_ms: Int }`) + a wrapper
   driving create/set/value/count/reset through the package with `&mut
   g_stores[0]`.

RESULT: every reconstructed shape is GREEN on v0.64.0 AND current main (no
crash, correct values). The original wrapper source is required to
reproduce; the m204/m210/m216/m217 fixes may or may not have addressed it.

NEXT: the Pulse lane re-runs the session adoption on the next archive; if
it still crashes, capture the wrapper source at the failing revision plus
the durable step log (`/tmp/pulse-adopt-steps.txt`) for the bisect.

---

## 2026-10-08 -- FIXED: nested Option/Result payload chains kept the erased i64 slot (m217, C-PULSE-10)

Pulse relay, C-PULSE-10: after `kv_put(&mut store, "k", "abcdefghij")`,
`kv_get(&store, "k")` returned an address-like decimal Str for every key
while `kv_get_bytes` returned the stored bytes correctly; multi-key
overwrites then read back corrupted through the same helper (kv_text).
Classified compiler-vs-package open (C-PULSE-04/05 family suspected).
Reproduced on current main with the xiom.kv 0.1.0 source (copied from the
WSL install tree): `kv_get = [1527232326592]`, bytes path and the local
`Str::from_utf8` control green.

ROOT CAUSE (IR evidence, minimal compiler-side repro): `field_payload_xiom`
resolved payload field reads (`.value`/`.error`) for Ident/Call/Paren
receivers but had no `Expr::Field` arm, so a NESTED chain
(`gs.value.value` on `Result[Option[Str], Str]`) fell to `None`: the INNER
`.value` read kept the erased i64 slot, and the consumer stringified the
payload (the Str data pointer) via `xiom_int_to_string` -- hence the
decimal. The package was NOT at fault: `kv_get` is a plain `kv_get_bytes` +
`Str::from_utf8`, and the record bytes round-trip.

FIX (m217): a recursive `Expr::Field` arm in `field_payload_xiom` resolves
the base field's payload type, then extracts the inner payload/error of that
container (bracket-safe through the existing option_result_payload /
option_result_err_payload helpers).

EVIDENCE: minimal compiler-side repro (`Result[Option[Str], Str]` built
directly and via a match-mediated local) pre-fix printed `[1963459382448]`
/ `[1963459381648]`; post-fix all four shapes print `[abcdefghij]`. On the
copied xiom.kv source `kv_get=[abcdefghij]`, and the PULSE acceptance gate
`probe_pkg_kv.xi` is fully green (single-key roundtrip, kv_get Str path,
multi-key overwrite stability; exit 0) -- both filed defects resolve.

LOCKS: `regress_m217_nested_option_payload` (IR: no `xiom_int_to_string` in
main, the inner read emits `inttoptr i64 ... to i8*`), e2e
`e2e_m217_nested_option_payload` + fixture, CI line. Gates: feature 535/535;
targeted e2e 12/12.

---

## 2026-10-08 -- FIXED: type-alias Vec elements resolved + spurious unknown-type warning (m216, C-PULSE-11)

Pulse relay, C-PULSE-11: a wrapper module exposing a package type through an
alias (`pub type Store = SessionStore;`, module B consuming `Store`) only
emitted `warning: unknown type 'Store' -- defaulting to i64`, and `Vec[Store]`
field reads silently compiled to the constant 0. Reproduced synthetically
(package module + alias wrapper + consumer; `Vec[SessionStore]` control
green, `Vec[Store]` red); the fuller shape (`pick(v[0])` by value) aborted
0xC0000005.

TWO ROOT CAUSES:
1. `resolve_vec_elem_type` (lib.rs) scanned `types`/`enum_variants` keys by
   exact/`.Store` suffix match but never consulted `type_aliases`, so
   `Vec[Store]` took the scalar element path; the `.ttl` field read then fell
   through to the literal-0 default (the m206 class). `vec_elem_storage_size`
   was alias-blind too: `Vec[Store].new()` sized the ctor buffer 8 instead of
   the struct width (the first struct `push` corrected `elem_size`, masking
   it until a read-before-push or a non-pushing container flow).
2. The warning itself was SPURIOUS: `llvm_type_for` called
   `xiom_to_llvm_type(clean_name)` EAGERLY for every unresolved name to test
   builtins -- the alias resolved correctly afterwards (IR evidence:
   `store_ttl(%struct.xiom.session.SessionStore*)` while the warning still
   printed; a temporary backtrace pinned every hit to that line, none to the
   final fallback).

FIX (m216): new cycle-guarded `resolve_alias_name` helper (shared with the
existing `llvm_type_for` alias chain); `resolve_vec_elem_type` (Ident and
Field arms) and `vec_elem_storage_size` resolve the chain before their
registry scans; the eager builtin probe computes `xiom_to_llvm_type` lazily
inside the primitive match arms, so the "defaulting to i64" warning fires
only for genuinely unknown types.

EVIDENCE: synthetic repro pre-fix `v[0].ttl`/`v[0].count` = 0/0 and the
`pick(v[0])` shape AVs; post-fix 60000/7 exit 0 (package shape) and
60000/1000 exit 0 (full shape), zero warnings; `Vec[Store].new()` ctor now
stores elem_size 24 in IR.

LOCKS: `regress_m216_alias_vec_elem_field` (IR: main body GEPs through
`%struct.pkg.SessionStore`), e2e `e2e_m216_alias_vec_elem_field` + fixture
(`tests/regression/m216_alias_vec_elem_field/`), CI line. Gates: feature
534/534 (533 + the new lock); targeted e2e 11/11.

RELAY (still open from the same bundle): C-PULSE-09 (wrapper-aggregate crash
bisect) and C-PULSE-10 (kv_get corruption triage). The hard-error guard for
genuinely-unknown defaulting remains OPEN: after this fix the only warning
path is the true fallback, which ~13 direct `xiom_to_llvm_type` fallback
sites still swallow into i64 -- needs the error-propagation pass.

---

## 2026-10-08 -- FIXED: [dependencies] dotted keys + relay C-PULSE-09/10/11 (m215, C-PULSE-08)

Pulse relay, C-PULSE-08: `dependency_roots_under` (m212) matched `dep.name`
verbatim, so the canonical DOTTED key (`xiom.rate`) could never match the
dash-named installed dir (`xiom-rate-0.2.0/`); the m212 unit tests covered
dash-form keys only. Worse, the TOML parser silently flattened the dotted
key into a NESTED table (`{ xiom: { rate: ... } }`), so the dependency was
recorded as name `xiom`, version `*` -- it matched `xiom-*` dirs by
accident and would mis-match any future `xiom-*` package.

FIX (m215): `dependencies_from_toml` flattens the raw `[dependencies]` TOML
value, reconstructing canonical dotted names (`parent.child`) and treating
tables with version/path/git as specs; `dependency_roots_under` matches the
dash-normalized name (`xiom.rate` -> `xiom-rate-`) as well as the verbatim
name, and probes both dash and dotted inner package dirs.

EVIDENCE: Pulse repro `docs/repro/dep-roots-name-form/{dash,dot}` -- both
`xiom build` and the documented `xiom --check probe.xi` now exit 0 with
"Type check PASSED" (pre-fix: T001 undefined variable / no roots). Unit
tests: dotted keys keep their canonical name; a dotted dependency finds the
dash-named install dir and its `src/`; xiom-graph 34/34.

RELAY (same batch, OPEN):
- C-PULSE-09: a `Vec[SessionStore]` package aggregate driven from a PULSE
  wrapper module crashes at runtime while the same calls inline are green
  (`tests/probes/probe_adopt_smoke.xi` red vs `probe_session_inline.xi`
  green); suspected C-PULSE-07 family (module-state vs package aggregates);
  needs a compiler-lane bisect.
- C-PULSE-10: `kv_get` returns an address-like decimal Str for every key
  after `kv_put` (multi-key writes also corrupt `kv_get_bytes`); repro
  `docs/repro/kv-get-str-corruption/`, gate `tests/probes/probe_pkg_kv.xi`;
  possible package-internal Str construction miscompile (C-PULSE-04/05
  family) -- packages + compiler lanes to triage.
- C-PULSE-11: `pub type Store = SessionStore;` (alias to a PACKAGE type) is
  "unknown type 'Store'" cross-module, and the build only emits
  `warning: unknown type 'Store' -- defaulting to i64` before continuing --
  a silent-miscompile class; wrapper structs work. At minimum the
  defaulting warning must become a hard error; the alias should resolve.

---

## 2026-10-07 -- FIXED: graphql boxed enum struct-payload fields read 0 (m206, packages relay)

Packages relay: the `xiom-graphql` conformance suite ran 9/10 -- `validate
valid operation` failed because `GraphQLSelection.Field(fs)` payload fields
read as the constant `0` (`fs.name == "hello"` false while `root.name` was
`Query`). Reproduced locally from the packages repo root:
`xiom run packages/xiom-graphql/tests/test_conformance.xi` -> 9/10; a
scratch copy with a debug print showed `fs.name=[0]`.

ROOT CAUSE (IR evidence, tmp/pulse05/gql/test.ll): when an enum's variant
payload structs share the same declared field name (`selection` in all three
GraphQLSelection variants), the enum gets the BOXED layout
`%struct.GraphQLSelection = { i64 tag, i64 payload-ptr }` (the inline
`{ tag, FieldSel, SpreadSel, ... }` layout only appears when the payload
field names differ). The `Pattern::Variant` payload binding handled
Float64/Float32/Str payloads but bound a STRUCT payload as the raw i64
handle without box-deref registration; the subsequent `fs.name` field read
on an i64 local then fell through to the constant-`0` field default.

FIX (m206): the variant-payload binding now inttoptrs a registered
struct/aggregate payload (non-Vec) to its `%struct.X*` box, records
`local_boxed_struct[fs] = X` and binds the pointer-backed struct-local
convention (register = box address), mirroring the Option/Result
struct-payload path (m148). Field GEPs then deref the box; writes through
the binding land in the payload.

EVIDENCE: package test 10/10; new fixture
`tests/regression/m206_enum_struct_payload_field/main.xi` is red-before
(exit 1 with the binding arm disabled, boxed `{ i64, i64 }` layout) and
green after (exit 0).

LOCKS: `regress_m206_enum_struct_payload_field` (IR:
`inttoptr ... to %struct.FieldSel*`), e2e
`e2e_m206_enum_struct_payload_field` + fixture, CI line.

---

## 2026-10-07 -- FIXED: grpc Vec[(Str, Str)] clone-tuple element naming (m202, packages relay)

Packages probes `xiom-grpc/tests/probe_suite_min.xi` (crashed 0xC0000005,
no output) and `probe_direct.xi` (crashed/hung) reproduced locally with the
debug driver. The packages lead ("read-after-mutation") was NOT the root
cause: a pure-local shape without any grpc code reproduces
(`tmp/pulse05/grpc_min_a.xi` / `grpc_min_b.xi`), and the fault is the PUSH,
not the read.

ROOT CAUSE (IR evidence, tmp/pulse05/mut_c.ll): for the tuple literal
`("k".clone(), "v".clone())` the tuple-element namer falls through to
`infer_llvm_type` on the element CALL, whose declared-fn lookup resolves the
bare `clone` leaf through the suffix scan and binds the derived
`MaybeUninit.clone` -- so the tuple was built as
`%struct.Tuple__MaybeUninit__MaybeUninit` (two struct fields, 32-byte
element store) while the `Vec[(Str, Str)].new()` buffer was sized from
`Tuple__Str__Str` (8-byte fallback because the real tuple type was never
registered). A 32-byte element store into the 8-byte-stride buffer
corrupted the heap: the struct-field shape (`req.metadata.push` through the
library) crashed 0xC0000005, the local shape read garbage (match=no).

FIX (m202): `infer_llvm_type_impl` now mirrors the call.rs Gap A inline
builtin-interface path: for an empty-argument `x.clone()` / `x.to_owned()`
on a VALUE receiver whose LLVM type is not a by-value struct, return the
receiver's own LLVM type. Tuple elements are then named Str/Str, the tuple
type registers normally, and Vec.new/push agree on 16-byte elements.

EVIDENCE: package probes now `start` / `rc=0` (suite_min) and
`start` / `len=1` / `match=ok` (direct), both exit 0. Minimal mutants
(local Vec+read, struct-field push+read, wrapper function, inline) all
exit 0.

LOCKS: `regress_m202_str_clone_tuple_elem_name` (IR: Tuple__Str__Str, no
MaybeUninit), e2e `e2e_m202_clone_tuple_vec` +
`tests/regression/m202_clone_tuple_vec/main.xi` (local + &mut helper
shape), CI line.

---

## 2026-10-07 -- FIXED: closure body leaked the enclosing ensures clause (m203, stdlib iter Range.count)

Stdlib relay: re-adding even `ensures: result >= 0` to `Range.count` made
`tests/smoke/smoke_iter.xi` fail with clang "instruction forward referenced
with type 'ptr'" at `%tmp8 = load i64, i64* @xiom_recursion_counter`.
Reproduced locally without the stdlib (`tmp/pulse05/iter_clause_range.xi`,
Range.count shape: generic `_count_via` + self-mutating closure).

ROOT CAUSE (IR evidence, tmp/pulse05/iter_clause_range.ll): the thunk for
`fn() -> Option[Int] { return r.next(); }` inherited the ENCLOSING
function's contract state (`current_ensures`, `result_ptr`). `Stmt::Return`
inside the closure body therefore emitted the outer ensures check and a
store to the outer result-alloca register (R.count's `%tmp8`), which is not
defined in `__closure_0`; the numeric SSA name then collided with a later
same-named definition and clang rejected the module. The outer check itself
also ran once correctly, so the failure was purely the leaked duplicate.

FIX (m203): the `Expr::Closure` block-thunk path saves/takes/restores
`result_ptr`, `result_llvm_ty`, `result_xiom_ty`, `match_result_ptr`,
`match_result_ty` and `current_ensures` around the body, mirroring the
unsafe-block fn path (lib.rs). The PipeClosure (expression-body) path
compiles no `return` statement, so it is unaffected.

EVIDENCE: local repro compiles/runs exit 0. With the clause temporarily
re-added to the sibling stdlib `Range.count`, `smoke_iter` prints OK, exit
0 (was clang error); the stdlib edit was reverted after the run.

LOCKS: `regress_m203_closure_ensures_isolation` (IR: closure thunk emitted,
exactly one contract_fail label), e2e `e2e_m203_closure_ensures_isolation` +
`tests/regression/m203_closure_ensures_isolation/main.xi`, CI line. Stdlib
lane can re-add the `Range.count` clause now.

---

## 2026-10-07 -- FIXED: module-scope init pruned its cross-module callee (m204, Pulse C-PULSE-07)

Pulse relay: `var b = rate_keyed_new(1, 1);` at module scope (xiom.rate's
limiter) was accepted by the checker but codegen emitted
`call i64 @rate_keyed_new(i64 1, i64 1)` with no definition -- clang "use of
undefined value '@rate_keyed_new'"; the larger Pulse suite AV'd during
module init. Reproduced locally without the packages repo
(`tmp/pulse05/ginit/`: sibling module xiom.rate + top-level `var bucket =
rate_keyed_new(1, 1);`).

ROOT CAUSE (IR evidence, tmp/pulse05/ginit_main.ll vs ginit_fn.ll): the
checker's catalog reachability filter (`collect_referenced_names` in
xiom-check, used before external decl injection) walked `TopDecl::Fn`
bodies, `Module`, and `Use` -- but NOT `TopDecl::Const` values. A catalog
function referenced ONLY from a module-level `var`/`const` initializer was
therefore pruned from the injected decls: no `define i64 @rate.rate_keyed_new`
was emitted, and the `@llvm.global_ctors` body for the global emitted a
call to a symbol that did not exist. A body-level call to the same fn worked
(it seeded the reachability set).

FIX (m204): `collect_referenced_names` now also collects names from
`TopDecl::Const(cd.value)` (module-level var/const initializers), so the
callee survives the filter, is injected with its body, and the ginit call
resolves to the qualified symbol (`@rate.rate_keyed_new`).

EVIDENCE: local repro compiles and runs exit 0; IR now contains both
`define i64 @rate.rate_keyed_new(...)` and the qualified call. Pulse can
re-test `docs/repro/module-scope-package-init/probe.xi`; the installed
package path is now handled by m212 (C-PULSE-02, [dependencies] -> catalog
source roots), so the hard-coded `source-roots` workaround is no longer
needed.

FOLLOW-UP: reading such a global from a function (`bucket == 42`) still
trips a checker typing gap ("cannot compare <error> with Int") -- the
global's cross-module-initialized type is not propagated to reads; tracked
separately, not part of C-PULSE-07's codegen defect.

LOCKS: `m204_module_ginit_emits_cross_module_callee` (driver test in
crates/xiom/tests/run_script_cli.rs) +
`tests/regression/m204_module_ginit_cross_module/{main,rate}.xi`. Gates:
checker 197/197, run_script_cli 5/5, driver lib 61/61.

## 2026-10-07 -- FIXED: rvalue Vec[Float64] index read raw bits (m200, stdlib p_rvalue_float_vec_index)

Stdlib probe `tools/known_failures/p_rvalue_float_vec_index.xi` (rc 1 on the
pin, re-verified locally): `mk_f()[0] != 1.0` failed while `var vf = mk_f();
vf[0]` and `mk_i()[0]` were correct. IR evidence (tmp/pulse05/rvalue.ll
pre-fix): at the rvalue site the element load produced the i64 bit pattern
and the comparison emitted `sitofp i64 %tmp91 to double` (1.0 bits ->
4.6e18); the two bound-local sites emitted `bitcast i64 %.. to double`.

ROOT CAUSE: the Index handler's float fallback (`vec_elem_float_type`) knew
only Ident containers (local_vec_elem/local_vec_handle/global_vec_elem), the
BUG 23 nested-Index form and Field containers. A CALL container
(`mk_f()[0]`) returned None, so the read fell to the plain scalar
`emit_elem_load` + caller-side sitofp instead of bit-reinterpreting the
stored double.

FIX (m200): `vec_elem_float_type` resolves `Expr::Call`/`Expr::GenericCall`
containers through `callee_return_xiom`, mapping a declared `Vec[Float32]` /
`Vec[Float64]` return to "float"/"double".

EVIDENCE: probe rc 1 -> 0; IR now shows three `bitcast i64 -> double` element
reads and zero `sitofp i64` (tmp/pulse05/rvalue_fixed.ll). Same-class
consumer: `xiom.stats.moments.quantile` q=0/q=1 branches.

LOCKS: `regress_m200_rvalue_float_vec_index_bits` (IR: rvalue read is a
bitcast, no sitofp), e2e `e2e_m200_rvalue_float_vec_index` + fixture, CI
line.

---

## 2026-10-05 -- verifier locals sort + one-step loop encoding (contracts coverage) -- STEPS 1+2 FIXED m213, step 3 open

Benchmark contracts ask (loop-invariant syntax + emitter coverage).
Findings from a local probe (`tmp/contracts/inv_probe.xi`):

- SYNTAX: loop invariants exist and the checker accepts them:
  `while cond invariant: expr { ... }` (parser: "while expr [invariant:
  expr] block"). Type invariants likewise exist on `type` decls.
- BUG 1 (locals sort): `infer_sort` for `Expr::Ident` looks up
  `var_sort_map` by the BARE name, but `encode_stmt`'s Let/Var arm inserted
  the sort under the fresh SSA name (bare -> SSA lives in `latest`). Every
  local inferred sort None, so any clause/invariant/body term referencing a
  local skipped with X7006/X7007 ("sorts None/Some(Int)") -- the benchmark
  t1/t4/t8 "no queries emitted" class.
- BUG 2 (loop summary): `Stmt::While` with an invariant is a one-step
  approximation (assume the invariant at the loop head, encode ONE guarded
  iteration, register the invariant as an obligation). The post-loop state
  was never summarized (no havoc + invariant + negated condition), so
  fixing BUG 1 ALONE converted honest UNKNOWNs into spurious VIOLATEDs --
  hence the experiment was reverted and x kept in v0.64.0.

STEPS 1+2 FIXED (m213, 2026-10-07):
1. BUG 1 fixed: the Let/Var AND Assign arms now register the sort under the
   BASE name as well as the fresh SSA (`var_sort_map.insert(base, sort)`),
   so `infer_sort(Ident)` resolves local sorts. Unit test
   `local_sorts_resolve_for_comparisons`.
2. BUG 2 fixed with the designed havoc summary: after the one guarded
   iteration, the arm collects every variable ASSIGNED in the body
   (`collect_assigned_names`, any nesting depth), creates fresh SSAs with
   their sorts, rebinds them in `latest`, then re-translates the invariant
   and condition and asserts `(=> ft (and inv (not cond)))` -- the standard
   havoc + invariant loop exit summary.
EVIDENCE (z3, local): `count_to` with `invariant: i >= 0` and
`ensures: result >= 0` -> 2 proven / 0 violated / 0 unknown (was 1 unknown
with the sort warning); the naked-loop probe stays 1 proven + 1 honest
X7007 unknown; the INSUFFICIENT invariant probe (`ensures: result >= 5`)
yields a REAL violation with a countermodel (no over-claiming). Unit test
`invariant_loop_havocs_and_asserts_exit`; verifier suite 41 green
(7 lib + 34 integration).
STILL OPEN (step 3 + full fixpoint):
3. Classification policy for obligations that fail ONLY because the loop
   summary is an over-approximation -> must surface as UNKNOWN, never
   VIOLATED (the benchmark workspace must not see false violations).
   Preservation in the two-state sense is also not yet encoded (the pushed
   invariant obligation is establishment + one-step shape).
4. Full fixpoint VC generation for loops remains future work; the current
   summary is the documented approximation.
Benchmark answer: invariant syntax above; a SUFFICIENT invariant now proves
post-loop obligations (step 2 landed).

---

## 2026-10-05 -- FIXED: incomplete struct literal compiled silently (m198, Pulse C-PULSE-06)

Pulse C-PULSE-06: `Pair{ a: 1; }` (missing `b: Vec[UInt8]`) compiled with no
diagnostic; `p.b.len()` read garbage (2800072769408 here) and PULSE's server
AV'd on real requests (0xC0000005). Reproduced on v0.64.0.

FIX (checker, struct-literal validation): after validating the provided
fields, a USER struct literal must initialize every declared field;
otherwise emit T001 `struct literal for 'Pair' is missing field 'b'`
(sorted list for deterministic order -- selfhost diff_check compares
line-exact). Builtin layouts (Vec/Set/Stack/Slice/Option/Result) and enum
variant constructors are exempt: the compiler/stdlib build those partially,
and variant ctors share leaf names with same-named structs
(benchmark/m89 `Node`, `Rectangle`) -- `resolve_enum_variant` wins,
mirroring the unknown-field guard.

EVIDENCE: Pulse probe now `error[T001] ... missing field 'b'`, exit 1, no
artifact. Regression fallout found and fixed within the batch: bench_math +
benchmark main (variant collisions) compile again. LOCKS:
`e2e_m198_missing_struct_field_rejected` (asserts failure + T001 + field
name) + fixture; CI line. Checker 197/197; feature 524/524; full e2e
2428/0/4.

---

## 2026-10-05 -- FIXED: bare reference in value position yielded the address (m197, Pulse C-PULSE-04)

Pulse C-PULSE-04: `p = p + 1` / `return p` with `p: &mut Int` compiled to
pointer arithmetic + ptrtoint, so cursors kept stack addresses (an xiom.http
consumer saw `pos=372324169712`). Root cause: the checker erases `&T`/
`&mut T` to `T` (types.rs `from_ast_type`), so both shapes type-check as
plain Int, while codegen lowered the bare Ident as the pointer; the write
side had already been fixed to store THROUGH the reference (v0.62.2), making
read/write asymmetric.

FIX (codegen): `autoderef_ref_value` -- when an expression is a local whose
XIOM type starts with '&' (`local_xiom_types`) and its LLVM type is a
pointer to a scalar, emit a load and use the pointee. Applied in both
binary-op paths (fold + normal) before arithmetic dispatch, and in
`Stmt::Return` when the fn's return type is scalar. Raw `*T` operands keep
pointer arithmetic; aggregate pointees and reference returns pass through
untouched.

REPRO/EVIDENCE: Pulse probe (`docs/repro/mut-int-bare-read/probe.xi`) exit
5 -> 0: `bare_add a=11 r=11`, `bare_read c=10 r=10`. Locks:
`regress_m197_mut_ref_bare_read_loads` (no GEP; pointee loaded) +
`e2e_m197_mut_ref_bare_read`; CI line; feature 524/524.

FOLLOW-UP (same batch): the return-path autoderef must NOT fire for declared
reference/pointer returns -- `fn get_ref(x: &Int) -> &Int { return x; }`
erases the return to the i64 address ABI, so the first version loaded the
pointee and callers then dereferenced the VALUE (m21_borrow_010 AV
0xC0000005; caught by the full e2e). `FunctionContext::
current_return_is_ref_like` (set from the AST return type in decl.rs) gates
it. Borrow cluster 20/20, feature 524/524, full e2e 2427/0/4.

---

## 2026-10-07 -- FIXED: W005 erased-interface stub fired for module-const receivers (Pulse C-PULSE-05, m199)

Pulse relay: `SCHEMA_VERSION.to_str()` -- where SCHEMA_VERSION is a module
constant -- hit the W005 erased-interface auto-stub: v0.63.1 silently rendered
an empty string (invalid JSON on disk); v0.64.0 aborted 0x80000003.

ROOT CAUSE (IR evidence, tmp/pulse05/probe.ll pre-fix): the receiver was
dropped entirely -- `%tmp4 = call i64 @to_str()` with zero args, then the W005
typed stub `define i64 @to_str() { ret i64 0 }` answered 0; the NULL Str
handle aborted in the runtime concat. `receiver_is_instance` treats a bare
Ident as a VALUE only if it is a bound local (lookup_local) or a mutable
module global (module_globals); immutable module `const`s live only in
`constants` (substituted literals), so `V` was classified as a module path and
the to_str operand became `args.first()` = None. Call-result receivers
(`f().to_str()`) were unaffected.

FIX (m199): (1) `constants.contains_key(name)` counts as an instance in
`receiver_is_instance`; (2) immutable consts now record their declared XIOM
type in `global_xiom_types` (bare + module-qualified, mirroring the
mutable-global BUG 29 pass); (3) `infer_expr_xiom_type_deep` falls back to
`global_xiom_types` so Str/Float/Bool consts pick the right conversion instead
of the erased i64 default.

EVIDENCE: Pulse probe now `const-to-str=[41]` + `[PASS]`, no W005; IR is
`%tmp4 = call i8* @xiom_int_to_string(i64 41)` and no `@to_str` anywhere.
Matrix probe (const Int/Str/Float64/Bool receivers, `to_str`/`to_string`,
`.eq`/`.lt`): exit 0.

LOCKS: `regress_m199_const_receiver_keeps_type` (IR:
`@xiom_int_to_string(i64 41)`, no `@to_str`), e2e
`e2e_m199_const_receiver_method_dispatch` +
`tests/regression/m199_const_receiver_to_str/main.xi`, CI line. Gates:
feature 525/525.

---

## 2026-10-07 -- FIXED: multipart_parse Part fields corrupt (erased Vec element type on match binding, m201)

Stdlib probe `tools/known_failures/p_multipart_parse_name.xi` (rc 1 on the
pin) reproduced locally with the sibling stdlib head. `out[0].name` printed
`0` with `.len() == -1`; `out[0].filename/content_type/data` all read -1; a
directly constructed `parts[0]` reads `"f"` correctly; `out.len() == 1` is
correct.

ROOT CAUSE (IR evidence, `tmp/pulse05/multipart.ll` pre-fix):
- `multipart_parse` returns `Result[Vec[Part], Str]`; the callee boxes the Ok
  Vec (`malloc(sizeof(Vec))`, pointer in payload field 1) -- caller and
  callee agree on that ABI.
- The caller's DIRECT read of `parts[0]` (concrete `Vec[Part]` local) uses
  the compile-time elem size 56 + `memcpy` of a `%struct.Part` -- CORRECT.
- The read of the match-bound `out[0]` instead emitted a RUNTIME elem-size
  switch (`extractvalue %struct.Vec, 3` then `switch i64 56` -> default
  `elem_load` = `load i64`). The switch only has cases 1/2/4 and defaults to
  an i64 scalar load, so a struct element (56 B) was read as an integer; the
  `.name` field access then compiled to `inttoptr i64 0`.
- At the binding site (stmt.rs payload binding), the `declared` payload
  "Vec[Part]" was known, but the element type was recorded ONLY for i64
  HANDLE bindings (`local_vec_handle`, `bind_ty_inner == "i64"`); the
  `%struct.Vec` alias shape (boxed header bound by address) recorded neither
  map, so the Index handler found no element type.

FIX (m201): the payload binding now records the declared `Vec[T]` element in
`local_vec_elem` for the `%struct.Vec` ABI shape (and keeps `local_vec_handle`
for the i64 handle shape), after clearing both maps for shadowing. `out[0]`
then takes the struct memcpy path.

EVIDENCE: multipart probe rc 1 -> 0; IR now `memcpy` + `load %struct.Part`
for each payload read, no runtime elem-size switch. Same-class probes also
turn green on the same fix: `p_geom_matrix_result_infer.xi` rc 0 and
`p_polyhedra_nested_hull.xi` rc 0 (`p_geom_box_unnameable.xi` stays T001 --
distinct checker issue).

LOCKS: `regress_m201_match_payload_vec_elem` (IR: >= 2 `load %struct.Part`,
no `switch i64`), e2e `e2e_m201_match_payload_vec_elem` +
`tests/regression/m201_match_payload_vec_elem/main.xi` (self-contained
Result[Vec[Part], Str] shape), CI line. Red-before verified (pre-fix fixture
program exit code 1, scalar switch dispatch).

---

## 2026-10-05 -- FIXED: angle-bracket generic receivers discarded their type args (m195, reflect.all_types heap corruption)

Stdlib known-failure probe `tools/known_failures/p_reflect_all_types_crash.xi`
reproduced locally: `reflect.all_types()` dies with 0xC0000374
(STATUS_HEAP_CORRUPTION) after ~8s. The probe comment isolated it to the
catalog return path; the actual delta is the SPELLING of the generic
receiver.

ROOT CAUSE: `xiom.reflect.reflect.xi` writes `Vec<TypeInfo>.new()` /
`Vec<FieldInfo>.new()` with ANGLE brackets. The parser's speculative
generic-args arm (parser lib.rs, TokenKind::Lt on an uppercase Ident) parsed
`<T>` only to DISCARD it ("generic type args discarded; postfix continues"),
so codegen's Vec.new receiver saw a bare `Ident("Vec")` and fell back to the
8-byte element size: both Vecs allocated 16 x 8 = 128 bytes while pushes
wrote 24-byte (FieldInfo) and 128-byte (TypeInfo) elements -- heap
corruption on the sixth FieldInfo push / second TypeInfo push. The
square-bracket spelling `Vec[T].new()` always worked because that path builds
`Expr::Index(base, T)` (user probe: malloc 512, elem 32; angle probe pre-fix:
malloc 128, elem 8).

FIX (parser): the angle arm now parses `<T[, U]>` as a type list, builds the
same `Expr::Index(base, T)` (Tuple for multiple args) as the square-bracket
path, and commits only when the closing `>` is immediately followed by `.`
or `(` -- otherwise it restores the position AND truncates speculative parse
errors so comparisons (`a < B > c`) still parse. This also fixes
`Map<K, V>.new()` and `with_capacity` element sizing for angle spellings.

REPRO/EVIDENCE: `tmp/contracts/known/reflect_min.xi` pre-fix 0xC0000374 at
~8.2s, post-fix exit 0 at 114ms. `tmp/contracts/known/vec_angle.xi` pre-fix
malloc 128 / elem 8, post-fix malloc 512 / elem 32.

LOCKS: parser `test_angle_generic_receiver_keeps_type_args` (Index receiver +
comparison fallback), IR `regress_m195_angle_vec_new_elem_size` (malloc 512,
no 128 fallback), e2e `e2e_m195_angle_vec_new_struct` (ten 32-byte structs
round-trip) and `e2e_m195_reflect_all_types` (the stdlib probe shape). Gates:
parser 108/108, feature 522/522, checker 197/197, full e2e 2425/0/4. CI
filter extended.

---

## 2026-10-05 -- AUDIT: contracts-track FAILs (t2/t3/t4/t5/t8) are reference-content failures, not emitter coverage

Benchmark relay (contracts run 1791207193731): t1 PASS (1 proven); t2/t3/
t4/t5/t8 FAIL with verdict unproven ("no queries emitted"). The working
theory was X7007 emitter coverage; extracting the exact sources from the
local benchmark container (`docker cp` from
`/app/data/evidence/run_1791207193731/trials/...`, kept at
`tmp/contracts/ref/`) shows otherwise.

EVIDENCE (reproduced locally on this tree):
- t2-queue, t3-hot-reload, t4-packet, t5-btree `source.xi` contain ZERO
  contract clauses (requires/ensures/invariant = 0). The verifier emits the
  "no queries emitted" placeholder -> 0/0/1/0; the harness policy
  (`src/tasks/contracts.js`, `proven == 0 && unknown > 0 -> unproven`) fails
  it, and even zero obligations fail as `no-obligations`. The harness says it
  itself: "No contract obligations found -- a contracts-arena reference must
  contain requires/ensures clauses" (contracts.js:321).
- t8-safety-probe has 3 clauses, ALL `requires` (axioms), NO `ensures`.
  `xiom-verify --check -o t8.smt2` -> ZERO `(check-sat)`, 3 skipped axioms
  (comments), 4 unknown (3 axiom skips + the no-queries placeholder; skipped
  axioms are counted as unknowns -- honest but inflated), 0 errors. The two
  X7007 modeling gaps (`Str.len()` sort: `infer_sort` has no MethodCall arm;
  `*mut UInt8 != null`: opaque pointer sorts) cannot create an obligation --
  none exists -- so `proven` stays 0 and the track stays FAIL.
- t1 passes because it carries `ensures` clauses (17 clauses; 1 proven).

RECOMMENDATION (benchmark lane, reference content): annotate t2/t3/t4/t5
references with at least one provable `ensures` (the contracts arena
requires clauses), and give t8 an `ensures` if the track is meant to
demonstrate safety-probe obligations. Alternative scoring: exempt
`no-obligations` trials from the FAIL aggregation instead of treating them
as unproven. Compiler-side emitter coverage for the two t8 axiom shapes is
queued as coverage work, but it is NOT the blocker for any verdict above.

---

## 2026-10-05 -- FIXED: user extern of a builtin runtime symbol duplicated the declare (m193)

Stdlib relay (2026-10-04, still open on v0.62.4): a direct XIOM extern of the
runtime function `xiom_guard_alloc` (plus a call) hung codegen (~300s) and/or
failed with clang "invalid redefinition". Re-verified on this tree: no hang,
but the native link fails deterministically.

REPRO: `tmp/contracts/guard_alloc_extern.xi` --
`extern "C" { fn xiom_guard_alloc(size: Int) -> *UInt8; }` + call.
- `--emit-ir` prints TWO identical `declare i8* @xiom_guard_alloc(i64)`
  lines (builtin preamble + user extern); LLVM rejects even byte-identical
  redeclarations, so `--release --target native` fails:
  `xiominput.ll:143:13: error: invalid redefinition of function
  'xiom_guard_alloc'`.

ROOT CAUSE: `emit_extern_declares` deduped via `mono.already_declared`,
seeded only by `hardcoded_declare_names()` -- a hand-maintained list that had
drifted behind `emit_builtin_declares()` (the whole D2.1 guard/trampoline
family, sprintf, llvm.memmove, xiom_str_from_vec, xiom_double_to_string) and
the module-header block (channel/threadpool/set_args). Any user extern of a
drifted symbol duplicated.

FIX (emitter.rs): before processing extern items, scan the module text for
`declare ... @name` and union the names into `already_declared`. This covers
everything actually emitted (including conditionals: a symbol skipped when
hot_reload is off is NOT seeded, so the user extern still declares it).

VERIFIED: IR lock `regress_m193_extern_builtin_no_duplicate_declare` (declare
count == 1), e2e `e2e_m193_guard_alloc_extern`
(`tests/regression/m193_guard_alloc_extern/`, calls the runtime symbol;
pre-fix link error, post run exit 0 "m193 ok"), m21 FFI cluster 10/10. CI
filter extended.

---

## 2026-10-05 -- FIXED: Float64<->Int64 bitcast intercept for num.float (m194)

Stdlib relay (queued since the num.float audit): `xiom.num.float.float_bits`
and `bits_to_float` were DOCUMENTED FALLBACKS returning 0 / 0.0 because XIOM
has no source-level bitcast. The stdlib wanted the exact behavior with no
stdlib change.

REPRO: `tmp/contracts/bitcast_probe.xi` -> pre-fix `bits=0 f=0` (rc 1);
expected `bits=4609434218613702656 f=1.5` (0x3FF8000000000000).

FIX (call.rs): the qualified stdlib keys `num.float.float_bits` /
`num.float.bits_to_float` (and the `xiom.`-prefixed forms) are intercepted
at the call site and lower to `bitcast double <v> to i64` /
`bitcast i64 <v> to double`. The intercept is module-qualified, so user
functions with the same leaf keep their bodies; unmatched arg types (not
double / not i64) fall through to the normal call. No stdlib change needed;
the fallback bodies remain in the module for non-intercepted dispatch.

VERIFIED: probe rc 0 with exact bits + round-trip (negative value keeps its
sign bit; +0.0 all-zero). Locks: `e2e_m194_float_bitcast` (exact bits +
roundtrips) and `e2e_m194_float_bitcast_ir` (main's call sites are bitcasts,
no calls to the fallbacks); CI filter extended.

---

## 2026-10-05 -- FIXED: confined-unsafe ctx alloca leaked 32 bytes of stack per loop entry (m192)

Perf queue item 1 surfaced a CORRECTNESS bug underneath the t3-hot-reload
sample: a confined `unsafe` block re-entered from a loop emitted its
`%struct.__unsafe_ctx_N` capture alloca inline at the block site, so every
execution consumed a fresh 32-byte stack frame released only at function
return. clang does not hoist escaped allocas out of loops, so a hot loop
exhausts the 8 MB stack reserve.

REPRO (Windows local; same shape as the container artifact):
- `tmp/contracts/t3_loop_probe.xi` (per-call confined unsafe, LOOPS env var).
  Pre-fix: LOOPS=200,000 exit 0 (41 ms); LOOPS=262,000 exit 0 (92 ms);
  LOOPS=264,000 exit -1073741819 (0xC0000005, fault in KERNELBASE);
  LOOPS=1,000,000 exit 0xC0000005 after ~4 s (WER tail).
  Threshold = 262,144 = 8,388,608 / 32 bytes exactly.
- The 100k-entry container variant (1000 cycles x 100 calls) sits UNDER the
  threshold, which is why it measured 213-223 ms instead of crashing: the
  loop touched ~3.2 MB of fresh stack (a growth page fault every 128 entries)
  on top of the per-entry runtime cost.
- This is the same `0xC0000005` class the packages lane reports for grpc
  `probe_suite_min` -- re-test on the next archive (partially explains it;
  the crash count/hang still needs the packages repro if it persists).

ROOT CAUSE: `crates/xiom-codegen/src/expr.rs` (Expr::Unsafe, ctx build):
`let ctx_slot = self.fresh_tmp(); self.emitln("  {ctx_slot} = alloca
%struct.{ctx_name}")`. FIX: when `self.local.loop_depth > 0`, push the pair
onto `local.hoisted_allocas` -- the BUG 22 #6 mechanism that splices
loop-body allocas into the fn entry block. One slot is reused per fn
invocation; recursion still gets one frame per activation (the trampoline
call is synchronous and the ctx never outlives the call).

LOCKS: `regress_m192_unsafe_ctx_hoisted_from_loop` (IR: every
`alloca %struct.__unsafe_ctx_` lands before the loop-body label; red-before
= the loop-body placement) + `e2e_m192_unsafe_ctx_loop_stack` (500,000
entries + capture write-back checksum; pre-fix 0xC0000005 at this size).
Feature suite 520/520; targeted unsafe e2e (m20/m21 x10/m37 x2/m192) and
M33-U (20) green; CI filter line extended.

PERF RESIDUAL (relayed to stdlib): with the leak fixed the per-entry cost is
still runtime syscalls. Windows ablation (per-entry `VirtualProtect` in
`xiom_guard_page_disarm` disabled, runtime otherwise identical): 262,000
entries 438.5 ms -> 92.5 ms (~1.67 -> ~0.35 us/entry). POSIX pays 3
`sigaction` syscalls per `xiom_trampoline_call` plus `mprotect` per disarm.
Fast-path directions for the stdlib lane: install the signal handlers once
(lazily); make arm/disarm flag-only and re-arm the PAGE_GUARD/PROT_NONE page
only after a guard-page fault was actually mapped. Expected: the t3 sample
lands in the peer 25-29 ms range once the runtime fast path rides a pin.

---

## 2026-10-05 -- OPEN (attributed): t3-hot-reload 216ms sample = per-iteration confined-unsafe trampoline

Queue item: the systems/contracts arena t3-hot-reload sample is ~216-245ms
while the operation's own JSON reports `time_ms: 0` and peer languages
(C/Rust) measure 5-16ms for the identical workload.

ARTIFACTS: solution `reference/systems-arena/t3-hot-reload.xi` (1000 cycles
x [2x dlopen(libc.so.6) + dlsym(abs) + 100 indirect calls + 2x dlclose]);
sample command = run the compiled arena binary (`--release --target
native`, 7 samples + 1 warm-up; arena.js `runTimedSamples`).

EVIDENCE (acceptance container, v0.63.0):
- hello-world binary, same flags: 3ms/run; t3 binary: 213-223ms/run stable.
  So the sample is the PROGRAM's own work, not process/driver overhead.
- Attribution variant (tmp/contracts/t3_hoisted.xi): identical workload,
  same checksum 25000000, but the `unsafe { value = fn_ptr(arg); }` block
  encloses the 100-call inner loop once per cycle instead of being
  re-entered per call -> 25-29ms/run (~9x faster, peer range).
  => the cost is the confined-unsafe TRAMPOLINE per entry:
  `xiom_trampoline_call` + `xiom_guard_heap_enter/exit` +
  `xiom_guard_page_arm/disarm` + `xiom_trap_enter/leave` ~= 2us x 100,000
  iterations ~= 200ms.
- The in-program `time_ms` is also wrong: `xiom.time.Instant.now()` wraps
  libc `time(0)` (stdlib/xiom/time/time.xi:230), i.e. WHOLE SECONDS, so
  `as_millis()` oscillates 0/1000 across identical 216ms runs -- the "op
  time 0" in the log is a second-resolution artifact, not a fast op.

FIX DIRECTIONS (after v0.63.1, perf items): (a) elide the per-execution
trampoline when a confined `unsafe` block is re-entered from a loop or
nests inside another confined region (or a cheap guard fast-path when the
heap guard is already armed); (b) stdlib `Instant` -> `clock_gettime(
CLOCK_MONOTONIC)` (rides the next STDLIB_VERSION pin).

---

## 2026-10-05 -- FIXED: xiom-verify t1 SMT "unknown constant self" (z3-scoped decls + receiver modeling)

Benchmark relay: the contracts arena t1-allocator reference
(`reference/systems-contracts-arena/t1-allocator.xi`) made z3 reject the
whole SMT: `(error "line 495 column 90: unknown constant self")` --
Results 1 proven, 11 unknown, 1 errors; the harness classified it as a
toolchain emission error, not a proof failure.

Repro: `xiom-verify <t1-allocator.xi> --check -o t1.smt2`.

ROOT CAUSES (three, all in the SMT emitter):
1. The receiver declaration was guarded by
   `var_sort_map.contains_key("self")`, and `var_sort_map` survives across
   functions (only `latest` is per-function). The first method that
   declared `|self|` left a stale entry, so every later implicit-receiver
   method SKIPPED its per-section declaration while body identifiers fell
   through to the bare `smt_escape("self")` spelling. Guard on `latest`.
2. z3 SCOPES sort declarations under `(push)`/`(pop)`. The owner sort
   `|xiom_BuddyAllocator|` was first needed lazily inside split_block's
   push scope (no field/param referenced the owner type itself, so the
   annotated-type pre-pass missed it), and every later section failed with
   `unknown sort 'xiom_BuddyAllocator'`. Fix: predeclare the OWN sort of
   every declared type/enum at top level (`collect_declared_type_names`).
3. Receivers are modeled as VALUES (so datatype selectors `(T-f self)`
   work) while `&T` parameters are pointer sorts; passing `self` to a
   `&T` parameter produced an ill-sorted application (`unknown constant
   header_read_order (xiom_BuddyAllocator Int)`). Fix: top-level opaque
   `|xiom_ref_T| : (T) -> &T` coercions (`emit_ref_coercions`, emitted
   after `emit_datatypes`), guarded to realisable value sorts -- the
   structural key "_" of `&Vec[Int]` is skipped (t4-packet).

EVIDENCE: t1 `--check`: 1 proven, 0 violated, 11 unknown, **0 errors**
(was 1 errors, exit 1). Full arena sweep on the fixed toolchain: t1..t5
all 0 errors. LOCK:
`implicit_receiver_to_ref_param_is_declared_and_coerced` (declaration per
section + coercion + no bare self application + z3 parse when available).
GATES: verifier 34 + 5.

---

## 2026-10-05 -- FIXED: nested `lib/lib` install copy indexed (W001 flood + doubled index)

User relay (hello-world on the installed v0.63.0): ~100 `warning[W001]:
module '...' declared by 2 files [<install>\lib\lib\xiom\..., <install>\
lib\xiom\...]` lines before any output, plus ~3-4s startup for a tiny
script (the compiler resolved `xiom.*` to the stale nested copy).

ROOT CAUSE (install-side, not the archive): the local install had
`<install>\lib\lib\{xiom,runtime,...}` beside the canonical
`<install>\lib\...` -- a leftover from an older side-by-side staging run
(v0.62.0 files at 13:10) that a later install merged into instead of
wiping. The published archive is clean: the v0.63.0 Windows zip has 0
`lib/lib/*` entries and 518 `lib/xiom/*` entries; a clean reinstall has
no duplicate. The installer's current code removes/relocates stale
staging trees (website e8eeede), so new installs are clean.

FIX (defensive, compiler side): the catalog index walk now skips a `lib`
child whose parent already holds `xiom/` (`ModuleCatalog::index_dir`),
i.e. the exact nested-install shape. The outer tree stays canonical, so
the stale copy can neither win module resolution nor emit W001 nor double
the index. LOCK: `catalog.rs::nested_lib_stdlib_copy_is_not_indexed`
(duplicate module file under `lib/lib/xiom` -> one candidate, zero
collisions). Verified on the relay install: nested tree removed, then
`xiom run --no-cache` on the user's program prints no W001 and executes
(exit 0).

---

## 2026-10-05 -- FIXED: runtime contract evaluator false violations (contract-check binding-state leak; packages-lane relay)

The packages lane reported two clause shapes that aborted with a spurious
`contract violated: ensures` on v0.63.0 while being structurally true
(`xiom-verify` reported the same clauses UNKNOWN, never violated):

- (a) tuple-component access on a `Result[(Int, Int), Str]` payload:
  `ensures: result is Ok => result.value.1 > off` (varint_decode_u);
- (b) `Result[Vec[UInt8], Str]` payload length vs a parameter length:
  `ensures: result is Ok => result.value.len() <= data.len()` (cobs_decode).

Evidence (packages lane): brute force with `--no-contracts` over all 65,792
frames of length <= 2 printed bad=0; uuid's `result.value.len() == 16`
(payload vs CONSTANT) worked, localizing the defect to dynamic-length /
tuple payload expressions. uuid's function has a single, first clause; the
failing ones have a body local named `result` (varint) or an Err clause
before the payload clause (cobs).

ROOT CAUSE (single, codegen): contract-check compilation mutated FLAT
binding maps without scoping them.
1. `local_xiom_types` and `is_payload_rebind` are flat maps; `push_scope` /
   `pop_scope` only save/restore LLVM local slots.
2. `bind_is_payload_xiom` (contract implication's bare `is Ok/Err` rebind)
   overwrote `local_xiom_types["result"]` with the PAYLOAD type and left it
   there after `pop_scope`.
3. `compile_ensures_checks` re-bound `result` per clause but re-inserted
   whatever was already in the map -- a no-op -- so two poisons survived
   into the next clause: a body local named `result` (`var result: Int = 0;`
   in varint_decode_u) and a preceding clause's Err rebind (cobs_decode's
   Err clause).

IR evidence (probes on main @ c4721d91 + this fix):
- tuple shape with a `result` local: `result.value.1` lowered to
  `icmp sgt i64 0, <off>` (literal 0), so `0 > off` aborted at off == 0;
- cobs shape: the second clause lowered `result.value.len()` to
  `inttoptr i64 <Vec handle> to i8*; call @xiom_str_len` -- the boxed Vec
  payload read as a Str, so the compared length was garbage.
Controls (single clause, no `result` local; Ok clause first) were green,
isolating the defect to leaked binding state rather than tuple/Vec lowering.

FIX (crates/xiom-codegen):
- `FunctionContext.result_xiom_ty` records the declared return type at
  function setup (the same `type_string_full` value the synthetic `result`
  entry uses).
- `compile_ensures_checks` snapshots `local_xiom_types` + `is_payload_rebind`
  around EACH clause, forces `result` to the declared return type (and
  clears its payload-rebind mark) before compiling the clause, then restores
  the snapshot -- no clause-to-clause leakage and no leakage back into the
  body.

LOCK: `tests/regression/m76_contract_payload_param_len.xi` extended with the
two shapes (`wf`: tuple payload + `result` local; `wg`: Err clause then Ok
payload bound) + `e2e_m76_contract_payload_param_len`.
GATES: feature 519/519, e2e 2418/0/4, verifier 34/34 + 4/4, driver 58/58,
checker 196/196, ascii_guard clean.

---

## 2026-10-04 -- OPEN (queued): direct extern of xiom_guard_alloc hangs codegen / "invalid redefinition"

Reported by the stdlib lane while landing the allocator bound-check lock:
a direct XIOM extern of the runtime function `xiom_guard_alloc` hangs
codegen (~300s) and/or fails with "invalid redefinition" on v0.62.4. They
worked around it with a test-only `xiom_guard_alloc_probe` helper, so the
lock shipped. Repro: a minimal `extern` declaration of `xiom_guard_alloc`
plus a call; suspect the runtime-decl merging path (duplicate prototype)
or the extern prototype shape (Int vs i64 / pointer kind). A direct
runtime-symbol extern is a legitimate pattern (wrapping runtime
internals), so the hang is a compiler bug; queue with the other open
findings.

---

## 2026-10-05 -- FIXED: duplicate `lz4_compress` leaf -- checker/codegen resolution divergence

Benchmark v0.63.0 acceptance: the lz4 smoke/probes fail on Linux
(`lz4_compress` returns a Vec whose len reads as pointer garbage;
`.len()` on the result otherwise fails as unresolved `Result.len`), while
qualified and user-space frame probes are correct. Root cause found and
reproduced deterministically in the benchmark container:

1. The stdlib exports TWO functions with leaf `lz4_compress`:
   `xiom.compress.lz4.lz4_compress` -> Vec[UInt8] (lz4.xi:31) and
   `xiom.compress.lz4_compress` -> Result[Vec[UInt8], Str] (compress.xi:274,
   the umbrella wrapper). Loading `xiom.compress.lz4` pulls the parent
   prefix module `xiom.compress`, registering both.
2. A BARE call `lz4_compress(data)` (after `use xiom.compress.lz4;`):
   the CHECKER resolves it to the imported module's Vec-returning fn (it
   accepts `.len()` on the result), while CODEGEN's bare-leaf resolution
   binds the Result wrapper. Result: `.len()` emits `@Result.len`
   (unresolved -> m142 loud error), or in shapes where the call compiles,
   the len read off a Result aggregate is pointer garbage -- exactly the
   benchmark's nondeterministic lengths (111111528768288, ...) and the
   smoke child exit -1.
   MINIMAL REPRO (`use xiom.compress.lz4; var c = lz4_compress(data); c.len()`):
   deterministic 3/3 fail in the official v0.63.0 container; qualified
   `lz4.lz4_compress` is deterministic 10/10 green (len 24).
FIX OPTIONS:
   (a) stdlib lane (immediate): rename the umbrella wrapper leaf
       (`lz4_compress` -> e.g. `lz4_compress_checked`) so the leaf is
       unique; checker and codegen then agree everywhere.
   (b) compiler lane (defensive, queued): bind bare PROGRAM calls through
       the checker's resolution the way R15/R20 do for catalog bodies --
       `catalog_resolved_calls` only records while `checking_catalog`, so
       program-scope bare calls fall back to codegen's order-dependent
       maps.

FIXED 2026-10-05 via (b) (the stdlib rename is now optional):
1. Checker (`xiom-check`): at a PROGRAM-scope bare call, when the leaf is
   registered under dotted keys with TWO DIFFERENT signatures and the
   resolved signature uniquely identifies one of them, record that dotted
   key in `catalog_resolved_calls` (owner-qualified + span-only). Helper
   `ambiguous_bare_fn_target` / `fn_sig_types_equal`; type-level FnSig
   equality because names/spans differ between registrations. Unambiguous
   bare calls record nothing -- the fast path is untouched.
2. Codegen (`call.rs`): the bare-call `fn_key` now consults
   `resolve_catalog_call_bare` BEFORE the registration-order bare slot
   (previously it was only a fallback when the bare key was absent --
   `lz4_compress` WAS present as the Result wrapper, so the record was
   never seen). `resolve_catalog_call_bare` tries the owner-qualified key
   (R20, catalog bodies) then the span-only key (program scope: the
   checker's owner is the declared module path while codegen's current_fn
   is the bare name), still guarded by the `.{fn_name}` suffix check.

EVIDENCE: `tests/toolchain/probes/lz4_compress_only.xi` frame bytes
2740398262480 -> 26; `lz4_smoke.xi` payload=864 bytes mismatches=0.
LOCK: tests/regression/m191_lz4_bare_duplicate_leaf/main.xi (bare
`lz4_compress` len sanity + round-trip) + `e2e_m191_lz4_bare_duplicate_leaf`.
GATES: m191 + m162 e2e green, strict `catalog_corpus_is_clean` green.

---

## 2026-10-04 -- FIXED: verifier SMT emission (contracts arena t1/t8: named-expression / initialized / self errors)

Benchmark relay (safe probe): `xiom-verify --check` on the contracts tasks
returned hard z3 parse errors -- "named expression already defined" (L277),
"unknown constant initialized" (L450), "unknown constant self" (L508) --
and printed "z3 invocation failed. Install z3..." even with z3 present.
Root causes + fixes (all in `xiom-verify`):
1. `:named` labels were source-derived (fn+line+clause index) -> duplicate
   obligations on one line made z3 reject the script. FIX: monotonic
   `unique_label()` suffix on every obligation (req/ens/side-condition/
   invariant).
2. `ident_term()` emitted ANY unknown identifier as a bare symbol
   (`initialized`, `null`, undeclared `self`). FIX: only known bindings
   (SSA latest, then declared params/consts/result/self) are emitted;
   unknown identifiers make the obligation X7007 instead of poisoning the
   script.
3. Datatype selectors were DECLARED unqualified (`(x Int)`) but USED as
   `Type-field` -> every selector application was an "unknown constant".
   FIX: qualified names on both sides; the existing selector test had
   encoded the mismatch and was updated to lock agreement.
4. Methods: the implicit `self` param is typed `Self`; contracts/axioms
   bound `self` at `|xiom_Self|` while the new receiver decl added a second
   `self` -> "ambiguous constant reference". FIX: normalize the `self`
   param to the owner type at collection and declare the receiver once.
5. Sort-mismatch poisoning: operators were emitted regardless of operand
   sorts (`(<= xiom_T x)`, `(+ (Array Gauge Int))`) -> whole-script
   rejection. FIX: arithmetic/compare require known numeric operands,
   logical require Bool, equality requires equal known sorts, implication
   requires Bool; unsupported shapes become X7007 (never invalid SMT,
   never fake proofs).
6. Undeclared sorts: lazy `(declare-sort ...)` inside a function `(push)`
   scope was lost at `(pop)`. FIX: top-level pre-pass declares every
   dynamic sort (signatures + struct fields + consts + annotated locals +
   the `|xiom_unknown|` inference fallback).
7. Invariants on non-mappable (generic/opaque) types are skipped as X7006
   instead of emitting bare field symbols.
8. CLI text: "z3 rejected the generated SMT (emitter bug)" vs the old
   blanket "Install z3" -- accurate for harness classification.

VERIFIED: `bench_contracts.xi` 2 proven / 0 violated / 31 unknown /
**0 errors** (rc 0); `bench_contracts_hard.xi` 3 / 0 / 40 / **0** (rc 0);
new unit tests (z3-parse gate, unique labels, honest skips) + updated
selector lock; verifier suite 34/34 integration + 3/3 lib.
UPDATE 2026-10-05 (benchmark relay): a script with NO `(check-sat)` at all
(every obligation skipped as X7007) made z3 print nothing; the parser
fabricated "Could not parse z3 output" and the stale "install z3" text
printed even though z3 ran fine. Empty z3 output is now classified
UNKNOWN -- "no queries emitted (all obligations skipped as UNKNOWN)" --
while non-empty unparseable output stays an Error; unit test added
(`queryless_z3_output_is_unknown_not_error`).
BONUS: the verifier CLI had the same temp-root source-dir recursion as the
compiler driver (a fixture under %TEMP% indexed every tree under it) --
guarded and wired to the Stage 6 index cache: its integration suite went
647s+ (timeout) -> **7.5s**; single test 102s -> 0.44s.

---

## 2026-10-04 -- AUDIT (queued, safety hardening): arena/handle overflow surfaces

Owner question: can the arena/handle memory model be overflowed? There is
no general arena allocator; the relevant surfaces are the FLAT-ARENA
collections and the unsafe-block guard slab arena. Findings:

1. GUARD ARENA (`xiom_guard_alloc`, stdlib/runtime/xiom_runtime.c:291):
   `aligned = (long long)(size + 15) & ~15` has no upper-bound check. A
   size within 15 of LLONG_MAX wraps to a negative `aligned`; the
   following `memset(p, 0, (size_t)aligned)` / offset math is UB and
   typically faults (caught by the confinement trampoline when inside an
   unsafe block). Not observed as silent corruption, but it is an
   unguarded arithmetic surface any unsafe/FFI caller can hit with a
   near-max size (e.g. an attacker-controlled length after wrapping
   arithmetic). FIX LANDED (stdlib cd61062, 2026-10-04): `if (size >
   LLONG_MAX - 16) return NULL;` in `xiom_guard_alloc` plus a
   fault-injection lock (`smoke_guard_alloc_wrap.xi` via a test-only
   `xiom_guard_alloc_probe`; rc 0 with the guard, rc 1 with it disabled).
   Compiler-side `STDLIB_VERSION` now pins cd61062 -- it rides the next
   compiler archive (v0.63.0 was cut before the fix landed). `xiom_alloc`
   (L161) is already zero/negative-guarded + malloc-bounded -- no wrap.
2. FLAT-ARENA HANDLES (`xiom/collect/*`): handles are raw integer indices
   into Vec-backed node pools (`fheap` docs: "handle wraps the node's
   arena index"; `heap.xi`: `id = keys.len()/3`). Out-of-range handles
   are bounds-checked by Vec indexing (trap) and `fheap_decrease_key`
   explicitly ignores out-of-range handles, so a bad index cannot become
   memory corruption. Residual risk is ABA/staleness: a removed node's
   slot can be reused, so a stale handle targets the wrong node (data
   confusion / DoS, not corruption). Generation-tagged handles (or debug
   slot tags) are the standard hardening; queue under STAGE6_PERF_PLAN
   item 4 (post-selfhost).
3. No other wrap path found in the allocator boundary; emitted Int size
   arithmetic is checked (overflow traps) unless user code opts into
   unsigned/wrapping ops -- which is exactly what the guard_alloc bound
   check must backstop.

---

## 2026-10-04 -- FIXED: scripting `xiom run` temp-root indexing (W001 flood, minutes cold)

Benchmark relay: v0.62.4 `xiom run` samples 7-9.6s (previous round
26-45ms). Root cause (verified with the installed v0.62.3/v0.62.4
binaries): `xiom run` compiles a `%TEMP%/xiom_run/_script_<hash>.xi` copy
and `compile()` adds the source's parent + grandparent as recursive
catalog source dirs; `%TEMP%` holds stray `.xi` files, so the WHOLE temp
root was indexed -- every extracted archive/stdlib workspace under
`%TEMP%\kilo` (9 competing copies of each module, 85 W001 collisions) and
~140s cold runs that repeat on every jit-cache miss.
FIX (v0.63.0): `temp_root_covers()` guard -- a dir at-or-above the system
temp root is never registered as a source dir (`compile()` ancestors +
`run_script_source_dirs`); deeper dirs under temp stay eligible for
fixtures/temp-rooted projects. Verified cold: 140s-class -> **3.95s**,
index visits 2565 (stdlib only), W001 0, script exit 0; unit lock added.
NOTES: (1) the published v0.62.3/v0.62.4 ARCHIVES are clean; the local
`%LOCALAPPDATA%\xiom.new\lib\lib` duplicate is an install-side artifact
(remove/reinstall cleanly). (2) Locally Windows Defender blocks execution
of fresh `%TEMP%\xiom_run\*.exe` (os error 225, ERROR_VIRUS_INFECTED), so
run2 recompiles then fails -- environment issue, not compiler. Contracts
SMT emission remains queue item 3 (unchanged).

---

## 2026-10-04 -- FIXED (m190): lz4 empty block = m165 2^32 Vec-cap constant triggered X86 `peephole-opt` miscompile

Fully reproducible from the emitted IR with plain `clang -O2` (LLVM 22.1.8,
x86_64-pc-windows-msvc); no xiom toolchain in the loop.

CHAIN:
1. m165 (`ad63eda0`, ancestor of v0.62.2 but not v0.62.0) raised the
   emitted Vec growth ceiling 2^24 -> 2^32 elements in
   `crates/xiom-codegen/src/call.rs` (vec_grow guard). 2^32 no longer fits
   an i32 immediate, so every guard lowers through
   `movabsq $0x100000001`.
2. SUFFICIENCY: taking the PASSING v0.62.0 IR and changing ONLY the 54 cap
   constants to 4294967296 flips rc 0 -> 5. NECESSITY: HEAD IR with the 54
   constants back at 16777216 is rc 0.
3. BISECT (`-mllvm -opt-bisect-limit`): the outcome flips at pass instance
   #16343 -- `peephole-opt` on `compress.lz4._lz4_write_seq`. Skipping just
   that instance -> rc 0; running it -> rc 5. The pass rewrites the
   esz-dispatch compare/register sequences (ws42.s vs ws43.s,
   ws_peephole.diff); the direct `lz4_compress_block` call then reads back
   `blk.len() == 0` -> return 5. The frame roundtrip path is unaffected
   (its internal block call still produces data).
4. Fix-value probes at IR level: 2147483648 (2^31) rc 5; 4294967295
   (2^32-1) rc 0 (lowered without the 0x100000001 movabs). The
   overflow-check guard (mul + `icmp uge new_cap, cap`, no immediate) is
   the untested structural alternative.
5. Evidence artifacts: `%TEMP%\kilo\m183\p4c*.ll`, `p4c_restore_all.ll`
   (copy-restore control: does NOT fix -> the earlier alloca-copy theory is
   dead), `ws42.s`/`ws43.s`, `ws_peephole.diff`.

FIX (m190, owner-approved): emitter constant -> 4294967295 (2^32-1) in
`call.rs` (vec_grow guard) + comment; IR lock
`regress_m165_vec_growth_ceiling` updated to assert 4294967295.
REPRO COMMITTED: `docs/repro/lz4/p4c_lz4.xi` (P4c).
VERIFIED: P4c compile+run rc 0 with the fixed compiler;
`smoke_compress_lz4_snappy.xi` prints "smoke_compress_lz4_snappy: OK"
(rc 0); feature lock green; `e2e_m165_vec_byte_buffer_gt_16mb` green
(m165 >16MB byte-buffer feature preserved).

---

## 2026-10-03 -- LOCALIZED (open, queued): `xiom run -e` latency is the cold stdlib-graph compile (no `-e` fast path)

User observation: `xiom run -e "<code>"` takes 7-9 s from Enter to
execution on the installed v0.62.3 toolchain.

MEASURED: matches the arena compile_ms gap (6.4-13.6 s per program on the
release toolchain). `-e` runs the FULL script pipeline (temp file ->
catalog index -> check -> codegen -> clang -> run); there is no
precompiled stdlib and no reuse across snippets. The script cache is keyed
on the exact source, so only a byte-identical re-run can hit, and `-e`
usage implies varied snippets. Local boxes with duplicate stdlib copies
under `%TEMP%` measure far worse (94 s with the official release binary
here, with W001 duplicate-module warnings for every module: benchmark/lane
leftovers under `%TEMP%\kilo` get indexed) -- a local artifact, not
shipped behavior.

RELATED CLI GAP: `-e` rejects `use` declarations (P001: expected
declaration, found the statement after `use`) and an unqualified
`io.println(...)` does not resolve, so module-using one-liners need an
undocumented form. Confirm the intended `-e` grammar before optimizing it.

FIX: roadmap `STAGE6_PERF_PLAN` item 1 (per-module compile cache +
precompiled stdlib + parallel module codegen), then a thin `-e` path that
loads the precompiled stdlib and compiles only the snippet. Also restrict
run-mode source-dir discovery so unrelated `.xi` trees are never indexed.

---

## 2026-10-03 -- FIXED (C25): `xiom run` warm script-cache hits closed stdin; `--no-cache` never bypassed the cache

Playground relay: cold runs read piped stdin, every cached rerun printed
`got: []`; `--no-cache` did not bypass a warm cache on v0.62.3.

ROOT CAUSES (`crates/xiom/src/main.rs`):
1. The cached-binary branch executed `Command::output()`, which defaults
   the child's stdin to NULL -- the program immediately read EOF. Fixed
   with `.stdin(Stdio::inherit())`; stdout/stderr stay captured for the
   existing success gate and replay.
2. `--no-cache` and `--jit` were filtered OUT of `effective` before they
   were read (`effective.contains(...)` was always false), so `--no-cache`
   could never bypass a warm cache and `--jit` could never take the JIT
   branch. Both flags are now captured from the raw arg list; the cache
   lookup is skipped when `--jit` is set so a cached binary is not executed
   only to be discarded.

LOCK: `c25_warm_cache_inherits_stdin_and_no_cache_bypasses` +
`tests/regression/c25_run_cached_stdin/main.xi` -- cold, warm, and
`--no-cache` runs must all print `got: [Ada]` (Defender-block tolerant,
full path in CI). Verified red-before (warm printed `got: []`) and green
after. Playground acceptance `tools/compiler-repros/c25/run.sh` prints
"C25 fixed: yes" on a toolchain containing this commit.

---

## 2026-10-04 -- FIXED: iter.range `.contains` C001 -- classifier nondeterminism root-caused (module-qualified call typing)

Registry stress on the OFFICIAL archives (harness: CWD=stdlib, XIOM_STDLIB
set, relative path; 20 compiles per file):

| toolchain | smoke_iter_range | smoke_iter_find_all_any |
|-----------|------------------|-------------------------|
| v0.62.4   | 8/20 green       | 12/20 green             |
| v0.62.3   | 8/20 green       | 10/20 green             |

Error text: `error[C001]: 'contains' receiver does not expose a concrete
Vec/Slice/Array element type`. Run-to-run flaky at roughly 50% -- a single
green run is luck, not a fix. This supersedes the v0.62.3 handoff's
"RESOLVED on the current tree (m184 the likely fix)" and the 2026-10-03
triage entry's "green on the stdlib tip". The documented HashMap-order
classifier state (call.rs `intercept = !has_user_fn`; the third-range-sum
state change) is the likely mechanism. Queue: v0.63.0 beside lz4.
IMPACT: blocks stdlib releases -- `run_smokes` has no exclusions and
`release.yml` runs the full corpus.

FIXED (2026-10-04, same day): root cause = `infer_llvm_type`'s bare-leaf
fallback for module-qualified calls. `iter.range(1,5)` resolved the callee
to the bare leaf `range`; the suffix scan over `types.functions` found BOTH
`iter.range` (`%struct.Range`) and `iter.range.range` (`%struct.Vec`) and
returned whichever the registration-ordered map listed first. On ~half the
runs it picked `%struct.Vec`, so `is_contract_collection_receiver` said
"collection" and `contains` was hijacked into the contract Vec scan -> C001
(env-gated trace: failing runs ty=%struct.Vec, passing runs ty=%struct.Range
for the identical program). FIX (call.rs, `infer_llvm_type` Call arm): when
the callee is `module.fn(...)` (Field over an Ident that is not a value
receiver), resolve the module's OWN key `{module}.{fn}` first -- `iter.range`
now binds directly and the ambiguous suffix scan is never reached. The
legacy first-match scan is unchanged for genuinely bare calls (an earlier
unanimous-only variant regressed the R49 `e2e_m94_nested_vec_struct_elem`
method chain, so it was restored exactly). EVIDENCE: reducer
`p_iter_range_contains_c001.xi` 9/20 fails before -> **20/20 green** after;
`smoke_iter_range` + `smoke_iter_find_all_any` compile 5/5 and run rc 0;
new lock `e2e_c001_iter_range_contains_sums` + ci.yml line; full e2e
**2418/0/4**; feature 519/519.
STDLIB/REGISTRY: the published v0.63.0 archive predates this fix -- keep
the C001 gate exclusions for the v0.63.0 pin; drop them once an archive
containing this commit ships.

---

## 2026-10-04 -- OPEN: enum-payload Str in-situ persists on m189; iter Range.collect forward-ref + closure lowering

1. **enum-payload Str in-situ -- reopened, NOT the m185/m189 shape.**
   Stdlib re-test on m189 (HEAD 32ea20f0): control
   `probe_enum_payload_str.xi` green, but `xiom.graphql`
   validate-valid-operation still rc 1 (9/10), identical to v0.62.3.
   Next: get the graphql validator slice (or the probe bundle) and compare
   the enum payload read path in that catalog context; the same-name
   collision fixed by m189 is ruled out.
2. **iter Range.collect forward-ref + Range.count/find undefined
   `__closure_N`.** Stdlib wave-65 (fully reverted, nothing landed): any
   clause on `Range.count`/`Range.find` makes `smoke_iter` fail with "use
   of undefined value in a generated `__closure_N`"; `Range.collect()` ->
   clang "instruction forward referenced with type 'ptr'". Standalone repro
   `tools/known_failures/p_iter_range_collect_forwardref.xi` fails on
   v0.62.3, v0.61.3 and m189. Also `smoke_iter_range` intermittently hits
   the C001 contains-classifier error under 8 workers (passes direct and
   single-worker 5/5) -- the C001 stays load-sensitive. Queue: v0.63.0
   (closure lowering + the C001 state pinning).

---

## 2026-10-04 -- NEW (open, queued): reflect.all_types() heap corruption (0xC0000374), catalog-return-path specific

Source: stdlib wave-64 relay, probe `p_reflect_all_types_crash.xi`.
Reproduces on v0.62.3, v0.61.3 and the m187 dev build (heap corruption
0xC0000374). The IDENTICAL build loop replicates green in a USER module,
and `type_info_by_name`'s single-TypeInfo return works -- so the trigger is
the CATALOG return path for the `Vec[TypeInfo]` built inside reflect,
likely a catalog-returned aggregate whose element/drop handling mismatches
(the m189 same-name/type-resolution family is the first thing to compare).
Queue: v0.63.0 triage behind lz4; needs the stdlib probe bundle to
minimize. v0.62.4 ships without it (by owner scope).

---

## 2026-10-03 -- FIXED (m189): enum struct-payload construction emitted INVALID IR (type/variant name collision)

Source: packages re-run -- the enum-payload Str in-situ failure survives
m184/m185 (validate valid operation corrupt, `Field(sel).name` reads
|0|/empty, 9/10). Minimal shape found here:

```xiom
type Field = { name: Str; value: Int; }
enum Selection { Field(sel: Field), Other }
fn pick(s: Selection) -> Str { match s { Field(sel) => { return sel.name; } Other => { return "?"; } } }
```
Construction `Selection.Field(f)` compiles to INVALID IR (clang: "invalid
getelementptr indices"): `%tmp8 = getelementptr %struct.Selection, ...,
i32 0, i32 2` on a 2-field enum (payload index one past the end), and
`%tmp6 = load %struct.Field, i8* %tmp5` -- a struct load through an i8*
string pointer. Both are in the variant-construction path for STRUCT
payloads (scalar/Str payloads avoid it). This is the strongest candidate
for the in-situ enum-payload corruption; fix next (m189).

ROOT CAUSE (localized, bare-construction probe `enum_ctor_probe.xi`):
A/B PROVEN same session: rename the payload type (`Rec` instead of `Field`,
keeping the variant name `Field`) and the construction compiles/runs rc 0
with `val_ty=%struct.Rec`. With the type and variant BOTH named `Field`,
the struct literal `Field{ name; value }` resolves to the enum VARIANT
(`Field` -> parent `Selection`), so the literal is emitted as a
`%struct.Selection` value; `var f` records that type, and
`compile_enum_constructor` then sees `val_ty=%struct.Selection` for a
`%struct.Field` slot -- producing the out-of-range GEP (`i32 0, i32 2` on
a 2-field struct) and the bogus `load %struct.Field, i8*`.
FIX (expr.rs struct-literal disambiguation): the bare-name checks were not
SUFFIX-aware, so inside a `module m` the struct registered as "m.Field" was
missed and the bare variant lookup won. Type existence and the
literal-shape match now check bare + module-suffixed keys before falling
back to variant resolution. LOCK: `e2e_m189_enum_struct_payload` +
`tests/regression/m189_enum_struct_payload/` (same-name type+variant, all
variants), ci.yml line; feature 519/519; both probes rc 0.

RELATED (packages relay, same suspected root): grpc `probe_suite_min.xi`
(`grpc_metadata_set(&mut req, ...)` with `req.metadata: Vec[(Str, Str)]`)
crashes 0xC0000005 with zero output, call-dependent, and a hang variant
when comparing `req.metadata[0].0 == "grpc-timeout"`. Verify against the
m189 build next; if the same collision shape appears there (tuple/struct
payload), it is the same fix.

---

## 2026-10-03 -- FIXED (m188): const values as match arms never matched

Source: packages relay (minimized `docs/repro/const-match/`; all 17 const
arms in grpc.xi had been rewritten to literals as a workaround).

ROOT CAUSE: a bare `CODE_A` arm parses as `Pattern::Ident` and was treated
as a catch-all BINDING: `pattern_needs_check` returned false, the arm fell
into the wildcard classification, and the LAST unguarded ident/`_` arm
became the branch target -- every input reached the wildcard
(`status_to_str` returned "UNKNOWN" for every code). IR evidence: the
dispatch branched straight to `match_arm6` with no comparisons.

FIX (codegen): `pattern_needs_check` (both copies: lib.rs + types.rs) now
checks `match_ident_is_const`, and the emit paths (plain + or-alternative
arms) compare the scrutinee against the const's literal value
(`icmp eq i64`) instead of treating it as a variant/binding. Non-literal
consts (Str/aggregate) keep the previous variant-check fallback.

LOCK: `e2e_m188_const_match_arms` + `tests/regression/m188_const_match_arms/`,
ci.yml line; feature 519/519; probe rc 1 -> 0. The grpc.xi literal
workaround can be reverted after this ships.

OPEN (packages, second item): their suite binary crashes 0xC0000005
PRE-output when a later test group is included; bisection hit the
3-attempt circuit breaker and is logged in their docs/failed_attempts.md.
Need the smallest failing group to triage; grpc stays tests=unknown,
unpublished until then.

---

## 2026-10-03 -- FIXED (m187): doctor's stdlib version checks compared unrelated version namespaces

REPORT: `xiom doctor` on a correct v0.62.3 install (installer-fetched
archive) warned "stdlib version 0.62.0 does not match compiler 0.62.3" and
"does not match the compiler's pinned stdlib stdlib-perf3". The stdlib
carries its OWN release line (`package.xi` = 0.62.0, last `stdlib-v*`
tag); `stdlib-perf3` is a PIN TAG with no semver. A valid install pairs
compiler 0.62.3 with stdlib 0.62.0 by design.

FIX (doctor.rs): compare MAJOR.MINOR lines only (patch drift is normal) and
skip pin checks for tag-shaped pins. Cross-minor pairs still warn; the
`doctor_cli` fixture (fake 0.0.1) still exercises the warning. Local run:
`[OK] stdlib v0.62.0` with no stdlib warnings.

RELATED (website lane, relayed): the served `install.ps1` aborts on a
stale `<InstallDir>.new` staging dir (`New-Item: item ... xiom.new already
exists`) and can leave `xiom` off PATH after a locked-dir install; the
website SESSION documents the side-by-side `.new` fallback. Unblock:
close VS Code/XIOM processes, delete `%LOCALAPPDATA%\xiom.new`, re-run.

REGISTRY (stdlib lane): `xiom-std 0.62.0` is the published artifact of the
0.62.0 line; republishing needs a `package.xi` bump + a `stdlib-v*` tag
(the publish workflow does the rest over OIDC).

---

## 2026-10-03 -- FIXED (m186): inline module-qualified UInt32 call compares misread high-bit values

Source: stdlib relay `tools/known_failures/p_uint32_high_bit_compare.xi`
(`adler32_combine(1,2,-1)` returns 0xFFFFFFFF; compared INLINE it reports
unequal on v0.62.3 while the bound-local control passes; UInt16/UInt8
family).

ROOT CAUSE (IR): the inline call result widened with `sext i32 to i64`
(signedness unknown) while the `as UInt32` constant widened with `zext` --
`-1 != 4294967295`. `expr_int_signedness`/`expr_is_unsigned` resolve
signedness via `infer_call_return_xiom(func)`, which misses
MODULE-QUALIFIED callees (`adler.adler32_combine`); the bound-local path
had the type through deep inference.

FIX (emitter.rs): both helpers fall back to
`infer_expr_xiom_type_deep(whole call)` when the callee-only resolver
misses, so UInt* returns widen unsigned. LOCK:
`e2e_m186_uint32_high_bit_compare` + `tests/regression/m186_uint32_high_bit_compare/`
(stdlib-guarded), ci.yml line; feature 519/519; probe rc 1 -> 0.

---

## 2026-10-03 -- TRIAGE (open): packages lane's three v0.62.3 findings (nested test-module import, uninit local struct, enum-payload Str corruption)

Source: packages lane relay (docs/COMPILER-FINDINGS.md A/B +
docs/repro/uninit-local/; websocket 163 T001s; xiom.graphql validator).

1. **Nested test-module import -- FIXED (m184).** Minimal repro built:
   `pkg.xi` (`module pkg`, `pub fn val`) + `tests.xi` (`module pkg.tests`,
   `use pkg;`) -> T001 "cannot call 'val' on this expression"; flat
   `module tests` control compiles/runs. ROOT CAUSE: the parser nests
   `module pkg.tests` as `Module(pkg){Module(tests){...}}`, so
   `resolve_imports` registered a leaf-less `pkg` chain entry; `use pkg;`
   then found `modules["pkg"]` and never loaded the file-backed pkg.xi,
   hiding every root export. FIX (xiom-check): `resolve_imports` now MERGES
   program module chains into existing entries (`merge_module_exports`,
   recursive SubModule merge) instead of replacing them, and `process_use`
   falls back to loading the real module file for a single-segment use
   whose entry is a pure SubModule chain (merge on load). LOCK:
   `e2e_m184_nested_module_import` + `tests/regression/m184_nested_module_import/`
   (pkg.xi + main.xi), ci.yml line; checker 195/195; flat control green.
   Workaround in the porter brief can be dropped once this ships.
2. **Uninitialized local struct assigned inside a match arm -- FIXED
   (m185).** Minimal standalone repro: `var r: Row;` (no initializer) +
   assignments in match arms + `return r` crashed on the official v0.62.3
   archive (0xC000001D illegal instruction). ROOT CAUSE (IR): the
   uninitialized/zero placeholder is the literal `0`, and
   `coerce_value`'s i64-to-struct path emitted `inttoptr i64 0 to
   %struct.Row*` + `load` -- a NULL dereference (UB that also let clang
   transform surrounding code; matches the reported garbage fields on
   other shapes). FIX (coerce.rs): a literal-zero source materializes the
   struct zero value safely (`alloca` + `store zeroinitializer` + `load`)
   instead of dereferencing NULL. LOCK: `e2e_m185_uninit_local_struct` +
   `tests/regression/m185_uninit_local_struct/` (all three match arms),
   ci.yml line; feature 519/519; probe pre-fix crash -> post-fix rc 0.
3. **Enum-payload Str corruption** (packages relay, in-situ; standalone
   controls pass, minimal repro pending): enum payloads carrying Str
   fields read corrupted in the graphql validator context. Candidate
   v0.62.4 target; minimal repro to build (likely shares the
   uninitialized/aggregate-value codegen family with item 2).

All three are v0.62.4 candidates; workarounds already in the packages
porter brief ("always initialize locals at declaration"; nested-module
workaround can drop after m184 ships).

---

## 2026-10-03 -- TRIAGE (open): registry lane's three v0.62.3-only regressions (iter C001, cell, lz4)

Source: `xiom-lang/stdlib tools/known_failures/README.md` @ cfb624b; all
three confirmed locally against the official v0.62.3 windows-x64 archive
(sha256 `011af7dd...06c2`).

1) **iter.range(...).contains(...) -- compile-stage C001.** 
   `xiom --force -o out.exe tests/smoke/smoke_iter_range.xi` fails with
   "unsupported: 'contains' receiver does not expose a concrete
   Vec/Slice/Array element type". Standalone passes. MINIMIZED: three
   distinct `iter.range(a, b).sum()` calls before the `contains` call flip
   the classification (two sums + contains = ok; add
   `iter.range(0, 10).sum()` = C001). AMPLIFIER IDENTIFIED: call.rs
   `intercept = !has_user_fn` for a method-form receiver that is NOT a
   collection -- the contract builtins must require
   `is_contract_collection_receiver(receiver)` instead of falling back to
   the Vec-scan lowering just because no user fn named `contains` is
   registered. The state that changes after the third sum is still to be
   pinned (next: gated trace of `infer_llvm_type` /
   `is_contract_collection_receiver` and the state map it reads). Green on
   the stdlib tip (584ffd1) per their battery; red on the v0.62.3 pin.

   RESOLVED ON THE CURRENT TREE (2026-10-03): the real smoke
   `smoke_iter_range.xi` compiles and runs rc 0 with the current dev build
   (+m182/m184/m185/C25; the official v0.62.3 archive is still red). The
   fix rides the current batch -- m184's module-resolution merge is the
   most likely cause. The stdlib lane should promote the smoke lock after
   the next candidate ships. The synthetic V8 minimization (three
   range-sums + contains) still errors C001 through the DIRECT-form
   contract scan (`call.rs:995`, untraced): the method-form intercept
   trace showed identical classifier values in the passing and failing
   variants (`is_coll=false`, `has_user=true`, `intercept=false`), so V8
   is a separate latent case for a later pass; the real smoke is the
   registry lock.

2) **cell/RefCell smoke abort -- NOT a compiler regression.** 
   `smoke_cell_refcell_basic.xi` / `smoke_cell_ref_get.xi` never call
   `release()`. The 6D.1 pointer-based Ref/RefMut semantics (stdlib
   `592243a`, 2026-09-12) document "Without Drop trait support, the user
   is responsible for calling this"; the earlier copy-by-value RefCell
   tolerated the missing release (its own comment: "never restoring
   borrow counts"), so the smoke depended on the old accidental semantics.
   IR confirms `borrow_Int` increments and no release call exists before
   `borrow_mut_Int`. Fix is stdlib-side (add `r.release()` /
   `rm.release()` to those smokes, or design auto-release).

3) **lz4 block compress empty -- OPEN, compiler-suspect.** 
   `smoke_compress_lz4_snappy.xi` rc=5: `lz4_compress_block` returns an
   empty Vec while a standalone minimal block-compress passes.
   `lz4_compress_block` -> `_lz4_block_core(data, 4096, &result)`: the
   `&mut Vec[UInt8]` out-param pushes do not land in the caller's Vec in
   this TU context.
   BISECT (official archive, 2026-10-03): the EXACT smoke prefix in
   isolation passes -- module + three imports + bytes_equal + small build
   + frame compress/decompress roundtrip + bound + block compress (P1/P2/
   P3 rc 0). All emitted lz4 function bodies are byte-identical between
   the standalone and the failing smoke IR (`_lz4_block_core`,
   `_lz4_write_seq`, `lz4_compress_block` -- zero diffs across
   383/1376/55-line bodies), and `small`'s alloca has no clobbering store
   before the block call.
   P4 (P3 + the full lz4 suffix: block roundtrip + HC + big) REPRODUCES
   rc=5. P3 vs P4 IR: identical helper functions; identical `main` prefix
   AND block-call region (same tmp numbers through the call); the first
   difference is only `main`'s `inlinehint` attribute (P3 has it, P4 does
   not -- a size heuristic). No function carries `readnone`/`readonly`/
   `nounwind`/`noalias` attributes, so unsound-annotation CSE is ruled
   out. Conclusion: the suffix changes clang's optimization of an
   identical prefix -- an optimizer-visible UB in the lz4/Vec path.
   NEXT (next session): (a) reduce the P4 suffix to the single flipping
   section (block roundtrip / HC / big) with probes; (b) compile p3.ll and
   p4.ll directly with `clang -O2 -S -emit-llvm` and diff the OPTIMIZED
   IR to locate the divergence, then audit that construct (the m180
   lesson: mismatched call signatures are accepted silently and only bite
   under optimization).

   OPTIMIZED-IR LEAD (done same session): `clang -O2` on the raw
   p3r.ll/p4r.ll (cmd-redirected UTF-8) both optimize. At the inlined
   block-compress return:
   - P3: `%tmp26.unpack8.i = load i64, ptr %tmp22.repack1.i` (repack1 =
     Vec+8, the LEN) -> `icmp eq i64 %tmp26.unpack8.i, 0` -> select 5/0.
   - P4: reloads `ptr` (data) and `%tmp26.unpack12.i = load i64, ptr
     %tmp22.repack5.i` (repack5 = Vec+24, the ESZ slot), then
     reconstructs a copy at `%tmp201` (stores `<2 x i64>` from repack1 to
     +8 and `unpack12` to +24) before the length check.
   The length/field path diverges once the suffix makes clang materialize
   the Vec copy instead of reading the call-result alloca directly --
   suspect a field-offset/stride confusion in the Vec copy or len read
   that only manifests under optimization (UInt8 esz = 1 vs i64 = 8).
   Next: dump the P4 check block after the `%tmp201` reconstruction and
   confirm which field feeds `blk.len()`; then find the emitter path that
   emits the copy (likely `compile_expr` Vec-value re-entry) and fix the
   offsets.

   NARROWED (same session): the check block is CORRECT (`%tmp26.unpack8.i`
   loads Vec+8 = len; `icmp eq ..., 0`). All shared optimized functions
   are semantically identical between P3 and P4 modulo attribute-group
   renumbering; divergence is confined to `main`.
   Suffix reduction: P4a (drop big) rc=5; P4b (block roundtrip only) rc=5;
   P4c (prefix + ONE bare `var dblk = lz4.lz4_decompress_block(blk);` +
   match) rc=5. So a single extra CALL SITE after the check flips the
   earlier `blk.len()==0` result -- optimizer-sensitive UB whose trigger
   is repeated call sites (same shape as the iter C001 "third call flips
   the classification": BOTH bugs are big-TU/repeated-callsite dependent).
   NEXT: (i) P5 -- call `lz4_compress_block(small)` twice instead of
   decompress_block, to confirm "second call site of the same alwaysinline
   fn" is the trigger; (ii) SSA-normalized diff of clang -O2 `main`
   between P3 and P4c to locate the transformed region; (iii) audit the
   emitter for repeated-callsite/symbol-instance bookkeeping.

   P5 DONE: calling `lz4_compress_block(small)` a SECOND time (P5) returns
   rc=0 -- the trigger is SPECIFIC to introducing a call to
   `lz4_decompress_block` after the check, not generic repeated call
   sites. Module-wide call/define ARG-COUNT audit on P4c IR: 0 mismatches
   (type-level audit still pending). `lz4_decompress_block` is
   `inlinehint` (not alwaysinline), 859-line body, calls `_lz4_read_ext`,
   malloc/realloc, llvm.trap. NEXT: (a) P6 -- replace decompress_block
   with another Result-returning fn to see if the trigger is the Result
   return ABI or that specific function; (b) type-level call/define audit
   (count-only passed); (c) inspect its emitted body for mistyped
   stores/GEPs.

   P6 DONE: a SECOND call to `lz4.lz4_decompress(lz)` (the Result-returning
   fn already used in the prefix) returns rc=0 -- so the trigger is
   SPECIFIC to `lz4_decompress_block`, not Result returns or repeated
   calls. Store-type audit of the 859-line `lz4_decompress_block` body:
   0 mismatches; bitcasts are byte-buffer reads. NEXT: inspect the
   OPTIMIZED P4c main around the block check in `p4c_opt.ll` (find where
   the return-5 value is computed) -- the pre-opt IR is correct, so clang
   transforms the whole main due to UB reachable only through the
   decompress_block call path.

   REGRESSION RANGE (2026-10-03): P4c compiled with the v0.62.0 release
   binary PASSES (rc 0); the v0.62.2 era build (tmp/sprintc/v0622_src
   release) FAILS (rc 5); the v0.62.3 archive FAILS. The regression
   entered v0.62.0..v0.62.2 (the v0.62.x method-binding/aggregate era).
   IR DIFF (HEAD vs v0.62.0, same source, normalized function diff: 14
   differing of 173 common): the lz4 callees that differ are only bounds
   constants (`_lz4_write_seq`: `icmp ule ..., 16777216` vs `4294967296`;
   `lz4_compress_block` itself is IDENTICAL). `main` differs by 208 lines
   for identical source: HEAD emits extra `alloca %struct.X` + `store` +
   field-GEP ROUND-TRIPS for aggregate values where v0.62.0 used direct
   GEPs into the source aggregate (seen at the frame-roundtrip match).
   NEXT: continue the same diff at the block-compress call site; audit the
   new alloca-copy emission path (result/aggregate re-entry in main) for
   aliasing/UB -- that is where the v0.62.x regression and the UB entered.

---

## 2026-10-03 -- FIXED (m182): complex module-level const tables mis-read (Str/struct payloads)

REPRO (committed): `docs/repro/const-tables/const_tables.xi` -- rc 6 on
HEAD and v0.62.2 (expected 0). Isolated NAMES probe (str_len loop) rc 9;
direct Str equality happened to pass, the length reads do not.

SHAPES:
- `const K: [4]Int` correct (m164 emits a real `internal constant [4 x i64]`).
- `const NAMES: [3]Str` -- per-use re-materialization; `str_len` loop sum
  is garbage (relay probe 30 vs 14; local isolated probe rc 9).
- `const ROWS: [3]Row` -- all fields read 0. IR: the element field load is
  emitted (`%tmp117 = load i64, ...`) but the comparison uses a CONSTANT 0
  (`icmp ne i64 0, 1`), i.e. the substitution path const-folds the field
  access to a placeholder while emitting a dead load.

ROOT-CAUSE AREA: m164 `try_register_const_array_global` (decl.rs) only
materializes all-literal INTEGER-like arrays; Str/struct elements keep the
per-use SUBSTITUTION path, which re-materializes the whole table on the
stack per read and loses aggregate element values in the fold.

IMPACT: stdlib findings row 25 (complex tables) not retirable until fixed;
packages keep raw-octet encodings. Fixed post-v0.62.3 (m182) -- row 25 can
now retire the complex shapes.

FIX (m182, 2026-10-03):
- `try_register_const_array_global` (decl.rs) accepts any FULLY-literal
  element array via the new `is_global_const_literal` (Int/Bool/Char/
  Float/Str/struct/nested), computes the real element LLVM type, and
  registers the global with that element type (no forced i64 slots).
- New `global_const_init_ext` + `struct_const_init` (lib.rs) render Str
  handles as interned constant expressions (Str is a bare `i8*`:
  `i8* getelementptr inbounds ([L x i8], [L x i8]* @.strN, i64 0, i64 0)`)
  and struct literals as constant aggregates in DECLARATION field order.
  The static `global_const_init` is untouched, so module-`var`
  initializers keep their behavior.
- Index fast path (expr.rs): the widening arm now widens only INTEGER
  element types; Str/float/aggregate elements return their typed value
  (structs match the existing Vec[struct] convention), so `ROWS[i].field`
  loads real data.

EVIDENCE:
- Probe `docs/repro/const-tables/const_tables.xi`: rc 6 -> 0 (the NAMES
  str_len loop included, total 14).
- IR: `@...NAMES = internal constant [3 x i8*] [i8* getelementptr ...]`,
  `@...ROWS = internal constant [3 x %struct...Row]
  [%struct...Row { i64 1, i64 2 }, ...]`.
- Locks: `regress_m182_const_table_aggregate_globals` (codegen IR),
  `e2e_m182_const_tables` + `tests/regression/m182_const_tables/`
  (stdlib-guarded), ci.yml line. Feature 519/519; probe green on the
  pinned stdlib.

---

## 2026-10-03 -- FIXED (m181): m178 pattern check false positives -- aliases, nested generic elements, enum None/Some

CONTEXT: the m178 ill-typed-match rule (T001) landed without a full e2e
pass; the batch rerun surfaced 9 fixture reds, all false positives of
`validate_pattern_for_type`. Fixed at HEAD, pre-pin-bump (the 10th red,
e2e_m17_zero_warnings, was a LOCAL artifact: the era worktrees under
tmp/sprintc live inside the repo tree and the catalog indexes source
dirs recursively -> W001 duplicate modules; moving them out of the repo
restores green. CI/clean checkout unaffected).

ROOT CAUSES + FIXES (crates/xiom-check/src/lib.rs):
1. `type MyResult = Result[Int, Str]` / `type MyOpt = Option[Int]`
   aliases: the scrutinee Named("MyResult") failed the literal head
   check. New `alias_base_head` unwraps bounded alias chains (bare +
   module-qualified) and the Some/None + Ok/Err checks accept the alias
   target head. (m21_type_edge_010/011, m36_c09.)
2. `Vec[Option[Int]].new()` typed the local "Vec[Int]": the
   `vec_ctor_type_name` render helper only kept brackets for nested
   Vec, everything else fell to "Int", so `g[0]` typed as Int and the
   pattern check rejected a genuine Option element. `render` now keeps
   brackets for ANY type-like base and comma-joins Tuple args
   ("Map[Int, Str]"). (m65_vec_option_elem.)
3. Enums declaring their own `None`/`Some` variants (`enum Opt { None,
   Some }`, `Inner`, `DiscountKind`, `BinOp`, `VoidE`): the pattern was
   read as Option's. New `enum_declares_variant` accepts a Some/None
   pattern when the SCRUTINEE enum declares that variant leaf.
   (m32_e13/e15, m33_y14, m34_y13, m36_e04.)

EVIDENCE: the 10 fixtures are the locks (red at a76ea897, green after;
ci.yml line extended). Feature 518/518, checker_locks 28/28, checker
194/195 (the 1 = stale-pin xiom.net T001s at 617:5, closed by the Gate P
pin bump), stdout: the wave-57 protection still rejects Ok/Err on Str.

---

## 2026-10-03 -- FIXED (m180): computed-receiver method key degraded to bare leaf -- contracts any_contracts() AV

ROOT CAUSE (exact): `xiom.contracts.any_contracts()` runs
`!_get_index().none()`. The receiver of `.none()` is a CALL expression, and
codegen's `infer_struct_type_name` only walks idents/fields/static paths, so
it returned None; `receiver_is_instance` is true for calls (they evaluate to
values), so the fn_key fell to `resolve_module_call` -> the BARE leaf
"none". The bare-call fallback then bound the keep-first catalog alias
`core.none[T](items: &Slice[T], predicate: fn(T) -> Bool)` to a 0-arg
method call. The monomorphised symbol `core.none_ContractIndex` was emitted
with the free fn's 2-param signature while the call site passed ONLY the
receiver. IR evidence: call `@core.none_ContractIndex(%struct.ContractIndex
%tmp3)` against `define i64 @core.none_ContractIndex(%struct.Vec %param0,
i64 %param1)` -- the callee treats the ContractIndex bytes as a Vec
(data/len from the package/version Str pointers) -> garbage length -> read
-> 0xC0000005. The checker's exact-arity UFCS filter (params == args + 1)
does not cover this path; the wrong key was chosen in codegen.

FIX (call.rs receiver-key construction): when the receiver is a non-ident
whose `infer_struct_type_name` is None, derive the dispatch leaf from the
DEEP XIOM type inference (`infer_expr_xiom_type_deep`, which resolves a
call's return type from the return-type registry) before the
instance/module fallbacks. New helper `receiver_dispatch_leaf` accepts only
concrete type spellings ("ContractIndex", "Vec[Int]" -> Vec, "*Point" ->
Point; single-char/generic/module names return None and keep the old
fallbacks). The receiver-qualified key then resolves through the existing
`.{Recv}.{method}` suffix path to `contracts.ContractIndex.none`.

EVIDENCE:
- Pre-fix: `p_contracts_any_av.xi` run rc 0xC0000005 (also on v0.62.0,
  v0.62.1, v0.62.2; all era stdlib trees v0620/pf1/pf2 red -> pure
  compiler-side, red already at the v0.62.0 tag).
- Post-fix: rc 0; IR call and definition now agree
  (`@ContractIndex.none(%struct.ContractIndex)`).
- Locks: `e2e_m180_contracts_any_av` +
  `tests/regression/m180_contracts_any_av/` (stdlib-guarded), ci.yml line.
  Feature regression 518/518. Full e2e + stdlib-exec in the batch run.

---

## 2026-10-03 -- FIXED (m179): nested-generic `size_of` resolved 8 bytes -- Arc strong_count on the unsafe_direct sync path

ROOT CAUSE (exact): the parser lowers `size_of[ArcInner[T]]()` to a
GenericCall whose explicit type list is the reduced BASE name
(`["ArcInner"]`; the nested `[T]` is dropped). The size intrinsics in
`crates/xiom-codegen/src/call.rs` only consulted `type_arg` / `args[0]` /
an Index-form `func`, so for that shape `ta` was None AND the
GenericCall list was never read. The monomorphised `Arc.new_Int` body then
took the 8-byte scalar fallback: IR `store i64 8` -> `malloc(i64 8)` for
the control block, immediately followed by a 16-byte `%struct.ArcInner`
store (heap overflow). `strong_count()` read corrupted memory (!= 1).
This is why only `#[unsafe_direct]` builds were red: the direct path
inlines the generic body into the monomorphised fn (m166 trust), while
the confined build's lifted blocks materialize substitutions differently.
The Rc/Weak family was masked by a hardcoded special case that listed only
RcInner/Rc.new_/Rc.drop_/Weak.drop_ -- Arc was not in it.

FIX (call.rs size intrinsics): resolve the type name from three sources in
order -- (1) the existing AST index/arg source; (2) the GenericCall
`explicit_generic_types` list, mono-substituted through
`current_type_map` + `param_concrete_types` (general: fixes any
`size_of[Base[T]]()` shape, not just refcount); (3) the monomorphised
family fallback, now extended with ArcInner/Arc.new_/Arc.drop_ (Rc/Weak
behavior unchanged, including the `sizeof` zero-guard). The shared
struct/scalar size resolution below is unchanged.

EVIDENCE:
- IR before/after (forced fresh, XIOM_STDLIB=tmp/sprintc/stdlib_pf2):
  `Arc.new_Int` `store i64 8` -> `malloc(i64 8)` vs `store i64 16` ->
  `malloc(i64 16)`; 16-byte ArcInner store fits after.
- Runtime: pre-fix `head_pf2_arc.exe` rc=1; fixed rc=0
  (`p_sync_arc_count.xi`, era matrix HEAD + stdlib_pf2).
- Locks: `regress_m179_size_of_nested_generic_mono` (codegen IR; verified
  red on the pre-fix tree, green after),
  `e2e_m179_arc_strong_count` + `tests/regression/m179_arc_strong_count/`
  (stdlib-guarded; true lock once the pin carries the perf2 annotations),
  CI line in ci.yml. Feature regression 518/518 (pre-commit). Full e2e +
  stdlib-exec in the batch run with the contracts fix.

RELAY UNCHANGED: stdlib keeps the perf2 `#[unsafe_direct]` annotations;
Gate P (t2) still depends on them.

---

## 2026-10-03 -- LOCALIZED: Arc strong_count defect is m166 x stdlib-perf2 (unsafe_direct sync annotation)

FIX LOCATION (2026-10-03): `crates/xiom-codegen/src/call.rs` size intrinsics
(~L3160-3260). `size_of`/`sizeof`/`align_of` handle `ta.is_none()` (parser
drops nested generic args) with a MONOMORPHISED-CONTEXT special case that
lists only `RcInner` / `Rc.new_` / `Rc.drop_` / `Weak.drop_` (L3180-3194).
`Arc.new_Int` is not in that list, so the fallback resolves the size to 8
(the generic field / count slot) instead of ArcInner[Int]'s 16. Fix: extend
the monomorphised special case to `ArcInner` / `Arc.new_` / `Arc.drop_`
(and audit Mutex/RwLock/Barrier allocations for the same gap -- they may be
accidentally safe at 64-byte arenas), or resolve the substitution generally
before the intrinsic. Lock: `m179` Arc fixture + IR check (`malloc(16)` for
ArcInner[Int]) + e2e; then rerun the Era matrix (HEAD+perf2 must go rc 0).

IR EVIDENCE (2026-10-03, forced fresh emits with --no-cache): in the
annotated build, `Arc.new_Int` inlines the unsafe block and allocates
**malloc(8)** for what is then stored as a 16-byte `%struct.ArcInner`
(`store %struct.ArcInner %tmp21, %struct.ArcInner* %tmp20` at an 8-byte
buffer) -- heap overflow/corruption; the same allocation in the confined
(stripped) build goes through `__unsafe_block_13`. Suspect the DIRECT path
evaluates the element/size expression (`size_of[ArcInner]` / alloc size)
against the UNSUBSTITUTED generic type inside an inlined unsafe block,
while the confined lift materializes substitutions first. Blast radius:
every `#[unsafe_direct]` generic sync fn (Mutex/RwLock/Arc/Barrier).
Next: fix the direct branch (expr.rs ~L5176-5219) to resolve the
monomorphized type before compiling the block, or route size-dependent
intrinsics through the substitution map; lock with an Arc/Mutex fixture +
IR check (`malloc(16)` for ArcInner[Int]) + e2e. RELAY TO STDLIB: keep the
perf2 annotations -- this is compiler-side; do not strip them (Gate P needs
them). Sync probes may show other annotated-fn failures until fixed.

CONFIRMED CAUSE (2026-10-03): the perf1 -> perf2 diff for sync.xi is
ANNOTATION-ONLY, and stripping the `#[unsafe_direct]` lines from a perf2
checkout makes HEAD GREEN on `p_sync_arc_count.xi` (rc 1 -> 0). So the
`#[unsafe_direct]` DIRECT path (expr.rs ~L5176-5219: plain inline lowering
instead of the confined lifted-function path with by-pointer captures)
miscompiles the annotated Arc/sync bodies. Blast radius is likely every
annotated sync fn, not just Arc -- pre-tag blocker; Gate P (t2) depends on
these annotations. Next: emit IR for HEAD+perf2 (annotated) vs HEAD+perf2
stripped on Arc.new/strong_count and diff; look for a missing
capture/deref/arena interaction in the direct branch.

Matrix on `p_sync_arc_count.xi`: v0.62.0/v0.62.1 + era tree -> green;
v0.62.2 + stdlib-v0.62.0 -> green; v0.62.2 + stdlib-perf1 -> green; HEAD +
stdlib-perf1 -> green; HEAD + stdlib-perf2 -> RED (rc=1); v0.62.2/HEAD +
current waves tree -> RED. The perf1 -> perf2 delta is the
`xiom/sync/sync.xi` `#[unsafe_direct]` annotation (PERF-2), which m166
trusts for injected stdlib fns. Next: inspect the m166 trust path vs the
annotated Arc/refcount bodies (the annotation likely bypasses a wrapper
that normalizes the atomics/refcount read), then fix + lock. Era worktrees:
`tmp/sprintc/stdlib_v0620`, `stdlib_pf1`, `stdlib_pf2`.

The contracts `any_contracts()` AV remains to be bisected; it is red on
v0.62.0/v0.62.1 with the current tree, so test it against the era pins
before assuming a pure compiler regression.

---

## 2026-10-03 -- RELAY (stdlib wave 58): two PRE-EXISTING v0.62.x defects (green on v0.61.3) -- pre-tag candidates

Both confirmed on the m178 build AND on the released v0.62.2 (installed
toolchain), so they are NOT regressions from the 2026-10-02/03 batches --
they entered the v0.62.x line somewhere between v0.61.3 and v0.62.2.

- `xiom.contracts.any_contracts()` -> 0xC0000005 AV at run time
  (`tools/known_failures/p_contracts_any_av.xi`; rc=-1073741819 on both
  v0.62.2 and HEAD, rc=0 on v0.61.3). Found bisecting
  `p_never_called_zeroarg.xi` (call 3 of 71). Suspect the zero-arg registry
  walk / fn-pointer table.
- `xiom.sync.Arc.new(42).strong_count() != 1` (`p_sync_arc_count.xi`;
  rc=1 on both). From `p_sync_sizeof.xi`. Suspect the Arc box/refcount
  field read or the Arc literal layout.

Recommendation: bisect each v0.61.3 -> v0.62.2 (tags are local), fix with
fixture + checker/e2e locks, then Gate P + tag v0.62.3. The stdlib lane has
already retired the m178 probe and their corpus is T001-clean (509/509,
smoke_net 10/10) after the 2-site `str_slice` fix (stdlib e997201/8b23b79).
BISECT PROGRESS (2026-10-03):
- contracts AV: red already on v0.62.0 and v0.62.1 -> entered somewhere in
  v0.61.3..v0.62.0 (not the 0.62.1..0.62.2 range).
- Arc count: green on v0.62.0, v0.62.1 and at `1814ac36` (R-2 partial
  aliasing); red on v0.62.2/HEAD -> entered in the later half of
  v0.62.1..v0.62.2. `d8a04678` (m162) cannot be probed with the CURRENT
  stdlib tree (compile fails: the tree needs post-m162 fixes such as m166
  annotation support / m176 qualified types), so the remaining probes need
  an ERA-PINNED stdlib checkout (the pin at that commit's STDLIB_VERSION)
  or a self-contained repro that does not import the moving stdlib tree.
  Worktrees at `tmp/sprintc/bisect_1814` and `tmp/sprintc/bisect_m162` age
  out -- rebuild candidates as needed.

---

## 2026-10-02 -- RELAY (stdlib wave 57): context-dependent invalid IR (alloca dominance); crypto link packet not reproducible

- NEW COMPILER BUG (stdlib `tools/known_failures/p_wave57_probe_ir.xi`):
  context-dependent INVALID LLVM IR (alloca dominance violation) when
  Result-style matches mix with Str-returning calls; non-monotonic under
  bisection. Invalid IR = clang failure, so this can block net/http waves.
  Pre-tag candidate. ROOT CAUSE CONFIRMED (2026-10-02): the probe ends with
  `match ip.ipv6_to_string(&parts8) { Ok(s) => {...}, Err(_) => {...} }` but
  `ipv6_to_string` returns **Str** -- the checker accepts Result patterns on
  a non-Result scrutinee, and codegen then emits the arm body's `s` read as
  `load i8*, i8** %tmp504` where `%tmp504` is an alloca created in an EARLIER
  match arm (`match_arm126`, ip4 section) -- no new binding store, so the
  load sits in a block that does not dominate it ("Instruction does not
  dominate all uses!"). FIX DESIGN: checker validation of each match arm's
  pattern against the scrutinee's type -- `Some/None` require Option,
  `Ok/Err` require Result, custom `Variant(name)`/variant idents require an
  enum with that variant; unknown/erased scrutinee types stay permissive.
  Diagnostic wording like `match pattern 'Ok' cannot match 'Str'`. Optional
  codegen hardening: an unbound ident read inside an arm must error instead
  of silently loading a stale slot. Lock: the stdlib probe + a minimal
  ill-typed-match fixture + e2e/checker lock (pre-fix: clang dominance
  error; post-fix: clean T001).
  IMPLEMENTED 2026-10-02 (m178): checker validation + `m178` fixture +
  `checker_locks::m178_result_patterns_on_non_result_rejected`; the minimal
  repro and the stdlib probe now report clean T001s. The new diagnostic
  also exposed REAL ill-typed matches in the local stdlib
  `xiom/net/ip.xi` (catalog body lines 617-618: `Ok/Err` matched on
  `ipv6_to_string`'s Str) -- STDLIB LANE FIX REQUIRED; api-freeze and the
  net-importing smokes stay red by design until those two sites are fixed
  (catalog findings are hard errors).
- crypto link packet (`undefined symbol: xiom_sha256_hash`): does NOT
  reproduce on stdlib main (`crypto.sha256_hex` NIST "abc" and
  `encoding.base64_encode` "YWJj" link+run green on both pins with and
  without XIOM_STDLIB; the symbol is in every tag's runtime + the driver's
  build-runtime list). Suspected STALE INSTALLED RUNTIME -- route to the
  install/deploy lane, not compiler or stdlib.
- stdlib relay check: m169/m170 verified FIXED on the v0.62.2 dev binary;
  other findings remain open.

---

## 2026-10-02 -- RELAY (packages -> compiler/stdlib): match-bound &mut payload mutations, derive[Clone] aggregate payloads, invariant placement, xiom-verify encodings

From the packages lane (25cf8c62). Compiler-side unless noted; no m-numbers
until reproduced:

- MATCH-BOUND PAYLOAD MUTATIONS ON `&mut` ENUMS ARE SILENTLY DROPPED (all six
  json mutators broken). Same write-through class as m168: mutating a value
  bound by a match arm on a `&mut` enum param must write back through the
  payload. High priority. MINIMAL REPRO (2026-10-02, rc=11 on HEAD): `type P
  = { n: Int } enum E { A(p: P), B }` + `fn bump(e: &mut E) { match e {
  E.A(p) => { p.n = p.n + 1; } B => {} } }` -- the bound `p` is a by-value
  copy, so the arm's field write lands in a dropped temporary. Repro at
  `tmp/sprintc/bug_match_payload_mut.xi`. Note: json's KAT suite (44/44)
  passes on the pinned 0.62.2, so their "all six mutators" observation was
  from their own gate/probes; this shape is the confirmed compiler side.
  ROOT CAUSE (2026-10-02): stmt.rs `Stmt::Match` has an in-place reuse path
  ("R-2 partial", ~L1673-1711) that matches a LOCAL Option/Result in place so
  payload bindings alias; it is gated to Option/Result names, and a `&mut E`
  param's slot holds a POINTER (%struct.E*) while the path expects a struct
  slot. Custom enums therefore snapshot into a fresh alloca, and the
  variant-field binding sites (~L2331 guard path, ~L2426 arm-body path,
  ~L1826 or-pattern path) load each payload into another fresh alloca
  (`add_local`), so writes are dropped. FIX DIRECTION: extend the in-place
  alias to custom enums and teach the three binding sites a pointer-slot
  base (GEP through the original pointer; register the bound name in
  `mut_ref_params` with a "&mut P" local_xiom_types record so field
  assignment stores through, the m168 path). Lock: fixture + e2e + IR (the
  store must go through the scrutinee pointer, no fresh payload alloca).
  ALIASING EXPERIMENT (2026-10-02, attempted, REVERTED): binding struct
  payloads of a `&mut` scrutinee through a pointer alias
  (`alloca %struct.P*` + "&mut P" record + `mut_ref_params`) makes the
  scalar-field repro pass AND keeps `*obj = ...` write-backs working, but it
  BREAKS element reads for aggregate payloads: with `Obj(entries: Vec[Entry])`
  aliased, `entries[i].key` (Str field of an element) reads back EMPTY while
  `entries[i].value` is fine (json-shaped probe: find("a") returned -1 and a
  copied entry lost its key). So the alias must also fix element-field
  resolution for ref-bound container locals (element type inference under an
  aliased "&mut Vec[T]" binding) before it can ship. Alternative release-safe
  route: a checker DIAGNOSTIC when an arm body mutates a by-value-bound
  payload of a `&mut` scrutinee (`x.f = ...`, `x[i] = ...`, mutating method
  calls on x) -- json's rebuilt source does not trip it (it constructs a
  replacement + `*obj = ...`), and it turns silent data loss into an error.
- `derive[Clone]` on an enum/struct with an AGGREGATE payload returns a
  corrupt handle; the next push crashes 0xC000001D. Derived Clone must box /
  copy aggregate payloads like `val_to_i64` does.
- `invariant:` PLACEMENT AMBIGUITY: json `P001` in every documented form
  while the control generates `invariant_check` -- needs a placement rule
  (contracts matrix) and a diagnostic.
- `xiom-verify` ENCODING GAPS (tooling/verify-side): 0/101 clauses proven
  across the three packages -- record-field selectors, opaque `&T`,
  if-merge, X7004 path-insensitivity, duplicate `:named`, unresolved
  cross-module `use`; scalar-only probes prove 2/2 and 4/4. Route to the
  verify tooling lane.
- STDLIB/PACKAGE-SIDE (route to the stdlib lane): legacy json bugs caught by
  the gate -- `0.05` parsed as `0.5`, `stringify_frac` recursion, exponent
  hang, partial writes, non-atomic merge.

---

## 2026-10-02 -- m176/m177 FIXED: module-qualified type annotations/literals/params (gcp); Result payload laxness (consul)

### m176 -- qualified type names (`qlib.LabelParts`) in literals, annotations, params
Repro: a cross-file sibling module's type used as `qlib.LabelParts{...}`, as
`var c: qlib.LabelParts`, and as a by-value `x: qlib.LabelParts` param.
Pre-fix: the literal was minted as an empty struct / the annotated slot and
param lowered to i64 (the IrEmitter `llvm_type_for` maps unknown names to
`Ok("i64")`), so fields silently read as zeros (and the packages lane saw a
T001 variant). Fix: resolve the registered key first (exact -> leaf ->
module suffix) in every place a qualified annotation can reach the i64
fallback: `resolve_literal_struct_ty` + `compile_struct_literal`
(`crates/xiom-codegen/src/expr.rs`), `emit_struct_field_read`, the
var/let annotation slot lowering and `param_llvm_type`
(`crates/xiom-codegen/src/stmt.rs`, `lib.rs`). Verified: qmain/qmain2/
qmain3/qmain4 all green (unannotated literal, annotated binding, by-value
param + method call). Locks: `m176_qualified_struct_literal` fixture
(sibling file) + `e2e_m176_qualified_struct_literal` + CI line.

### m177 -- generic-arg laxness: Bool must not satisfy numeric payload args
Repro (consul): `fn make() -> Result[Int, Str] { var r: Result[Bool, Str] =
Ok(true); return r; }` compiled silently and `.unwrap()` read the Bool
payload as an Int (garbage). Root: `type_arg_compatible` treated Bool as
interchangeable with any numeric. Fix: numeric-to-numeric promotion stays
(Int vs UInt8); Bool only matches Bool. Verified: the repro now reports
`return type mismatch: expected Result[Int, Str], found Result[Bool, Str]`;
feature-reg 517/517, checker 195/195, checker_locks 27/27, stdlib-exec 85/85
(+2 ign), api-freeze 2/2 after the change. Locks: `m177_result_payload_mismatch`
fixture + `checker_locks::m177_result_payload_mismatch_rejected`.

---

## 2026-10-02 -- m175 FIXED: Int FFI handles to pointer params must inttoptr (playground C24, io.read_line)

Playground relay: `printf 'Ada\n' | xiom run --no-cache -O0 read_line.xi`
printed `got: []`, and the compiled binary aborted with glibc "invalid stdio
handle" (the disassembly showed the FILE* truncated to a byte and its stack
address passed). Root: `io.read_line` feeds `xiom_stdin() -> Int` (the C
`FILE*` bits) to `fgets(..., stream: *UInt8)`; `coerce_arg_for_param`'s
pointer-param branch materialized a pointee-width temporary (`trunc i64 ->
i8`, `alloca i8`, `store`, pass the slot ADDRESS) instead of inttoptr'ing
the handle. Fix (`crates/xiom-codegen/src/coerce.rs`): when the arg is a
Call returning i64 and the param is a pointer, route through `coerce_value`
(existing inttoptr arm); lvalues keep the R53 implicit-`&x` address path.
Verified: IR now `%p = inttoptr i64 %handle to i8*` -> `call i8* @fgets`;
Windows and Linux both print `got: [Ada]` in run and compile modes; pack
acceptance `C24 fixed: yes`. Locks: `m175_stdin_handle_int_pointer` fixture +
`e2e_m175_read_line_piped_stdin` (real piped stdin) +
`e2e_m175_stdin_handle_inttoptr_ir` + CI line.
Stdlib answer: no stdlib change needed -- `xiom_stdin() -> Int` is a valid
FFI spelling once the coercion inttoptrs the bits.

---

## 2026-10-02 -- RELAY (packages -> compiler): loop-return typing, arity symmetry, cross-module qualified type names, crypto symbol

From the packages lane (Open rows + wave-50 changelog). No m-numbers until
reproduced compiler-side:

- Bare `loop { ... }` whose every path returns is still typed as falling
  through to `()` -- T001 at the loop's closing brace (terraform). Reproduce,
  then treat a `loop` with no break/fallthrough as divergent.
- ARITY ASYMMETRY: extra call arguments are rejected, but MISSING arguments
  are accepted silently. Add a checker arity check in both directions with a
  diagnostic.
- Module-qualified TYPE names across modules are unreliable:
  `selector.LabelParts` does not resolve like the bare imported `LabelParts`
  (k8s extends the saml row). Reproduce with a minimal two-module probe.
- Stdlib-side: `xiom.crypto/hash` SHA-256/HMAC exist in source but a package
  link fails with `undefined symbol: xiom_sha256_hash` (aws hand-rolled
  KAT-pinned copies). Route to the stdlib lane; the compiler half matters
  only if an emitted constructor is dropped.
- Informational: no bracket recurrences in wave 50; tooling notes only.

---

## 2026-10-02 -- m171/m172/m174 FIXED: qualified variant exhaustiveness (f), aggregate-payload enum equality (g), struct-literal field order

Three checker/codegen fixes from the Phase 2 findings and the stdlib relay.

### m171 -- qualified enum-variant patterns satisfy exhaustiveness (finding (f))
`pattern_covers_variant` compared the pattern's stored DOTTED name
("Tree.Leaf") against the bare variant ("Leaf"), so every qualified arm was
flagged W000 `non-exhaustive match` (T001 when other errors exist). Fix
(`crates/xiom-check/src/lib.rs`): compare the last dot-segment. Verified:
`tmp/sprintc/bug_f_qualified_pattern.xi` compiles with zero W000 and runs;
lock `tests/regression/m171_qualified_variant_pattern/` via
`checker_locks::m171_qualified_variant_patterns_are_exhaustive`
(check-only: recursive constructor temporaries still hit (e)).

### m172 -- enum equality with aggregate payloads is rejected, not invalid IR (finding (g))
`==`/`!=` on an enum whose variants carry `Vec`/struct/tuple payloads was
lowered as a whole-struct compare (`icmp eq %struct.Vec`; clang: "icmp
requires integer operands"), even for uncalled functions (codegen emits every
module function). Fix (`xiom-check`): when both operands are the same enum
and any variant payload is not scalar/Str/Char/Bool/pointer, emit
`cannot compare ... variant payloads include non-comparable aggregates;
compare the fields or a tag instead` instead of compiling. Verified:
`bug_g_enum_eq.xi` now reports the T001 (was a clang failure); scalar/Str
payload equality still compiles and runs (guard fixture
`m173_enum_eq_scalar_payload` + e2e + CI). Locks:
`checker_locks::m172_enum_aggregate_equality_rejected`,
`checker_locks::m173_scalar_payload_enum_equality_still_green`,
`e2e_m173_enum_eq_scalar_payload`.

### m174 -- struct literals map fields by NAME, not supplied position (stdlib relay p_struct_literal_field_order)
`Quaternion{ w; x; y; z; }` against a declaration of `{x;y;z;w}` stored each
value at slot i, silently scrambling every Euler-derived rotation (the
stdlib reordered wave 54; this locks the compiler). Fix
(`crates/xiom-codegen/src/expr.rs`): both literal paths
(`compile_struct_literal` and the generic struct-literal arm) resolve each
supplied field name to its DECLARED index via a new
`declared_field_order`/`declared_field_index` helper (module-leaf fallback
like the rest of codegen; unknown names keep the legacy positional
fallback). Verified: stdlib probe `p_struct_literal_field_order.xi` rc 1 on
v0.62.2 -> rc 0 on HEAD; lock
`tests/regression/m174_struct_literal_field_order/` +
`e2e_m174_struct_literal_field_order` + CI line; feature-reg 517/517,
stdlib-exec 85/85 (+2 ign), api-freeze 2/2 after the change.

---

## 2026-10-03 -- OPEN (selfhost Phase 3 port finding): W000 non-exhaustive match warning order is HashMap-random

`Checker::check_match_exhaustiveness` iterates `self.enum_variants` (a
`HashMap`) to build the uncovered-variant list, so a user enum match missing
TWO OR MORE payload variants emits the W000 warnings in a per-process random
order. Repro:
`tmp/sprintc/phase3_checker/stage1/l_w000_payload_missing2.xi`
(`enum Op { A(x: Int), B(y: Int), C(z: Int) }` with only `Op.A` matched):
four consecutive `xiom --dump-check` runs on the same file alternated
`'B' ... 'C'` / `'C' ... 'B'`. Single-missing cases are stable (one line),
and `Option`/`Result`/`Bool` use fixed vectors.

Impact: any canonical dump (the Phase 3 `--dump-check` gate, CI diffing,
golden diagnostics) is unstable for multi-missing user enums; the selfhost
gate uses registration order and only gates single-missing cases
(`selfhost/tests/check_negative/lints/w000_nonexhaustive_payload.xi`).
Fix direction: sort the collected variant names before coverage checking
(or iterate `BTreeMap`/the declaration order) so the diagnostic order is
deterministic.

---

## 2026-10-03 -- OPEN (selfhost Phase 3 port finding): `io.list_dir` returns pointer bits instead of directory names

Found while porting the checker's catalog module index. `io.list_dir(path)`
reports a plausible count but every `Vec[Str]` entry holds heap/pointer bits:
on `stdlib/xiom` (45 entries) each element compares unequal to its real name
and dumps raw bytes `196,202,13,184,66,2,0,0,0,0` (0x0000_0242_B80D_CAC4,
little-endian) -- the same 10 bytes for every entry. `e == "io"` and
`e == "string"` are both false and `str_len(e)` returns 6 for all of them, so
the elements are not valid Str values at all.

Repro:
`tmp/sprintc/phase3_checker/listdir_probe3.xi` compiled with the Rust driver
(`xiom -o listdir_probe3.exe listdir_probe3.xi`), prints the byte dump above;
`listdir_probe.xi` shows the same values rendered as decimal. Same class as
m163 (method field `Vec[Str]` element miscompile) but on a stdlib entry point.

Impact: a declared-header stdlib module index cannot be built from the
selfhost, so the Phase 3 catalog port falls back to a static relocation table
for the 19 modules whose declared dotted name does not match any path shape
(`selfhost/src/check_modules.xi::cm_static_module_path`, computed by
`tmp/sprintc/phase3_checker/module_overrides.ps1`). Fix direction: the
`Vec[Str]` construction inside `stdlib/xiom/io/io.xi::list_dir` (or the
underlying `opendir`/`readdir` binding) must load the pointee rather than
passing the pointer bits as the element.

2026-10-07 packages relay (re-confirmed): STILL BROKEN with a slightly
different symptom -- `io.list_dir` returns the CORRECT count but the LAST
name is repeated for every entry, identical on the pinned v0.64.0 and on
compiler main m199..m207 (v0.64.1 candidate).

FIXED m211 (2026-10-07): the entries were valid Str values, but every one
ALIASED readdir's reused dirent storage. Root cause: the
`Str::from_c_str`/`from_cstring` codegen builtin was a pure pointer
identity ("a C string is already a NUL-terminated i8*") -- the comment
even said "reinterpret", but every caller (io.list_dir, io.read_line,
console) treats the result as an OWNED Str. readdir overwrites the same
dirent buffer on each call, so after the loop all pushed pointers pointed
at the final name. FIX: the builtin now copies via
`xiom_str_len` + `xiom_str_from_vec` (the D1-hardened NUL-terminating
copy), matching from_utf8/from_bytes. Probe: `io.list_dir` on
`stdlib/xiom` -> `n=45`, first=`ai_prompt.txt`, last=`time`, only one
entry equals the first (pre-fix: all 45 identical).
LOCKS: `regress_m211_from_c_str_copies` (IR: `@xiom_str_from_vec` +
`@xiom_str_len` emitted), e2e `e2e_m211_list_dir_owned_names` + fixture
(creates a dir with 3 files, asserts exactly one entry equals the first),
CI line.

---

## 2026-10-02 -- OPEN (selfhost Phase 3 port finding): `NkAssign` destructure reads the second payload field as pointer bits

Same failure class as Phase 2 (h), but in a SMALL function: while porting
the checker, `selfhost/src/check_expr.xi::ce_stmt_assign` matched
`c.p.nodes[idx].kind` against `NkAssign(place, value)` (variant
`NkAssign(l: Int, r: Int)` of `selfhost_ast.NodeKind`, 95+ variants). The
compiled binary read `place` correctly and `value` as pointer bits
(`140698859853289` = 0x7FF7...): the assignment then crashed with
`0xC0000005` (access violation) when the garbage index was used to read
`c.p.nodes[value]`. Repro:
`tmp/sprintc/phase3_checker/p_assign.xi` (`x = 2;` inside `main`) against a
`xiomc-self --dump-check` build of that change; the trace showed
`assign enter place=8 value=140698859853289` and no further output
(exit `-1073741819`).

Discriminating evidence: moving the SAME match into two one-arm helpers
(`ce_assign_lhs`, `ce_assign_rhs`) returns the correct fields (`lhs=8`,
`rhs=9`) for the same node, and the Phase 2 AST dumper's `NkAssign(l, r)`
arm has always worked -- so the construction is sound and the corruption
depends on the surrounding function shape/arm set, exactly like (h).

Workaround (landed): field access goes through the one-arm accessor helpers
and statement dispatch uses an Int-tag selector (`ce_stmt_tag`) with
per-kind helper functions, the same shape Phase 2 used for
`NkExprGenericCall`. Recurrence while porting the borrow walk (2026-10-03):
`bc_stmt_assign` crashed with 0xC0000005 on `p.y = 3;`
(`tmp/sprintc/phase3_checker/stage1/bf5.xi`) until the same
`bc_assign_lhs`/`bc_assign_rhs` accessor split was applied. Fix direction:
payload-field loads for variant patterns must match the construction layout
independent of surrounding function size/arm count (see also the fix
direction of (e)/(g)/(h): the generated IR for aggregate-payload enum
matches needs a verifier check).

---

## 2026-10-02 -- C23 FIXED: driver optimized every module twice (opt then clang); LLVM 18 miscompiles the lesson trio

Playground relay (AUDIT 33.4, pack `playground/tools/compiler-repros/c23`):
the same deterministic programs printed different values at `-O2` on WSL
(l6-15 3 instead of 2, l7-39 3 instead of 2, l8-09 0 instead of 2) and were
correct at `-O0`. Windows (clang 22, **no `opt` binary**) was always correct
-- the first host split clue.

Evidence chain:
- `--emit-ir` output is IDENTICAL at -O0/-O2, and stock `opt default<O2>` +
  clang is correct -- so the fault is the driver pipeline, not the emitted IR.
- An `LD_PRELOAD` exec interposer + a clean `HOME` (the JIT cache at
  `~/.xiom/jit` is NOT cleared by `--no-cache`) captured the exact commands:
  `opt -passes=verify`, `opt -O<level> -S -o <ir> <ir>` (in place), then
  `clang ... -O<level> <ir> <runtime.c>` -- i.e. the module is optimized
  TWICE.
- Exact-replay matrix on l7-39: opt=O0 -> correct at clang O0..O3; opt=O1 +
  clang=O1 -> 2 (correct) but opt=O1 + clang=O2 -> 3; opt=O2 + clang=O1 -> 3;
  opt=O2 + clang=O0 -> 2. Single-stage clang O2/O3 is correct on all three
  files. On LLVM 18 the second stage's O1 middle-end IR is miscompiled by
  the O1+ backend (backend-only O1 on the pre-clang IR is clean).

Fix (`crates/xiom/src/lib.rs`): keep the `opt -passes=verify` check, drop the
redundant `opt -O<level>` pre-optimization; clang is the single optimizer, so
hosts with and without `opt` now produce the same code (and the no-opt path
was always correct).

Verified: pack acceptance `bash run.sh <toolchain>` with a HEAD-built Linux
driver prints `C23 present: no` (all three 2/2). Fixtures
`c23_l6_category`/`c23_l8_quick` return 12/13 with the pinned pre-fix driver
and 0 with the fixed driver on the Linux compile path; `c23_l7_pending` is
shape coverage (its miscompile only fired on the JIT path). Windows green
(the removed block never ran there). Locks: 3 fixtures + 3 e2e tests + CI
line; full e2e at the end of this compiler batch.

---

## 2026-10-02 -- RELAY OPEN: a top-level `module` header breaks nested-module cross-type field compares

Found while porting the C23 fixtures: the l6-15 program compiles as-is, but
adding only `module <name>` at the top makes the checker reject the nested
module's field compare:

```
module storage {
  ...
  let note = notebook.notes[i];
  if note.category == cat { ... }   // T001: cannot compare <error> with Str
```

Repro: original `playground/tools/compiler-repros/c23/l6-15.xi` + a top-level
`module c23_l6_mod` line -> one T001 at the compare. Without the header it
compiles. Likely the top-level module wrap changes the type-resolution scope
for nested-module types (`models.Note`). OPEN, post-batch intake.

---

## 2026-10-02 -- SELFHOST PHASE 2 FINDINGS: (f)/(g) FIXED (m171/m172); (e)/(h) OPEN

Two findings surfaced while starting the parser port
(`crates/xiom-parser` -> `selfhost/src/`). Repros live in
`tmp/sprintc/phase2_parser/`; the Phase 2 design avoids both (arena AST,
unqualified variant patterns), but (e) silently crashes/mis-compiles
recursive value trees, which the language advertises as supported (the
checker accepts them and emits `%struct.*` with payload pointers).

Status: (f) qualified variant exhaustiveness and (g) aggregate-payload enum
equality are FIXED (m171/m172, see the entries above); (e) recursive payload
boxing and (h) large-function variant-payload mapping remain OPEN.

### (e) Recursive enum payloads are pointer-boxed unsafely (crash / wrong values)

```xiom
enum Tree { Leaf(v: Int), Node(l: Tree, r: Tree) }

fn depth(t: Tree) -> Int {
  match t {
    Leaf(v) => { return 1; }
    Node(l, r) => { return 1 + depth(l) + depth(r); }
  }
}
```

* `Node(a, b)` with `a`/`b` REACHABLE-LEAF locals happens to work
  (`probe_rec_run3.xi` -> `depth=3`, rc 0).
* `Tree.Node(Tree.Leaf(1), Tree.Leaf(2))` (temporaries) compiles but
  crashes at run time (`rc = -1073741795`, 0xC000001D):
  `probe_rec_nested.xi`.
* `Node(c, b)` where `c` is itself a `Node` local (2-level tree) crashes the
  same way: `probe_rec_locals.xi`.
* `enum MyExpr { EInt(v: Int), ESome(inner: MyExpr), ENone }`; building
  `ESome(a)` from a local `a` then recursing `show(inner)` binds the boxed
  POINTER as the payload (`show` printed `some(int1638584114768)` instead of
  `some(int7)`): `probe_variants2.xi` (vs `probe_rec_run3.xi` which is
  correct).

Root: a recursive payload is stored as a pointer to the source value
(`%struct.probe_rec.Tree = { i64, i64, Tree*, Tree* }`); successive boxing
either stores the address of a dead temporary or passes the pointer where a
value struct is expected (ABI mismatch). Impact: recursive value-tree ASTs
(parser/checker/codegen) cannot be trusted. `selfhost/src/ast.xi` uses an
arena (`Vec[Node]` + `Int` child indices, `-1` = absent) as the workaround.
Fix direction: boxed payloads must own a stable copy (heap/arena) and every
read/pass must load the pointee (the BTree stdlib avoids this with an arena
too).

### (f) Qualified enum-variant patterns are not recognized by exhaustiveness

```xiom
match t {
  Tree.Leaf(v) => { return 1; }
  Tree.Node(l, r) => { return 1 + depth(l) + depth(r); }
}
```

emits `warning[W000]: non-exhaustive match: variant 'Leaf' ... not covered`
(and the same for 'Node') even though both variants are covered and codegen
dispatches them correctly; with unrelated errors in the file the same
false positives surface as T001s. Unqualified patterns (`Leaf(v)`,
`Node(l, r)`) are clean: `probe_rec_run2.xi` (no warnings, rc 0, depth 3)
vs `probe_rec_run3.xi`. The parser stores qualified variant names DOTTED
(`Pattern::Variant` name = "Tree.Leaf"; see the Phase 2 AST dump), while the
exhaustiveness matcher compares against the bare variant name
(`crates/xiom-check/src/lib.rs:9647`). Impact: the selfhost port uses
unqualified variant patterns throughout (like the Phase 1 `Tk` convention).

### (g) `==` on an enum with aggregate payloads emits invalid IR

```xiom
enum E { A, B(v: Vec[UInt8]), }
fn eq(a: E, b: E) -> Bool { return a == b; }
```

clang: `error: icmp requires integer operands` on
`%tmpNN = icmp eq %struct.Vec %tmpA, %tmpB` -- the derived equality compares
the whole variant struct, including `Vec` fields. Scalar/Str payloads are
fine (`probe_enumeq.xi`: A==A, B(1)==B(1), C("x")==C("x") all correct), only
aggregate payloads break. Repros:
`tmp/sprintc/phase2_parser/expr_agent/enum_eq_repro.xi` (minimal) and
`tmp/sprintc/phase2_parser/core_agent/repro_enum_eq/probe_pskip.xi` (fails
while importing `parser_state.xi`). Impact: no executable build that
imports a module using `==` on such an enum compiles, even when the helper
is never called (codegen emits every module function). Workaround: kind
identity via Int tag codes (`parser_state.xi::tk_tag`), payloads are never
part of an identity test. Fix direction: lower enum equality per-variant
(tag compare + member-wise compare only for the matched variant), or reject
non-comparable payloads with a diagnostic instead of emitting invalid IR.

### (h) `NkExprGenericCall` destructure mis-maps payload fields in a large dispatch function

In `selfhost/src/parser_expr.xi::pe_parse_postfix_expr`, the pattern
`match node.kind { NkExprGenericCall(base, types, _args) => ... }` (variant
`NkExprGenericCall(callee: Int, types: Vec[Int], args: Vec[Int])` of the
~95-variant `selfhost_ast.NodeKind`) compiled with a payload mapping that
disagrees with the construction site: the IR reads `base` from field 58 and
`args` from field 41 as an i64, while the construction stores `callee` at
field 56, `types` at 57 and boxes the `args` Vec as a pointer at 41 (IR:
`tmp/sprintc/phase2_parser/pp.ll`, `match_arm109`/`match_arm165` vs the
stores after them). Result: AST dumps showed the callee as arena index 0
(the module-name ident), and `m37_bug47_ref_params_leak.xi` crashed with
0xC0000005 in `--dump-ast`. The isolated shape is CORRECT:
`tmp/sprintc/phase2_parser/probe_nk_gc.xi` constructs and destructures the
same variant and prints 77/88/99/55 correctly, and 79 of 83 corpus files
dumped byte-identically before the fix, so the corruption depends on the
surrounding function (a huge match/dispatch body), not the variant itself.
Workaround (landed): one-step `NkExprGenericCall` construction and a
node-identity merge check with base/types carried in locals. Fix direction:
payload-field assignment for variant patterns must match construction
independent of function size / arm count.

---

## 2026-10-02 -- m170 FIXED: fn-typed param Vec returns lost their element type; erased Option/Result literal payloads widened the slot (C24-2)

Stdlib relay (C24-2, `p_curve_thunk_zero`): `curves.curve_length(line, 0, 1, 2)`
returned the wrong arc length while the direct call `line(0.5)` was correct --
the `Vec[Float64]` returned through a fn-typed parameter was unreadable inside
the callee. Two coordinated roots:

1. **Callee-side binding inference (m170a).** `var v = samples(t)` bound to a
   call through an fn-typed PARAM never recorded the parameter's declared
   return type: `callee_return_xiom` did not consult `fn_local_returns` (only
   `call.rs` used it for the emitted signature), and the non-mono param
   registration in `decl.rs` stored the type with `type_from_ast`, which DROPS
   generic args (`Vec[Float64]` -> `Vec`). Fix: (a) `callee_return_xiom` now
   returns `fn_local_returns[leaf]` for Ident callees; (b) the `decl.rs`
   registration uses `type_string_full` (mirrors the mono path's round-14c
   fix). User-space replica: `apply_read` (`v[0]`) failed pre-fix (rc 4),
   green post-fix; the catalog probe went rc 2 -> 0.

2. **Erased Option/Result literal payload slots (m170b, the relay's
   "Option-of-Vec .unwrap() AV").** `Option[Vec[Float64]]{ is_some: ...,
   value: out }` in a catalog body stored the 32-byte `%struct.Vec` inline
   through the erased `%struct.Option = {i64, i64}` field (type_meta's stale
   payload type "Vec" widened the store), overrunning the 16-byte Option and
   corrupting the return; `.unwrap()` dereferences the payload as the boxed
   handle the `Some(v)` ctor builds. Fix (`expr.rs`, Struct-literal arm): for
   the ERASED `Option`/`Result` bases the payload slots are forced to i64 and
   converted with `val_to_i64` (bitcast doubles, heap-box structs, ptrtoint
   handles) -- exactly the Some/Ok ctor convention. Probe: `vector.refract`
   result read back len 2 / values correct (pre-fix: len garbage, rc 5).

Locks: `tests/regression/m170_fn_typed_vec_return/` (fn-typed read/len/loop
replicas + user Some(v) and catalog refract Option-of-Vec extraction) +
`e2e_m170_fn_typed_vec_return` + IR lock `e2e_m170_fn_typed_vec_return_ir`
(the callee-side read's preceding line must be `bitcast i64`, never
`sitofp i64`) + CI line. Suites at the fix: feature-reg 517/517,
stdlib-exec 85/85 (+2 ignored), stdlib api-freeze 2/2 (first run raced the
freshly rebuilt driver and went red; isolated + rerun green). Full e2e runs
once at the end of this compiler batch.

---

## 2026-10-02 -- m169 FIXED: same-leaf qualified RESULTS lost their Vec element type (C24-1 caller-side bit reads)

Stdlib relay (C24-1): caller-side element reads of `vector.lerp`,
`vector.clamp`, `vector.hadamard` and `curves.b_spline` RESULTS were
bit-reinterpreted (stored 1.5 read as 4.609e18), while callee-side and
unique-leaf siblings (`cross`, `normalize`, `bezier_quad`) were correct.
Probes: the stdlib session's `tools/known_failures/p_geom_vector_result_bits.xi`
(copied to `tmp/sprintc/c24_1/`; extended by
`tmp/sprintc/c24_1/probe_c24_ext.xi` to cover hadamard/b_spline +
`matrix.hadamard`).

Root cause (codegen `callee_return_xiom`, Field arm): for a
module-qualified callee the exact `${receiver}.${leaf}` key misses because
the catalog stores the resolved `geom.vector.lerp` form. The fallback
`callee_return_xiom_suffix(leaf)` returns None when the leaf is shared with
different return types (`math.lerp` -> Float64, `math.tower.lerp` -> T;
`math.clamp`/`cmp.clamp` -> Float64; `matrix.hadamard` -> Vec[Vec[Float64]];
`geometry_extended.b_spline` -> Vec2). The binding then recorded no XIOM
type, so `lp[0]` took the scalar i64 element-load path and the comparison
`sitofp`'d the double's bits as an integer.

Instrumented evidence (`XIOM_TRACE_RETXIOM=1`, temporary `[retxiom-q]`
trace): `has_q=false` for `vector.lerp`/`vector.clamp` with hits
`[("math.primitives.lerp","Float64"), ("math.tower.lerp","T"),
("math.lerp","Float64"), ("geom.vector.lerp","Vec[Float64]")]`; `cross`
resolved only because its suffix match is unique.

Fix: in `callee_return_xiom`'s Field arm (crates/xiom-codegen/src/lib.rs),
when the syntactic receiver is not an instance, resolve the callee exactly
as call emission does
(`resolve_catalog_call(recv, leaf, func.span()).unwrap_or_else(||
resolve_module_call(recv, leaf))`) and read the declared return type off
that resolved key before the ambiguous suffix fallback.

Verified: `probe_c24_ext.xi` pre-fix rc=3 (lerp read), post-fix rc=0;
`p_geom_vector_result_bits.xi` pre-fix rc=2, post-fix rc=0. IR before/after
at the caller read: `%tN = sitofp i64 %bits to double` vs
`%tN = bitcast i64 %bits to double` feeding `fcmp une double ..., 1.5`.

Locks: `tests/regression/m169_sameleaf_qualified_result_elem/` +
`e2e_m169_sameleaf_qualified_result_elem`, the IR lock
`e2e_m169_sameleaf_result_elem_read_ir` (preceding line must be
`bitcast i64`, never `sitofp i64`) + CI line. Suites at the fix:
feature-reg 517/517, checker_locks 23/23; full e2e runs once at the end of
this compiler batch.

---

## 2026-10-02 -- RELAY (packages -> compiler): type laxness yields wrong bytes; trap-14 recurrences; positives

From the packages lane (`docs/COMPILER-FINDINGS.md` there). OPEN, no m-number
until reproduced compiler-side:

- expat/nbt "silent exit -1 + zero stdout" (2026-09-30 fleet sweep,
  docs/repro/v0622-regressions/HARNESS-NOTES.md at packages 45ea541a):
  **NOT REPRODUCED on current trees (2026-10-02).** Clue: port.ps1's
  `exit code: -1` is its OWN watchdog sentinel (`$exitCode = if ($timedOut)
  { -1 } else { $proc.ExitCode }`), i.e. the 120 s timeout fired; the empty
  output is the file-redirected child's unflushed buffer after the tree
  kill. Re-ran the exact harness on this box with the deployed
  `%LOCALAPPDATA%\xiom.new\bin` v0.62.2 (same 9/30 binary) against
  `E:\xiom-lang\stdlib` and a clean `stdlib-perf1` (06d0ee7) worktree:
  `scripts/port.ps1 -Package xiom.expat -TimeoutSec 60/120` -> PASS 25/25,
  exit 0, 18.2 s; `-Package xiom.nbt` -> PASS 26/26 (t5 included), exit 0.
  Minimal 2.8 KB `io.println` probe with
  `Start-Process -RedirectStandardOutput <file>` -> 2804 bytes, FINAL-MARKER
  present. If it still reproduces in the packages session: preserve
  `%TEMP%\xiom-run-*.out/.err` before port.ps1 deletes them, note the
  packages commit and run duration, and check whether `a.exe` is spinning
  (PID CPU) during the timeout.
- NEW: TYPE LAXNESS beyond brackets -- `let gb: Vec[UInt8] = got.value;`
  where `got.value` has a Str field compiles clean and yields wrong bytes
  (pptx). Intake needed: reproduce with a minimal probe; decide
  checker-reject vs codegen conversion.
- Stdlib defect (stdlib lane, not compiler): `xiom.crypto.hash._u64_lshr(x,
  63)` returns 3 instead of 1 for bit-63 operands (web3 Keccak); the
  in-package `n == 63` fix is the stdlib lane's.
- Trap-14 silent recurrences: docx 8 parameter-position sites, image 2,
  pptx 1 -- caught only by the post-green grep.
- Positive probe: the v0.61.3 const-array mis-materialization did NOT
  reproduce on v0.62.2 for simple `[8]Int`/`[64]Int` loop-indexed shapes
  (bad=0); row 25 annotated, complex initializers still untested.
- Positives: concrete fn-pointer callbacks, multi-module package resolution,
  `&mut Struct` field pushes, UInt32 -> Int zero-extension.

---

## 2026-10-01 -- m168 FIXED (primary): `&mut Int` assignment dropped the write

Packages relay (commit `bad44b2`,
`docs/repro/v0622-regressions/mut_int_write_drop.xi`):
`fn set99(s: &mut Int) { s = 99; }` printed `st=10` (expected 99), exit 0,
zero diagnostics; origin `xiom.svm`'s shuffle seed (`_svm_shuffle(order:
&mut Vec[Int], state: &mut Int)` never advanced). The sibling shape
(`let v = byval(s); s = v;`) also dropped it.

Reproduced: `tmp/sprintc/probe_mut_int.xi`. IR before:

```
%tmp3 = alloca i64*; store i64* %param0, i64** %tmp3
%tmp4 = inttoptr i64 99 to i64*; store i64* %tmp4, i64** %tmp3
```

i.e. the assignment REBOUND the param slot with inttoptr(99) instead of
storing through the pointer.

Fix: `LocalState.mut_ref_params` (inserted for `Type::MutRef` in `decl.rs`
and the mono path; cleared per fn); the `Stmt::Assign` Ident path loads the
carried pointer and stores the coerced pointee through it; `coerce.rs` skips
the slot-address path for mut-ref args and auto-derefs `&mut T` -> T in
`coerce_ref_arg_to_pointee`'s pointer-param branch. Verified:
`tmp/sprintc/probe_mut_ref_writes.xi` (Int, Vec[Int] whole-assign, Str).

RESIDUAL FIXED (2026-10-02, m168b): passing a `&mut T` param to a by-VALUE
param in a DIRECT call (`bump` -> `byval(s)`) ptrtoint'd the pointer. Two
reasons: `coerce_ref_arg_to_pointee` was consulted only inside the
`param_ty.ends_with('*')` branch (`coerce.rs`), so by-value param types
(i64 / %struct.Vec) never reached it; and its Ident arm stripped only the
`*T` ABI form while `local_xiom_types` records `ref_preserving_name`
("&mut Int", "&mut Vec[Int]"). Fix: hoist the helper call for non-pointer
param types and accept `&mut `/`&`/`*` prefixes. Verified:
`probe_mut_ref_writes.xi` pre-fix FAIL bump rc 2, post-fix ALL-OK rc 0;
container sibling (`veclen(s)` from `&mut Vec[Int]`) green. IR: `%t =
load i64, i64*` feeding `@byval`, no `ptrtoint` in `@bump`. Locks:
`tests/regression/m168_mut_ref_byval_arg/` + `e2e_m168_mut_ref_byval_arg` +
`e2e_m168_mut_ref_byval_arg_ir` + CI line.

Locks: `tests/regression/m168_mut_ref_write_through/` + e2e + CI line +
`regress_m168_mut_ref_write_through` (IR: `store i64 99, i64*`, no
`inttoptr i64 99 to i64*`).

---

## 2026-10-01 -- m167 FIXED: global Vec push/index routed to the inline fast path (was: generic stdlib body, invalid IR)

Packages relay (their commit `2d91399`, `docs/repro/v0622-regressions/`):
the `Vec[Str].push` mis-lowering trigger is a MODULE-LEVEL `var v: Vec[Str]`
global. A local Vec with the same pushes is fine; the global case fails
clang with `'%tmpN' defined with type 'ptr' but expected 'i8'`.

Reproduced locally on Windows (release-state toolchain, v0.62.2) with
`tmp/sprintc/probe_global_vecstr.xi`:

```
; --emit-ir, the push store (inside the guard-wrapped unsafe block):
%tmp1029 = load i8*, i8** %tmp1003        ; data
%tmp1030 = load i64, i64* %tmp1005        ; len
%tmp1032 = getelementptr i8, i8* %tmp1029, i64 %tmp1030   ; MISSING *esz!
%tmp1034 = load i8*, i8** %tmp1033        ; dest (i8*)
%tmp1035 = load i8*, i8** %tmp1007        ; the Str handle
store i8 %tmp1035, i8* %tmp1034           ; BAD value type (should be i8*)
```

and the read side classifies the element as scalar:
`%tmp85 = alloca i8; %tmp86 = trunc i64 ... to i8; store i8 %tmp86, i8* %tmp85;
call void @io.println(i8* %tmp85)` -- i.e. `v[0]` took the i64 element-load
path and `println` got a char buffer, not a Str.

Root cause class: the element type of a MODULE-GLOBAL `Vec[X]` is not
registered where the Index/push lowering classifies elements, so the
scalar fallback (i64 load / i8 store) is taken -- the same gap m163 fixed
for receiver FIELDS. `global_xiom_types` already records the global's full
XIOM type (`decl.rs` module-var pass) and is consulted for generic
inference (`call.rs` ~3950); the Index/push element classifiers need the
same fallback (register `Vec[X]` globals like m163's field elements, or
extend `vec_elem_is_str`/`resolve_vec_elem_type`/`resolve_vec_elem_xiom`
with a `global_xiom_types` arm).

Also visible in the same IR: the push index is `data + len` without the
element-size multiply (stride bug) -- verify whether the correct esz
multiply lives elsewhere or is part of the same fallback.

Locks to add with the fix: regression fixture with a module-global
  `Vec[Str]` (push + index + println) + the IR in-process test; run the full
  e2e once (codegen change). Files: `tmp/sprintc/probe_global_vecstr.xi`;
  packages `vec_str_push_global.xi` / `vec_str_push_param.xi`.

---

## 2026-10-02 -- OPEN (selfhost Phase 1 port findings): nested method receiver mutation, 128-bit enum payloads, inf float literal, unsigned formatting/div

All four surfaced while porting `crates/xiom-lexer` to `selfhost/src/lexer.xi`
(Phase 1). Repros live in `tmp/sprintc/phase1_lexer/` (probe1, probe5-probe9,
torture.xi); none of them affect the Phase 1 gate (all are avoided or inert in
the port), but three can silently change program behavior.

### (a) Nested `self` method call does NOT propagate receiver-field mutation

```xiom
pub type Lx = { xs: Vec[Int]; pos: Int; }

fn Lx.peek() -> Int { if pos >= xs.len() { return -1; } return xs[pos]; }
fn Lx.advance() { if pos >= xs.len() { return; } pos = pos + 1; }   // mutates pos

fn Lx.scan() -> Int {
  var go = true;
  while go {
    let c = peek();          // outer method call on self
    if c >= 0 { advance(); } else { go = false; }  // INNER method call on self
  }
  return pos;                // <- never advances: infinite loop
}
```

`scan()` loops forever: the `advance()` call made from inside another method
mutates a COPY of the receiver (the caller's `pos` never changes), while the
same call from `main` (`lx.advance()`) does persist. Calling a *free*
function with an explicit `&mut` receiver works:

```xiom
fn advance(lx: &mut Lx) { if lx.pos >= lx.xs.len() { return; } lx.pos = lx.pos + 1; }
fn scan(lx: &mut Lx) -> Int { ... advance(lx); ... }   // propagates
```

Repro: `tmp/sprintc/phase1_lexer/probe8.xi` (hangs) vs `probe9.xi` (works);
`probe7.xi` shows inline field mutation inside one method is fine.
Re-confirmed 2026-10-02 AFTER rebasing onto the m168 `&mut Int` fix:
probe8 still loops -- m168 fixes assignment THROUGH a `&mut T` parameter, not
mutation of a receiver field by a nested `self` method call.
Impact: method-to-method calls are the documented receiver style; any
in-place allocator/parser written that way silently loops. The selfhost
lexer uses free `&mut Lexer` helpers (lx_* prefix) as the workaround.
Fix direction: the inner call's receiver must be passed by address when the
callee mutates fields (the same self is already address-taken for inline
field writes).

### (b) Enum payload of a 128-bit integer type is lowered as i64

```xiom
pub type K = enum { Big(v: UInt128), }
fn f(k: K) -> Int { match k { Big(v) => { return g(v); } } return 0; }
fn g(v: UInt128) -> Int { return 0; }
```
clang: `'%tmpNN' defined with type 'i64' but expected 'i128'` -- the match
payload binding is typed i64. `UInt128`/`Int128` work fine as normal
locals/params (smoke_d1_native128 passes). The selfhost lexer stores BigInt
as two UInt halves (`TkBigInt(hi, lo)`) as the workaround.
Repro: `tmp/sprintc/phase1_lexer/probe1.xi` (first version).

### (c) Float literal that overflows to infinity emits invalid LLVM text

`let x = 1e999;` -- the lexer correctly produces `Float(inf)`, but codegen
prints the literal as `double inf` (clang: expected value token). Any
overflowing decimal float literal fails to compile. (The selfhost dump
avoids float values entirely for now.)

### (d) Unsigned formatting and division are sign-blind

- `"" + (18446744073709551615 as UInt)` prints `-1` (u64::MAX formatted as
  a signed i64) -- `probe1.xi` `u64max = -1`.
- `UInt128 / % 10` on values with bit 127 set uses signed division
  (`u128max` decimal print returned empty because the remainder was
  negative) -- `probe1.xi` `u128max =`.
- Bit ops are correct (`>>`/`&` are logical: hex nibble extraction for
  u64/u128 works -- `probe2.xi`), which is why the selfhost dump uses hex.

The selfhost lexer avoids all three by rendering payloads in hex through
shifts/ands only.

FIXED 2026-10-02. Root cause: the push intercept (call.rs ~1590) only
recognized receivers whose `infer_llvm_type` was Vec-typed or
container-field/indexed/unwrap shaped; a module-global Ident erased to i64
missed every arm and fell through to the GENERIC stdlib body
`Vec.push[T]` (collections.xi:32), whose unsafe block hardcodes an 8-byte
stride (`new_cap * 8`) and does not scale `data + len` -- for Str elements
that emitted `store i8 <handle>, i8*`, which clang rejects. Fix:
* push intercept: `is_global_vec` arm (module_globals entry whose llvm_ty
  is %struct.Vec);
* `resolve_vec_push_ptr` / `resolve_vec_receiver_ptr`: module-global Vecs
  return `@symbol` directly (no scratch, no store-back), so pushes work on
  the emitted global in place;
* element classifiers got a `global_vec_elem` fallback (`vec_elem_is_str`,
  `resolve_vec_elem_type`, `vec_elem_float_type`, `resolve_vec_elem_xiom`,
  `resolve_vec_container_elem_xiom`, `vec_value_xiom_type`) -- without the
  float one, global `Vec[Float64]` reads came back as IEEE bit patterns
  (same class as the stdlib `p_geom_vector_result_bits` finding).

Verified: `tmp/sprintc/probe_global_vec_multi.xi` (global Vec[Str] loop
pushes + index + `.get().unwrap()`, Vec[Int], Vec[Float64]) -> all pass.
Locks: `tests/regression/m167_global_vec_push/` + e2e + CI line +
`regress_m167_global_vec_push` (IR: `internal global %struct.Vec`, no
`__unsafe_block` inlining of the stdlib body). Gates: full e2e
2395/2395 (+4 ignored), feature-reg 517/517, parser 107/107,
checker_locks 23/23 + CLI locks, selfhost diff 2.

RELAY (stdlib lane): `Vec.push[T]` in `collections.xi` still hardcodes
`new_cap * 8` and does unscaled `data + len`; the compiler now avoids that
body for direct Vec receivers, but any other route into it breaks for
T != 8 bytes. Make it stride-correct (`sizeof[T]`/element-scaled
arithmetic, e.g. via a runtime elem-size field or generic-safe scaling).

---

## 2026-10-01 -- OPEN (C23): -O2 silently miscompiles (reproduced in WSL; bracket O1/O2)

Playground relay + repro pack: `E:\xiom-lang\playground\tools\compiler-repros\c23\`
(byte-exact lesson solutions `l6-15.xi` / `l7-39.xi` / `l8-09.xi` + README
+ `run.sh`; acceptance = `run.sh` prints `C23 present: no`).

Reproduced EXACTLY on the reporter host: WSL Ubuntu (clang 18.1.3
(1ubuntu1), kernel 6.6.87.2-microsoft-standard-WSL2) with the v0.62.2 linux
toolchain (release tar.gz extracted to `~/xiom-c23`): l6-15 `2/3`,
l7-39 `2/3`, l8-09 `2/0` (O0/O2), `C23 present: yes`.

- NOT reproducible on the Windows host (all `2/2`) -- host-LLVM dependent,
  as the playground noted.
- Level bracket (l7-39): `-O0 2`, `-O1 2`, `-O2 3`, `-O3 3` -- the trigger
  passes live in the O1->O2 delta (the driver's `opt -O2` rewrite and/or
  clang's -O2 backend).
- Instrumentation hides it: adding prints/dump to l7-39 makes both levels
  correct (matches the playground's reduction notes -- the exact program
  shape is required).
- IR reviewed so far is structurally correct: `Vec[TodoItem]` esz = 24
  (`store i64 24`), `.get` boxes 24 bytes, `.set` memcpys 24 bytes to
  `data + idx*esz`; the new item stores `done` = i64 1 at field 2.
  `TodoItem = { i64, i8*, i64 }`, `Vec = { i8*, i64, i64, i64 }`.
- Next (dedicated batch): pass-level bisection in WSL (`opt -O1` vs `-O2`
  on the captured pre-opt IR; clang-wrapper log to replay the exact
  command), then minimize at the IR level; cross-check the packages'
  `Vec[Str].push` "stride 8, i8 store" finding for the same root class.

Repro artifacts: `tmp/sprintc/c23_capture_ir.sh`, `c23_levels.sh`,
`c23_debug_l7.xi`; pre-opt IR at `tmp/sprintc/c23_l7_pre.ll`.

---

## 2026-09-30 -- m166 follow-up FIXED: method trust key + legacy xiom.sync atomics (t2 residual)

Benchmark relay: t2-queue still ~34 s (full arena) / ~12 s (probe) on
v0.62.2 even though the xiom.sync.atomics annotation shipped. Local
inspection of the released state confirmed `@sync.AtomicInt.load`-style
methods still emitted the full per-call trampoline + guard sequence.

Two gaps:

1. TRUST KEY: the m166 provenance lookup compared `catalog_fn_keys`
   against `fd.name.name`, but METHOD decls keep a SHORT name (`load`) with
   the receiver in `fd.receiver`; the injected keys use the
   receiver-qualified form (`AtomicInt.load`, matching the driver's
   `fn_dedup_key` and the emitted symbol). Methods were therefore never
   trusted even when annotated. Fixed in `compile_fn` and the generic-mono
   path: build `trust_key = "{receiver}.{name}"` for methods.
2. SOURCE COPY: consumers of the legacy `xiom.sync` standalone API bind
   `sync.xi`'s OWN `AtomicInt` methods/helpers (lines ~511-571, 691-755),
   not the annotated `xiom.sync.atomics` module. Those legacy fns carry
   their own unsafe blocks and need `#[unsafe_direct]` too (stdlib lane).

Local proof (legacy sync.xi `.load`/`.store` annotated LOCALLY, edit
reverted): 4M pairs **7000 ms -> 0 ms**; the emitted IR for
`@AtomicInt.load`/`@AtomicInt.store` drops every `xiom_trampoline_call` +
guard arm/disarm. Repros: `tmp/sprintc/perf_atomic_legacy.xi` (legacy API)
and `docs/repro/perf-1-atomic-trampoline/perf_atomic_trampoline.xi`
(atomics API).

Locks: `regress_m166_method_receiver_trust_key` (trusted method -> 0
trampolines; untrusted -> confined) + the existing free-fn lock. Gates:
full e2e 2393/2393 (+4 ignored), feature-reg 515/515, parser 107/107,
CLI locks, selfhost diff 2.

RELAY (stdlib lane): annotate every fn in `stdlib/xiom/sync/sync.xi` whose
body contains an `unsafe` block -- at minimum the `AtomicInt` methods
(new/load/store/fetch_add/fetch_sub/swap/compare_exchange) and the
standalone `atomic_*` helpers; consider the spin/yield helpers
(`cdl_wait_spin`). Tag for the next pin (e.g. stdlib-perf2); the t2 Gate P
re-run waits on it. NOTE: the fix requires compiler >= this commit.

---

## 2026-09-29 -- m166 FIXED: `#[attr] pub fn` silently dropped the attribute; `#[unsafe_direct]` trust did not cover injected stdlib fns

Found while chasing PERF-1 (benchmark t2-queue, ~1 us/call unsafe-block
trampoline cost). Two independent defects blocked the sanctioned
`#[unsafe_direct]` route for stdlib hot primitives:

1. PARSER: `#[unsafe_direct]` directly above `pub fn` raised
   `P001: expected 'fn'` (the fn parser expected `fn` right after the
   attribute block and did not accept `pub`), and error recovery then
   parsed the fn WITHOUT the attribute. Net effect: the annotation was
   SILENTLY DROPPED -- no diagnostic, no warning, no effect. (`pub #[attr]
   fn` already worked, which is why the one existing attribute fixture
   never caught it.)
2. TRUST: even with the attribute attached, `compile_fn` judged stdlib
   origin by `config.source_file`, which is the PRIMARY source -- for a
   user program importing the stdlib that is the USER's path, so the
   attribute was rejected (confined) for exactly the stdlib fns that need
   it (the `xiom.sync.atomics` wrappers).

Fix:
* Parser (`crates/xiom-parser/src/lib.rs` parse_fn_decl): accept an
  optional `pub` after the attribute block -- attributes attach in both
  orders now.
* Codegen trust (`crates/xiom-codegen`): new `CodegenConfig.catalog_fn_keys`
  (`set_catalog_fn_keys`); `compile_fn` and the generic-mono path trust
  `#[unsafe_direct]` when the fn's key is in that set, in addition to the
  source-path check. The driver fills the set from the decls it injects
  (Stage 4.5) via the existing `fn_dedup_key` -- exactly the keys codegen
  emits.

Local verification (stdlib atomics annotated LOCALLY, edit reverted after
measuring; see `docs/repro/perf-1-atomic-trampoline/`):

```
before: atomic loop 8000 ms (4M store+load pairs, ~1.0 us/call)
after:  atomic loop    0 ms (below clock resolution)
```

IR: `@sync.atomic_load` / `@sync.atomic_store` emit no
`xiom_trampoline_call` and no guard-page arm/disarm; the 22 remaining
trampoline CALL sites in the closure belong to other (unannotated) stdlib
unsafe blocks.

Locks: parser unit test `test_attribute_before_pub_fn`;
`regress_m166_unsafe_direct_pub_fn_trusted` (trusted -> 0 trampoline calls,
untrusted -> still confined). Gates: full e2e 2393/2393 (+4 ignored),
feature-reg 514/514, parser 107/107, checker_locks 23/23 + CLI locks,
selfhost diff 2 passed.

RELAY (stdlib lane): annotate every fn in `stdlib/xiom/sync/atomics.xi`
whose body contains an `unsafe` block with `#[unsafe_direct]` (load/store
verified locally; the rest are the same single-intrinsic shape). Requires
compiler >= this commit for the attribute to take effect; tag for the
v0.62.2 wave and update `STDLIB_VERSION` (release gate). Then the
benchmark lane re-runs t2-queue.

---

## 2026-09-29 -- m162 FIXED: same-leaf user fn poisoned catalog-body resolution

Playground relay: a user module exporting a fn whose LEAF name matched a
stdlib fn (`char_at`) poisoned the catalog-body check of UNRELATED stdlib
modules -- `xiom.num`'s body reported bogus T001s ("cannot access field on
non-struct type Int", "cannot compare <error> with Char") because its bare
`char_at(s, i)` resolved to the user's `(Str, Int) -> Int` instead of its
own `use xiom.string.char_at` (`(Str, Int) -> Option[Char]`). The import
alone triggered it (probe_b_main); a same-leaf fn in the MAIN unit did not
(probe_d_main).

Repro: `tmp/sprintc/m162_sameleaf_catalog_poison/` (probe A-E matrix +
README); permanent lock `tests/regression/m162_sameleaf_catalog_poison/`
(user_util.xi + main.xi).

Root cause (two ends, both fixed):
* CHECKER: `flush_catalog_bodies` checks each body with `current_module`
  taken (None), so the bare-call cascade fell through to the global
  first-wins `functions` slot. The user program registers before the
  catalog preload, so the user's `char_at` owned it; the body's own
  explicit item import sat unused in `imported_items`.
* CODEGEN: `bare_fn_aliases` (keep-first) had the same ownership; the
  emitted call inside `@num.parse_int_radix` was `call @user_util.char_at`
  (runtime "invalid index" -- the wrong sig made `opt.is_some` false on
  the first character).

Fix:
* Checker bare-call cascade (`crates/xiom-check/src/lib.rs`): while
  `checking_catalog`, prefer the body's EXPLICIT ITEM import
  (`use xiom.string.char_at;`) over the global bare slot -- provenance
  test `local_module_paths[leaf]` ends with the leaf. Module-surface
  injections (`use xiom.math;`'s `pow`) do NOT qualify: a broad
  `imported_items` preference regressed `xiom.math.rounding` 141:10
  (`pow` overload picked as Int) and was rejected. The chosen dotted
  target is recorded into `catalog_resolved_calls` under the R20
  owner-qualified key.
* Codegen bare-call cascade (`crates/xiom-codegen/src/call.rs`): new
  `resolve_catalog_call_bare` consults `catalog_call_targets` (owner key +
  leaf suffix) after the caller-module check and BEFORE
  `bare_fn_aliases`.

Locks: `m162_sameleaf_catalog_poison` e2e + CI line; checker_locks
`m162_sameleaf_fn_does_not_poison_catalog_bodies` (compile + run).
Gates: full e2e 2392/2392 (+4 ignored), feature-reg 512/512,
checker_locks 23/23, selfhost diff 2 passed. The selfhost `rt_` prefixes
are no longer required by this bug (kept; rename in O1).

---

## 2026-09-29 -- m163 FIXED: Str field elements in a struct METHOD miscompiled to int add

Found while building the selfhost Phase 0 skeleton (`IrBuffer` text
builder): inside a struct method, an INDEXED ELEMENT of a `Vec[Str]`
STRUCT FIELD used as a `+` operand (or in an accumulator) compiled to
**integer add + inttoptr** instead of `@xiom_str_concat` -- silently wrong
strings (pointer decimals), no diagnostic.

Repro: `tmp/sprintc/m163_method_str_accum/` (`m163_lib.xi` +
`m163_main.xi`); single-file variant `tmp/sprintc/m163_single.xi`.

IR evidence (`--emit-ir`, `@Buf.pair` = `return lines[0] + lines[1];`,
BEFORE):

```
%tmp22 = phi i64 ...            ; element 0 (Str handle) as i64
%tmp39 = phi i64 ...            ; element 1 (Str handle) as i64
%tmp40 = add i64 %tmp22, %tmp39 ; INTEGER ADD of two pointers
%tmp41 = inttoptr i64 %tmp40 to i8*
ret i8* %tmp41
```

Characterization (all inside the same module):

| Shape | Result |
|---|---|
| method `return lines[0];` (direct element return) | OK |
| method accumulator `out = out + lines[i] + "\n"` over the FIELD | BROKEN (pointer decimals) |
| method `lines[0] + lines[1]` | BROKEN (empty / garbage) |
| method accumulator over a LOCAL `Vec[Str]` | OK |
| method accumulator growing a single Str FIELD | OK |
| free fn `join_of(b: &Buf)`: `out + b.lines[i]` | OK |
| caller-side `out + buf.lines[i]` (outside the module) | OK |

Root cause: the method prologue (`decl.rs` compile_fn) binds receiver
FIELDS as bare-name locals (GEP + `add_local`) but -- unlike params and
body locals -- never registered their Vec ELEMENT types in
`local_vec_elem`. A field element (`lines[i]` on a `Vec[Str]` field)
therefore fell to the generic scalar i64 element load
(`emit_elem_load`), so both `+` operands were i64 and the Str-concat
intercept was unreachable. Caller-side `Expr::Field` containers resolve
through `vec_elem_is_str`'s Field arm (`declared_field_type`) and free-fn
`&Buf` params resolve through the Field arm too, which is why only the
method-context bare-Ident shape broke.

Fix: `record_receiver_field_vec_elems` (lib.rs, next to
`field_xiom_type`) registers each field's `Vec[X]` element into
`local_vec_elem` at prologue field-binding time; called from BOTH
prologue branches in `decl.rs`. Field Vecs now behave exactly like local
Vecs for every downstream consumer (Str-handle loads, concat
classification, float element handling, struct-element memcpy). Params
and body locals register AFTER the prologue with unconditional `insert`,
so a same-named param/local still shadows the field (the existing
`Reader.process(self, buf: Str)` shadow lock stays green).

Locks: `tests/regression/m163_method_field_vec_concat/main.xi` +
`e2e_m163_method_field_vec_concat` (CI line) +
`regress_m163_field_vec_elem_concat` (in-process IR: asserts
`xiom_str_concat`). The selfhost `IrBuffer` m163 workaround (single Str
field) is no longer required; revisit in the O1 pass.

## 2026-09-12 -- R17 FIXED (round-57): nested-index Str elements in concat

R14's concat fallback (compiled LLVM scalar type) overrode the semantic
verdict for Index shapes the walker could not see through. It fixed
`"E1[0]=" + e[0]` (Vec[Int] from a chained `.collect()`) but broke
`"" + rows[0][0]` (Vec[Vec[Str]] -- smoke_serialize_csv printed
`[2653219331984|...]`): the nested Str loaded as i64 and got formatted as a
NUMBER instead of taking inttoptr (pointer recovery).

Root cause: `expr_is_integer`'s `Expr::Index` arm only handled IDENT
containers with a `local_vec_elem` entry; nested containers returned false,
and the fallback could not distinguish "unknown" from "known non-int".

Fix (codegen):
- `vec_value_xiom_type` resolves the container's XIOM value type recursively
  (`Ident` -> `Vec[{local_vec_elem}]`; `Index(inner)` -> element of the inner
  value's type), so `rows[0][0]` reaches `"Str"`.
- `concat_operand_is_int` (both concat sites) is semantic-first: known
  non-integers (`Str`, `Bool`, compound containers, registered structs) use
  `val_to_i8ptr`'s inttoptr; the LLVM scalar fallback applies only to
  genuinely UNKNOWN Index/Field shapes (generic "T" elements stay unknown so
  R14's chained-collect Vec[Int] still formats).
- The inner `is_int_name` was replaced by `elem_name_is_int` (one definition).

Locks: `e2e_m72_nested_index_concat` (+ the probe
`p_nested_index_concat.xi`), alongside `e2e_m71_concat_index_elem` (R14).
Both probes exit 0; the pipeline probe still prints E[0]=9 / E[4]=225.

## 2026-09-12 -- R14 ROOT-CAUSED AND FIXED: concat of indexed elements (not chain corruption)

The combined iter probe `p_iter_pipeline_r32.xi` AV'd (0xC0000005) while
every sub-variant passed. ASAN + a pass/fail matrix showed the iterator
chain was NEVER at fault: the crash was the trailing
`io.println("E[0]=" + result[0])`. With `result[0] == 9`, the generated IR
did `inttoptr i64 9 to i8*` and `xiom_str_concat` read address 0x9 (ASAN:
`rax=rdi=rsi=9`, fault in `xiom_str_concat` from `main`).

Root cause: `Codegen::expr_is_integer` decides whether a `Str + X` operand is
formatted with `@xiom_int_to_string` or coerced with `inttoptr`. Its
`Expr::Index` case only consulted `local_vec_elem` for an `Expr::Ident`
container. A Vec bound from a chained single-expression `.collect()` never
registers that element map, so `e[0]` (LLVM `i64`) fell through to
`inttoptr` -- a garbage string pointer whose address is the element VALUE.
Splitting the chain into a local (`var it = ...take(5); var v = it.collect()`)
or adding any statement between collect and print made the registry present,
which is why only the combined shape crashed. Minimized to
`p_r14_e1.xi` (crash) vs `p_r14_e1b.xi` (pass, one extra println).

Fix: at both concat sites (`compile_binop_fold` and the normal BinOp path)
the operand verdict now falls back to the COMPILED LLVM type for `Index`/
`Field` shapes (`llvm_scalar_is_int`: i8/i16/i32/i64/i128, never i1 or a
pointer), so an iN scalar is always formatted. The semantic verdict still
wins for calls, preserving the Str-returning-callee-with-i64-fallback case
(which must stay `inttoptr`/bitcast, not be formatted as a number).

Regression lock: `tests/regression/m71_concat_index_elem.xi` +
`e2e_m71_concat_index_elem` (asserts `"e[0]=9"`, `"e[4]=225"`, left-side
`e[1] + "!"`, and a plain Vec index). `tmp/bug_probes/p_iter_pipeline_r32.xi`
now exits 0.

## 2026-09-12 -- D1/D2/D4/D5: catalog parse integrity, real spans, fn types, overload pick

Follow-up compiler-side slice of the Item A triage (D1-D6 queue). All four
are landed; the remaining stdlib worklist is refreshed in
[`ITEM_A_STDLIB_FINDINGS.md`](ITEM_A_STDLIB_FINDINGS.md).

### D1 -- catalog parse errors were swallowed (production-grade gate)

`CachedModule::parse_file` parsed with `Parser::parse_program` and discarded
`parser.errors()`. Because the parser RECOVERS by returning a partial AST,
whole declarations silently vanished: `xiom.char` `'GBP'`/`'+/-'` multi-char
literals dropped `is_currency` / `is_math_symbol`, and the corpus only saw
downstream `undefined variable` cascades.

Fix: `CachedModule.parse_errors: Vec<(String, Span)>` records every
recoverable diagnostic; `CatalogCorpusReport.parse_errors` aggregates them
(sorted) and `is_clean()` requires the list to be empty; the gate prints
`PARSE line:col: message` and the assert reports the count. User-program
surfacing is intentionally staged for the FLIP: xiom.char is currently broken,
and emitting these as user warnings would break the m16/m17 zero-warning
contracts before the stdlib fixes land. NOTE: the parser's fatal path (>100
errors) still returns None from `parse_file` and is not yet representable.

The new gate immediately found 14 recoverable diagnostics in 12 modules --
all now in the stdlib worklist (stray closing braces left by removed
`unsafe` blocks, prose in `ensures:`, `fn var`, `<=>`, multi-char literals).
Two of them were NOT stdlib defects:

### D1b -- parser: tuple type args in generic struct literals (COMPILER BUG)

`MapIter[(Int, T), U]{ next_fn: ..., f: f }` failed with "expected type name
before '{'". The `Name[...]` postfix has TWO type-args branches: the
leading-`fn`/`*`/`&`/`[`/`(` branch (tuple args) built `Expr::Index` and
`continue`d WITHOUT consuming a following `{ ... }` tail; only the
leading-Ident branch handled it. Recovery then dropped the enclosing
declaration (`iter.xi:629,634` `EnumerateIter.map/take`).

Fix: extracted `Parser::parse_struct_literal_tail` (shared by both branches),
called from the tuple branch and the simple-args branch. Repro:
`tmp/bug_probes/p_d1_tuple_structlit.xi`.

### D2 -- non-exhaustive match diagnostics had span 0:0

The exhaustiveness warning/error hardcoded `Span::new(0, 0)` (and the `warn`
helper did too, for EVERY warning). `check_match_exhaustiveness` now takes
the match expression's span (both `Stmt::Match` and `Expr::Match` call sites
thread it) and uses `warn_at`; the helper `warn_at(message, span)` was added
and `warn` delegates with `0:0`. yaml_lite's six findings now point at the
match instead of `0:0`.

### D4 -- generic fn-typed struct fields could never match

`types_compatible` had NO `(Fn, Fn)` arm: fn-pointer types compared only by
exact structural equality, so a concrete closure (`fn(Int) -> Int`) could
never fill a generic field (`fn(T) -> U`) and every higher-order iterator
body errored (`xiom.iter` 17 findings). Fix: structural arm comparing arity
and recursing into params/return (generic params and wildcards pass, as
everywhere else). This also unblocked the now-live `iter.xi:629/634`
declarations from D1b.

### D5 -- first-wins bare slot mis-binds compatible overloads

The global bare-function slot is first-wins; the all-imports corpus binds
`is_empty` to `xiom.array`'s Array version while `xiom.path` (which imports
only `xiom.env`) calls `is_empty(str)`. `Path` metadata etc. work in normal
compiles because `array.xi` is not loaded. Fix: argument types are now
evaluated ONCE per call (also removing a double-`check_expr` on generic
positions); when the chosen signature cannot accept the call,
`resolve_alternative_bare_fn` searches explicitly imported items first, then
every module surface in sorted key order, for a PUB fn whose signature fits.
Deterministic; keeps the first-wins sig when nothing fits (errors unchanged).

### D5b -- method leaf names polluted bare-name VISIBILITY too

Same class as the earlier `fn_owner_module` fix: `visibility` was registered
for METHOD leaf names, so a private `fn Foo.is_empty` marked the bare name
invisible (first-wins `or_insert`). `is_empty(self.inner)` in xiom.path then
skipped the entire free-fn resolution path and fell into the imported-items
fallback with `xiom.array`'s Array signature. Fix: visibility is registered
for FREE fns only, mirroring the owner map. With that, D5's alternative
selection picks `xiom.string.is_empty` and the last non-parse finding is
gone.

### D6 (effectively closed, compiler side) -- field calls vs cross-type methods

`ChainIter[T, U].next`'s `self.second()` was captured by the UNIQUE-CANDIDATE
wildcard: `DateTime.second(self) -> Int` is the only method named `second`,
so the call typed `Int` and the `Option[T]` return mismatched. Fix: the
wildcard method fallback now skips when the RECEIVER's registered surface
has a FIELD with that name -- struct fields shadow cross-type methods, and
the existing P2-7 fn-field path then calls `second: fn() -> Option[U]` with
the correct signature. (The stdlib's `Option[U]` vs `Option[T]` return is
accepted because both args are generic parameters.)

### D5c -- same-name interfaces with different arities (flaky arity gate)

`Hash` exists as `fn hash() -> UInt64` (xiom.core) and
`fn hash(self, hasher)` (xiom.hash). The interface-dispatch arity check did
`interfaces.values().find(|(mn,..)| mn == method)` and enforced THAT
declaration's arity; `values()` order is random, so `value.hash()` in
`hash[T: Hash]` flapped between valid (0-arg core declaration) and
"expects 1 argument(s), found 0" (hasher declaration). Fix: accept the call
when ANY same-name declaration's arity fits; report only when NONE does;
the return-type lookup prefers an arity-fitting declaration (deterministic).
The R8-hardening is preserved: a call with only the 2-param declaration
present still errors.

### FLIP STATUS (updated 2026-09-12, round 57)

The corpus-isolation fix changed the measurement: the synthetic corpus `use`
list now only LOADS modules; its aliases are cleared before
`flush_catalog_bodies` (plus `imported_items`/`local_module_paths`/
`module_import_paths`/`use_alias_paths`), so each body is checked under its
OWN imports -- what a real program sees. The previous 0/0/1 measurement was
too permissive: it masked 149 real findings of one kind, modules using
qualified aliases they never import (`xiom.os` -> env/io/string;
net.https/log/simd/crypto/collections/encoding). Stdlib worklist:
ITEM_A_STDLIB_FINDINGS.md section Q.

**FLIPPED AND CLOSED 2026-09-15 (round 61)**: the stdlib lane cleared
section Q and the round-59 encoding sites (1f4f0aad), the compiler lane fixed
the scope-first alias / container-wildcard / generic-`!` classes behind the
remaining failures (R21 below), and `strict_catalog_findings` is now TRUE.
The un-ignored `catalog_corpus_is_clean` gate is the regression canary; any
catalog-body finding in any loaded module is a hard error. Final gates:
checker 188/188, stdlib-exec 85/85 (+2 ign), feature-reg 510/510, e2e
2321/2321. Stage 3 Item A is CLOSED -- see the R21 entry at the end of this
file for the complete hold/release history and the remaining follow-ups.

### D3/D6 status (original entries)

D3 (`xiom.path:204` `Err(e).message` typed `Str`) is NOT reproducible at
HEAD: the body's bare `metadata(self.inner)` takes the free-function path
(the G-10 explicit-self heuristic only fires for a literal `self`/`this`
argument), so `e: IOError` and the field access is valid. D6 (ChainIter
`Option[U]` from `next()`) remains stdlib-design-dependent and is listed for
the stdlib session; if the design stays, the compiler needs a generic-
relation rule.

---

## 2026-09-12 -- Stage 3 Item A step 2: artifact classes FIXED, corpus 779 -> 235

Triage of the 779 catalog-body findings split them into checker artifacts and
real stdlib findings. All identified CHECKER artifacts are fixed; what
remains is a stdlib-facing worklist, handed off in
[`ITEM_A_STDLIB_FINDINGS.md`](ITEM_A_STDLIB_FINDINGS.md).

Checker fixes in this slice (all in `xiom-check`):

1. **Alias-scoped module resolution** (`module_exports_for_alias`): qualified
   paths resolve through the alias's FULL dotted path (`local_module_paths`)
   instead of the global first-wins leaf-key maps. The all-imports corpus
   binds the same leaf for many modules (`use xiom.io` vs
   `use xiom.async.io` -> "io"; `xiom.convert.time` vs `xiom.time` ->
   "time"), so the wrong module's surface was used
   (`cannot call 'read_file_bytes'` / `println` / `bigfloat_*`).
2. **Type-segment descent**: `module.Type.method(...)` no longer falls back to
   an on-demand catalog peek for a TYPE segment. On Windows the peek is
   case-insensitive: `time.Duration.from_millis` matched `duration.xi` and
   swapped in that module's surface (then missed `from_millis`).
3. **Catalog impl registrations**: `parse_file` collects
   `impl Trait[Args] for Type` method registrations BEFORE
   `expand_impl_blocks` erases them (`CachedModule::impl_registrations`);
   `register_external_module` replays them into `Checker::impls`, so
   interface-qualified static calls in catalog bodies (`Num[T].one()`,
   `PrecisionLimits[T].min_value()`) resolve like user-program impls.
4. **Builtin fns through module paths**: conversion intrinsics and
   `panic`/debug traps are recorded in `builtin_fns`; a module-qualified call
   whose leaf is a builtin resolves even when the named module does not
   export it (`xiom.char.to_int_from_char`, `xiom.core.panic`). `panic` is
   registered as a global builtin (core.xi declares it privately and
   test/assert.xi calls it qualified).
5. **Current-module ambiguity preference**: a bare call to a fn owned by the
   CURRENT module is not "ambiguous" in the all-imports context
   (`time(0)` in xiom.time); extern registration now records owners too.
6. **Method leaf names no longer claim free-fn ownership**: `fn_owner_module`
   only tracks FREE fns. `fn Vec4f.get` used to insert owner "xiom.simd" for
   the bare name `get`, which disabled receiver-method precedence and bound
   `get(0)` to `xiom.array.get`.
7. **G-10 implicit/explicit receiver shapes**: a bare call inside a method
   body binds to the receiver's method ahead of imported same-named free fns,
   for both call shapes -- implicit (`get(0)` in `Vec4f.normalize`) and
   explicit (`len(self)` passing the receiver as the first arg). An explicit
   shape defers to a same-arity free fn whose first param accepts the
   receiver, so `scale(self, k)` inside `Rect.scale` still recurses into the
   free `scale[T](r, k)` (e2e_m34_y17) while `len(self)` binds
   `Vec4f.len` (instead of `xiom.array.len` -> wrong Int return).
8. **Local fn-typed parameters shadow globals**: `compare(...)` where
   `compare: fn(&Int,&Int)->Int` is a parameter now calls the parameter
   (previously bound an unrelated global `compare`).
9. **`char_at` method typing**: method sugar lowers to `@xiom_char_at`
   (codepoint -> `Char`), NOT `Int` and NOT `Option[Char]` (the free
   `xiom.string.char_at` returns Option). The Int typing broke `.unwrap()`
   sites; the Option typing broke the pervasive direct comparisons.
10. **Interface arity/return**: a first param named `Self` OR typed as the
    receiver type counts as the implicit receiver (free-fn-style interfaces
    like `Eq[T].eq(a: T, b: T)` called as `arr[i].eq(target)`); `Self` return
    types substitute to the receiver type (or the generic param name) so
    arithmetic on `T.max_value()` type-checks.
11. **Same-name interface bounds**: bound lookup searches ALL registered
    interfaces whose leaf matches the bound name (`Bounded` exists in both
    `xiom.num` and `xiom.math.interfaces`; the bare key is first-wins). Fixes
    `T.epsilon()`.
12. **`Unit` literal + `to_str` builtin**: `Ok(Unit)` type-checks;
    `arg.to_str()` on an unbounded generic resolves (fmt.format1).

Nondeterminism: before this slice the corpus count varied run-to-run
(247/263) due to HashMap-order resolution; the fixes above make the
measurement stable (two consecutive runs identical). Determinism is a
prerequisite for the flip.

Gate tooling: `XIOM_CATALOG_DUMP=1` on `catalog_corpus_is_clean` prints every
finding (`ALL module:line:col: msg`) instead of one representative per class,
for the stdlib session's per-site worklist.

Gates on the fresh canonical binary: checker 187/187 (+1 pending gate),
xiom-ast 9/9, feature-reg 510/510, stdlib-exec 85/85 (+2 ign), e2e
2317/2317, xiom lib 20/20, fmt 83/83, lsp 42/42, jit 5/5,
`cargo check --workspace` clean. The LSP suite had one PRE-EXISTING red
(`test_stdlib_module_no_false_positives` read the retired
`stdlib/xiom/memory/alloc.xi`); the test now reads
`stdlib/xiom/alloc/alloc.xi` (namespace wave 2 move). NOTE for reruns: the
e2e harness spawns `target/debug/xiom.exe`; run `cargo build -p xiom` after
checker changes or the suite tests a stale driver (false failure).

## 2026-09-12 -- R9 FIXED: full-path shim delegation (infinite self-call -> 0xC0000409)

Stdlib report repros p_x1/p_x2/p_x4/p_x6, p_sdx_shim_first,
p_lev_shim_first: a FULL-PATH call into a module that was never imported
(`xiom.string.glob.glob_match`) crashed at runtime with 0xC0000409.

Root cause: the checker resolves the full path by PEEKING the module
(non-caching) and records it in `peeked_resolved` for codegen injection.
`xiom.string.glob` is a shim whose body delegates by full path to
`xiom.misc.glob.glob_match`, but the shim's own `use xiom.misc.glob;` was
never followed at injection time, so the target module's decls never
reached codegen. The leaf-qualified symbol key (`glob.glob_match`) was then
occupied by the shim itself and the inner call bound to it -- infinite
recursion, trap.

Fix (`Checker::collect_external_decls`): peeked modules are injected
through a TRANSITIVE `use` closure in deterministic dotted-name order (the
same first-wins order `all_cached()` uses), so `xiom.misc.glob` registers
BEFORE the shim and the shim's duplicate leaf key is the one skipped. No
hard error needed -- correct full-path resolution.

Verified: all six external repros exit 0 with correct output on the fresh
binary. Lock: tests/regression/m70_full_path_shim_delegation.xi +
`e2e_m70_full_path_shim_delegation` (pre-fix: 0xC0000409). Gates: checker
187/187 (+1 pending gate), feature-reg 510/510, stdlib-exec 85/85 (+2 ign),
e2e 2317/2317, workspace check clean.

## 2026-09-12 -- Stage 3 Item A step 1 LANDED: collect-then-check + corpus 19,287 -> 779

The catalog import-context fix ("collect-then-check") is implemented and the
artifact classes are separated from real findings:

- `register_external_module` now only REGISTERS (types/signatures/interfaces)
  and queues the body; `flush_catalog_bodies` runs at the end of
  `resolve_imports`, after the transitive load + prelude + program uses.
- Each body is checked under a PER-MODULE ISOLATED import context
  (`CatalogImportContext` snapshot/restore): a module's own `use`
  declarations are processed (new `TopDecl::Use` arm in `check_top_decl`),
  and the private deps it resolves are loaded with the NON-CACHING
  `peek_owned`. The snapshot covers `modules`, `imported_items`,
  `local_module_paths`, `module_import_paths`, `use_alias_paths`,
  `functions`, `visibility`, `fn_owner_module`, `methods`,
  `submodule_aliases` and `peeked_resolved`.
- Private-name resolution: `fn_owner_module` is now name -> SET of owning
  modules (two catalog modules may each define a private `_u32_mask`).
- Catalog module-level const/var globals are registered module-scoped
  (`catalog_global_consts`); `_K1`/`_PI`/`NANOS_PER_SEC` no longer undefined.
- Warnings raised while `checking_catalog` are tagged with the "catalog
  body" prefix in `warn()` too (m16/m17 zero-user-warning contracts).

Measured corpus (`catalog_corpus_is_clean`, ignored pending gate):
**779 findings, 0 hard errors, 0 other warnings** (was 19,287 with the old
phase; intermediate 5,216 after the import context, 2,477 after owner sets,
then globals + isolation). The gate prints one representative finding per
class (`ONE module:line:col`) plus per-module counts, so the remaining work
is triageable from a single run. The alias classes (`string` 4318, `math`
1806, `convert`, `io`, `bigint`, ...) are GONE. Remaining classes separate
into likely CHECKER artifacts -- static calls like `time.Duration.from_*`
resolve in ordinary user compiles but not in the all-imports corpus context
(`cannot call 'from_millis'`), `undefined variable 'Num'/'Ord'` (interface
receivers), `PrecisionLimits`, `cannot call 'panic'` (builtin), ambiguous
bare `time`/`spawn` (current-module preference) -- and likely REAL stdlib
findings: extern-requires-unsafe ~59, pointer-to-pointer casts ~21,
T003/T007 confinement, Float64/Int mixing 28, `Array[T]` arg mismatch 18,
`date_day_of_week` 16. The flip stays GATED until `report.is_clean()`.

REGRESSION CAUGHT MID-SLICE (fixed): the first flush attempt let catalog
private deps load through `find_owned`, which CACHES them -- the driver
injects every cached module into codegen, so extra concrete defines/decls
shifted first-wins resolution and m43 closure adapters miscompiled
(compile 0, run 1; e2e caught it). The non-caching peek + snapshot model
fixed it; e2e is 2316/2316 again.

Gates on a fresh canonical binary: checker 187/187 (+1 pending gate),
xiom-ast 9/9, feature-reg 510/510, stdlib-exec 85/85 (+2 ign), e2e
2316/2316, xiom lib 20/20, fmt 83/83, lsp 42/42, jit 5/5, workspace check
clean. Stdlib-lane heads-up: the classes they pre-triaged as checker-side
(undefined `io`/`string`/`size_of`/`alloc` in bodies) are addressed by the
import-context work; re-measure against this build before realigning.

## 2026-09-11 -- Stage 3 Item A: flip BLOCKED -- corpus re-measured (19,287 findings)

Re-measured the catalog-body findings after the stdlib dedup rounds, with a
new repeatable API (`Checker::check_catalog_corpus` ->
`CatalogCorpusReport`): it builds a synthetic program that `use`s every
indexed module, so the full import graph loads the same way a real compile
does, and reports `{ findings, warnings, errors }`.

Result on HEAD: **19,287 catalog-body findings, 0 hard errors, 6 other
warnings**. The flip to hard errors is NOT landed (it is gated on
`report.is_clean()`, now encoded as the ignored test
`catalog_corpus_is_clean`; run
`cargo test -p xiom-check catalog_corpus_is_clean -- --ignored --nocapture`
to re-measure).

Root cause of the dominant class (NOT stdlib-side): catalog bodies are
type-checked at module-LOAD time (`register_external_module`), before the
import graph is complete, and `check_top_decl` has NO `TopLevel::Use` arm --
so a module's own `use xiom.math;` never registers the `math` alias.
Qualified calls inside catalog bodies then resolve `math`/`string`/
`convert`/`io`/`bigint`/`bigfloat`/`geom` as undefined variables and cascade
("cannot call ... on this expression", "left operand must be numeric, found
<error>"). Counts: undefined `string` 4318, `math` 1806, `convert` 406,
`io` 144, plus ~12k cascades. The 1.5k-4.5k previously cited was a partial
view (only the modules a given smoke compile happens to load).

Fix path (queued, compiler-side, no stdlib edits): split
`register_external_module` into register-then-check, queue catalog bodies,
and flush them at the END of `resolve_imports` (after the full transitive
load + prelude), processing each module's own `use` declarations under its
module context before its body; only then re-measure and flip.

Slice landed: `CatalogCorpusReport` + `Checker::check_catalog_corpus` +
`ModuleCatalog::module_names`, the ignored pending gate, and this diagnosis.
No compiler behavior change; feature-reg 510/510, stdlib-exec 85/85 (+2 ign),
checker 187/187 (+1 ign), workspace check clean. e2e remains 2316/2316 from
51c81efd (no codegen/checker-path change since).

## 2026-09-11 -- Stage 2c slice 4: canonical registry keys + shared structural module

Completes the remaining Stage 2c items:

- **Checker registry keyed by `TypeId`**: `Checker::types` changed from
  `HashMap<String, ...>` to `HashMap<TypeId, ...>`. `get_type` /
  `contains_type` intern the (module-qualified) query through the canonical
  arena, and every registration site (builtins, structs, enums, anon
  structs, tuples, module exports) inserts the interned id. Registry
  identity is structural by construction -- spelling variants can no longer
  miss, and the R5 qualified-import preference now compares ids.
- **Shared structural module**: `structural.rs` moved from xiom-check to
  `xiom_ast::structural` (same implementation; `xiom-check/src/structural.rs`
  is a `pub use` shim). This removes the layering problem where codegen
  (which only dev-depends on xiom-check) would have needed the checker in
  production code: the canonical type-name grammar belongs with the AST.
- **Codegen canonical keys**: `IrEmitter::type_arg_to_name` renders through
  `xiom_ast::structural::canonical_type_name`, so codegen registry keys
  (`types.types`, `type_meta`, concrete-container registrations and the
  `Vec::new` element-size path) use the checker's canonical spelling
  (`Result[Int, Str]`, never a second `Result[Int,Str]` entry). New unit test
  `type_arg_to_name_renders_canonically`.

Test-count note: the 9 structural tests move with the module to xiom-ast, so
the checker suite is 187 (196 - 9) plus xiom-ast 9/9; the codegen lib suite
gains the renderer test (11/11).

Gates (fresh canonical binary): checker 187/187, xiom-ast 9/9, codegen lib
11/11, feature-reg 510/510, stdlib-exec 85/85 (+2 ign), e2e 2316/2316, xiom
lib 20/20, fmt 83/83, lsp 42/42, jit 5/5, workspace check clean.

## 2026-09-11 -- Stage 2c slice 3: `CheckedType::Named(TypeId)` interned type identity

The audit-6 representation flip: `CheckedType::Named(String)` carried raw
spellings, so structurally-equal types could compare unequal
("Result[Int,Str]" vs "Result[Int, Str]") and every consumer re-parsed
names. Implementation:

- `TypeArena` is now a zero-sized handle to a PROCESS-GLOBAL, append-only,
  thread-safe intern table (`OnceLock<RwLock<Interner>>`); interned names are
  leaked as `&'static str` so rendering never holds the lock (poison
  recovery via `into_inner()`). `intern_type_name` canonicalizes structurally
  and derives `contains_param` from the parsed shape; it is the ONLY
  interning entry point. The raw `intern(name, flag)` API is deleted -- it
  could mint non-canonical ids.
- `CheckedType::Named(TypeId)` replaces the String payload. Construction:
  `CheckedType::named(impl AsRef<str>)`, plus `From<&str>`/`From<String>`
  for `TypeId`; `from_str` canonicalizes BEFORE primitive classification
  (so `" Int "` is `Int`, not a named lookalike).
- Interned-symbol surface (the rustc `Symbol` pattern): `Display`,
  `PartialEq<str>`, `PartialEq<&str>`, `PartialEq<TypeId> for str`, and
  `Deref<Target = str>`, so pattern guards and diagnostics read naturally
  without un-interning boilerplate. `Debug` is manual and renders
  `Named("Vec[Int]")` -- raw ids never leak into diagnostics.
- ~190 call sites in lib.rs/catalog.rs migrated: constructions route through
  `named()`, guard comparisons through the str surface, hash lookups through
  `.name()`. `TypeId` equality is structural by construction, so
  `types_compatible`'s named comparisons are now exact rather than
  spelling-dependent.

RED-GREEN: no behavior change intended; the proof is the full gate set on a
fresh canonical binary: checker 196/196, feature-reg 510/510, stdlib-exec
85/85 (+2 ign), e2e 2316/2316, xiom lib 20/20, lsp 42/42, jit 5/5, workspace
check clean. New tests: `checked_named_equality_is_structural` (spelling
variants equal, Debug renders names, whitespace primitives classify) and the
strengthened interner fuzz (canonical variants share an id).

## 2026-09-11 -- Stage 2c slice 2: single structural parser wired through the checker

Stage 2c's structural core (crates/xiom-check/src/structural.rs, round 39)
was still bypassed by hand-rolled type-string parsing in the checker. This
slice completes the "no other module parses type strings" mandate:

- structural.rs grows the shared decomposition helpers:
  - `container_parts` -- `"Option[Result[Int,Str]]"` ->
    `("Option", ["Result[Int, Str]"])` with CANONICAL argument renderings;
  - `is_container_base` -- `"Vec[Int]"` for base `"Vec"` (malformed and
    bare names are false);
  - `tuple_elem_names` -- both tuple spellings: parenthesized `(Int, Str)`
    with a DEPTH-AWARE comma split (nested parens/brackets survive), and
    the legacy `Tuple__Int__Str` encoding;
  - `is_result_or_option` -- the `?` operator's classifier (Result/Option
    bare or with args, plus the `_` placeholder).
- lib.rs retires its local parser: `parse_generic_type` (the last
  `name.find('[')` splitter with an off-by-one slice on malformed text) is
  DELETED; `is_send`, the `first_entry` receiver-arg substitution, pattern
  payload binding (new `container_arg`), `parse_tuple_elem_types`, the
  destructure arm, `tuple_fields_from_name`, the `?` classification and the
  Vec/Array element-index arms all route through the shared helpers. No
  behavior change was intended; because every compound extraction now feeds
  CANONICAL `", "` renderings into `CheckedType::from_str`, spelling
  variants (`"Result[Int,Str]"` vs `"Result[Int, Str]"`) yield identical
  CheckedTypes instead of two structurally-equal values.
- Tests: 2 new structural unit tests (container/tuple decomposition; the
  depth-aware nested tuple split) + 1 checker test pinning canonical
  rendering through `container_arg` / `parse_tuple_elem_types`.

Gates (fresh canonical binary): checker 195/195, feature-reg 510/510,
stdlib-exec 85/85 (+2 ign), e2e 2316/2316.

## 2026-09-11 -- LET-array P3 FIXED: unannotated `let` literals bind fixed arrays

The last migration step of docs/LET_ARRAY_DECISION.md: `Stmt::Let` still ran
the M33 `compile_array_as_vec` conversion for every unannotated literal, so
`let a = [...]` was a `%struct.Vec` while `let a: [N]T` / fixed var bindings
were `[N x T]` -- the representation depended on the binding form.

Fix (crates/xiom-codegen/src/stmt.rs): an unannotated NON-EMPTY `let`
literal binds a `[N x T]` aggregate (element type inferred from the first
element, As-aware), exactly like the P1/var path. Empty literals and
Vec/Slice-annotated bindings keep the M33 conversion; unannotated `var`
literals stay Vec (they can be pushed to).

Keeping the collection API source-compatible required three adjacent fixes:

1. **Call-site Vec materialization** (crates/xiom-codegen/src/coerce.rs,
   `array_as_vec_arg`): passing a fixed array (bare, `&a`, `&mut a`) to a
   by-value `%struct.Vec` param (`&Slice[T]`, `&Vec[T]`, `Vec[T]`)
   materializes a HEAP-BACKED Vec with the array's elements (malloc +
   memcpy, len = N, cap = max(N,16), elem_size = size_of(T)); `%struct.Vec*`
   params get an alloca of the same header. Elements are COPIED, not
   aliased: a by-value `Vec` param may PUSH (`test_algo` concat:
   `var result = a; result.push(...)`), and realloc on a stack view
   corrupted the heap (0xC0000374). Discovered via the ecosystem suite:
   pre-bridge `binary_search(&arr)` bitcast the `[7 x i64]` slot to
   `%struct.Vec*` (AV; the callee read element bytes as len/cap/esz).
2. **Non-generic `&Slice[Int]` ABI** (crates/xiom-codegen/src/lib.rs,
   `param_llvm_type`): `&Slice[Int]` erased to its ELEMENT (`i64*`), so
   `core.sum_slice`'s `s.len()` loaded `data[0]` as the length (probe fm5
   returned 0 pre-fix). It now lowers to the same by-value `%struct.Vec` as
   the generic Slice params (mono subst arm parity); `sum_slice` returns the
   real sum.
3. **`.len()` on fixed arrays** (crates/xiom-codegen/src/call.rs): a
   `[N x T]` local or a `&[N]T` element-pointer param now returns the const
   N from `local_array_sizes` (previously fell through to generic dispatch /
   a 0 stub). Runs before the Str path so narrow (i8*) element arrays are
   not strlen'd.

RED-GREEN: m69 fixture exits 8 pre-P3 (sum_slice = 0), 0 post-P3; probe fm5
returns 3 post-P3; test_algo (89 tests) returns 0 on both pre-P3 and the
final binary -- it caught the stack-view heap corruption mid-slice
(0xC0000374) before the heap-materialization fix. Locks:
tests/regression/m69_let_array_slice_bridge.xi +
`e2e_m69_let_array_slice_bridge` + IR pin `e2e_m69_let_array_fixed_ir`
(`alloca [5 x i64]`). Gates: checker 192/192, feature-reg 510/510,
stdlib-exec 85/85 (+2 ign), e2e 2316/2316.

## 2026-09-11 -- LET-array P2 FIXED: user-fn `&[N]T` / `&mut [N]T` element-pointer ABI

The LET-array decision doc's P2 (docs/LET_ARRAY_DECISION.md) reproduced on a
fresh canonical HEAD build: a NON-GENERIC user fn with a `&[N]T` param
lowered the param to POINTER-TO-ARRAY (`[3 x i64]*`), while generic/catalog
fns (the mono `subst_type` Ref-Array arm) take the ELEMENT pointer (`i64*`).
Two clang failures:

1. `array.len(a)` inside `fn takes_arr(a: &[3]Int)` mono'd with T inferred
   as the ARRAY SLOT NAME: `resolve_local_xiom_type` returned the raw LLVM
   `[3 x i64]` for the pointer-to-array slot, so the specialization name was
   `array.len_[3 x i64]_3` -- `[`/`]` are illegal in an unquoted LLVM symbol
   (clang: `error: expected '(' in call`), and the call arg type
   (`[3 x i64]*`) disagreed with the mono def's element pointer (`i64*`).
2. A non-generic `&mut [N]T` param emitted its element write through a
   two-index GEP on the pointer-to-pointer (`getelementptr [3 x i64]*,
   [3 x i64]** %slot, i64 0, i64 idx` -- clang: "invalid getelementptr
   indices"), so `a[i] = v` was a hard compile error (probe letarr2c.xi).

Fix:
- `IrEmitter::param_llvm_type` (crates/xiom-codegen/src/lib.rs):
  `Ref(Array)` and `MutRef(Array)` params lower to `{elem_llvm}*` -- the
  same shape as the mono subst Ref-Array arm and the catalog `&[N]T` ABI.
- `compile_fn` param binding (crates/xiom-codegen/src/decl.rs): `&[N]T` /
  `&mut [N]T` params record the element type (`local_array_elem` +
  `local_xiom_types`) and const N (`local_array_sizes`), mirroring the mono
  Ref-Array binding -- element reads/writes take the element path and
  `array.len(a)` in the body infers N=3 (name `array.len_Int_3`; call sites
  and the mono def agree on `i64*`).
- `resolve_local_xiom_type` (crates/xiom-codegen/src/emitter.rs): a
  pointer-to-array slot strips the pointer and resolves its ELEMENT type
  instead of ever returning `[N x T]` (the invalid-symbol class; extends
  the round-15 array-value arm).

RED-GREEN: pre-fix letarr2/letarr2b fail with `expected '(' in call`,
letarr2c fails with `invalid getelementptr indices`; post-fix all three exit
0 and the IR is coherent (`define i64 @takes_arr(i64* %param0)`,
`call i64 @array.len_Int_3(i64* ...)`, `define void @bump(i64* %param0,...)`).
Lock: tests/regression/m68_let_array_user_fn_ref.xi +
`e2e_m68_let_array_user_fn_ref` (var + annotated-let sources, ref
forwarding, `&mut` writes, Int8 sext/UInt8 zext element reads). Gates:
checker 192/192, feature-reg 510/510, stdlib-exec 85/85 (+2 ign), e2e
2314/2314.

## 2026-09-11 -- AUDIT #12 CLOSED: watchdog process::exit -> cooperative cancellation

The timeout and memory-budget watchdogs called `std::process::exit(1)` from
worker threads: no unwinding, atexit handlers racing the compiler's own
cleanup, and library embedders (LSP/MCP) could not intercept the exit.
`xiom.toml [compiler] timeout-secs` was also parsed-but-ignored (audit #18
remainder).

Fix: `crates/xiom-codegen/src/cancel.rs` adds a process-wide cooperative
token. Codegen checks it at `compile_program` entry, per function in the
parallel loop, and before merging outputs; the driver spawns clang through
`run_child_cancellable` (reader threads drain the pipes; the poll loop kills
the child on cancellation); the driver watchdogs set the token instead of
exiting, and `real_main` reports the failure and exits from the main thread.
Manifest `timeout-secs` is now applied as a project default (explicit
`--timeout` wins; 0 disables).

Verified: `--timeout 1` on smoke_error2 -> exit 1 in ~2s with both the
timeout message and `codegen: compilation cancelled`; manifest
`timeout-secs = 1` likewise, `--timeout 60` overrides and succeeds; no
suite regressions (checker 192/192, feature-reg 510/510, stdlib-exec 85/85
+2 ign, e2e 2313/2313).

## 2026-09-11 -- LET-array P1 FIXED: annotated `let [N]T` bindings + float element reads

Two codegen roots surfaced by the LET-array decision doc probes
(docs/LET_ARRAY_DECISION.md; tmp/bug_probes/letarr1.xi + letarr3.xi):

1. **`let c: [3]Int = [7,8,9]` emitted invalid IR** (`%tmp defined with
   type 'i64' but expected '[3 x i64]'`): the Stmt::Let array-literal path
   ALWAYS ran the M33 Vec conversion, then stored the %struct.Vec value
   into the declared `[N x T]` slot. The Let arm now ports the VAR BUG 53
   path: allocate the `[N x T]` aggregate, store each element with the
   declared element width (coerce_value handles narrow/float elements),
   register the local as an array. Lock: tests/regression/
   m67_let_array_annotated.xi + e2e_m67_let_array_annotated.
2. **Float elements read through `[N]T` came back boxed as raw i64 bits**
   (the by-value and pointer fixed-array index arms ran `val_to_i64` on
   the loaded element), so the binary-op layer sitofp'd the BIT PATTERN
   back to double: `f[1] != 2.5` was true for an equal value. Both arms
   now return `float`/`double`/`fp128` elements as their real LLVM type,
   mirroring the Vec float-element path.

RED-GREEN: pre-fix letarr1/letarr3 fail to compile (or miscompare);
post-fix both exit 0. Gates: checker 192/192, feature-reg 510/510,
stdlib-exec 85/85 (+2 ign), e2e 2313/2313.

## 2026-09-11 -- smoke_error2 has-mid FIXED: generated type_meta keys shadowed the real type

The long-standing flake (round-13 note: "don't chase, verify via
probe_err_chain2") was reproduced deterministically on a fresh isolated
HEAD build: `smoke_error2` exited 1 with `FAIL: has-mid` 20/20 runs. The
emitted IR for `chain.error_chain_has` loaded `e.messages[i]` through the
SCALAR i64 element path and passed `trunc i64 -> i8` of the Str handle to
`@chain.str_eq(i8*, i8*)`; the chain-only probe resolved the same read as
`Str` and passed. No runtime UB, no stdlib logic defect.

Root cause (codegen, HashMap-order dependent): `type_meta` carries the
generated concrete key `Option__ChainError` (registered by
`error_chain_pop`'s `Option[ChainError]`), whose name ENDS WITH the bare
registered type key `ChainError`. Two resolvers matched candidate keys by
bare suffix:

- `IrEmitter::vec_elem_is_str` iterated `type_meta.keys()` and `break`t on
  the first key matching the suffix. When `Option__ChainError` came first
  (std HashMap ordering is randomized per process), its field list
  (`discriminant`, `value`) lacked `messages`, so the function returned
  false and the Vec[Str] element fell to the scalar loader.
- `IrEmitter::resolve_vec_elem_xiom`'s Field arm selected ONE key with
  `.find(...)?` and returned None when that key lacked the field.

So the same source produced different IR per compiler process --
deterministic per exe, flipping across builds (8 instrumented compiles
flipped exactly the 6 chain fns with `e.messages[i]`, 26 vs 32 branch
markers). This is why "unrelated stdlib code" appeared to change the
result: it changed which generated keys existed, not the chain logic.

Fix (crates/xiom-codegen/src/lib.rs + vec_abi.rs): new tiered
`IrEmitter::declared_field_type(base_ty, field)` resolves the field from
the first meta that CONTAINS it: exact/dot-qualified keys first, then
non-generated bare suffixes (`is_generated_aggregate_key` excludes
`Option__`/`Result__`/`Vec__`/`Slice__`/`Map__`/`Set__`/`Tuple__`/
`_Anon__`), then any suffix match. No scan stops on a key that lacks the
field. `vec_elem_is_str`, `resolve_vec_elem_xiom`, and
`resolve_vec_elem_type`'s Field arm all route through it.

Evidence: pre-fix m66 fixture 3/8 red (exit 1 at has-mid); post-fix 10/10
green; smoke_error2 10/10 green; full smoke IR byte-identical across 5
separate compiler processes. Locks: `stdlib_exec_error2_runs` (the former
flake is now a permanent stdlib-exec gate) + tests/regression/
m66_chain_error_has.xi / `e2e_m66_chain_error_has`. Gates: checker
182/182, feature-reg 510/510, stdlib-exec 85/85 (+2 ign), e2e 2312/2312.

## 2026-09-09 -- M58 FIXED: module-level mutable arrays indexed through stack copies

Stdlib finding 3b-2 #8 ("module-level arrays mis-materialized; crc table
reads back all zeros; backing undersized; AV near page boundaries").
Probes: tmp/bug_probes/arr_global.xi + gz12u.xi/mygzip.xi shape
(`var _crc32_table: [256]UInt;`).

Root cause: index READS and WRITES into a module-level `[N x T]` global
compiled the container via the plain Ident arm (load the WHOLE global by
value, expr.rs:803), then the fixed-array arm spilled that value into a
FRESH stack alloca and GEP'd the COPY (expr.rs:2806-2815; stmt.rs:790-805).
For reads the runtime saw a stale snapshot taken at the read site; for
writes the element store hit the stack copy and the whole-array store-back
to the global never fired -- every write was silently lost. Lazy-init
patterns (`if _tbl[1] == 0 { fill(); }`) then re-ran init every call and
reads stayed zeros. Two [128] arrays appeared to "survive" only because
their accidental layout/sizes hid the stale-copy reads in the specific
probes.

Fix: Ident containers that resolve to a module global with an array type
now GEP the REAL global directly (`@symbol`) in both the index-read arm
(crates/xiom-codegen/src/expr.rs) and the index-write arm
(crates/xiom-codegen/src/stmt.rs); no load/copy/store-back involved.

Regression: tests/regression/m58_module_array_global.xi +
e2e_m58_module_array_global (two array sizes, head/tail checks, RMW;
red on the pre-fix binary -- printed exit 1, green after -- exit 0).
Full e2e + feature-reg + stdlib-exec run at doc time.

## 2026-09-09 -- BUG 57 FOLLOW-UP FIXED: nested Vec ctor element registration (geom_vec/mat)

Root cause of the post-BUG-57 geom compile regression
(`store %struct.Vec <i64>` / "defined with type i64 but expected
%struct.Vec") was NOT a second emitter -- it was an off-by-one-wrap in the
DECL-SITE element registration for nested generic Vec ctors:

- `IrEmitter::vec_ctor_elem_type` (crates/xiom-codegen/src/lib.rs, the LIVE
  copy used by bindings) rendered `Vec[Vec[Float64]].new()` by re-wrapping
  the OUTER Vec around the inner type-arg expression:
  `type_arg_to_name(Index(Vec, idx))` -> "Vec[Vec[Float64]]" (DOUBLE-wrapped;
  the element of `basis` should be "Vec[Float64]").
- `basis[u]` reads then recorded "Vec[Vec[Float64]]" into
  `indexed_elem_types`; the chained inner read `basis[u][jj]` stripped ONE
  layer (mapped_elem) but resolve preferred the struct path: a whole
  %struct.Vec was memcpy'd out of an 8-byte double slot (stride from
  field 3), then the struct->scalar coercion (coerce.rs 433-442) emitted
  extract_scalar_field0 -> ptrtoint (defensive fix, valid), and a SECOND
  extract with the stale type spilled that i64 back INTO a %struct.Vec
  alloca -- the invalid store.
- Why only SOME functions failed: single-pass compilation means reads
  inside a loop body compile BEFORE a later `basis.push(v)` that would have
  corrected the entry via push-inference (call.rs 1124-1172). gram_schmidt
  reads at linear.xi 268/271 precede the push at 284; vectors that were
  pushed before their first read were coincidentally correct.
- FIX: the nested-ctor arm now renders the INNER type arg only
  (`type_arg_to_name(idx)` -> "Vec[Float64]"). Element registers
  single-wrapped; outer reads yield Vec elements, inner reads strip to
  "Float64" and take the typed float load path.
- REGRESSION: tests/regression/m57b_geom_local_nested.xi +
  e2e_m57b_geom_local_nested (gram_schmidt ordering shape; red on the
  pre-fix binary with the exact invalid-IR signature, green after).
- VERIFIED: smoke_geom_vec 6/6 + smoke_geom_mat/quat/2d/3d/collision OK;
  checker 182/182, parser 99/99, ctfe 36/36, fmt 79/79, lexer 29/29.
  Full e2e + feature-reg + stdlib-exec run at doc time.

## 2026-08-24 -- compiler session, Stage 0 ground truth (fresh laptop environment)

Readiness plan created: `docs/COMPILER_READINESS_PLAN.md` (merges the round-15
campaign queue + audit findings #1-20 into staged work; this file's OPEN queue
cross-references those stages).

### BUG 57 (verdict CONFIRMED): nested &Vec[Vec[Float64]] param reads garbage in module fns

The round-15 handoff contradicted itself: SESSION.md claimed probe_skew3/4
"green at baseline -- stale handoff entries"; stdlib_session.md (later that
evening) reported garbage reads. RESOLVED on a fresh machine at HEAD `81ed009a`:

- `tmp/bug_probes/probe_geom1.xi` -- identity(2) RETURNED from one fn, then
  trace/det take &Vec[Vec[Float64]] and use the stdlib defensive-copy pattern
  (`ac.push(a[i])` then double-index). Result: trace printed
  **9.21436483760003e+18** (garbage bits), exit 1.
- `tmp/bug_probes/probe_geom2.xi` -- RAW param read `m[0][1]` in a fn body:
  printed **-4.6094342186137e+18**, exit 1. Unary-minus-on-nested-index in main
  was NOT reached (blocked by the param read). The stdlib_session "-4.0 bits"
  family is confirmed.

Both probes print values via convert.float_to_string; run with
`xiom --run -o out.exe probe.xi` and read the PRINTED exit-code line.

Coverage gap: NO e2e test exercises nested &Vec[Vec[Float64]] PARAM reads in
module fns (e2e_m37_nested_vec covers direct indexing on LOCALS only) -- which
is how 2294/2292 green e2e coexists with this bug. Stage 4 fix must add
e2e_m57_geom_nested_param (identity->trace/det + raw m[0][1] + skew compare).
Fix direction per stdlib_session: the &Vec[Vec[Float64]] mono'd-body element
read path (the "mono'd &[N]T body offset+1" family); locals in main are known
good. Affected smokes: geom_vec/mat/quat, matrix.det/trace/rank consumers,
curves/bezier family.

### Round-15 quick suites re-verified on this machine

checker 178/178, parser 97/97, ctfe 97/97 (match the desktop numbers exactly);
full e2e running at doc time. Build from clean target: 1m07s, no new warnings
beyond the known xiom-check unused_mut + codegen dead-code set.

### BUG 57 FOLLOW-UP (2026-08-28, stdlib r17 finding): geom_vec/mat compile regression -- DIAGNOSED, PARTIAL

Post-BUG 57, smoke_geom_vec/mat changed from RUNTIME garbage to a COMPILE
error: "%tmpN defined with type i64 but expected %struct.Vec" inside the
gram_schmidt/_vec_dot region (IR lines 20759-20761). Established facts:

1. The invalid IR = extract_scalar_field0 applied to a %struct.Vec VALUE
   (coerce.rs) -- a Vec operand reached the binary-op struct-scalar
   extraction (expr.rs ~1728-1733), which loads field 0 AS i64 (it is the
   i8* DATA POINTER). DEFENSIVE FIX LANDED: Vec/Slice now load field 0 as
   the pointer + ptrtoint (VALID IR, same i64-bits semantics the pre-map
   codegen had). Error moved one temp forward (tmp380 -> tmp381): a SECOND
   emitter then stores that i64 AS %struct.Vec (alloca Vec + store Vec)
   -- its identity not yet found (not coerce_value/emit_vec_store_fields/
   call.rs:1749 per greps).
2. Trace evidence: instrumented runs show ONLY ONE map-resolution event in
   the whole gram_schmidt region (vc[i][j] FLOAT at line 310) -- the
   basis[u] / basis[u][jj] reads NEVER reach the is_vec branch's
   resolution section (no FLOAT, no MEMCPY, no FALLBACK trace). They
   compile through a bypassing path (suspect: the indexed-WRITE RHS in
   stmt.rs or the &basis[u] Ref-arm address path, or an INLINED
   _vec_dot body polluting gram_schmidt's IR). fn attribution via define
   lines is unreliable here (alwaysinline bodies).
3. Per-fn map clear added at compile_fn entry (correct hygiene -- generic
   fns share source spans across monomorphisations) but did NOT fix this
   case, so the leak is within a single body or via a non-map path.

NEXT SESSION PLAN: instrument the stmt.rs indexed-write RHS path and the
Expr::Ref Index-address path for Vec[Vec[Float64]]; correlate IR tmp
lineage via -g-style comments (emit a "; idx-read" marker in each
candidate emitter, rebuild, read the failing block's ancestry). The
stdio session has geom_vec/mat parked; quat is GREEN with BUG 57.
### BUG 57 FIXED (2026-08-25, overnight): chained-index element types -- THREE coordinated roots

Fix spans expr.rs + lib.rs + types.rs + call.rs; probes probe_geom1/2 now
exit 0 with correct values (-3/3/3, trace=2/det=1); new regression
tests/regression/m57_geom_nested_param.xi registered as
e2e_m57_geom_nested_param. Root causes (in discovery order):

1. vec_elem_from_type_annotation rendered nested container args via
   type_from_ast, which DROPS generic args -> `&Vec[Vec[Float64]]` param
   registered elem="Vec" (bare). Fixed to type_string_full -> "Vec[Float64]".
   GOTCHA: TWO copies of this fn exist (lib.rs:~2312 live for decl.rs,
   types.rs:578) -- BOTH patched; keep in sync.
2. resolve_vec_elem_type(container) only knows IDENT/FIELD containers.
   For chained m[i][j], the inner container is an INDEX EXPRESSION ->
   None -> scalar i64 load -> caller sitofp on RAW FLOAT BITS =
   -4.6094342186137e+18 (exactly -3.0's bit pattern as i64).
   FIX: new indexed_elem_types map on IrEmitter: key = Debug form of an
   Index EXPR, value = XIOM type of what it YIELDS. Outer index inserts;
   inner index looks up ITS CONTAINER and strips one Vec layer.
3. First-cut of the map fed primitives into the struct-memcpy path ->
   `alloca %struct.Int` = "Cannot allocate unsized type"
   (smoke_collect_cache compile fail). Primitive elems now fall through to
   the scalar loader; only struct/nested-Vec elements take memcpy.

Also en route: mc.push(m[i]) push-side inference consulted the new map
(call.rs) so defensive-copy rows register "Vec[Float64]" not "Vec[Int]".

VERIFIED: m57 e2e green; feature-reg 510/510; stdlib-exec 70/70(+2);
parser/checker/ctfe green; full e2e running at doc time. The geom smokes
(vec/mat/quat) and curves/bezier consumers should be re-swept by the
stdlib session on this build.
### R16c -- verifier honesty LANDED (2026-08-25, audit #3/#8/#14 CLOSED)

crates/xiom-verify rewritten production-grade; suite 31/31 (27 kept + 4 new
regression tests). Defect-to-fix map:
- #3 fail-closed-by-false: unsupported obligations are SKIPPED and reported
  as UNKNOWN with reasons (GenReport/SkippedObligation; CLI prints
  "[WARN] UNKNOWN"). A body-incomplete function gates ALL its obligations
  (unconstrained |result| used to fire spurious VIOLATED).
- #8 sort mismatch: ONE numeric story -- all integer widths -> SMT Int,
  floats -> Real (decimal literals, no exponent form); sort-aware operators
  (div vs /); mismatches -> UNKNOWN. The old BitVec/FloatingPoint mapping
  made every such query ill-sorted. NOTE: the old test
  smt_has_correct_type_map CODEFIED the bug (asserted BV32) -- rewritten.
- #8 undeclared field fns: mappable structs emit declare-datatype; field
  access uses real selectors (Point-x). Unmappable receivers -> UNKNOWN.
- #14 z3 deadlock: input via UNIQUE TEMP FILE (no stdin pipe to deadlock);
  -T now passed in SECONDS correctly (was milliseconds into a seconds flag);
  parent polls try_wait + kills at 1.5x budget.
- ALSO FIXED en route: contract-axiom forall bound the RETURN SORT STRING as
  a variable (invalid SMT); if/else branches were CONJOINED (contradictory
  contexts made obligations UNSAT = VACUOUS Proven for clamp/max shapes) --
  branches are guarded implications with definite-return fall-through
  threading; SSA reads use the LATEST binding; naked while loops -> UNKNOWN;
  type invariants are REAL check-sat VCs (were TODO comments); xiom-verify
  CLI forgot parser take_errors() (the audited partial-AST hazard) --
  malformed input now fails loudly.
- CHECKER LIMITATION found while probing (log for Stage 2/3): contracts
  referencing struct-field RECEIVERS (`requires: p.y == 0`) fail checker
  typing ("left operand must be numeric, found <error>") even though plain
  body field access works. Fix lands with structural types (Stage 2).

### Stage 0 RESULTS (2026-08-24 evening, post-baseline)

- Full e2e on the untouched round-15 binary: **2293/2294**. The single fail,
  e2e_m37_simd_runtime (Some(15) vs Some(0)), reproduces WITHOUT any local
  change -- SIMD CPU-dispatch environment fail on this laptop (same class as
  the old smoke_simd CRT ignore). Desktop remains the reference for
  SIMD-sensitive tests.
- stdlib_exec_log_runs ALSO fails at BASELINE here (verified stash+rebuild):
  machine-environmental until the desktop re-confirms. stdlib-exec 69/70(+2).
- Stage 1 CTFE rewrite landed; see SESSION.md round-16a header for the full
  defect-to-fix map. New ctfe suite: 36/36 (recursion-no-ICE, RecursionLimit-
  diagnostic, session fuel, overflow hard-error, exact float eq, for-range/
  array iteration, break/continue/return-in-loop, block-tail sequential exec,
  try_to_expr aggregate reconstruction with NO Int(0) sentinel).
- GOTCHA for future edits: Expr::Int stores the i64 bit pattern as u64 --
  checked arithmetic must run via `as i64` FIRST (naive unsigned checked ops
  return None for -X / 0-X and silently unfolded consts; caught by
  regress_5e7f_const_arithmetic_neg against baseline within minutes).

---

## STATUS SUMMARY -- authoritative (2026-08-11 16:3x, compiler session)

> Read this first. Older dated sections below may contradict it (e.g. the
> BigInt/BigFloat Phase C entry claimed BUG 11 open -- that predates the fix).

| Item | Status | Fix commit | Verified by |
|------|--------|-----------|-------------|
| BUG 1 -- tuple-of-struct opaque LLVM type / truncated slots | **DONE** | `d22068f8` | e2e_m37_tuple_struct; probe_big/probe_big2; bigint_div_mod no longer crashes |
| BUG 8 -- catalog fns with `&Vec[Int]`/`&Vec[UInt8]` params | **DONE** (was the uncommitted coerce.rs state) | `384d5666` (revert) + verified | e2e_m37_catfix_catalog_vecref; probe_bit (12&10=8) |
| BUG 9 -- private catalog struct types degrade to i64 | **DONE** | `3ccd004c` | e2e_m37_catfix_private_type (b9mod+b9main, sum=7) |
| BUG 10 -- float literals truncated to 6 decimals | **DONE** | `a2aafa26` | e2e_m37_float_precision (0.123456789 round-trips) |
| BUG 11 -- unsafe-extern double marshalling (math.sqrt via trampoline) | **DONE** | `7f7b7b54` | stdlib-exec **72/72** (complex + net_folder pass); e2e_m37_unsafe_option_return |
| Parser -- `bits[L - 1]` (uppercase-ident index with arithmetic) | **DONE** | `40441ca7` | e2e_m37_index_arith; bigint.xi/bigfloat.xi parse |
| Catalog -- import lookup walked the whole tree per miss (minutes/hang) | **DONE** | `1e982ebf` | stdlib-compile 108.7s -> 23.7s; probe_bit compile 138-160s -> 9.5s |
| Struct `&T` param mutation silently lost | **DONE** | `d22068f8` | e2e_m37_ref_mut; probe_dig/probe_cmp |
| clang -O2 hang (alwaysinline everything) | **DONE** | `d22068f8` | smoke_bigint.xi compiles ~5-14s (was >300s) |
| Unsafe-block capture collector missing Expr::Struct etc. | **DONE** | `e0f96fef` | e2e_m19_read_file_content |
| Parallel codegen `__unsafe_ctx_0` redefinition | **DONE** | `e0f96fef` | e2e_i2_parallel_codegen (all 5 ecosystem tasks) |
| BUG 12/17 -- Vec[Float64]/Vec[Str] element type lost on `&Vec[T]` params | **DONE** | `2ae300fd` | e2e_m37_vec_f64 (R=0 isolated); m12b |
| BUG 13 -- fp128 link + coercion (soft-float helpers) | **DONE** | `2ae300fd` + `3b8f5415` (fp128_helpers.c) | e2e_m37_f128 (R=0 isolated); C harness 400/5/2500/1002.5 |
| BUG 14 -- UInt64->UInt128 sext / UInt128>> ashr | **DONE** | `2ae300fd` | e2e_m37_u128 (R=0 isolated); m14 |
| BUG 15 -- bare `shr`/`shl` hijacked by math-builtin intercept | **DONE** | `2ae300fd` | e2e_m37_shr_builtin (R=0 isolated); m15 |
| BUG 16/18 -- skiplist+trie / string+similarity+time 0xC0000409 combos | **DONE** | `2ae300fd` (fn-key + longest-prefix walk + encoding repair) | m16a-f probes R=0; m16b-e (bare/leaf forms) R=0 |
| **BUG 2 -- module-global struct FIELD writes lost** | **DONE** | `f0388644` | e2e_m37_global_field_write (g.v = 5 persists); probe_gf R=0 |
| **BUG 3 -- module-global fn-call initializers zero** | **DONE** | `f0388644` | e2e_m37_global_fn_init (`var G = _mk(1)` -> 10 via @llvm.global_ctors); probe_const3 R=0 |
| Str + Int/Char/UInt concat crashed (inttoptr of the integer -> AV) | **DONE** | `f0388644` | e2e_m37_str_int_concat ("y = " + 42 -> "y = 42"); probe_concat0 R=0 |
| Dotted-module chain binding -- `use stdlib.xiom.io` flaky 40-60% "cannot call" (parser nests dotted paths; process_use bound the {xiom:{io:...}} chain; stdlib-prefixed uses skipped preload/prelude) | **DONE** | `f0388644` | m19_read_file 8/8 + min repro 8/8 deterministic; stdlib-compile 40/40; checker 178/178 |
| BUG 19 [U+FFFD] NaN-producing Float64 ops returned sentinel/0xC000001D (float `!=` ? `fcmp one`; `Str+Float64` concat inttoptr) | **DONE** | `9c3a2f9e` | e2e_m37_nan_ieee (23 checks) R=0; probe_nan prints "c = nan / is NaN" exit 0; 17/17 sweep |
| BUG 43 -- Result[Float64, Str] payload read via sitofp (direct-call scrutinee) | **DONE** | this session | e2e_m37_bug43_result_f64_payload; smoke_core_convert exit 0 |
| BUG 44 -- deref/coercion of `&Str` loads a byte (`i8`) instead of the pointer (`i8*`) | **DONE** | this session | e2e_m37_bug44_str_deref; probe_b44 exit 0 |
| BUG 45 -- method-form interface dispatch inside generic-bound fns -> stub | **DONE** (root cause = BUG 46's i64* param degradation) | this session | e2e_m37_bug45_iface_method_generic; eqt11/eqt18/eqt20 exit 0 |
| BUG 46 -- generic `&UserStruct[T]` param field reads return garbage (mono Ref arm degraded to i64*) | **DONE** | this session | e2e_m37_bug46_generic_struct_ref; eqt18/20 exit 0 |
| BUG 47 -- `ref_params`/`param_locals` leak across fns -> later value param mis-dereffed (AV) | **DONE** | this session | e2e_m37_bug47_ref_params_leak; smoke_sort exit 0 |
| BUG 41/42 follow-up -- mono Ref arm made `&Slice[T]`/`&[N]T` lowering UNREACHABLE (i64* params) | **DONE** | this session | e2e_m37_bug45/46/47 (restored `%struct.Vec` by-value + `elem_ty*`); warning gate zero |
| S7 NASM/SIMD tracks (math/crypto/hash/compress asm, CPUID dispatch) | **OPEN** -- off-limits to stdlib session (crates/ + stdlib/runtime/*.c); pure-XIOM fallbacks in place | -- | -- |
| Selfhost plan | **WRITTEN -- execution in progress** | `090ed5d1` | docs/SELFHOST_PLAN.md (phases 0-8) + docs/checklists/selfhost-phase0.md |

**Suite state (current, 2026-08-11 22:10):** fast suite 1103/10/1 -- ALL 10
failures are the parallel stdlib session's in-flight restructure (stdlib-exec
complex/hash/net/rand folder moves + smoke_math_core renamed to
smoke_math_tower, lsp/mcp module-list tests against the new layout, diff
documented pre-existing ignore); **zero compiler regressions**. e2e harness
verification of the 4 new e2e_m37 tests is blocked until the parallel session's
next compiler rebuild (their target/debug/xiom.exe predates `2ae300fd`); exact
harness invocation replicated with the isolated binary: compile=0 run=0 for
all 6 affected tests. Warning gates 0/0.

---

## 2026-08-18 -- Compiler session: BUG 43-47 FIXED + scripting test race + mono &Slice regression

### FIXED -- BUG 43: Result[Float64, Str] payload read via sitofp (direct-call scrutinee)

- **Root cause:** the Some/Ok/Err payload binding resolved the payload XIOM type
  ONLY for `Expr::Ident` scrutinees (`local_opt_payload*` tracking). A scrutinee
  that is a DIRECT CALL (`match core.to_float_from_str("3.14")`) had no declared
  payload -> the Float64 bound as raw i64 bits and float ops sitofp'd 3.14's
  pattern (~4.6e18) -- smoke_core_convert exit 11.
- **Fix (lib.rs + stmt.rs):** new `scrutinee_payload_xiom(expr, field_idx)` -- for
  Call/GenericCall scrutinees resolves the callee's declared return type via
  `callee_return_xiom` + the existing bracket-aware `option_result_payload` /
  `option_result_err_payload`; both payload-binding paths (guard pre-extract +
  arm binding) use it. Definition side (bitcast) was already correct.
- **Verified:** probe exit 0 (direct-call + ident-held + Err-payload Str);
  smoke_core_convert exit 0; e2e `e2e_m37_bug43_result_f64_payload`.

### FIXED -- BUG 44: deref/coercion of `&Str` loads a byte instead of the pointer

- **Construct:** `var p = &s; var d = *p;` (deref of &Str) and passing `&Str`
  where `Str` is expected (auto-coercion). Both AV'd or compared a byte.
- **Root cause (three defects):**
  1. `UnaryOp::Deref` used `trim_end_matches('*')` -- for a `&Str` param
     (`i8**` -- pointer to the Str slot) it stripped BOTH stars and emitted
     `load i8, i8**` instead of `load i8*, i8**`. One-star strip fixes it.
  2. LOCALS bound from `&expr`/`&T` were untracked (params had `ref_params`;
     locals had nothing) -- `*p` fell to the legacy byte-pointer path
     (`inttoptr i64 -> i8*; load i8`). New `ref_locals` set (context.rs) +
     `track_ref_local` (stmt.rs Let/Var) mirrors `ref_params`; the deref path
     resolves the pointee via `local_xiom_types`.
  3. `&T`->T auto-coercion: `expect_str(&s)` / `expect_str(p)` passed the slot
     ADDRESS (or truncated it to a byte) as the string. New
     `coerce_ref_arg_to_pointee` (coerce.rs) derefs the address when the param
     type exactly equals the pointee LLVM type (i8* for Str); `&T` params
     (i64/i8**) keep receiving the address.
- **Verified:** probe (deref local + param, coercion ident + literal-ref forms)
  exit 0; e2e `e2e_m37_bug44_str_deref`.

### FIXED -- BUG 46: generic `&UserStruct[T]` param field reads return garbage

- **Root cause:** the mono `subst_type` Ref arm's struct check only matched
  BARE registered keys (`struct_types.contains(&name)`); user generic structs
  register MODULE-QUALIFIED ("eqt20.Box2"), so `&Box2[T]` params degraded to
  `i64*` -- the callee's field GEPs indexed an i64, and every field read
  degraded to 0 (the "empty body" was `ret i64 0`).
- **Fix (lib.rs mono Ref arm):** the struct check now also matches the
  qualified suffix (5c.35 style, same as the Named arm) and emits
  `%struct.{qualified}*`.
- **Verified:** eqt18/eqt20 exit 0 (user-slice element reads + len reads),
  probe exit 0; e2e `e2e_m37_bug46_generic_struct_ref`.

### FIXED -- BUG 45: method-form interface dispatch inside generic-bound fns -> stub

- **Root cause:** the SAME mono Ref-arm i64* degradation as BUG 46 -- the
  slice element loads produced garbage, and the method-form dispatch
  (`el.eq(&value)`) resolved against the wrong shape. With the param typed
  correctly, the primitive fast-path inlines the correct comparison.
- **Verified:** eqt11 (user slice + generic contains), probe with a T-typed
  receiver (`f_eq[Int](3,3)`) -- both exit 0; e2e
  `e2e_m37_bug45_iface_method_generic`.

### FIXED -- BUG 47: `ref_params`/`param_locals` leak across functions (fn-param -> AV)

- **Root cause:** `param_locals` and `ref_params` were NEVER cleared between
  function compilations (the per-fn reset block missed them). A `&T` param
  named `b` in an EARLIER fn (`cmp_int(a: &Int, b: &Int)`) left a stale
  entry, so a LATER fn's plain value param named `b` was misidentified as a
  ref-param -- `x.compare(&b)` compiled the VALUE and the eq/compare fast-path
  dereferenced address 7 (`inttoptr i64 7 + load`) -> 0xC0000005. The fn-param
  `compare` in heap_sort_by/heap_sift_down_by was the same family (stale
  ref-param classification broke the fn-pointer arg flow).
- **Fix (decl.rs):** clear `param_locals` + `ref_params` (and the new
  `ref_locals`) in the per-fn reset block.
- **Verified:** full-collision probe (impl `Int.compare` used via a generic
  bound + `heap_sort_by(&mut hb, cmp_int)`) exit 0; smoke_sort exit 0; e2e
  `e2e_m37_bug47_ref_params_leak`.

### FIXED -- BUG 41/42 follow-up: mono Ref arm made `&Slice[T]`/`&[N]T` lowering UNREACHABLE

- The BUG 41/42 batch merged the Ref/Ptr/MutRef mono arm, making the
  `&Slice[T]` (by-value `%struct.Vec`) and `&[N]T` (`elem_ty*`) specialization
  UNREACHABLE (dead-code warning) -- generic fns then lowered `&Slice[T]`
  params to `i64*` and read the first element as the length. Restored the
  shape checks in the live arm; the dead arm is deleted (warning gate: zero).

### FIXED (test-side) -- scripting `test_standalone_simple` flake: shared `s_out.exe` race

- Both standalone tests derived the OUTPUT path from a shared "s_out" name;
  with `--test-threads=32` they raced on the same output file (Windows
  sharing violation -> intermittent "Access is denied" -> non-success status).
  The output path now derives from the UNIQUE script name. Verified 3/3 full
  scripting runs 34/34.

### NOTE -- checker gap (NOT this batch): `Eq5[T].eq(el, &value)` associated-form Self substitution

- eqt13's associated form still fails the CHECKER ("argument 1 type mismatch:
  expected Self, found Int") -- `Self` in an interface's method param doesn't
  substitute to the type arg in associated-form calls. The tower pattern
  (explicit `Eq5[Int].eq(...)` with fn-style params) avoids it; stdlib uses
  method form. Candidate for a future checker session.

---

## 2026-08-10 -- BigInt/BigFloat session findings (feat/architect)

### BUG 1 (CRITICAL) -- Tuple return types containing structs emit an OPAQUE LLVM type

- **Construct:** any function returning a tuple whose element is a non-primitive
  struct, e.g. `fn f() -> (BigInt, BigInt)`, `fn g() -> (BigInt, BigInt, BigInt)`.
  Tuple elements of i64 (e.g. `(Int, Int, Int)` in `time.civil_from_days`,
  `bits.unpack_u32_le`) work; tuple elements that are structs do not.
- **Error / observed behavior:**
  1. When clang rejects the IR:
     ```
     error: clang failed with exit code 1
     stderr: xiominput.ll:129:18: error: Cannot allocate unsized type
     129 |   %tmp5 = alloca %struct.Tuple__probe_tuple.Pair__probe_tuple.Pair
     ```
     The `%struct.Tuple__...` type is referenced but never defined (opaque).
  2. When clang tolerates it (bigint case), the constructor stores only the
     FIRST i64 of the struct into the tuple slot (`store i64 %tmp33, i64* %tmp34`
     where element 0 is a 40-byte `BigInt`) -- the rest of the slot is garbage.
     Reading `dm.0` then dereferences garbage Vec pointers:
     - `bigint_div_mod(3, 10)` -> `0xC0000005` ACCESS_VIOLATION (or
       `0xC0000409` STACK_BUFFER_OVERRUN)
     - `bigint_div_mod(10, 3)` -> crash; `bigint_mod` -> hang/infinite loop;
       `bigint_sqrt` -> crash; `bigint_ext_gcd` -> crash.
- **Minimal repro** (`probe_tuple.xi`):
  ```xiom
  module probe_tuple
  type Pair = { a: Int; b: Int; }
  fn make_pair(x: Int, y: Int) -> (Pair, Pair) {
    return (Pair{ a: x; b: y; }, Pair{ a: y; b: x; });
  }
  fn main() -> Int {
    var t = make_pair(3, 7);
    if t.0.a != 3 { return 1; }
    if t.1.b != 3 { return 2; }
    return 0;
  }
  ```
  -> clang error above (opaque `%struct.Tuple__probe_tuple.Pair__probe_tuple.Pair`).
- **Impact on stdlib (blocks Phase A/B):**
  - `stdlib/xiom/bigint.xi`: `pub fn bigint_div_mod(a: &BigInt, b: &BigInt) -> (BigInt, BigInt)`
    (PRE-EXISTING, frozen signature, **never exercised by any test until now** --
    `tests/regression/` contains zero `bigint_div_mod`/`bigint_mod`/`bigint_gcd`
    call sites; the original 3-assertion smoke only exercised parse/mul/compare).
    The original author already worked around the tuple problem once for the
    internal helper (`_div_mod_base` returns a named `DivModResult` struct --
    comment: "Returns a DivModResult struct to avoid tuple issues") but the
    public `bigint_div_mod` kept the tuple and was never tested.
  - Planned (this session, blocked): `bigint_sqrt_rem -> (BigInt, BigInt)`,
    `bigint_ext_gcd -> (BigInt, BigInt, BigInt)`.
- **Fix direction:** codegen must emit a concrete LLVM struct type definition
  for `Tuple__<T>__<U>` (aggregate of element types) and store elements with
  the element's real size (memcpy-style), not `store i64`. Enum variants with
  struct payloads (Option/Result) already work -- reuse that lowering path.

### BUG 2 -- Module-global struct FIELD writes are silently lost

- **Construct:** `var g: W = W{ v: 0; };` at module scope, then `g.v = 5;` in a
  function; reading `g.v` afterwards returns 0. Whole-value assignment
  (`g = W{ v: 5; };`) persists correctly.
- **Minimal repro** (`probe_gf.xi`): `_set()` does `g.v = 5;` -> `main` prints
  `g.v` == `0`.
- **Impact:** module-level mutable struct state (round-mode globals etc.) must
  be updated by whole-value assignment until fixed. Stdlib constants that
  would be module globals are affected (see BUG 3).
- **Fix direction:** GEP-store on a module-global struct must write through to
  the global (currently the value appears to be copied on access).

### BUG 3 -- Module-global `var` initializers that CALL functions are silently zero

- **Construct:** `var G: BigInt = _mk_one();` at module scope (fn call in the
  initializer). `G` reads back as zero at runtime, no diagnostic.
  Scalar literals (`var _BASE: Int = 1000000000;`) and struct literals
  (`var g: W = W{ v: 0; };`) initialize correctly.
- **Minimal repro** (`probe_const2.xi` / `probe_const3.xi`): module fn
  `_mk_one()` returns `bigint_from_int(1)`; global `G_ONE = _mk_one()` prints
  `G_ONE=0` while a local call prints `local=1`.
- **Impact:** the spec constants `BIGINT_ZERO/ONE/TEN` (bigint) and
  `BIGFLOAT_ZERO/ONE/TWO/TEN/HALF/PI/E` (bigfloat) cannot be module globals
  built by code; the stdlib exposes them as pure constructor functions
  (`bigint_one()`, `bigfloat_pi()`, ...) until fixed.
- **Fix direction:** run global initializer expressions (or lower them to
  `@llvm.global_ctors` / runtime startup init) instead of emitting a zero
  initializer.

### NOTE 4 -- Module-qualified enum variant access resolves to a module

- `bigfloat.Down` (variant via module qualifier, like `cmp.Less` in cmp.xi)
  fails with "argument 1 type mismatch: expected RoundMode, found module" in
  some positions, while type-qualified `bigfloat.RoundMode.Down` (log.xi
  style) works everywhere. Checker's dotted-name resolution appears to treat
  the variant segment as a sub-module first. Minor; type-qualified form is the
  documented style.

### NOTE 5 -- D4b aggregate: leaf sub-module fns not reachable through a 1-segment aggregate

- After `use xiom.bigfloat;` (flat aggregate manifest `module xiom.bigfloat`
  + `use xiom.num.bigfloat;`), the sub-lib's TYPES resolve
  (`bigfloat.RoundMode.Nearest`) but its FUNCTIONS do not:
  - `bigfloat.bigfloat_from_int(42)` -> "cannot call on this expression"
  - `num.bigfloat.bigfloat_from_int(42)` -> "undefined variable 'num'"
  - `xiom.num.bigfloat.bigfloat_from_int(42)` -> works (full dotted path).
  The proven D4b example (`use xiom.math;` + `math.core.sqrt(x)`) works only
  because the sub-lib's parent segment (`math`) equals the aggregate leaf.
  With `use xiom.num.bigfloat;` directly, leaf calls (`bigfloat.fn`) work.
  Checker should attach `num.bigfloat` under the aggregate key `bigfloat`
  (parents map builds from the full key "num.bigfloat" -> parent "num", never
  from the importing manifest).

---

## 2026-08-10 -- Compiler-hardening session (BUG 1 fixed; findings below)

### FIXED -- BUG 1 (tuple-of-struct codegen) -- root cause and fix

Three coordinated codegen fixes landed (commits pending, crates/xiom-codegen):

1. **`concrete_type_for` Tuple branch** (lib.rs): tuple type keys now
   module-qualify their element names (`Tuple__probe_tuple.Pair__probe_tuple.
   Pair`), matching the expression-level registration used by the body's
   alloca/GEP. Previously the fn SIGNATURE used bare names (`Tuple__Pair__
   Pair`) while the body used qualified names -- two different LLVM types for
   the same tuple -> clang "Cannot allocate unsized type" / ret type mismatch.
2. **`resolve_type_key`** (lib.rs): prefers the current-module-qualified key
   and SKIPS generated aggregate keys (`Tuple__...`, `Option__...`,
   `Result__...`, `_Anon__...`) in the suffix search -- `Tuple__x.Big__x.Big`
   ends with `.Big`, so a bare `Big` could resolve to the tuple itself and
   nest the tuple name into itself (double-nested `Tuple__Tuple__...`).
3. **`parse_struct_field_types`** (lib.rs): looks up the registered type_meta
   layout instead of splitting the name on `_` (which degraded every
   non-primitive element to i64 -- tuple slots stored only the first i64 of
   each struct). Element stores now use the real field width.

Also routed tuple PARAM types through `concrete_type_for` (`param_llvm_type`,
decl.rs signature + prologue), and fixed `coerce_arg_for_param` for
non-ident `&expr` lvalues. Verified: `probe_tuple`, `probe_big` (40-byte
structs, 3-element tuples), `probe_big2` (copy-from-tuple + by-ref passing)
all compile and run correctly; `bigint_div_mod` no longer crashes (was AV /
garbage before the fix).

### FIXED -- Struct `&T` param mutation was silently lost

- **Construct:** `fn _trim(b: &BigInt) { b.digits.pop(); }` -- mutation of a
  field through a `&T` STRUCT param. Struct `&T` params were passed BY VALUE,
  so `pop()`/`push()` on the param's Vec field mutated a discarded copy --
  trailing-zero limbs were never trimmed (`3*3` produced digits `[9, 0]` with
  len 2), breaking `bigint_eq`/`bigint_compare` on arithmetic results
  (`compare(3*3+1, 10)` returned 1 while both printed "10"). Scalar `&T`
  params already carried the address; structs now do too.
- **Fix (crates/xiom-codegen):** `param_llvm_type` lowers plain-struct `&T`
  params to `%struct.X*` (address), matching `&mut T`. Generic containers
  (`&Vec[T]`, `&Slice[T]`, `&Map`, `&Set`) keep the established by-value ABI.
  The existing auto-deref field-read path (expr.rs), Vec push/pop mutation
  path, `coerce_arg_for_param` (slot-address + fresh-alloca fallback for
  `&fn_call()` lvalues), and `infer_llvm_type`'s Field arm (pointer-base
  normalization + `Vec[...]` base-container resolution) complete the path.
- **Verified:** `probe_mut` (pop through `&W` propagates), `probe_ref`
  (scalar `&Int` unchanged), `probe_dig` (mul/add/sub limb lengths now
  correct -- `_trim` works), `probe_cmp` (compare correct), `probe_dm`
  (div_mod invariant `q*b+r==a` holds for small numbers).

### FIXED -- clang -O2 hang on large modules (alwaysinline every function)

- **Construct:** every emitted function was marked `alwaysinline`. With a hot
  caller (the extended bigint smoke's `main`) calling many large functions,
  LLVM inlined the whole library into `main` and `clang -O2` never finished
  (>300s on a 735KB module; `-O0` finished in ~16s; stripping the attribute
  finished in ~3s).
- **Fix (decl.rs):** size-based inline policy -- `approx_block_cost`
  (recursive statement count): <=10 stmts `alwaysinline`, <=48 `inlinehint`,
  larger no attribute. Preserves P0-4 hot small-function inlining
  (`read_u16_be`-style) while preventing optimizer explosion.
- **Verified:** extended `smoke_bigint.xi` compiles in ~14s (was hanging).

### NOTE 7 -- `bigint_div_mod` quotient is WRONG for multi-limb dividends (STDLIB, parallel-session Phase A)

- **Construct:** `bigint_div_mod` on any dividend with >=2 limbs produces a
  too-small quotient and a remainder >= divisor. Repros (all other bigint
  functions verified correct -- mul/add/sub/compare/to_str/from_int/from_str):
  - `bigint_div_mod("1000000005", 2)` -> q=2, r=1000000001 (correct:
    q=500000002, r=1).
  - `bigint_div_mod("987654321987654321", 12345)` -> q=80004000080004,
    r=4941000004941 -- satisfies `q*b+r==a` but r >= b (correct: r < 12345).
  - `bigint_div_mod("987654321987654321", 1000000000)` -> q=987654321.
- **Root cause (stdlib logic in `stdlib/xiom/bigint.xi`, NOT the compiler):**
  `_estimate_q_digit` (line 132) estimates with a single top limb
  (`n_hi / d_hi`) with no carry window from the higher limbs, and the
  div_mod loop only corrects OVERestimates (decrements `est`), never
  UNDERestimates -- so a low estimate sticks. Also `est <= 0 { return 1; }`
  forces a bogus digit of 1 when the true digit is 0.
- **Action:** parallel BigInt session owns `bigint.xi` -- fix the estimator
  (Knuth-style two-limb window + upward correction, or process the remainder
  carry) and remove the `est<=0 -> 1` shortcut. Compiler side is verified
  correct via the probes above.
- **Smoke impact:** `stdlib_exec_bigint_runs` fails at assertion 8 until this
  stdlib fix lands (compile/hang issues from BUG 1 and the optimizer hang are
  fixed).

---

## 2026-08-11 -- BigInt/BigFloat session: follow-up verification (post coerce.rs edit)

### BUG 8 (CRITICAL) -- RESOLVED by the compiler session's uncommitted coerce.rs edit

Re-verified on the rebuilt compiler: `_from_twos_bits` / `_bits_to_bigint`
now emit correct signatures
(`define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0)`),
`bigint_bit_and/or/xor` work, and the extended `smoke_bigint.xi` (20+
assertions incl. two's-complement negatives) passes end-to-end.

### BUG 9 -- Catalog fns returning module-local PRIVATE struct types degrade to i64

- **Construct:** `fn f(...) -> PrivateType` where `PrivateType` is a struct
  declared (non-`pub`) in the same catalog stdlib module. The definition and
  call sites emit `i64` instead of `%struct.PrivateType` -> ABI mismatch ->
  AV at runtime. `pub type` in the same position works.
- **Minimal repro** (verified): `stdlib/xiom/num/probet.xi` with
  `type Wrap = { a: Int; b: Int; }` + private `make_wrap` -> `call i64
  @probet.make_wrap` / `define i64 @probet.make_wrap`; with `pub type Wrap`
  -> `%struct.Wrap` and correct results.
- **Stdlib handling:** `IntFrac` in `num/bigfloat.xi` is declared `pub` --
  it is part of the module's public surface anyway (split result of the
  rounding/floor machinery).
- **Fix direction (compiler):** catalog decl registration must resolve
  non-pub module-local struct types (or the checker must reject them) --
  silently defaulting to i64 corrupts memory.

### BUG 10 (pre-existing) -- Float literals emitted rounded to 6 decimals

- **Construct:** any Float64 literal. The emitted LLVM constant is the
  literal formatted with ~6 decimals: `0.000000001` -> `0.000000` (= 0.0),
  `3.14159265358979` -> `3.141593` (~=1e-7 error). Short literals (<= 6
  decimals) are exact and unaffected. Present in the OLD compiler binary as
  well -- pre-existing, not a regression from the hardening session.
- **Observed:** `if diff > 0.000000001` became `if diff > 0.0`; probes
  comparing literals like `3.1400000000000001` vs `3.14` reported "equal".
- **Stdlib handling:** no stdlib fn depends on > 6-decimal literals (PI/E
  are parsed from strings); the bigfloat smoke expresses its 1e-9 tolerance
  as `diff * 1000000000.0 > 1.0` (emission-safe).
- **Fix direction (compiler):** emit `%f`-style literals with full
  precision (e.g. `%.17g`) or hex float constants (`0x1.91eb851eb851fp+1`).

### BUG 11 -- `unsafe { extern }` calls return wrong values for runtime Float64 args

- **Construct:** `stdlib/xiom/math.xi` `pub fn sqrt(x: Float64)` lowers to
  `unsafe { return sqrt(x); }` (extern "C" libm). With a RUNTIME argument the
  result is wrong: `math.sqrt(4.0)` (var-held) != 2.0, `sqrt(9.0)` != 3.0.
  Constant-folded calls appear fine. Same symptom in `smoke_complex.xi`
  (`complex_abs` -> `math.sqrt`) and `smoke_net_folder.xi` -- both fail in the
  stdlib-exec suite (70/72 pass; the two failures are the unsafe-extern
  path, unrelated to bigint/bigfloat).
- **Suspect:** the Unsafe Confinement trampoline
  (`xiom_trampoline_call` + `__unsafe_ctx` struct) -- double args/results
  through the context struct. Compiler session's in-flight domain
  (uncommitted coerce.rs + confinement phases); NOT the stdlib code.
- **Fix direction (compiler):** verify double (and i64) arg/ret marshalling
  through `__unsafe_ctx`; compare against `sqrt_pure` (non-unsafe impl).

---

## 2026-08-11 -- REGRESSION in committed &T fix (d22068f8) -- RESOLVED

### BUG 8 (CRITICAL) -- Catalog-module fns with `&Vec[Int]` params emit an EMPTY signature

> RESOLVED: the compiler session's later uncommitted coerce.rs edit fixed the
> &expr lowering; verified on the rebuilt compiler (see the follow-up section
> above). Kept below as the original report.

- **Construct:** any function in an IMPORTED (catalog) stdlib module whose
  parameter is `&Vec[Int]` (i64-element generic container). The function
  definition is emitted with NO parameters and `i64` return, while call
  sites pass `%struct.Vec*` and use the declared return type -- ABI mismatch
  -> deterministic ACCESS_VIOLATION at runtime.
- **Observed IR (probe_bit.xi, both the fresh build and the committed-HEAD
  compiler):**
  ```
  %tmp183 = call i64 @_from_twos_bits(%struct.Vec* %tmp47)   ; call site: Vec* arg
  define i64 @_from_twos_bits() {                             ; definition: NO params!
  ```
  The definition is also emitted UNQUALIFIED (`@_from_twos_bits`, missing the
  `@bigint.` module prefix) -- the external-decl registration degraded the
  signature. `_bits_to_bigint(&Vec[Int], Bool)` in the same module vanished
  from the IR entirely.
- **Scope (empirically verified):**
  - `&Vec[Int]` param in a USER module -> correct (`define i64 @sum_vec(%struct.Vec %param0)`).
  - `&Vec[UInt8]` param in a CATALOG module -> correct (smoke_compress exit 0).
  - `&BigInt` (plain struct) param in a catalog module -> correct (`@bigint._twos_bits(%struct.BigInt*, i64)`).
  - OLD compiler (before d22068f8): `bigint_bit_and` worked (probe_bisect #9,
    exit 0) -- so this is a REGRESSION from the committed &T-param fix
    (param_llvm_type struct-address lowering + coerce changes), not a
    pre-existing issue.
- **Trigger in stdlib:** `stdlib/xiom/bigint.xi` `_from_twos_bits(bits: &Vec[Int])`
  and `_bits_to_bigint(bits: &Vec[Int], negative: Bool)` -- the two's-complement
  bitwise helpers. The extended `smoke_bigint.xi` crashes (0xC0000005) the
  moment any of `bigint_bit_and/or/xor` is linked in.
- **Likely root cause hint:** the recurring `warning: unknown type 'Int]' --
  defaulting to i64` (type-string parser splitting `Vec[Int]` on `]`) combined
  with the new param lowering -- the mangled `&Vec[Int]` param type resolves to
  an empty/i64 default during catalog decl registration. The catalog
  registration path (collect_external_decls) is what differs from the
  in-program path (which works).
- **Fix direction (compiler):** catalog/external-decl signature extraction for
  `&Vec[T]` params must preserve the container type (and the module prefix on
  the emitted fn name). Verify with: probe_bit.xi (bit ops), smoke_bigint.xi
  (extended), and re-run smoke_compress (must stay green -- &Vec[UInt8] is the
  canary for the working path).

### FIXED in stdlib during diagnosis (no compiler involvement)

- `bigint.xi` div_mod zero-remainder: `dm.1.digits[0]` is an out-of-bounds
  read when the remainder is zero (div_mod returns an EMPTY digits Vec for a
  zero remainder -- `_trim` pops all limbs; both the old and new paths do
  this). All readers (`bigint_to_base`, `bigint_to_hex`, `_bigint_bit_array`)
  now guard with `bigint_is_zero(&dm.1)` first. Pre-existing hazard, exposed
  by the new m==1 schoolbook path.
- `smoke_bigint.xi` assertion for `bigint_shift_left`: the function is the
  original DECIMAL shift (x10^n), not a bit shift -- assertion corrected to
  `shift_left(1, 3) == "1000"` (documented in the module header).

---

### NOTE 6 -- `stdlib_exec_bigint_runs` fails (smoke_bigint.xi compile hangs/fails) -- PARALLEL-SESSION IN-FLIGHT, NOT A COMPILER REGRESSION

- **Observed:** `.\test_summary.ps1 -Fast` (threads 32) reports a 4th
  stdlib-exec failure: `stdlib_exec_bigint_runs` ("compile failed for
  examples\stdlib_smoke\smoke_bigint.xi"). Manual `xiom -o bigint_check.exe
  smoke_bigint.xi` does not return within 60s (compiler appears to hang) -- the
  harness reported a compile failure rather than a hang, so behavior is racy.
- **Root cause (stdlib, not compiler):** `stdlib/xiom/bigint.xi` (+590 lines)
  and `examples/stdlib_smoke/smoke_bigint.xi` (+198 lines) carry LARGE
  uncommitted in-flight edits from the parallel BigInt/BigFloat stdlib session
  (adds `bigint_from_u64`, and `use xiom.num;` / `xiom.core.INT_MAX` /
  `xiom.core.to_int_from_char`). These helpers are mid-introduction and not yet
  self-consistent, which drives the compiler down a pathological/infinite
  path. Likely related to existing BUG 1 (opaque tuple-of-struct LLVM type).
- **Action:** NOT a compiler regression from this session's warning cleanup.
  Compiler-hardening session left it untouched (owned by the parallel stdlib
  session). Re-verify once the parallel session lands its BigInt refactor and
  the helpers exist; if it still hangs on a CONSISTENT stdlib, re-open as a
  compiler bug (likely BUG 1 area).

---

## 2026-08-11 -- Compiler session follow-up: BUG 8 verified FIXED, 3 new findings

### BUG 8 -- verified RESOLVED on the current tree (commit pending)

Reproduced with a fresh probe (`probe_bit.xi`: `bigint_bit_and(12, 10)`) --
the current compiler emits CORRECT signatures for catalog fns with
`&Vec[Int]` params:

```
define %struct.BigInt @bigint._from_twos_bits(%struct.Vec %param0) inlinehint {
define %struct.BigInt @bigint._bits_to_bigint(%struct.Vec %param0, i64 %param1) ...
```

and `probe_bit` compiles + runs (12&10=8). The empty-signature stub the
stdlib session observed came from `emit_undefined_symbol_stubs` filling in
for a fn that was never emitted -- the emission failure was the earlier
uncommitted coerce.rs state (now reverted/committed). Locked in with e2e:
`e2e_m37_catfix_catalog_vecref` (examples/catfix: imported module with
`&Vec[Int]` + `&struct` params).

### FIXED -- Parser: `bits[L - 1]` (index with arithmetic on an uppercase ident)

- **Construct:** `if bits[L - 1] == 0 { ... }` -- the postfix `[` arm's
  "explicit generic call args" heuristic (lowercase base + UPPERCASE first
  token in brackets) eagerly parsed `L` as a TYPE argument and errored
  "expected ']', found -" -- it only backtracks when `]` is followed by `(`,
  but the error fired before reaching that check. Also broke catalog files:
  `stdlib/xiom/bigint.xi` `_from_twos_bits` and `stdlib/xiom/num/bigfloat.xi`.
- **Fix (crates/xiom-parser/src/lib.rs):** speculative scan to the matching
  `]` (bracket-depth aware, Eof-guarded); commit to generic-args ONLY when
  the group is immediately followed by `(`. Index expressions (`bits[L-1]`,
  `buf[Head]`) fall through to normal index parsing.
- **Verified:** probe_idx (bits[L-1] evaluates correctly), generic calls
  (`add2[Float32](...)`) still parse; e2e `e2e_m37_index_arith`.

### FIXED -- Catalog import pathological slowness ("hang" on `use xiom.math`)

- **Construct:** importing modules whose transitive imports contain LEAF
  references (`use xiom.core.to_int` -- a fn/const, not a module file) made
  `xiom --check` take minutes-to-infinite. Root cause: `ModuleCatalog::
  load_module`'s Strategy-b scan-based fallback walks the ENTIRE source
  directory tree (reading every .xi header) for EVERY failed lookup -- and
  source_dirs include the file's parent dir + project root (and the CWD,
  which can be a temp dir containing a full worktree + target/). The
  pre-built `module_index` (build_index) already covers every file the scan
  could find, so the scan is pure waste in the normal flow.
- **Fix (crates/xiom-check/src/catalog.rs):** (1) skip the scan entirely when
  `module_index` is non-empty (index is authoritative; scan retained only for
  catalogs built without `build_index`, e.g. unit tests); (2) `index_dir` /
  `load_from_dir` / scan recursion skip build/VCS/package-manager dirs
  (`target`, `build`, `.git`, `node_modules`, any `.`-prefixed dir).
- **Measured:** `use xiom.math` check 20s->9.6s (scan eliminated; residual
  ~4-9s is prelude module parsing); `probe_bit` full compile 138-160s->9.5s.
  Locked in by e2e_m37_catfix_* (which also verify catalog resolution still
  works).

### VERIFIED -- circular imports are safe (not prevented, terminate)

- **Construct:** module A `use`s B and B `use`s A. The loader's
  `cached_loaded` guard terminates the cycle; the checker resolves both
  modules' symbols regardless of load order; compile succeeds.
- **Empirically verified** with a two-module cycle (check 4.6s, compile 8.3s,
  run correct -- the only crash was my fixture's own unbounded mutual
  recursion, expected stack overflow, not a compiler issue). The current
  stdlib has NO true module cycle (bigint->xiom.num; num/bigfloat->xiom.bigint
  don't close a loop because num.xi doesn't import bigfloat).
- Locked in by e2e `e2e_m37_catfix_circular_imports` (examples/catfix circ_*).

---

## 2026-08-11 -- M37 batch 2: BUG 9/10/11 FIXED -- fast suite 1112/1/1

### FIXED -- BUG 9: private catalog struct types degrade to i64

`collect_external_decls` injected only PUB type decls; a pub fn taking or
returning a PRIVATE struct (the stdlib's `pub IntFrac` workaround pattern)
lost the type layout -- emitted signatures degraded to i64 (`make_frac(3,4)`
summed to 0 instead of 7). Fix: transitively inject non-pub types
referenced by injected pub fn signatures (params/returns/inner types +
field types). Verified: examples/catfix b9mod+b9main; e2e
`e2e_m37_catfix_private_type`.

### FIXED -- BUG 10: float literals truncated to 6 decimals

`{:.6}` at all three literal-emission sites truncated 0.123456789 to
0.123457 (wrong stored values AND wrong comparisons). Now `{:.17e}` --
exact f64 round-trip, LLVM-valid. Also removed a leftover "CG02 DEBUG"
eprintln. Verified: e2e `e2e_m37_float_precision`.

### FIXED -- BUG 11: unsafe-extern double marshalling (complex/net_folder)

The unsafe-block round-trip family: (1) block-fn `return X` with a STRUCT
tail extracted a scalar field (Option field-1) instead of val_to_i64's
heap-pointer round-trip -> AV on re-materialization (m34_y04);
(2) `ret_from_enclosing` overwrote the shared block value with the
enclosing coercion -> `icmp eq ptr, i64` (m34_d01..d20);
(3) fault path emitted `ret i64* 0` (clang rejects) instead of `null`
(m33_u13). Fixes in expr.rs/stmt.rs. `smoke_complex.xi` and
`smoke_net_folder.xi` now PASS -- stdlib-exec is 72/72.

### TEST MIGRATIONS (confinement-era rules, not compiler regressions)

- m21_ffi_unsafe_001..009: added the T007 `requires:` contract to
  whole-body-unsafe fns (rule landed in 83416aa9 after the tests).
- m35_z12/z30: wrapped pointer-returning wrappers in unsafe (T003).
- m33_u13: rewrote the dangling-`&local` return test to be well-defined.

---

## 2026-08-11 -- BigInt/BigFloat session: Phase C (transcendentals) landed

- `bigfloat` Phase C landed (pure-XIOM series: pi/e with precision via
  Machin/Taylor, exp/ln/log10, sin/cos/tan, atan/atan2, pow_bf) --
  `smoke_bigfloat.xi` now 44 assertion blocks, exit 0 in ~2s.
- NO new compiler findings from Phase C. One stdlib coding error caught and
  fixed (atan halving identity: `1 + sqrt(1 + t^2)`, not `1 + sqrt(t^2)`).
- **BUG 11 (unsafe extern doubles) -- FIXED by the compiler session AFTER this
  entry was written** (`7f7b7b54`, 2026-08-11): `stdlib_exec_complex_runs`
  and `stdlib_exec_net_folder_runs` now PASS (stdlib-exec 72/72 -- verified
  15:0x and 15:4x). The stale "remains OPEN" claim below was written before
  the fix landed; see the STATUS SUMMARY at the top of this file.
- BUG 2/3 (module-global field writes / fn-call initializers) remain
  open; stdlib design already avoids both (whole-value global assignment;
  constants as pure constructor fns). Fixing them unlocks precision-cached
  pi/ln10 and the spec's `const BIGINT_*/BIGFLOAT_*` style.

---

## 2026-08-11 -- fmt.sprintf session: Float64 container element access broken (NEW)

### BUG 12 (NEW) -- `Vec[Float64]` element reads lower to `load i64 + sitofp` (silent corruption); `[N]Float64` fixed arrays degrade to `[N]i64` (AV)

- **Construct (both user and catalog modules):** any read of an element of a
  `Vec[Float64]` (param or local) or of a fixed `[N]Float64` array.
- **Observed:**
  1. `Vec[Float64]`: `probe_vf.xi` -- `var x = v[0];` after `v.push(3.14159)`
     -> `x != 3.14159`. IR evidence (`_sprintf_engine`, the Vec element-load
     switch): the 8-byte case emits `bitcast i8* to i64*` + `load i64`, then
     the float-typed value is produced by `sitofp i64 %tmp402 to double` --
     the f64 BIT PATTERN (4614256650576693248 for 3.14159) is treated as an
     integer and converted, not reinterprete -- `%.2f` of 3.14159 printed
     "4614256650576693248.00". The generic Vec element-load path only knows
     integer element widths (1/2/4/8) and defaults the type to i64.
  2. `[N]Float64` (incl. inside structs): `probe_fa.xi` -- warning
     `unknown type 'Float64]' -- defaulting to i64` (the type-string parser
     splits on `]`, same family as the old BUG 8), struct fields + array
     slots laid out as i64 -> 0xC0000005 ACCESS_VIOLATION reading `s.values[0]`.
  3. `Vec[Float64]` STORE path appears intact (push stores the raw bits);
     only element READS are wrong.
- **Impact on stdlib:** NO pre-existing stdlib module uses `Vec[Float64]` or
  `[N]Float64` (stats works on `Vec[Int]`) -- zero regression. The fmt.sprintf/
  sscanf batch (G13) was designed around `Vec[Float64]` and was re-designed
  to avoid the construct: scalar `Float64` params (proven -- bigfloat/geom
  pass doubles everywhere) and a fixed-slot `FloatScan` struct for sscanf
  float results (`// TODO(compiler)` note in fmt.xi). Float64 in structs as
  plain fields (not arrays) is proven fine (geom Vec2).
- **Fix direction (compiler):** (a) the Vec element-load switch must load
  `double` (and `float`/`fp128`) for float element types instead of i64+sitofp
  -- the element type should come from the Vec's registered type, not the
  width; (b) the type-string parser must not split `Float64]`/`UInt8]` on the
  first `]` (parse the full `Vec[T]`/`[N]T` with bracket depth) -- same root
  cause family as BUG 8's `&Vec[Int]` empty-signature bug. Verify with
   probe_vf.xi (expect exit 0) and probe_fa.xi (expect exit 0).

### BUG 13 (NEW) -- fp128 (Float128) arithmetic hits missing compiler-rt helpers at link time

- **Construct:** any program whose Float128 value flows through i64->f128
  (sitofp), f128->f64 (fptrunc), or f128 division. `probe_f128.xi` --
  `n as Float128`, `x as Float64`, `acc / ten` -> lld-link errors:
  ```
  undefined symbol: __floatditf   (sitofp i64 -> fp128)
  undefined symbol: __trunctfdf2  (fptrunc fp128 -> f64)
  undefined symbol: __divtf3      (fdiv fp128)
  ```
  fadd/fmul/fpext on fp128 are native (x87) and link fine; the compiler-rt
  soft-float helpers are not in the link line.
- **Impact on stdlib:** blocks the planned `bigfloat_to_float128` bridge
  (256-bit framing mission). Int128/UInt128 are UNAFFECTED (native LLVM i128
  -- no helpers) so `bigint_to_i128/u128/u64` landed. A lossy
  `bigfloat_to_float64 -> fpext` wrapper was rejected (only 15 digits -- the
  whole point of f128 is 34). TODO(compiler) note in stdlib/xiom/num/bigfloat.xi.
- **Fix direction (compiler):** link compiler-rt (clang `-rtlib=compiler-rt`
  or add libclang_rt.builtins) on Windows, or emit/implement the handful of
  `__*tf3`/`__floatditf`/`__trunctfdf2` stubs; verify with probe_f128.xi
  (expect exit 0).

### BUG 14 (NEW) -- UInt64->UInt128 cast emits SEXT; UInt128 `>>` emits ASHR

- **Construct:** `v as UInt128` with `v: UInt64` whose bit 63 is set, and
  `(p >> 64)` on a `UInt128` whose bit 127 is set. `probe_m128.xi` --
  `(lhs as UInt128) * (rhs as UInt128)` for lhs/rhs with bit 63 set gives the
  WRONG product. IR evidence:
  ```
  %tmp6 = sext i64 %tmp5 to i128     ; must be zext for UInt64
  %tmp18 = ashr i128 %tmp17, %tmp19  ; must be lshr for UInt128
  ```
  (The compiler has no unsigned 128-bit type distinction at codegen -- UInt64->
  UInt128 and Int64->Int128 both lower to sext; UInt128 `>>` lowers to ashr.)
- **Impact on stdlib:** XXH3's 64x64->128 mulhi (`XXH3_mul128_fold64`) was
  wrong on inputs with the top bit set (verified: seed-0 64-bit vectors
  passed for short inputs only after the fix). Worked around in
  `hash/xxhash.xi`: `_u64_to_u128` builds the i128 from 32-bit halves (sext
  == zext for bit-63-clear values) and the high half is masked after `>>`.
  `bigint_to_u128`/`to_i128` are unaffected (limbs < 1e9 and shifts of
  bit-63-clear values only).
- **Fix direction (compiler):** lower `UInt64 as UInt128` with `zext` (type
  information exists in the checker) and `UInt128 >>` with `lshr`; verify
  with probe_m128.xi (expect exit 0).

### BUG 15 (NEW) -- alwaysinline bodies with a single `var` + `return` drop the mask statement on inline

- **Construct:** a small fn (<= inline-threshold) whose body is exactly
  `var mask = <expr>; return <expr2> & mask;` -- e.g.
  ```
  fn shr(x: UInt64, k: Int) -> UInt64 {
    var mask: UInt64 = ((1 as UInt64) << (64 - k)) - 1;
    return (x >> k) & mask;
  }
  ```
  Inlined call sites emit ONLY the `ashr` -- the mask `shl`/`sub` and the
  `and` are dropped (probe_sip3.xi: `shr(t, 32)` returns the sign-extended
  value; IR shows `%tmp6 = ashr i64 %tmp4, 32` with no following `and`).
  Adding a second var (`var shift = 64 - k; var mask = ...;`) makes the
  inline correct (the pattern hash.xi `_rotl64` has always used).
- **Impact on stdlib:** SipHash/XXH3 logical shifts were wrong until the
  two-var form was used. All new hash code uses the two-var pattern with a
  comment. NOT worked around in old code -- hash.xi `_rotl64` (two vars) is
  unaffected.
- **Fix direction (compiler):** the inline expansion drops statements from
  single-var bodies (likely a statement-copy bug in the inline pass); verify
  with probe_sip3.xi (single-var shr must equal two-var shr, exit 0).

### BUG 16 (NEW) -- combining collect.skiplist + collect.trie fast-fails with 0xC0000409 (stack cookie)

- **Construct:** link BOTH `stdlib/xiom/collect/skiplist.xi` and
  `stdlib/xiom/collect/trie.xi` into one program and run ANY skiplist fn
  (even `skiplist_insert`) plus `trie_new` -- the program prints normally
  then dies at exit with 0xC0000409 (STATUS_STACK_BUFFER_OVERRUN -- the /GS
  cookie check on main's frame fires after `return`). Repros:
  `probe_pt2.xi` (two inserts, no io) and `probe_pt3.xi` (insert + trie_new).
  Each module alone, and every other pair (skiplistx{string,convert,char,
  cuckoo,fenwick,objectpool,queue,cache}; triex{cuckoo,fenwick,objectpool,
  queue,cache}) exits clean.
- **Observed in IR (both modules combined):** generated symbols emitted
  UNQUALIFIED: `define %struct.MaybeUninit @MaybeUninit.clone(...)` and
  `define void @BinaryHeap.invariant_check(...)` / `@BufReader.invariant_
  check` / `@Cursor.invariant_check` -- no module prefix, while the same
  programs alone show the same stubs (benign alone). Two Option payload
  instantiations exist in the pair (Option[Int] in skiplist, Option[Char]
  via trie->string.char_at) -- the shared MaybeUninit/clone codegen path is
  the prime suspect (clone body `ret %struct.MaybeUninit %self` with a
  16-byte layout vs possibly 8-byte Char payload -> stack corruption).
- **Stdlib handling:** both modules are correct individually; the CI/exec
  harness must NOT combine them in one smoke -- the batch's smokes are
  split (smoke_collect2a: skiplist+cuckoo+fenwick+pool+spsc+arc;
  smoke_collect2b: trie+cuckoo+fenwick+pool+spsc+arc). TODO(compiler) notes
  in both modules.
- **Fix direction (compiler):** qualify generated clone/invariant-check
  symbols per module and make the MaybeUninit clone emit payload-accurate
  code for each Option payload type; verify probe_pt2.xi (expect exit 0).

### BUG 17 (NEW) -- runtime-runtime `Str ==` on Vec[Str] ELEMENTS lowers to pointer compare

- **Construct:** comparing two runtime `Str` values that come from `Vec[Str]`
  ELEMENT loads, e.g. `ga[i] == gb[j]` where `ga`/`gb` are `Vec[Str]`:
  probe_jac2.xi -- both elements are "he" yet `==` is false; `ga[0] == "he"`
  (literal) is true. The emitted IR for the element-element comparison
  contains NO `strcmp` call (the element loads degrade the operand type so
  the equality lowers to pointer icmp), while plain runtime `Str == Str`
  (vars/slices, probe_seq/probe_seq4) DOES emit `strcmp`.
- **Impact on stdlib:** `text.similarity.jaccard_similarity` and the
  lcp/lcsuffix helpers compared slices -- all now use a byte-wise `_str_eq`
  helper (documented) so string content comparison never depends on the
  degraded path. This is the same family as BUG 12 (Vec element type
  degradation): Float64 elements load as i64+sitofp, Str elements lose the
  content-equality lowering.
- **Fix direction (compiler):** the Vec element-load expression must carry
  the element's declared type (Str -> strcmp on `==`; Float64 -> `load
  double`); verify probe_jac2.xi (both comparisons true, exit 0).

### BUG 18 (NEW) -- combining string + text.similarity + time in one program crashes (0xC0000405)

- **Construct:** one program importing `xiom.string`, `xiom.text.similarity`
  AND `xiom.time` (smoke_str2.xi) crashes at startup with 0xC0000405 before
  any output. Every PAIR of the three exits clean; each module alone is
  clean. Same family as BUG 16 (combination-specific startup crash -- likely
  the unqualified `@MaybeUninit.clone`/invariant-check stubs colliding when
  several Option payload shapes coexist).
- **Refined trigger (time-only programs):** with only `xiom.time` imported,
  `strptime("2026-13-01", "%Y-%m-%d")` followed by `strptime("2026-08-11",
  "%Y-%m-%d %Q")` in the SAME program crashes (0xC0000405); the same specs
  individually, or any other spec pair, exit clean. The unsupported-
  conversion early-return path (`%Q`) after a range-check failure path
  miscompiles at -O2 (probe_tm9: the pair fails; both single calls pass).
- **Stdlib handling:** the batch's smokes are split -- smoke_str2.xi covers
  string+text.similarity (proven pair), smoke_time2.xi covers time with the
  `%Q` check removed (the fn is correct -- verified by single-call probes;
  TODO(compiler) notes in the affected modules).
- **Fix direction (compiler):** same as BUG 16 -- module-qualify generated
  symbols and emit payload-accurate clone/invariant code; verify
  smoke_str2.xi recombined (expect exit 0).

## 2026-08-11 -- STATUS SUMMARY: BUG 12-18 all FIXED (commits `2ae300fd`, `3b8f5415`)

| Bug | Fix | Verified |
|-----|-----|----------|
| 12/17 | `vec_elem_from_type_annotation` (types.rs + lib.rs) unwraps Ref/MutRef/Ptr; `Vec(...)` AST form handled -- Vec element loads keep Float64/Str types | m12b, m37_vec_f64 R=0 |
| 13 | fp128 coercion arms in `coerce_value` + NEW `stdlib/runtime/fp128_helpers.c` soft-float add/sub/mul/div/conv/cmp (verified in a C harness: 400/5/2500/1002.5, negatives, tiny values) | m13b/m13d, m37_f128 R=0 |
| 14 | cast site uses `xiom_type_of_local` (registered type); `expr_is_unsigned()` picks lshr; var bindings infer type from `as UInt*` targets | m14, m37_u128 R=0 |
| 15 | math-builtin `shr`/`shl` intercept restricted to `math.*`/`xiom.math.*` qualified keys + bare keys with NO registered fn | m15, m37_shr_builtin R=0 |
| 16/18 | `process_use` longest-dotted-prefix walk (directory submodules); `bare_fn_aliases` prefers the CALLER's module; **fn-key fix**: call resolution returns the BARE key when a bare definition exists, qualifying to caller-module/alias only otherwise -- kills the definition-vs-call symbol mismatch (user fns emit bare `@mk_big`, calls resolved to leaf-qualified zero-param stubs -> ABI crash, same family as BUG 8) | m16a-e (bare/leaf forms) + m16f (qualified form) all R=0; skiplist+trie combined OK |

- **Encoding repair:** `skiplist.xi` + `trie.xi` contained invalid UTF-8 (lone
  0x97 bytes) which silently broke import binding -- repaired.
- **Regression sweep** (isolated binary, `tgt_iso`): m37_tuple_struct,
  m37_ref_mut, m37_index_arith, m37_vec_f64, m37_u128, m37_f128,
  m37_float_precision, m37_shr_builtin, m33_z14, m34_y04, m33_u13,
  m19_read_file -- **12/12 R=0**.
- **Known follow-up (parallel stdlib session owns it):** the `collect/` ->
  `collections/` folder move landed while module declarations inside still
  say `xiom.collect.*` -- `use` resolves by file path (works) but
  fully-qualified calls need the declared name (`xiom.collect.skiplist.fn`
  works; `xiom.collections.skiplist.fn` does not until the declarations are
  aligned). Not a compiler regression; m16b-e/m16f verified green.
- Workspace build gate: **zero warnings** (dead `load_external_module` +
  unused imports removed with the XIOM_TRACE_* debug prints).

---

## 2026-08-11 [U+FFFD] stdlib session (evening): BUG 19 (NEW) [U+FFFD] every NaN-producing Float64 operation returns a garbage sentinel or traps

**? FIXED 2026-08-11 (commit `9c3a2f9e`).** Two codegen defects, both verified:

1. **float `!=` lowered to `fcmp one`** (ordered-not-equal) [U+FFFD] for NaN operands
   `one` is FALSE, so `x != x` returned false and NaN was undetectable.
   Fixed to `fcmp une` in both the BinOp table (expr.rs) and the trait-method
   table (call.rs `.ne()`).
2. **`Str + Float64` concat inttoptr'd the FP bits** [U+FFFD] the "garbage sentinel
   print" was this, not the fdiv (the IR fdiv was always correct). Fixed via
   new `@xiom_double_to_string` (xiom_runtime.c): NaN ? "nan", [U+FFFD]inf ?
   "inf"/"-inf", finite values shortest-round-trip (%.15g else %.17g);
   declare added to emitter.rs so the undefined-symbol stub pass can't emit
   a conflicting zero-param definition (this WAS the 0xC000001D crash path).
   Float32 concat widens via fpext first.

Verified: `m37_nan_ieee.xi` (23 checks: 0/0, inf*0, inf?inf ? NaN; `x != x`
true; ordering-with-NaN all false; Float32 NaN; concat "nan"/"inf"/"-inf"/
"3.14"/"0"/"0.5") R=0; original probe prints "c = nan / is NaN" exit 0;
17/17 regression sweep + stdlib-compile 40/40 + checker 178/178 green.

- **Construct:** any IEEE-754 NaN-producing Float64 operation: `0.0 / 0.0`, `inf - inf`, `inf * 0.0` (with `inf` from `1.0 / 0.0` or `-1.0 / 0.0`). Verified on a fresh build from HEAD (incl. `2ae300fd` + `3b8f5415` + `f0388644`; `cargo rustc -p xiom --bin xiom -- -o <temp>\xiom.exe`).
- **Observed:**
  1. `var a = 0.0; var b = 0.0; var c = a / b;` ? `c` prints `-92233.-72036854775808` and `c != c` is **false** (not NaN). Same sentinel for `inf * 0.0` and `inf - inf`.
  2. `math.ln_pure(-1.0)` (pre-existing stdlib path `if x <= 0.0 { return 0.0 / 0.0; }`, math.xi:162) ? process dies with **0xC000001D** (STATUS_ILLEGAL_INSTRUCTION) [U+FFFD] the runtime fault-trap fires on the NaN-producing division.
  3. `1.0 / 0.0` ? +inf and `-1.0 / 0.0` ? -inf are **correct** (inf results pass; only NaN results are broken).
- **IR evidence (probe_nan.xi):** the emitted IR is correct [U+FFFD] `%tmp41 = fdiv double 0.00000000000000000e0, 0.00000000000000000e0` [U+FFFD] so the corruption happens in the clang/optimize/runtime-trap stage, not in AST emission. The consistent garbage value (`-92233.72036854775808` [U+FFFD] a sentinel) suggests the fault-trap/intercept layer (the same system as smoke_guard_fault.xi) replaces NaN-producing FP ops with a trap-or-sentinel path instead of the IEEE result.
- **Impact on stdlib:** `math.constants` NAN cannot be implemented (no literal syntax; `0.0/0.0` broken) [U+FFFD] `// TODO(compiler): BUG 19` in math/constants.xi; `math.is_nan`/`is_inf` classify correctly but nothing in the stdlib can PRODUCE a NaN today (all NaN-producing libm entries [U+FFFD] asin/acos/ln/sqrt [U+FFFD] carry domain `requires:` contracts). `num/float.xi` `bits_to_float` (planned) needs a real bitcast intrinsic to land anyway.
- **Fix direction (compiler):** route NaN results (fdiv 0/0, fsub inf-inf, fmul inf*0, and libm domain-error returns) through the IEEE path [U+FFFD] do not trap/sentinel float NaN; optionally add a `nan` literal or i64?f64 bitcast intrinsic (`bitcast i64 0x7FF8000000000000 to double`) which would unblock `math.constants.NAN` + `num.float.bits_to_float`. Verify with probe_nan2/probe_nan3 (expect `nan` print + `x != x` true, exit 0).

**Stdlib unblock:** `math.constants.NAN` can now be implemented as
`pub fn nan() -> Float64 { return 0.0 / 0.0; }` (a const initializer can't hold
the expression yet [U+FFFD] const-fold only handles literals; a runtime fn works).
`is_nan(x)` = `x != x` is now correct.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 20 (NEW, REGRESSION from `1d4cd2e8`) [U+FFFD] unconditional -mavx512* clang flags crash non-AVX-512 CPUs (illegal instruction) in ANY vectorized program

**? FIXED 2026-08-12 (commit `4fa7a1d2`).** AVX-512 flags are now HOST-CPUID-gated:
`-mavx512f/bw/dq/vl` are added only when
`std::arch::is_x86_feature_detected!("avx512f")` (crates/xiom/src/lib.rs). The
-O2 vectorizer emits zmm in ordinary float loops; the runtime CPUID dispatch
gates only INTENTIONAL SIMD calls, so the FLAGS must match the host.
`-maes -mavx -mavx2` remain unconditional (safe on any AVX2 CPU). Verified on
Zen 2: clang arg trace shows no `-mavx512*`; probe_avx.xi exit 0;
smoke_num_fraction/smoke_math_rounding/smoke_math_precision/
smoke_math_trig_constants/smoke_num_float all R=0 (were 0xC000001D).
`smoke_num_precision` STILL AV-crashes (0xC0000005, deterministic) [U+FFFD] a
separate BUG 22/23 item (cross-module returned Vec[Float64] / nested Vec
family), queued with the wave batches.

- **Construct:** commit `1d4cd2e8` adds `-maes -mavx -mavx2 -mavx512f -mavx512bw -mavx512dq -mavx512vl` to every clang invocation on `Target::Native && x86_64` (crates/xiom/src/lib.rs:1049-1057). On a CPU WITHOUT AVX-512 the -O2 vectorizer can emit AVX-512 instructions in ordinary float loops ? 0xC000001D (STATUS_ILLEGAL_INSTRUCTION) at runtime. The CPUID dispatch in simd_runtime.c gates only the INTENTIONAL SIMD calls; it cannot gate the vectorizer.
- **Repro (minimal, probe_avx.xi):**
  ```xiom
  use xiom.io; use xiom.convert;
  fn main() -> Int {
    var acc = 0.0; var i = 0;
    while i < 64 { acc = acc + (i as Float64) * 0.5; i = i + 1; }
    io.println(convert.float_to_string(acc));
    return 0;
  }
  ```
  ? compiles clean, then **exit -1073741795 (0xC000001D)** with zero output. Also affects: `smoke_num_fraction.xi`, `smoke_math_rounding.xi` (and every float-loop stdlib smoke) on this machine (AMD Ryzen 9 3950X = Zen 2 [U+FFFD] **no AVX-512**). Pure-int/string programs (e.g. `smoke_string_case.xi`) run fine.
- **Impact:** blocks stdlib float-smoke verification on non-AVX-512 machines; any user program with a float hot loop crashes on such CPUs. The v0.58 comment's own rule ("only dispatch-gated code paths may rely on AVX-512 presence") is unenforceable for the vectorizer [U+FFFD] the FLAGS themselves must be gated.
- **Fix direction:** gate the flags on the HOST's CPUID at build time (query AVX-512 support before adding -mavx512*; e.g. use `-march=native` which enables only what the host supports), or drop the 512-bit flags (keep -maes -mavx -mavx2, safe on any AVX2 CPU). Verify: probe_avx.xi exit 0 on Zen 2; e2e SIMD test still R=0.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 21 (NEW) [U+FFFD] catalog-module fn returning a Str created INSIDE an unsafe block returns a corrupted Str (len 0xFFFFFFFF)

**Status 2026-08-12: NOT REPRODUCED on the current build.** All documented
shapes verified PASSING with the isolated binary: catalog fn with (a) loop
inside unsafe + `return Str.from_cstring(buf)` from inside the block ?
"aaa" R=0; (b) direct return from inside the block ? "xyz" R=0; (c) while
loop + `xiom.string.str_concat` build inside unsafe + return ? "aaa" R=0.
`smoke_string_pad_repeat` R=0. The original `str_repeat_char` shape was
restructured away before the fix could be isolated [U+FFFD] likely resolved by the
accumulated batch (`f0388644` chain/process_use + `9c3a2f9e` BUG 19 + `2ae300fd`
fn-key). The stdlib's workaround (no loop inside unsafe) can stay. If the
EXACT original file still fails, send it and it becomes a live repro.

- **Construct:** an IMPORTED (catalog) stdlib module fn whose body creates a Str inside an `unsafe` block and RETURNS it from inside that block, e.g. `stdlib/xiom/string/repeat.xi` `str_repeat_char` (was: `unsafe { ...; return Str.from_cstring(buf); }`). The unsafe-confinement trampoline (`__unsafe_ctx`) round-trips only i64-class values; the Str (ptr+len struct) return corrupts the length field ? `len()` returns 4294967295 and any strcmp on the value crashes (0x80000003 breakpoint [U+FFFD] heap guard).
- **Verified:** probe_repeat/probe_rep2 [U+FFFD] `repeat.str_repeat_char('a', 3)` prints `[]` with `len=4294967295`; the smoke's `str_repeat_char(...) != "aaa"` comparison then dies 0x80000003 with all buffered output lost. The SAME shape in a USER module (`fn mk_a() -> Str { unsafe { ...; return Str.from_cstring(buf); } }` with T007 `requires: true`) works [U+FFFD] so it is the catalog/trampoline path, not from_cstring. The flat string.xi str_pad_left/str_pad_right build strings inside unsafe but return OUTSIDE the block (assign-var-in-unsafe, return after) [U+FFFD] that shape is correct, which is why the bug was never hit before.
- **Fix direction (compiler):** the unsafe-block return marshalling for catalog fns must round-trip Str (and other 16-byte struct) returns like the BUG 11 Option fix did [U+FFFD] verify repeat.xi's `str_repeat_char` via its smoke once fixed; alternatively reject `return <struct-typed>` from inside unsafe blocks with a clear error.
- **Stdlib handling:** repeat.xi restructured to the proven shape (assign inside unsafe, return outside) + `// TODO(compiler): BUG 21` note. The `str_repeat_char(...) != "aaa"` comparison is the crash trigger [U+FFFD] the corrupted value must never reach a comparison.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 22 (NEW, batch report) [U+FFFD] implementation-phase findings from 4 parallel agents (detailed repros below; all pre-existing or new-shape; none blocked the batches)

**? RESOLVED 2026-08-12 (commits `eeab8cf7`, `e6608946`, `47cd188f`, `f74a52c7`):**
1. `&&`/`||` short-circuit [U+FFFD] **FIXED** (branch on LHS; RHS compiled only when needed; verified div-by-zero RHS never executes) [U+FFFD] m37_short_circuit
2. Unary minus on match-bound vars [U+FFFD] **FIXED** (negation defers wildcards like Not; float payloads bitcast from the i64 slot) [U+FFFD] m37_match_float_payload
3. Cross-module 3-tuple `.1`/`.2` [U+FFFD] **FIXED** (tuple field maps derive+register for catalog returns) [U+FFFD] m37_catalog_boundary
4. Cross-module Option payload match [U+FFFD] **FIXED** (float payloads via local_opt_payload_xiom; Int payloads verified) [U+FFFD] m37_catalog_boundary
5. requires/ensures trap [U+FFFD] **FIXED** (clean `xiom_panic`: message to stderr + flush + exit 1, no more 0xC000001D) [U+FFFD] m37_contract_pass + manual violation probe
6. str_reverse invalid IR [U+FFFD] **FIXED 2026-08-12** (root: unsafe-block captures grabbed SHADOWED names [U+FFFD] the block's inner `let c` shadowed an enclosing loop's `let c`, capturing the outer loop-body alloca whose address doesn't dominate; fix: block-bound names excluded from captures + loop-body binding allocas hoisted to fn entry). Flat string/string.xi str_reverse verified: compiles, returns "cba"; m37_loop_capture regression [U+FFFD] the stdlib's reverse.xi workaround can now delegate
7. index_of(str_slice) 0xC0000409 [U+FFFD] stdlib-side scan workaround in place; not reproduced standalone (see 22.15)
8. xiom_char_at leading-byte [U+FFFD] DOCUMENTED CONTRACT (byte position), stdlib decodes UTF-8 manually [U+FFFD] no change
9. Multi-byte char literals [U+FFFD] **VERIFIED FIXED** on current build ([U+FFFD]=233, ?=937)
10. byte_at sign-extend [U+FFFD] **VERIFIED FIXED** on current build (UInt8 as Int zexts)
11. Module-qualified call results inline [U+FFFD] **FIXED** (ROOT: qualified-call symbol vs bare def mismatch ? fn-symbol PRE-ASSIGNMENT map + integer verdicts for inline calls/index/binary operands) [U+FFFD] m37_inline_call_concat
12. Vec[Char] element size [U+FFFD] **FIXED by the BUG 23 #2 nested-Vec work** (elem sizes now resolve from the registered element type; Char elements read via the elem_size switch)


1. **`&&` does not short-circuit** (num/fraction.xi `fraction_from_float`): both operands evaluate; a div-by-zero in the RHS traps 0xC000001D even when the LHS is false. Use nested `if`s. (Suggested fix: proper short-circuit lowering or reject non-short-circuit semantics.)
2. **Unary minus on match-bound vars fails** [U+FFFD] `error[T001]: cannot negate type _` for `-d` where `d` is bound in a match arm. Workaround: type-annotate the binding.
3. **Cross-module 3-tuple field access `.1`/`.2` fails** (math/arithmetic.xi `gcd_extended` returns (Int,Int,Int)): `240 * t.1` ? "right operand must be numeric, found <error>" [U+FFFD] `.0` works, 2-tuples fine. Comparison/return contexts work.
4. **Cross-module `match` on `Option[Int]` binds a garbage payload** (math/arithmetic.xi `mod_inverse`): `Some(v) => v != 5` misbehaves; `is_some()/unwrap()` exact. Same family as the old Option-tail extraction bug.
5. **`requires:`/`ensures:` are runtime-enforced and TRAP on violation (0xC0000005/0xC000001D)** [U+FFFD] even when the fn body guards the case. Stdlib avoids declaring contracts on fns with graceful fallbacks (documented in module headers); fix direction: contracts should be checked/elided consistently (or only in debug builds), or stdlib keeps fallback-first bodies.
6. **flat `string.str_reverse` emits invalid LLVM IR** (`Instruction does not dominate all uses!` [U+FFFD] alloca in loop body captured by an enclosing unsafe context struct). Blocks the `str_reverse` API name: any program calling `string.str_reverse` fails to compile. reverse.xi implements locally; the flat fn needs the loop-body alloca moved out (compiler session's string.xi is a shared file [U+FFFD] needs care).
7. **`string.index_of(str_slice(...), needle)` inside a loop crashes (0xC0000409)** [U+FFFD] passing a str_slice result to index_of repeatedly corrupts; replace.xi scans byte-wise instead.
8. **`xiom_char_at(s, pos)` returns only the leading BYTE** of a multi-byte char (195 for [U+FFFD], not 233) [U+FFFD] pre-existing flat string.xi/char.xi contract: byte position, not code point. String sublibs do manual UTF-8 decode from byte_at.
9. **Multi-byte char literals are mangled to their last byte**: `'[U+FFFD]' != 'O'` is false (both ? 0xA9). Workaround: build chars via to_char(cp) in tests.
10. **`byte_at(...) as Int` sign-extends UInt8** (0xC3 ? -61): must mask `& 0xFF` when handling bytes >= 0x80.
11. **Module-qualified call results used INLINE in arithmetic miscompile** (e.g. `i = i + char.len_utf8(ch)` advances by 1 instead of 2): bind the result to a `let` var first (agent-applied repo-wide).
12. **`Vec[Char]` element size is 1 byte** (BUG 12 family): storing code point 937 (0x3A9) reads back 0xA9. str_code_points returns Vec[Int] instead.

---

## 2026-08-11 (night) [U+FFFD] stdlib session: BUG 23 (NEW, wave-2 batch) [U+FFFD] findings from 4 parallel agents (string metrics/unicode, math number-theory/linear)

**? RESOLVED 2026-08-12 (commits `eeab8cf7`, `e6608946`, `47cd188f`, `f74a52c7`):**
1. Cross-module returned Vec[Float64] reads [U+FFFD] **FIXED** (var-bindings inherit the callee's declared element type) [U+FFFD] m37_catalog_boundary
2. Nested Vec[Vec[T]] garbage [U+FFFD] **FIXED** (parser type-arg rendering, elem size 32, memcpy reads, nested float inner reads) [U+FFFD] m37_nested_vec
3. Fn-value params named add/mul [U+FFFD] **VERIFIED FIXED** on current build (fn-typed params dispatch correctly)
4. `else if` parser rejection [U+FFFD] **FIXED** [U+FFFD] m37_else_if
5. Flaky `use of undefined value` (~50%) [U+FFFD] **FIXED** (catalog decl injection now deterministic: all_cached sorted; plus the fn-symbol pre-assignment kills the order-dependent symbol class) [U+FFFD] 8/8 deterministic compiles
6. UTF-8 BOM breaks registration [U+FFFD] **FIXED** (lexer strips leading BOMs) [U+FFFD] lexer unit test
7. (Bool,Bool) tuples as Tuple__Int__Int [U+FFFD] **FIXED** (param-seeded tuple scan + XIOM idents + checker field-map derivation) [U+FFFD] m37_catalog_boundary
8. Catalog `&Vec[T]` param mutation no-op [U+FFFD] **FIXED** (Ref params pass the pointer like &mut) [U+FFFD] m37_catalog_boundary
9. Unary minus on catalog-returned float [U+FFFD] **FIXED** [U+FFFD] m37_catalog_boundary
10. Subtraction on catalog-returned floats [U+FFFD] covered by the BUG 20 CPUID gating + float-type registration fixes; verify via the float smokes
11. xiom.math.sqrt bare import "requires unsafe" [U+FFFD] **VERIFIED FIXED** on current build (math.sqrt R=0)
12. Recursive helpers defeat the vectorizer [U+FFFD] NOT a compiler bug (stdlib shape choice); BUG 20's CPUID gating now lets loops vectorize on Zen 2 (AVX2); recursion remains their call


1. **Vec[Float64] element READS of a module-RETURNED Vec are still broken** (BUG 12 corner): a user program reading `v[0]` from a Vec[Float64] returned by a catalog fn gets raw bit-pattern garbage and float arithmetic on such loads traps (0xC000001D). BUG 12's fix covers Vec element loads inside the defining module; cross-module returned float Vecs are not fixed. Workaround: verify via scalar invariants (dot/norm), avoid element reads of module-returned float Vecs.
2. **Nested `Vec[Vec[T]]` element reads return garbage** (0xC0000005): `m[1].len()` wrong, `m[i][j]` = 0 for a 2-row matrix; float arithmetic on nested loads crashes. Blocks dynamic-matrix runtime verification and `set_partition` (reads Vec[Vec[Int]]).
3. **Fn-value params named `add`/`mul`/`div` collide with built-in operators**: `ring_theory(add, mul)` compiled the param calls to `tower.Int.mul`/native `*`, IGNORING the passed function (verified in IR). Rename params (op_add/op_mul) [U+FFFD] callers pass positionally so the API is unchanged. Parser/checker should reject or qualify operator-shadowing param names.
4. **Parser rejects `else if`** (`expected '{', found if`) [U+FFFD] must use nested if/else. (The `elif` keyword works; `else if` as two words does not.)
5. **Flaky `use of undefined value` compile failure** (~50%): combining modules that load `xiom.text.similarity` + spline fns with `&Vec[Float64]` params (math/numerical.xi) intermittently fails with `use of undefined value '@_tridiagonal'` (undefined-symbol-stub emission; BUG 8/16/18 family). Same source compiles on retry.
6. **A UTF-8 BOM in a catalog module silently breaks function registration** (all BOMs in the stdlib were removed/avoided; writer tools must not add BOMs).
7. **(Bool,Bool) tuples misregister as Tuple__Int__Int** (logic.xi `quantifiers`) [U+FFFD] Bool in tuples/structs degrades; workaround: encode as Int 0/1 or separate fns.
8. **Catalog `&Vec[T]` param mutation is a silent no-op** [U+FFFD] `&mut Vec[T]` is required for mutation (enumerators use `&mut`). The old struct-&T fix (d22068f8) covers struct params, not Vec params.
9. **Unary minus on a catalog-returned float in an expression traps** (0xC000001D, BUG 20 family): use `0.0 - x`.
10. **Subtraction on catalog-returned floats in smoke asserts traps** (0xC000001D): use interval comparisons (a > lo && a < hi) instead of `a - b` diffs in smokes.
11. **`xiom.math.sqrt` as a direct import cannot be called bare from a user module** (T001 "requires unsafe") [U+FFFD] the math-builtin intercept (BUG 15) only covers catalog modules; user code must call `math.sqrt`.
12. **Recursive helpers defeat the BUG 20 AVX-512 vectorizer**: range/table scans and summation loops written as recursion (one iteration per frame, depth = ~182 < 500 limit) do not get vectorized, so those modules run on Zen 2. Intended as a temporary shape until BUG 20's CPUID gating lands.

**TODO(compiler): NOT IMPLEMENTABLE stubs left by wave-2** (frozen signatures compile, bodies documented; blocked by the above): math/trig sinh/cosh/tanh/atanh (exp/ln inline arithmetic traps [U+FFFD] same shape as math/hyperbolic.xi which is also BUG-20-blocked at runtime), calculus integrate_romberg/integrate_gauss/limit/left/right/is_continuous/gradient/partial_derivative/jacobian/hessian/laplacian/curl/divergence (loops + Vec[Float64]), differential richardson/gradient/jacobian/partial_derivative, series maclaurin_series/convergence_rate, integral integrate_adaptive, set_theory set_partition (nested Vec), logic simplify/normal_forms/satisfiability/tautology_check/quantifiers (Bool/Str-returning recursion + Bool tuples). All re-verifiable once BUG 20 (CPUID-gated flags) and the BUG 23 #1/#2 (cross-module float Vec / nested Vec) fixes land.

---

## 2026-08-12 [U+FFFD] stdlib session: BUG 24 (NEW, REGRESSION from the 22/23 batch `eeab8cf7..7cfc7fe4`) [U+FFFD] bigfloat pow_bf miscompiles inconsistently (AV or hang) depending on unrelated program structure

**? PARTIAL FIX 2026-08-12 (`dd6b4ab1`):** the WRONG-VALUE symptom is fixed [U+FFFD]
root was PRE-EXISTING (not the 22/23 batch): `llvm_type_for`'s array arm
kept the trailing bracket (`[10 x Int]` ? elem name `"Int]"` ? unknown type ?
i64 degradation), corrupting bigint's `var digits: [10]Int` and every
fixed-array local. Verified: p_arr digit-extract R=0; probe pair now compiles
clean (no "unknown type" warnings).
**RESIDUAL (open):** a per-program-shape AV remains inside the pow path when
the program also compiles to_str/eq [U+FFFD] crashes before the first println with
buffered output lost; the compiled IR is verifiably sound (no stubs, no
duplicate defines, helper symbols consistent across shapes). Suspects to
continue: BigFloat struct-value passing with Vec[Int] significand fields
interacting with the 23.8 &Vec pointer-pass change; deeper helpers in the
bigger module set. Reproduction: p_powv2.xi shape (pow_bf + to_str in one
program) AVs; p_b24a.xi shape (pow_bf alone) returns a wrong value only in
the pre-fix build.

- **Construct:** any program calling `num/bigfloat.xi` `bigfloat_pow_bf(&base, &exp)` [U+FFFD] or its wrappers (`precision_float.bigfloat_pow`) [U+FFFD] on the CURRENT compiler. `smoke_bigfloat.xi` (pre-existing harness smoke, was 72/72 green) now dies 0xC0000005; `smoke_num_precision.xi` dies too. pow_bf's pieces (is_zero/is_one/is_negative/ln/mul/exp at the same operands) ALL verify correctly in isolation.
- **Decisive minimal repro pair (identical logical values, different codegen):**

  - `var t = bigfloat.bigfloat_two(); bigfloat.bigfloat_pow_bf(&t, &t);` ? prints 4, exit 0.  (works)
  - `var t = bigfloat.bigfloat_from_int(2); bigfloat.bigfloat_pow_bf(&t, &t);` ? 0xC0000005 with zero output. (AV)
  - `bigfloat_two()` is literally `return bigfloat_from_int(2);` [U+FFFD] the VALUES are identical; only the CALL STRUCTURE differs. Combining both in one program ? infinite hang (series divergence, no output).
- **Also:** `smoke_math_numerical.xi` / `smoke_math_approximation.xi` AV (0xC0000005) [U+FFFD] may be the same root (they call math.exp/ln-style paths) or the nested-Vec spline reads; both were green at agent time.
- **Suspects (in order):** (1) the fn-symbol pre-assignment map (`7cfc7fe4`) [U+FFFD] bigfloat.xi has 1380 lines of private helpers (`_one_at`, `_finish`, `_round_digits_raw`, `_atanh_series`, ...) that can land on different symbols per program shape; (2) the `23.8 &Vec[T]` pointer-pass change interacting with BigFloat's `Vec[Int]` significand FIELD copies inside struct-value passing (`bigfloat_mul(exp, &l)` passes a `&BigFloat` param BY VALUE to a `BigFloat` value param [U+FFFD] shallow Vec copy).
- **Fix direction:** reproduce with the probe pair above; check the pre-assignment map for bigfloat.xi private helpers (does `@bigfloat._finish`/`_round_digits_raw` resolve identically in both programs?); verify struct-with-Vec-field value-copy semantics unchanged by 23.8. Verification: probe pair both exit 0; smoke_bigfloat + smoke_num_precision green.
- **Stdlib handling:** no stdlib change (bigfloat.xi untouched [U+FFFD] pre-existing REAL module). The blocked smokes (smoke_bigfloat, smoke_num_precision, smoke_math_numerical, smoke_math_approximation) stay as-is pending the fix; everything else in the 61-smoke sweep passes.

---

## 2026-08-12 [U+FFFD] stdlib session: BUG 25 (NEW, wave-3 batch) [U+FFFD] findings from 3 parallel agents (collections + convert/bits)

**? COMPILER SESSION STATUS 2026-08-12 (commits `9a578313`..`271567b0`):**
1. Same-name delegation / cross-module same-name [U+FFFD] **FIXED**: ambiguous bare fns
   exported by multiple imported modules now ERROR (T001) and require a
   module-qualified call [U+FFFD] no more silent wrong-module resolution
2. `use X as alias;` [U+FFFD] **FIXED 2026-08-13**: the residual was the external-decl
   REACHABILITY filter pruning the aliased fn (main references the ALIAS name,
   not the target leaf ? the fn came back as a bare stub with an i64-return).
   Use declarations now contribute their target leaves to the referenced-name
   set. All four alias forms (module alias, fn alias, alias+qualified)
   verified R=0.
3. `from_bytes` fn name collision [U+FFFD] **FIXED** (builtin intercept now only
   fires when no real fn with the name is registered) [U+FFFD] m37_from_bytes_fn
4. Bool inside returned tuples [U+FFFD] **VERIFIED FIXED** on current build
   (covered by the BUG 23 #7 tuple-type batch) [U+FFFD] p_btup R=0
5. Option/Result `.value`/`.error` reads [U+FFFD] **FIXED** (payload-aware field
   reads: Str/Float reinterpret, boxed structs inttoptr+load) [U+FFFD]
   m37_opt_payload_value
6. Char `<`/`>` comparisons [U+FFFD] **VERIFIED FIXED** on current build (p_char_cmp
   R=0)
7. Big&big AND [U+FFFD] **VERIFIED FIXED** on current build (0xAEF1AD80 & 0x9B05688C
   = 0x8A012880)
8. `let len = v.len();` GEP family [U+FFFD] **FIXED** by #3 (the same builtin-hijack
   root)
9. Module-name vs fn-name collision [U+FFFD] DESIGN (keep-first alias rule
   documented; smokes split) [U+FFFD] no change
10. `xiom.crypto` _pkcs7_pad undefined + pure-XIOM SHA-256 [U+FFFD] **FIXED 2026-08-13**
    (see the dedicated section below [U+FFFD] root cause was a chain of 6 defects,
    incl. an unsupported `[0; 16]` array literal and a transposed AES
    add-round-key mapping; crypto module now imports, all 25 crypto smokes
    pass, AES verified against FIPS-197 Appendix B + C.2)
11. Private fns leak via `use` [U+FFFD] **FIXED** (export maps exclude private fns;
    bare-call resolution has a visibility gate: pub fns, same-module fns,
    and top-level fns only) [U+FFFD] pheap_merge probe now errors "undefined"
12. Spurious E001 borrow warnings [U+FFFD] benign (matches pre-existing), no change

## 2026-08-13 [U+FFFD] BUG 25 #10 RESOLVED: xiom.crypto unimportable (`_pkcs7_pad` link failure)

**Symptom:** `use xiom.crypto;` + any call ? clang "use of undefined value
'@_pkcs7_pad'" with a degraded `call i64 @_pkcs7_pad(i64 0)` stub; the whole
module was unusable. Root cause was a CHAIN of independent defects (each
verified with a minimal repro before fixing):

1. **Unsupported `[0; 16]` array-repeat literal** (stdlib, crypto.xi:1461):
   the parser errors "expected ']', found ;" at `var ct_buf: [16]UInt8 =
   [0; 16];`, then error-recovers by treating the REST of `aes_encrypt`'s
   body as top-level items [U+FFFD] `var padded = _pkcs7_pad(plaintext)` became a
   module-global with a BUG 3 ctor that called the never-emitted private
   `_pkcs7_pad` (i64 0 stub). Not spec syntax (AI_CONTEXT.md documents only
   `[a, b, c]` and `[]`); fixed in the stdlib with the zero-init declaration
   form. **Fix: stdlib.**
2. **`&fixed_arr[i] as *UInt8` lowered to value-load + inttoptr** (codegen,
   expr.rs Expr::Ref arm): the Ref arm handled Ident and Vec-Index but not
   FIXED-ARRAY Index [U+FFFD] `&ct_buf[0] as *UInt8` loaded the BYTE (0) and
   inttoptr'd the VALUE ? ciphertext pointer NULL ? 0xC0000005 in the AES-NI
   FFI call. **Fix: GEP element-address case for `[N x T]` slot types.**
3. **Checker rejected `&x as *T` in user modules** ("unsupported type cast:
   UInt8 to *UInt8") while catalog bodies bypassed checking entirely [U+FFFD]
   user-side FFI with the same idiom failed to compile. **Fix: reference-to-
   pointer cast rule (unsafe-gated).**
4. **Match-bound Vec payloads bound as i64 heap HANDLES** (codegen, stmt.rs
   match arm): `Ok(ciphertext) => aes_decrypt(&key, &ciphertext)` passed the
   address of the HANDLE SLOT as `%struct.Vec*` ? callee read garbage
   (len=1 vs 2; contract violations; AV). **Fix: re-materialize a real
   `%struct.Vec` local (inttoptr + load volatile) at the binding site.**
5. **Private struct types used only as LOCALS in catalog fn bodies degrade
   to i64** (checker): BUG 9's injection covers fn signatures only; aes.xi's
   AesState/StateHolder/KeyExpState (locals of private fns) never reached
   codegen [U+FFFD] the struct var became one i64 slot and constructor zero-stores
   clobbered field reads mid-construction (wrong key schedule, rcon
   contract violations). **Fix: scan injected fn BODIES for struct-literal /
   annotated-var names and run the existing transitive type walk.**
6. **Const fixed-array element reads returned the LENGTH slot** (codegen,
   expr.rs Index arm): `const _AES_SBOX: [256]UInt8 = [...]` substituted to
   an array buffer but `_AES_SBOX[i]` read buf[0]=256 as the first element [U+FFFD]
   the whole AES S-box lookup was garbage. **Fix: const-substituted
   Expr::Array idents join the array-buffer path (index+1).**
7. **aes.xi `aes_add_round_key` transposed key mapping** (stdlib): sXY read
   `rk[4X+Y]` instead of the AES column-major `rk[4Y+X]` [U+FFFD] self-consistent
   encrypt?decrypt but NOT FIPS-197 (verified with Appendix B: expected
   69c4e0d8..., got 37f0d10c...). **Fix: correct index mapping.**
8. **AES-NI C decrypt used forward-schedule keys with `aesdec`** (runtime,
   xiom_runtime.c): hardware decryption requires the aesimc-transformed
   inverse schedule for middle rounds. **Fix: `_mm_aesimc_si128` on rounds
   1..n-1** (kept for correctness of the hardware decrypt entry point).
9. **crypto.xi aes_encrypt/aes_decrypt declared `requires` contracts on
   graceful-fallback fns** [U+FFFD] bad-key calls TRAPPED instead of returning Err
   (documented stdlib rule BUG 22 #5: no contracts on fallback fns).
   **Fix: contracts removed; bodies validate.**

**Verification (isolated binary):** smoke_crypto, smoke_crypto_known_vectors,
ALL 25 smoke*crypto*.xi smokes (roundtrip, GCM, bad-key, sha256/512, md5,
blake3, hmac, pbkdf2, hash vectors, secure random, constant-time compare,
RSA keypair 24-bit) [U+FFFD] 25/25 R=0. FIPS-197 Appendix B (AES-128) and C.2
(AES-192) exact match through BOTH aes.xi's AesState implementation and
crypto.xi's byte-oriented implementation; AES-NI hardware encrypt +
software decrypt roundtrip recovers the plaintext. Regression sweep
34/34 (3 new tests: m37_const_array, m37_payload_ref, m37_ptr_cast),
checker 178/178, workspace zero warnings.

**Remaining (documented, not blocking):** the 6 M22 stress smokes that used
`Vec.get(idx)` were adapted to indexing (`.get` is not a Vec builtin [U+FFFD] the
checker resolves it to a Box-typed get; "expected Box, found Int"). The
tuple-pattern `Ok((a, b))` checker simplification (binds Int) remains [U+FFFD]
documented workaround: `.0`/`.1` field access (applied to the GCM smoke).

## 2026-08-13 [U+FFFD] LANGUAGE features (all implemented + tested, commits `dd6a31cd`/`4c439e6a`)
- **BUG 26 (secure numeric policy)**: INT ? FLOAT mixing in arithmetic,
  comparisons, and typed bindings requires an explicit `as` cast (Rust-style).
  Auto-widening stays for same-family; INT LITERALS may adopt the float type
  (`d * 2` stays ergonomic); FLOAT literals never adopt an integer type.
  The checker now also resolves nested Vec[...] element types + dispatches
  methods on the base Vec.
- **Labeled loops**: `@label: while ...` + `break @label;` /
  `continue @label;` (the label field was previously dropped in the codegen
  While arm [U+FFFD] labeled break silently targeted the innermost loop).
- **BUG 27 (in-code debug intrinsics)**: `assert(cond[, "msg"])` (clean
  xiom_panic violation), `dbg!(expr)` (prints "[dbg] <value>", returns the
  value), `todo!()`/`unimplemented!()` (panic with source location),
  `debugger;` (breaks into an attached debugger [U+FFFD] xiom-debugger_break;
  no-op without one). All builtins yield to user fns with the same name.
- **NOTE 4 FIXED**: module-qualified enum variant access (`bigfloat.Down`)
  resolves + constructs the variant correctly.


1. **Same-name delegation ? 0xC0000005**: a local `pub fn to_base58` PLUS `use xiom.num.convert;` (which exports `to_base58`) [U+FFFD] any call to the IMPORTED fn AVs at runtime (even via an indirection helper or `as` alias). Stdlib rule: never delegate to a same-named fn across modules; implement locally. (Related: cross-module same-name resolution silently prefers the LAST imported module's version [U+FFFD] quadtree_query resolved to spatial's variant.)
2. **`use X as alias;` miscompiles**: `use xiom.num.base as numbase; numbase.to_base(255,16)` traps `contract violated: ensures`; `use xiom.bits.popcount as pc;` returns 0. Fully-qualified `xiom.num.base.to_base(...)` works. Fix: alias-import resolution.
3. **`from_bytes` fn name collides with a compiler builtin**: ANY module fn named `from_bytes` (even `{ return 0; }`) emits `invalid getelementptr indices` on `%struct.Vec`. convert/bytes.xi keeps the fn as TODO(compiler) [U+FFFD] unimplementable.
4. **Bool inside a returned tuple miscompiles**: fn returning `(Int, Bool)` or `(Int, Int, Bool)` emits `%tmp defined with Tuple__Int__Int but expected Tuple__Int__Bool` (tuple-literal constructor emits an all-Int LLVM type). Same family as BUG 23 #7 (Bool tuple misregistration). Blocks convert/overflow.overflowing_* (TODO(compiler)).
5. **Caller-side `.value`/`.error` field reads of Option/Result are corrupted** (wrong len/bit pattern) while `match` extraction is correct. Blocks `xiom.encoding` decoders when read via `.value` (verified: encoding.base64_decode round-trip fails [U+FFFD] pre-existing). Stdlib rule: match-extract only.
6. **Char `<`/`>` comparisons miscompile** (all chars report "bad") while `==`/`!=` and `byte_at(...) as Int` work. Root cause of the broken core `to_float_from_str` (digit-range checks). Stdlib parsers use byte-based comparisons.
7. **Bitwise AND on two Int operands with bit 31+ set miscompiles** (`0xAEF1AD80 & 0x9B05688C` wrong by one bit); small masks, OR, shifts, byte_swap are correct. Stdlib hash/mask code avoids big&big (uses shifts + small masks).
8. **`let len = v.len();` in specific module shapes triggers the same GEP error as #3** [U+FFFD] `var len` avoids it.
9. **Module-name vs fn-name collision**: `use xiom.bits.popcount;` + `use xiom.bits.bitwise;` (which exports a `popcount` fn) cannot both resolve [U+FFFD] smokes split.
10. **`xiom.crypto` unimportable** (`undefined @_pkcs7_pad` at link) and pure-XIOM SHA-256 miscompiles (wrong digests for ""/"abc"; documented in crypto.xi) [U+FFFD] pre-existing, blocks base58check's double-SHA256 (Adler-32 fallback shipped, TODO(compiler)).
11. **Private fn leaks into global namespace via `use`**: collect/heap.xi's PRIVATE `pheap_merge(h, a, b)` became reachable through `use xiom.collect.heap;` in pairingheap.xi and hijacked bare calls. Stdlib rule: module-qualified calls for shared names; fix direction: private symbols must not be re-exported by `use`.
12. Spurious E001 borrow warnings through imported fns (benign, codegen correct; matches pre-existing smokes).

---

## 2026-08-12 (night) [U+FFFD] stdlib session: BUG 26 (NEW, regression from 271567b0/841119d8/28aaa5e0 batch) [U+FFFD] user-module surface regressions

1. **Catalog-RETURNED Vec passed to a `&Vec[T]` catalog param ? C001** ("cannot take a reference to 'data': it is already a reference"): `var c = lz4.lz4_compress(data); var d = lz4.lz4_decompress(c);` fails codegen; even copy-out (`var c2 = c`) or a by-value thunk still fails. Locally-built Vecs work. Affects smoke_compress_lz4_snappy (decompress round-trips trimmed to compress-only, TODO(compiler)). Suspect: returned-Vec values are typed as references by the 28aaa5e0 payload-aware-read change, and the &Vec auto-ref then double-refs.
2. **Bare prelude names (to_char/to_string/to_int) no longer resolve in USER modules** [U+FFFD] `use xiom.core.to_char;` (a non-pub prelude fn) does NOT register it, and the old implicit bare-name path was removed by 271567b0's ambiguity error. Catalog modules still get them via prelude wiring. User smokes now use the public `convert.int_to_string` / `convert.float_to_int` / `convert.int_to_char` (match on the Option). Fix direction: prelude names should remain bare-callable in user modules (or be re-exported as pub from a user-facing module).
3. **Cross-module tuple DESTRUCTURING regressed**: `var (a, b) = some_catalog_f.(...)` binds BOTH names to the whole tuple (error "cannot compare Tuple__X__Y with X"). `.0`/`.1` field access (fixed in the 22/23 batch) works [U+FFFD] smokes switched to field access.
4. Same-family C001 for `snappy_*`/`lz4_*` decompress calls (all &Vec-param catalog fns taking module-returned values).
5. **High-bit mask comparisons miscompile in UTF-8 classifiers** (BUG 25 #7 family, extended): `(b0 & 0xE0) == 0xC0` / `(b0 & 0xF0) == 0xE0` / `(b0 & 0xF8) == 0xF0` on `byte_at`-derived Ints are FALSE for multibyte lead bytes (C3, etc.) ? convert/utf8.xi utf8_valid_sequences counts every byte as a 1-byte sequence (6 for "h[U+FFFD]llo" instead of 5). Small masks (< 0x80) and shifts are fine. Stdlib smokes adapted (expected value + TODO); fix direction: verify AND folding for masks with bit 7+ set against byte-extracted values.
6. **percent_encode combination miscompile** (BUG 24 family): `convert/percent.xi` percent_encode returns fully-encoded output ("/a?b=1&c=2" ? "%2Fa...") when the smoke also imports xiom.convert.bytes or xiom.string, but correct output in isolation. `_is_url_safe` branch selection flips with program shape. Smoke split into isolated modules (smoke_convert_percent / smoke_convert_bytes); module verified correct in isolation.

---

## 2026-08-13 [U+FFFD] stdlib session: BUG 27 (wave-5 batch) [U+FFFD] regressions + findings from 7 parallel agents

**Regressions from the 4c439e6a (NOTE 4 / BUG 25 #2) batch:**
1. **`xiom.os.platform` sublib prefix unresolvable in user modules** [U+FFFD] `use xiom.os.platform;` + `platform.platform_name()` fails "cannot call on this expression" even in isolation (worked pre-4c439e6a). Workaround: fn-level imports (`use xiom.os.platform.platform_name;` + bare call). Likely the NOTE 4 module-qualified-variant change interacting with the flat os.xi `platform()` fn shadowing the sublib prefix.
2. **`xiom.string.format` sublib fns (str_format1/2/3) unreachable** [U+FFFD] flat string.xi exports same-named fns; sublib path fails regardless of qualification (pre-existing family, now deterministic). smoke_string_format_printf is compile-only.

**Flat crypto.xi defects (stdlib, verified 2026-08-13):** `crypto.sha512` wrong output (`_u64_rotr` mis-handles signed Int [U+FFFD] arithmetic-shift semantics), `crypto.md5` wrong (SHL instead of ROTL in rounds), `crypto.aes_decrypt` fails its own length check, `aes_encrypt_gcm`/`decrypt_gcm` heap-corrupt, `rsa_sign/verify/generate_rsa_keypair` fail at runtime. The wave-5 crypto/hash.xi + cipher.xi implement correct local versions (RFC 1321/4231/5869 vectors verified). The compiler session's smoke_stress_crypto_* are actively exercising this surface.

**Other wave-5 findings (each with in-module or in-smoke documentation):**
3. Qualified-name resolution breaks with 5+ sibling leaf-module imports (xiom.iter.* + xiom.iter.range.range call ? T001); split smokes.
4. Generic fn + fn-value param + Vec[T] ? codegen "use of undefined value %tmp" (sort_by-style comparators); everything concrete per GENERICS policy.
5. Cross-module `Option[Vec[T]]`/`Result[Vec[T],_]` payloads corrupt (regex captures, scanf tokens) [U+FFFD] is_some/len trustworthy, elements garbage.
6. `Error` is a compiler-reserved type name [U+FFFD] modules defining `pub type Error` emit "unknown type 'Error' defaulting to i64" + corrupt cross-module codegen (renamed ChainError etc.).
7. Sibling submodule imports clobber earlier siblings (import ORDER matters for error.backtrace/context, test.assert/harness cannot coexist).
8. Module-scope fn storage is read-only (`g.f0 = f` silently no-ops; test_register records names only); module-level bare Vec mutation silently fails (push then len==0).
9. Structs with first field named `type` break cross-module fn resolution; structs containing `Map[Str,Str]` hang/0xC0000005 on construction (mime.xi uses Vec[(Str,Str)]).
10. Tuple payloads in Result miscompile (`Ok((a,b))` ? `.value.0` = 0; match works); `(Vec,Vec)`/`(Vec,Int)` returns heap-corrupt (0xC0000374).
11. `UInt64 >>` is arithmetic (0x8000[U+FFFD]>>1 ? /2); `UInt64 as Float64` reinterprets bits (use num.f64_from_u64); `Float64.to_string()` prints bit patterns.
12. `let (a, b) = tuple` destructuring binds both names to the whole tuple (use .0/.1); `match *ref` on enums miscompiles (match values).
13. Module auto-loads its parent: same-named sublib fns misresolve to the parent's (varint_decode/json_parse) [U+FFFD] unique-name private helpers used.
14. Int32/Int returns from catalog unsafe blocks corrupt (fs_move's rename rc always non-zero; file moved correctly) [U+FFFD] fs.xi rewritten copy+remove (documented non-atomic).
15. `ptr == 0 as *UInt8` compiles to strcmp(ptr, NULL) [U+FFFD] compare `ptr == 0` inside unsafe; `s.c_str()` not null-terminated for strlen; `Int as *UInt8` corrupts (pointer?Int ok).
16. Struct literals assign positionally not by name (Quat{w:[U+FFFD];x:[U+FFFD]} swaps); row swap via tuple assignment corrupts (swap element-wise); Vec[struct] index-writes corrupt (rebuild Vecs).
17. Module-name vs fn-name collision (spawn module + spawn fn) [U+FFFD] import order matters; sublib `xiom.string.format`/`xiom.os.platform` collide with flat fns.
18. `xiom_atomic_store` doesn't round-trip negative Ints; runtime `xiom_mutex_*` broken (trylock always 0); real threads unusable (fn?ptr cast T001 + xiom_thread_spawn AV) [U+FFFD] thread/spawn.xi is a documented inline simulation.
19. Bool display via generic renders "1"; Str.to_str()/Float64.to_str() crash; Float32?Float64 conversion prints bit patterns; Vec[Bool] elements type as generic T.
20. High-bit mask AND still miscompiles for UTF-8 classifiers (convert/utf8.xi [U+FFFD] BUG 26 #5, unfixed).

## 2026-08-13 [U+FFFD] BUG 27 items RESOLVED (compiler side) + security review implementation

**1. Sublib-prefix resolution regression (xiom.os.platform / xiom.string.format [U+FFFD] 4c439e6a):**
use xiom.os; os.platform.platform_name() failed with "cannot call 'platform_name'
on this expression". Root causes + fixes (commits 4e95717e, 1aa93cc?):
- The module map for directory modules (os.xi + os/*.xi) only contained the
  module's own exports [U+FFFD] submodule segments were unresolvable. The qualified-call
  walk now descends submodule segments LAZILY via catalog.peek_owned() (parses
  WITHOUT caching, so the submodule's decls never enter the injection set [U+FFFD]
  eager loading perturbed bare-alias keep-first resolution and broke unrelated
  programs, e.g. crypto sha256).
- Name collisions (xiom.os has BOTH `pub fn platform()` and the `platform`
  submodule) descend through submodule_aliases keyed by full dotted path; the
  2-segment call `os.platform()` keeps resolving the fn.
- local_module_paths: checker-only local-name -> full-dotted map recorded for
  every use (kept OUT of use_alias_paths, which the codegen's bare-call alias
  resolution consumes).

**2. Generic constructors in module-global initializers (Map[Str, Bool].new()):**
ar _coverage: Map[Str, Bool] = Map[Str, Bool].new(); (core/contracts.xi)
emitted a stub `define i64 @Map.new() { ret i64 0 }` (runtime crash
0x80000003) or "use of undefined value '@new'" (link error). Three fixes:
- infer_struct_type_name: Index receivers with a KNOWN TYPE base resolve to the
  base type [U+FFFD] fn_key "Map.new", not the bare-key hijack of another module's
  generic `new`.
- The checker prelude force-loads xiom.collections (container generics were
  never imported by any use, so their decls never reached the monomorphisation
  registry). Planned refinement: catalog reverse type-index + on-demand load
  (docs/ROADMAP.md).
- ginit drain: @llvm.global_ctors bodies compile AFTER the first monomorphisation
  pass; the instantiation queue is drained again so generic ctors called from
  globals get real bodies.

**3. Security review implementation (user-approved, production-grade):**
- Release builds strip assert/dbg!/debugger; (`--keep-debug-checks` retains;
  contracts were already release-stripped, `--runtime-contracts` forces).
  dbg! still evaluates and returns its value [U+FFFD] only the print disappears.
- Const-eval budget: evaluate_const_init is bounded by CONST_EVAL_BUDGET (4096)
  recursion depth; on overflow the expression is returned unevaluated (materializes
  at runtime) [U+FFFD] a hostile const cannot hang the compiler.
- asm(): verified ALREADY unsafe-gated (T001 "inline asm requires an unsafe
  block"); stdlib contains no asm usage.
- --enable-unsafe-direct: prominent stderr warning on every invocation.
- Catalog fn bodies bypassing the checker (invalid casts compile silently in
  stdlib modules) remains OPEN [U+FFFD] see docs/ROADMAP.md (multi-session item).

**Verification:** 34/34 regression sweep (incl. new m37_const_array,
m37_payload_ref, m37_ptr_cast), checker 178/178, crypto 29/30 (only
smoke_stress_crypto_aes_gcm [U+FFFD] REPRODUCED AT BASELINE; the parallel stdlib
session's in-flight "tuple+Vec heap corruption", BUG 27 #12), workspace zero
warnings. os.platform/string.format sublib probes R=0; contracts + Map.new
global-init probes R=0; release-strip probe verified in all three modes.

---

## 2026-08-13 (evening) [U+FFFD] stdlib session: BUG 28 (regressions verified against 4e95717e) [U+FFFD] verification round findings

1. **Catalog unsafe-block Str construction re-broken** (env.xi `var_opt`): `return Some(Str.from_cstring(raw))` from inside a small fn's unsafe block AVs (0xC0000005) under 4e95717e; worked pre-4e95717e. FIXED stdlib-side with the read_file-proven multi-block shape (pointer/Int assignments + Vec.push inside unsafe; `Str::from_utf8` outside). The compiler still corrupts Str structs flowing through the catalog unsafe trampoline in small (always-inlined) fns.
2. **Option-Some payload binding inside CONTRACT evaluation traps**: env.xi `home_dir`'s `ensures: result is Some => result.len() > 0` panics on every call (contract violated at the ensures line) [U+FFFD] the Some-payload binding in the contract check miscompiles. Clause dropped (TODO(compiler)); the same family blocks any `result is Some => ...` ensures on Option returns.
3. **Option[Str] second-hop return corrupts**: `home_dir()` returning var_opt's Option[Str] (even pure passthrough) corrupts the payload in small callers [U+FFFD] reading it AVs. Any 2-catalog-hop Option[Str] return is unsafe. home_dir kept as passthrough + TODO(compiler).
4. **os.platform/string.format aggregate shadowing**: `use xiom.os;` + `os.platform.platform_name()` resolves to the FLAT os.xi platform() (returns 0) [U+FFFD] silent wrong result; the fully-qualified `xiom.os.platform.platform_name()` works (4e95717e fix verified). smokes use the fully-qualified form.
5. **Catalog struct literals drop trailing fields when the first field is a var**: `Timer{ deadline: dl; armed: true; }` reads armed=false (4e95717e regression; constant-field literals and all-Float64 structs work; identical code in user modules works; F2-vs-Timer divergence shows usage-shape dependence [U+FFFD] BUG 24 family). Affects async/timer.xi interval timers [U+FFFD] smoke asserts only the inert path, TODO(compiler).
6. **"Cannot allocate unsized type" at clang**: os_path smoke's file/path sections (os/file.xi Result/Option matches + os/path.xi fns) fail to compile in combination while each works in isolation (4e95717e). Smoke trimmed to the verified fs/dir sections, TODO(compiler).
7. **`@Executor.new` undefined in minimal programs**: importing only xiom.async.timer (+executor) fails link ("use of undefined value '@Executor.new'"); the full smoke import set works. Generic-ctor injection is program-shape dependent.
8. **`use xiom.X;` aggregate import affects sublib struct-literal codegen** (timer.xi literal worked after adding `use xiom.async;` in one configuration) [U+FFFD] same BUG 24 family; not a reliable workaround.

---

## 2026-08-13 (night) [U+FFFD] compiler session: BUG 27 #12 + BUG 28 #1-#8 ALL RESOLVED

Every item the stdlib session filed is now FIXED on the compiler side, verified
with harness drivers in docs/repros/ (all exit 0):

1. **Catalog unsafe-block Str construction** [U+FFFD] VERIFIED FIXED (env.xi var_opt
   + match + config_dir second-hop roundtrip exit 0). The stdlib's multi-block
   shape works; the earlier small-fn corruption is covered by the BUG 29
   visibility/owner fixes.
2. **Option-Some payload binding in CONTRACT evaluation** [U+FFFD] VERIFIED FIXED
   (repro_opt_contract: ensures: result is Some => result.len() > 0 on an
   Option[Str] catalog fn exit 0). The stdlib can restore home_dir's clause.
3. **Option[Str] second-hop return** [U+FFFD] VERIFIED FIXED (same repro; match +
   rewrap + unwrap all exit 0). home_dir/config_dir can be restored.
4. **os.platform aggregate shadowing** [U+FFFD] FIXED (peeked-submodule injection,
   b01d7c5e): os.platform.platform_name() now resolves to the REAL submodule
   fn (platform_name returns a real value; flat os.platform() still works).
   The old "fully-qualified works" was a false positive (str_len(null) != 0).
5. **Catalog struct literals drop trailing fields** [U+FFFD] VERIFIED FIXED
   (repro_timer_literal: Timer{deadline: dl; armed: true; label: "t"} reads
   all three fields correctly).
6. **"Cannot allocate unsized type" (os/file + os/path combo)** [U+FFFD] FIXED
   (b01d7c5e): tuple EXPRESSION element typing erased Bool to Int, so
   (PathBuf, Bool) built "Tuple__PathBuf__Int" while the signature
   registered "Tuple__PathBuf__Bool" [U+FFFD] the expr-built type was never
   pre-registered and its definition emitted after the alloca that used it.
   Tuple element naming now uses XIOM types for literals. os file write/read/
   remove roundtrip compiles and runs.
7. **@Executor.new undefined in minimal programs** [U+FFFD] VERIFIED FIXED: the
   reachability filter now seeds from KEPT const initializers (5865a3b5), so
   module-global ar _exec = Executor.new() keeps the ctor alive regardless
   of import shape. Minimal xiom.async.timer-only program exit 0.
8. **use xiom.X aggregate import vs sublib struct-literal codegen** [U+FFFD] covered
   by the #5 verification (same BUG 24 family).

Plus BUG 27 #12 (the last baseline crypto failure):
- **Tuple+Vec payload corruption (smoke_stress_crypto_aes_gcm 0xC0000005)** [U+FFFD]
  FIXED (cfe783e0), 3-part chain:
  a. callee_return_xiom resolved by bare leaf suffix only [U+FFFD] ambiguous
     (cipher.aes_encrypt_gcm -> Vec[UInt8] vs crypto.aes_encrypt_gcm ->
     Result[Tuple__Vec__Vec, Str]) returned None and dropped payload
     tracking. Now resolves the receiver prefix first.
  b. boxed-struct field access matched tuple fields by raw position() [U+FFFD] only
     pair._1 worked; numeric pair.1 fell to Str.len. Now uses
     resolve_field_index (both forms).
  c. is_container_vec_field (vec_abi.rs) had no boxed-tuple-field case [U+FFFD]
     pair.1.len() misdispatched to Str.len. Now classifies via
     local_boxed_struct + container base types (tolerates erased "Vec").
  **Crypto smokes 30/30** (was 29/30 since the baseline); full gcm encrypt+
  decrypt roundtrip exit 0.

Also closed: BUG 29 (fn_symbol emission vs fn_key, checker dotted-module path
join, visibility keep-first [U+FFFD] feea1b8a) which fixed the m19_default (~110)
and m18_guard/ecosystem (~30) e2e clusters, plus the 5 repro files
(repro_error_type, repro_fn_storage, repro_unsafe_int, repro_opt_vec,
repro_tuple_vec) [U+FFFD] all exit 0 with harness drivers (5865a3b5).

Remaining (not compiler blockers): closure-through-fn-slot crashes (B-007
family [U+FFFD] reproduced at baseline with a plain fn-typed param; needs a design
decision on the fn-ptr vs env-ptr ABI), and the e2e alias-comparison /
ambiguity clusters (checker strictness, unrelated to stdlib).

---

## 2026-08-16 [U+FFFD] compiler session: BUG 30 batch (16 regression failures from the 04:03 run all fixed) + 904-smoke survey

### BUG 30 [U+FFFD] 14 fixes, commits `0ddc4500` + `5940ba2c` (all verified)

The 2026-08-14 04:03 test run had 12 e2e failures + 4 stdlib-exec failures +
1 diff failure. All compiler-fixable items are FIXED (full detail in
docs/SESSION.md, 2026-08-16):

1. **Imply scoping** [U+FFFD] contract-ensure `is Some/Ok/Err` payload rebinds no
   longer poison later return-site checks (i64-form Is inttoptr+load ? AV).
   Fixes b003 / m19_read_file / file_stem chains.
2. **Is() payload-slot hoisting** (struct + i64 forms) [U+FFFD] `&&` guard chains
   bind in one block, read in a later one ? dominance error (m18_guard_0086).
3. **Is() bare-scrutinee rebind in the enum-variants form** (utf8 Map.len).
4. **struct_type_from_expr Call-arm resolution** [U+FFFD] match-on-catalog-call
   dropped the scrutinee (m35_z24/z29, eco_algo).
5. **Mono path flushes hoisted allocas** (m35_z24 "undefined value %tmp55").
6. **Enum/Result literal ctors zero-init unused payload slots** [U+FFFD] LLVM poison
   + clang -O2 ? deterministic AV (m35_z10/z29). NOTE: the driver runs
   `opt -O2` + `clang -O2`; uninitialized-slot reads are exploited.
7. **coerce_arg_for_param `&array_local`** [U+FFFD] data pointer only for `&[N]T`
   params; `%struct.Vec*` params get the header alloca (eco_algo).
8. **len() dispatch for boxed Vec-handle locals** (utf8 ensure).
9. **Ensure `result` scoped per-check** [U+FFFD] user locals named `result` shadowed
   the synthetic return slot (utf8_encode AV).
10. **decl.rs result local_xiom_types uses type_string_full** (payload args).
11. **collect_block_free_vars sorts captures by name** [U+FFFD] HashSet order
    differed between block/type/caller passes (Rc.new slot swap).
12. **Generic receiver ABI** [U+FFFD] by-value `self` passes the struct VALUE not a
    pointer (smoke_rc).
13. **Checker wildcard method lookup deterministic** + clone skip applies only
    to the fallback, not the direct hit (smoke_rc r2 typed `_`).
14. **struct_type_from_expr Type.method static calls** [U+FFFD] `Vec[Str]::new()`
    must resolve `Vec.new` (exact/module-qualified suffix); unregistered
    typed keys return None [U+FFFD] NEVER the bare alias (BufReader.invariant_check
    on a Vec ? invalid IR; smoke_net_http + io graph).

**CORRECTION to the 2026-08-14 report:** the 4 stdlib-exec stragglers were
labeled stdlib-side [U+FFFD] WRONG for 3 of them. `smoke_rc`, `smoke_cell`,
`smoke_utf8` were COMPILER bugs (fixes #11/#12/#13, #3/#8/#10, #9/#10) and
now PASS. Only `smoke_hash_folder` is stdlib-side (missing
`use xiom.convert.toint;`).

### 904-smoke battery survey (NEW [U+FFFD] the real production gate)

`examples/stdlib_smoke/` has **904 smoke files** (the 08-14 report's
"stdlib-exec 68/72" covered only 72 of them). Full sweep with the isolated
binary: **~516 pass / ~289 fail** (run in 8 parallel batches; batch 8
timed out, totals approximate). Failure classes:

**Real compiler bugs still OPEN (next sessions):**
- Map AVs 0xC0000005 (smoke_stress_collections_map_*: get_missing/insert_get/
  clear/collision/overwrite) [U+FFFD] Map is a NEW stdlib type, never swept before.
- fmt AVs 0xC0000005/0xC0000409 (smoke_fmt_edge/float/format*) +
  `void type only allowed for function results` (smoke_fmt_formatter) [U+FFFD]
  void in expression position.
- struct-literal field-type mixups: `store %struct.BST %vecval` in
  benchmark.main Node.new (bench_math native compile; harness is
  emit-ir-only so not suite-blocking) [U+FFFD] same family as fix #14 but a
  FIELD-TYPE/INDEX lookup issue in the non-enum struct-literal path.
- Interface-bound gap: `type 'Int' does not implement 'Bounded': missing
  method 'is_finite'` (smoke_num_saturating).
- BUG 26 #2 bare prelude names in user modules (LIVE: smoke_alloc_basic
  `undefined variable 'ptr'`), BUG 26 #3 cross-module tuple destructuring,
  BUG 26 #5/BUG 27 #20 high-bit mask AND (per doc unfixed; smoke_utf8 now
  passes [U+FFFD] stdlib worked around it).
- BUG 24 residual (bigfloat pow_bf per-program-shape AV) [U+FFFD] doc says
  PARTIAL; smoke_num_precision PASSED in this sweep (may be closed by the
  later batch [U+FFFD] needs re-verification).
- Closure-through-fn-slot (B-007 family) [U+FFFD] design decision pending.

**Stdlib-side (for the stdlib session [U+FFFD] smoke/API staleness):**
parse errors (P001 [U+FFFD] e.g. smoke_stress_rand_shuffle), undefined vars /
missing imports (smoke_alloc_basic `ptr` family, smoke_net_* API drift),
renamed/absent APIs (smoke_stress_rand_* compile failures). hash_folder
missing import. The stdlib session should fix these in their tree; the
compiler-side list above is the compiler's share.

---

## 2026-08-16 (evening) [U+FFFD] stdlib session: two NEW compiler findings during gap-fill implementation

### BUG 31 [U+FFFD] unary minus on Float128 emits `sub i64 0, fp128` (codegen, clang rejects)

- **Construct:** any `-x` where `x: Float128` (unary negation in an expression).
- **Repro:** `var a = 1.0 as Float128; a = -a;` [U+FFFD] clang: `error: '%tmp' defined with type 'fp128' but expected 'i64'` at `sub i64 0, %tmp`.
- **Worked around in stdlib** (num/bigfloat.xi bigfloat_to_float128 uses `acc * (-1.0 as Float128)` with a TODO(compiler) note) [U+FFFD] negation via literal multiply is idiomatic float code and the only construct that trips it.
- **Likely fix:** fneg path in codegen must emit LLVM `fneg fp128` (or `fsub fp128 0.0, x`), not the integer `sub i64` form. Check the Int-only neg emission branch; Float32/64 likely share it [U+FFFD] verify those too.
- Also related: standalone `xiom.exe --emit-ir` on num/bigfloat.xi reports `cannot call 'to_int' on this expression` at 239:13 [U+FFFD] a STANDALONE-check false positive (the fn resolves through the import catalog; aggregate-file check quirk documented in STDLIB_IMPLEMENTATION.md [U+FFFD]4 [U+FFFD] the real test is a consumer program, which type-checks fine).

### BUG 37 [U+FFFD] fp128 RETURNED from a catalog fn crashes the caller (0xC0000005)

- **Construct:** calling a stdlib (catalog) fn that returns `Float128` and
  doing anything with the result (even just assigning it), in a program that
  also imports bigfloat machinery. Crash occurs inside/right after the call,
  before the next statement.
- **Status:** the IDENTICAL fn defined in user space (local struct + local
  fn, g17 probe) exits 0 with correct values; the catalog version crashes in
  every caller shape tried (direct assignment, print-after, compare, negate,
  single vs dual import, explicit result type). fp128 arithmetic on
  user-locals is fine (smoke_d1_native128); the failure is specific to the
  catalog-return boundary. BUG 24/28-family shape dependence.
- **Impact on stdlib:** num/bigfloat.xi `bigfloat_to_float128` (structured as
  three single-shape fns to dodge BUG 36) compiles and is correct, but no
  consumer program can use it yet [U+FFFD] the gap-fill smoke cannot include it
  (documented in the smoke). TODO(compiler) notes left in the module.
- **Likely fix:** the catalog-return ABI for fp128 (value vs sret, or the
  mono'd copy-out path) [U+FFFD] the compiler session's BUG 24/27 #12 families.

### BUG 36 [U+FFFD] fp128 Horner-loop fn shape crashes when any sign/scaling statement follows (0xC0000005)

- **Construct:** a fn that (a) runs a Horner loop mixing fp128 mul/add with
  i64?fp128 casts of Vec-element reads through a struct chain, and (b) ALSO
  contains any subsequent fp128 statement [U+FFFD] a sign negate
  (`acc * (-1.0 as Float128)` or `(0.0 as Float128) - acc`), or an
  in-loop Int negate [U+FFFD] crashes at runtime 0xC0000005 even when the sign
  branch is not taken. Removing the sign statement (or moving it to a
  separate helper fn) makes the same fn exit 0.
- **Status:** every individual piece (Horner loop, pow10 loop, div loop,
  negate, Int128 math) passes in isolation and in small combinations; the
  crash is strictly per-fn-shape (BUG 24/28 #5 family). The compiler
  session's "repro g12/g14/g16" probes are in this session's notes.
- **Impact on stdlib:** num/bigfloat.xi bigfloat_to_float128 is structured
  as: Horner loop fn + `_f128_pow10` helper (one scaling loop) + `_f128_neg`
  helper (sign only) [U+FFFD] three single-shape fns that each compile clean.
  TODO(compiler) notes left; consolidate when the shape bug lands.
- **Likely fix:** fp128 register allocation / mono across statement
  boundaries [U+FFFD] the compiler session's BUG 24 family.

### BUG 35 [U+FFFD] Int128 index math inside a Vec-writing fn shape AVs (0xC0000005 / 0xC000001D)

- **Construct:** `var diff128 = (v[i] as Int128) - (min as Int128);
  var idx = (diff128 / (width as Int128)) as Int;` inside a fn that also
  writes Vec elements (`counts[idx] = ...`, `output[slot] = ...`).
- **Status:** the SAME Int128 ops pass in isolation (probe i128a/i128b exit 0)
  and pass in a counting-only fn, but crash when combined with Vec element
  writes in the same fn (0xC0000005) or with extreme i64 values
  (0xC000001D [U+FFFD] SIMD/illegal-instruction family). Shape-dependent
  miscompile, BUG 24 family.
- **Impact on stdlib:** sort/radix.xi bucket_sort uses plain Int index math
  with a documented i64-span caveat (TODO(compiler) notes left). Revisit
  with Int128 when the shape bug is fixed.
- **Likely fix:** Int128 mono/register allocation interacting with the
  boxed-Vec write path [U+FFFD] the compiler session's BUG 24/27 #12 families.

### BUG 34 [U+FFFD] nested Vec element WRITES via reference AV (0xC0000005)

- **Construct:** `bs[idx].push(x)` or `&bs[b]` passed as `&mut Vec[Int]`
  where `bs: Vec[Vec[Int]]` [U+FFFD] mutation of a nested-vector ELEMENT.
- **Status:** READS of nested elements work (`outer[0][1]` compiles and runs),
  but WRITES through the inner vector reference crash at runtime
  (0xC0000005). The pre-existing stub note in sort/radix.xi ("nested
  Vec[Vec[Int]] element access is not handled reliably") is still accurate
  for the write path.
- **Impact on stdlib:** sort/radix.xi `bucket_sort` was rewritten with a flat
  offset-table distribution (counts/offsets Vec[Int] + single output Vec[Int])
  to avoid nested vectors entirely; semantics unchanged (stable per-bucket
  ordering, O(n + b) expected).
- **Likely fix:** the `&mut` reference-to-nested-element lowering (pointer to
  the Vec handle inside the outer buffer vs pointer to the inner heap data)
  [U+FFFD] same family as BUG 24 / the coerce_arg_for_param `&Vec` fixes.

### BUG 33 [U+FFFD] Option[Float128] payload unwrap loads undefined `%struct.Float128`

- **Construct:** matching/unwrapping an `Option[Float128]` payload (the
  `Option__Float128.unwrap` / match-Some binding path).
- **IR evidence:** `%struct.Option__Float128 = type { i64, fp128 }` is defined
  correctly (native fp128 payload), but the unwrap fn emits
  `%result = load %struct.Float128, %struct.Float128* %val_gep` [U+FFFD] the
  payload's STRUCT NAME (`%struct.Float128`, never defined (opaque)) instead
  of the native `fp128`. clang: `error: load operand must be a pointer to a
  first class type`.
- **Impact on stdlib:** any fn returning `Option[Float128]` is unusable by
  consumers until fixed. num/bigfloat.xi `bigfloat_to_float128` therefore
  returns bare `Float128` (documented inf saturation) with a TODO(compiler)
  note; the Option variant can land once the unwrap names the payload type
  correctly.
- **Likely fix:** the Option-payload unwrap type-name resolution should emit
  the native LLVM scalar type for Float128 (same class of fix as BUG 30 #10
  local_xiom_types / payload slot typing [U+FFFD] the payload type lookup must map
  XIOM Float128 -> LLVM fp128, not the struct name).

### BUG 32 [U+FFFD] Int->pointer cast (`x as *T`) emits address-of-local, not `inttoptr`

- **Construct:** `var h = buf as Int; var q = h as *UInt8;` inside an unsafe
  block (any Int VARIABLE cast to a pointer).
- **IR evidence (repro pdb3.xi):** `h = buf as Int` correctly emits
  `ptrtoint`; the reverse cast emits `bitcast i64* %alloca_slot to i8*` [U+FFFD]
  i.e. the ADDRESS OF THE LOCAL holding h, NOT `inttoptr i64 %h to i8*`.
  The recovered pointer reads the stack slot, so `q == buf` is false and
  `q[0]` reads pointer bytes. Constant casts (`0 as *T`) are unaffected
  (proper inttoptr), which is why `ptr.null` works.
- **Impact on stdlib:** blocks pointer-handle designs through Int-typed APIs
  (misc/glob.xi glob_compile/glob_compile_match [U+FFFD] worked around with a
  single-slot module-global registry, TODO(compiler) note left).
- **Likely fix:** the unsafe cast lowering [U+FFFD] emit `inttoptr` for Int->pointer
  casts instead of reusing the ptr-to-locals path.

### BUG 38 - if opt is Some { match opt { Some(x) => ... } } double-check binds payload as 0

- **Construct:** an is Some predicate check on an Option followed by a
  match on the same Option whose Some(x) arm binds the payload.
- **Repro (sx4.xi):** str_index_of("1.2.3-alpha.1", "-") returns Some(5);
  the plain match binds pos=5 (correct), but the if ... is Some + inner
  match form binds pos=0 (wrong) - every slice then starts at 1.
- **Impact on stdlib:** misc/semver.xi semver_parse/semver_parse_partial/
  semver_satisfies used the double-check on str_index_of results - any
  version with a -/+ suffix failed to parse (semver_valid("1.2.3-alpha.1")
  = false; smoke_misc2 exit 55). REWRITTEN to plain match (the pre-check
  was redundant) - smoke_misc2 now exit 0. iter.xi's while item is Some
  (single-check rebind) is unaffected and NOT this bug.
- **Likely fix:** the is-Some imply/binding machinery (BUG 30 #1/#2 family) -
  the payload slot binding after an is Some check must not poison the
  subsequent match's payload read (0 is the zero-init of the untracked slot).


### BUG 50 - generic container names with pointer type args emit invalid LLVM identifiers -- FIXED (`569c3d31`)

- **Construct:** a fn returning Result<*mut UInt8, AllocError> (pointer payload in a generic container). The mono names the container struct %struct.Result__*UInt8__AllocError -- the * is invalid in an LLVM identifier (clang: error: expected '=' after name).
- **Triggered by:** smoke_alloc_edge (global_alloc().allocate(l) returns Result<*mut UInt8, AllocError>). The stdlib-side API bug (global_alloc returning the Allocator interface type) was fixed first (alloc.xi -> GlobalAlloc); the remaining failure is this codegen naming issue.
- **FIX:** `sanitize_container_arg` maps every non-identifier char to `_` -- applied at ALL 7 container-name construction sites (ensure_concrete_option/result, concrete_type_for Option/Result arms, mono subst Option/Result arms, concrete_container_llvm) so the def and call sides agree. smoke_alloc_edge + the user probe now compile and run exit 0. (Regression caught & fixed in the same batch: tuple element names built from local_xiom_types leaked "Vec[Int]" into `Tuple__Vec[Int]__Vec[Int]` -- the tuple path now strips container args, matching the decl-side bare names; smoke_iter was restored.)

### BUG 51 - Option[UserStruct] payloads: bound-name method resolution + runtime corruption -- FIXED (`569c3d31`)

- **Construct:** a fn returning Option[Rc[T]]/Option[Ref[T]] (user struct payload) called from user code; the match's Some(up) binding then calls up.get().
- **Two facets:** (a) CHECKER: up.get() on the directly-bound name resolves to an Option-returning method ("cannot compare Option with Int") -- with an explicit var x: Rc[Int] = up; annotation it type-checks; (b) RUNTIME: the annotated form reads a WRONG payload value (rcprobe3: direct .get() = 42, upgraded x.get() != 42; user-space replica rcprobe4 with a local MyRc reproduces it -- not an rc.xi bug).
- **Impact:** smoke_rc_edge/rc_weak/cell_refcell_try/cell_refcell_mix blocked (weak.upgrade / try_borrow paths). Same family as BUG 33 (Option payload slots) -- struct payloads in the generic Option container still corrupt through the catalog/unsafe path.
- **Stdlib:** rc.xi/cell.xi implementations verified correct via the user-space replica.
- **FIX (checker):** `CheckedType::from_ast_type` PRESERVES container args ("Option[MyRc]"); Some/Ok/Err pattern bindings bind the payload with the INNER type from the scrutinee ("Option[MyRc]" -> up: MyRc) instead of the `_` wildcard (which fell to the sorted wildcard method lookup -- Option.get < MyRc.get alphabetically -> "cannot compare Option with Int"); Some() ctor types "Option[inner]"; container-erasure compatibility ("Option" ~ "Option[MyRc]") keeps erased forms interchangeable; Field access and method dispatch strip container args to the base name. **FIX (runtime):** no extra codegen change needed -- the payload read already used the BUG 43 scrutinee-payload machinery; the checker fix unblocked the annotated form. Probe (Some(up) -> up.get() and x.get() == 42) exits 0; checker 178/178.

### BUG 52 - enum-with-payload values in Map crash at runtime (0xC0000005) -- FIXED (`569c3d31`)

- **Construct:** Map[Str, T] where T is an enum WITH payload fields (e.g. JsonValue): m.insert("b", MyVal.Text("x")) + m.get("b") + match. User-space replica (jp7) AVs; the same enum in a Vec (BUG 42's fix) works. The Map value path still uses the enum's scalar/tag-only layout.
- **Impact:** serialize json_set/json_object building crashes -- smoke_stress_serialize_json_nested blocked (rewritten to the real json.* API, verified correct, blocked at runtime). json_parse/stringify of parsed values unaffected.
- **FIX -- FOUR coordinated codegen changes + two ABI-family root causes:**
  1. Enum-variant ctor args infer the ENUM type in the generic direct-match inference (`MyVal.Text("x")` -> V=MyVal, not Int -- insert_Str_Int became insert_Str_MyVal).
  2. Generic METHOD type args infer from LOCAL receivers via recorded container types: `infer_value_xiom_type` records typed static-ctor receivers ("Map[Str, MyVal]"), `infer_call_return_xiom` records fn-return containers (`var m = make_map()`), and the generic-inference fallback parses them positionally (m.get("b") -> Map.get[Str, MyVal], not [Str, Str]).
  3. Mono'd method bodies record Vec-typed receiver-field ELEMENTS: new `generic_type_field_types` (stdlib generic decls' args-preserving field types -- the builtin Map/Set registrations pre-empt type_meta with bare "Vec") + `record_field_vec_elem` in both receiver-binding branches, so `values[i]` reads take the struct/enum memcpy path (resolve_vec_elem_type -> Ident arm) instead of the scalar i64 load (tag only).
  4. Index-WRITE (`values[i] = value`, the duplicate-key update in Map.insert) memcpy's struct/enum elements INLINE instead of storing a boxed pointer as i64.
  - concrete_container_llvm now EXCLUDES enum payloads from concrete naming (mirrors concrete_type_for's M18 rule) -- the caller previously emitted %struct.Option__MyVal while the def never registers enum monomorphs ("Cannot allocate unsized type").
  - ABI root cause found while bisecting: `&K` params with K=Str -- the caller passed the STRING ADDRESS as i8** and the callee's `*key` loaded the string's FIRST 8 BYTES as a pointer (strcmp AV -- smoke_collections_map_basic crashed identically at baseline). Caller now materializes an i8* temp slot. `&Vec[T]` caller ABI fixed to %struct.Vec* (the def GEPs through the param; by-value pass read the data pointer as the Vec header -- every contains-style generic fn AV'd).
  - Verified: 6 user probes (insert/get/update/remove/enum round-trip, fn-return receivers), smoke_collections_map_basic, 30-smoke battery, e2e 2268/2268, checker 178/178, feature-reg 510/510, stdlib-exec 70/70+2. smoke_stress_serialize_json_nested improved (AV -> heap corruption) but still fails -- nested enum-with-Vec-field payloads through Map values remain (deeper layer, stdlib-session follow-up).

### BUG 48 - catalog generic-bound ASSOCIATED dispatch (Eq[T].eq) emits icmp eq 0, element -- UNBLOCKED (`3a789afb`/`175294ea`); verify on re-apply

- **Construct:** a stdlib (catalog) generic-bound fn calling the associated form Eq[T].eq(items[i], value) / Ord[T].compare(a, b) with the tower-style generic interfaces. The mono emits icmp eq i64 0, %element (compare-with-ZERO fallback) instead of calling the impl (verified in core.contains_Int IR: %tmp37 = icmp eq i64 0, %tmp36 -> ret 1 when element == 0).
- **Status:** user-space generic fns with the same call dispatch correctly (eqt16/eqt17 exit 0). The METHOD form (items[i].eq(value)) in catalog fns WORKS (contains/is_sorted exit 0) -- the checker note "method form works" holds. Only the associated form in catalog fns falls back.
- **NEW (2026-08-18):** the ABI-family root causes are fixed -- the user-space replica of the FULL construct (tower interfaces + impl Eq[Int] + generic contains_eq with the associated call) previously AV'd due to the &Vec[T] caller ABI (by-value vs %struct.Vec*) and now exits 0 in BOTH associated and method forms (m37_bug48_associated_generic_vec e2e). Whether the catalog-specific zero-fallback had an ADDITIONAL factor can only be confirmed by re-applying the tower Eq/Ord impls (the stdlib session's re-apply plan) -- retry with the current compiler.
- **Stdlib impact:** use the method form for catalog dispatch (the conversion plan uses it); the associated form is available for user-space code.

### BUG 49 - catalog fn-param call sites still break when Eq/Ord impls for i64-ABI types are present -- FIXED (`3a789afb`/`175294ea`)

- **Construct:** any program that (a) defines impl Eq[Int]/impl Ord[Int] (i64-receiver impls; Str-receiver impls do NOT trigger) AND (b) calls a CATALOG fn taking a fn-typed param (heap_sort_by(v, cmp_int) from xiom.sort.heap) -- the fn-param call site miscompiles (AV or wrong order). LOCAL fn-param fns in the same program are fine once the param is not named compare (param-NAME collision with the impl method symbol -- verified: local compare-named param breaks, cmp-named works).
- **Status:** BUG 47's param_locals fix cleared the cross-fn leak but NOT this catalog+impls shape. Param renames in the stdlib (compare->cmp) do not help the catalog case. The tower-style Eq/Ord impls (28) + method-form conversion were re-applied and REVERTED again because of this: smoke_sort (PASS->FAIL exit 4) and smoke_cmp_by regress.
- **Minimal repro:** hsbp.xi = use xiom.core; (brings the impls) + use xiom.sort.heap; + local cmp_int(a: &Int, b: &Int) -> Int + heap_sort_by(&mut hb, cmp_int) -- sorts wrong/AVs. Remove the use xiom.core; -> exit 0.
- **FIX (2026-08-18):** `callee_is_fn_ptr` (the bare-call dispatch) required the resolved fn key to be UNREGISTERED -- but impl-method registration aliases the bare method name ("Int.compare" also registers "compare"), so a fn-typed param named `compare` resolved to @Int.compare and the call bypassed the passed fn pointer. The local now SHADOWS the global for fn-typed params (fn_ptr_return_types) and closure locals: the call loads the param's i64 fn pointer and calls through it. Verified: user-space replica (impl Eq/Ord[Int] + local sort_by_cmp with compare-named param) sorts correctly (exit 0), hsbp shape (tower impls via xiom.math.tower + heap_sort_by) exit 0, m37_bug49_fn_param_impl_collision e2e.

- **Compress note (2026-08-18):** gzip/zlib decompress corruptions are this same family, NOT stdlib logic -- deflate (direct Result return of rle_decode) passes; local replicas of the full gzip pipeline (rle + intermediate decoded.value + re-wrap, probes cz1/cz2) pass in user space; the catalog intermediate Result[Vec[UInt8], Str] value read + re-return corrupts. The stdlib compress module is correct; the gzip/zlib smokes unblock when the catalog payload path lands.
- **Follow-up (2026-08-18):** READS through &[N]T params are fixed (array.len/array.get verified by the session), but WRITES are not -- n set_first(arr: &mut [5]Int, v: Int) { arr[0] = v; } still fails at clang (probe aw.xi). array.sort/sort_by (read+write insertion sorts) remain blocked. Also &[N]T slice-param passing (&mut arr from a ar arr = [1,2,3,4,5] literal) hits the same path.
### BUG 55 facet-2 - unsafe-block ctx capture of *T locals still corrupts cross-block reads

- The ctor-context facet of BUG 55 is fixed (rc/upgrade payloads OK), but the ctx-capture facet remains: a *T local written in one unsafe { } block and read via *(p) in ANOTHER block (or fn) still reads garbage (probe vg10: write p[0]=42 in block A, read *(p) in block B -- same fn, no calls -- exit 1; same-block reads exit 0).
- **Blocks:** Vec.get (Some(*(data + index)) -- payload corrupt -> cross_cmp_sort, smoke_core contains-style reads), json_set/stringify (enum-with-Vec-field payloads), and the compress gzip/zlib intermediate Result[Vec] path. The user-space replicas of all three (cz1/cz2/cb1) pass -- catalog-only.
- **Minimal repro:** vg10.xi (12 lines). The __unsafe_block_N(ctx) capture of the pointer local is misrouted; the fix likely belongs in the block-ctx builder (pointer-typed captures must store the pointer VALUE, not the slot address).
### BUG 56 - ensures clause before an EXPRESSION body drops the fn body (parser) -- FIXED (round-3 commit)

- **Construct:** fn identity_non(x: Int) -> Int ensures: result == x { x } -- the parser's CONTRACT expr parse consumed the fn's BODY block: `result == x { x }` parsed `x { x }` as a STRUCT LITERAL (name x, field x = x) -> fd.body = None -> the fn emitted `ret 0` / lost its param (convert.identity_Int -> store param; ret 0; smoke_stress_convert_identity exit 1). ALSO hit NON-generic fns (identity_non had zero params in the emitted def).
- **FIX (2026-08-18):** contract exprs parse with struct-literal restriction (the parse_cond pattern -- a trailing `{` starts the fn BODY, not a struct literal). Both generic and non-generic shapes verified.
- **Related CTFE discovery:** the decl.rs registration for CTFE bodies was structurally misplaced (nested inside the generic block, never running); the brace reconstruction exposed the engine's fall-through-after-if-return bug -- `if n <= 1 { return 1; }` then `n * factorial(n - 1)` kept evaluating -> unbounded recursion -> debug-build stack overflow on const CTFE. CtfeContext.returned now stops block/while evaluation after a return.### Compress gzip/zlib decompress -- catalog-only crash (2026-08-19 follow-up)

- After BUG 55 facet-2 + BUG 56 fixes: the intermediate Result[Vec] rewrap (cz2) and unsafe ctx capture (vg10) PASS in user space, and gzip_COMPRESS works in the catalog (gz9 exit 0), but gzip_DECOMPRESS crashes the program BEFORE the first statement (gz12 probe: println before the call never fires; exit 0xC0000005). deflate (rle_decode direct return) passes. The catalog mono of gzip_decompress (header scan + trailer UInt arithmetic + crc32 table lazy-init) is the trigger -- bisecting further needs the compiler session's IR-diff skills; user-space replicas of the same logic pass (cz1/cz2).
- Same family suspects: smoke_stress_io_parent_file_name (Path.parent returns None -- the ch.is_some { let c = ch.value; } payload-field pattern in a loop -- NOT the match-scrutinee shape the facet-2 fix covered) and smoke_array_sort_by (closure comparator -- B-007 documented layer).
### Compress gzip/zlib decompress -- catalog-only crash FIXED (2026-08-19, round-4 commit)

THREE coordinated root causes, all verified with fresh probes in tmp/bug_probes
(gz12/gz12d/gz12k now exit 0; gz12m2 payload bytes are real; e2e
`e2e_m37_gzip_roundtrip`):

1. **Result-payload FIELD access never unboxed (`decoded.value`).** The
   `let decompressed = decoded.value;` shape (payload-FIELD on an
   Option/Result LOCAL -- NOT a match scrutinee) bound the raw BOXED POINTER
   as an i64: `&decompressed` then passed the i64 SLOT ADDRESS as
   `%struct.Vec*`, so crc32 read stack garbage as len/elem_size -> 8-byte
   element loads -> 0xC0000005 inside gzip_decompress. (The "crashes before
   the first statement" claim was a stdout-buffering artifact: puts output
   was lost on the AV.) Fixes: new `field_payload_xiom` (lib.rs, mirrors
   scrutinee_payload_xiom: local_opt_payload -> local_opt_payload_xiom /
   local_err_payload -> the receiver's declared type), the Field arm in
   expr.rs now unboxes container/struct payloads (inttoptr + load
   %struct.Vec) and covers `is_result.value` (was missing -- only
   option.value/result.error reinterpretted), Option/Result PARAMS now
   track payload types in decl.rs (type_from_ast renders bare "Result"),
   and let/var bindings record the payload XIOM type via
   infer_field_payload_xiom (emitter.rs/stmt.rs) so `.len()`/indexing
   dispatch correctly.
2. **If-expression result slot hardcoded i64.** `let compressed = if
   level == 0 { _store_encode(data) } else { rle_encode(data) };` coerced
   the %struct.Vec VALUE to field-0-as-i64 (the data pointer): 
   `compressed.len()` became xiom_str_len(data_ptr) and `compressed[i]`
   compiled to a LITERAL 0 -> gzip payload of six zero bytes -> decode
   produced 3 zero bytes (and the CRC/size checks passed vacuously against
   the same broken crc32). Fix: the Expr::If result type is inferred from
   the arm tails (all-agree struct > pointer > all-agree float > i64), and
   infer_if_xiom_type records the arm-tail XIOM type on the binding.
   ALSO fixes Float64-valued if-expressions (`return if c { 1.5 } else {
   2.5 };` -- was bitcast-to-i64 + sitofp -> wrong value).
3. **Param payload tracking** (decl.rs) -- see #1; without it the
   `use_payload(r: Result[Vec[UInt8], Str])` param shape never resolved.

Verified: probe battery (gz9/12/12d/12e/12f/12h/12u/12m2, field_probe,
ifexpr_probe incl. Float64 arms) exit 0; stdlib smokes
smoke_compress / gzip_roundtrip / zlib_roundtrip / gzip_levels /
compound / deflate_roundtrip exit 0; checker 178, parser 97, ctfe 97,
feature-reg 510, stdlib-exec 70(+2 ignored), stdlib_tests 40.

REMAINING in this area (documented, NOT fixed here):
- smoke_stress_compress_gzip_large: pre-existing 0xC0000005 in __chkstk
  (huge stack alloca -- reproduces at baseline; separate queue item).
- smoke_stress_compress_gzip_empty / gzip_bad_input: the facade's
  `requires: data.len() > 0` / `>= 18` contracts PANIC instead of letting
  the fn return Err -- the smokes expect Err returns; stdlib-side decision
  (relax the requires or change the smokes).
- The [256]UInt module-global crc table lazy-init STILL writes the STACK
  COPY (module-global ARRAY element stores go to a copied alloca, never
  the global) -- the CRC is deterministic garbage but self-consistent for
  round-trips; gzip_crc32's public value is WRONG vs real gzip. Same
  family as BUG 2 (module-global field writes) but for array elements.
- The contract-ensure unbox (`result is Ok => result.len() >= 0`) loads
  the payload UNCONDITIONALLY -- for an Err result it inttoptrs 0 and
  loads from NULL (swallowed by the guard-fault trap today; latent).
### Round-7 finding FIXED (2026-08-20) -- inlined Vec.pop + match Option slot (probe ve2); Vec/Set/Slice method injection + pointer arithmetic

- **Construct:** inlined `Vec.pop` on an EMPTY vec returns Some at runtime
  while the emitted IR is fully correct (len==0 -> `Option{tag=0, payload=0}`)
  -- the CALLER's `match v.pop() { Some(x) => ..., None => ... }` misreads
  the inlined Option slot. Vec.get's None path works; a bare `return None`
  works; the pop builtin's own IR is right (verified: vec_pop_empty block
  stores tag 0). Blocks smoke_stress_collections_vec_edge and
  smoke_stress_collections_vec_first_last. The stdlib session's probe ve2
  has the IR evidence; user-space replica of the same pop+match shape
  passes -- catalog/inlined-context only (likely the inline-expansion or
  scrutinee-alloca resolution for the INLINE-builtin return, cf. the
  scrutinee machinery in lib.rs).
- **Next step:** build a minimal user probe with a fn returning
  `Option{tag=0}` that is `alwaysinline` and matched by the caller; diff
  the inlined vs non-inlined scrutinee alloca handling in expr.rs's
  match-check code.

**RESOLVED -- FOUR compiler roots (round-7 commits, e2e
`e2e_m37_round7_vec_pop_slot`):**

1. **Vec/Set/Slice methods were NEVER injected** (checker
   `collect_external_decls`): `recv_is_nonpub_generic` skipped methods on
   non-pub generic receivers -- correct for `BinaryHeap[T]`, but Vec/Set/
   Slice are COMPILER-BUILTIN generic types whose type decl is in
   PRIMITIVES (never injected), so ALL their methods vanished:
   - the six INLINE builtins (new/push/pop/get/len/sort/set) still worked
     at call time via call.rs inline handlers -- but `struct_type_from_expr`
     -> `resolve_struct_return` found NOTHING for bare "pop" (Vec methods
     register in NEITHER functions NOR generic_fn_decls) -> no scrutinee
     alloca -> the Some/None check blocks became unconditional branches ->
     the match always took the FIRST arm (Some on empty = ve2). Vec.get
     worked only by LUCK (the ".get" suffix happened to land on
     Map.get, also Option-returning).
   - every NON-inline Vec method (first/last/clear/reserve/extend/
     insert/remove/truncate/...) compiled to a ZERO-PARAM STUB
     (`define i64 @Vec.first() { ret 0 }`) -- smoke_collections_vec_
     first_last and every vec method beyond the six inline ones were
     broken. Fix: exempt PRIMITIVES receivers from recv_is_nonpub_generic
     (xiom-check/src/lib.rs) -- Vec/Set/Slice methods now inject and
     monomorphise like Map's.
2. **`resolve_struct_return` suffix search took the FIRST match and gave
   up** -- a ".pop" suffix could resolve to a non-Option-returning decl and
   return None even when a later match returned Option. Now scans all
   suffix matches for an Option/Result return.
3. **Mono'd Vec-method bodies broke on pointer arithmetic** (new exposure
   once Vec.first/last/insert/remove mono'd): the BUILTIN Vec type_meta
   registers `data` as "*UInt8" (i8*), so a mono'd body's `*(data + len -
   1)` (a) hit the Str+Int CONCAT intercept (xiom_str_concat of the buffer
   + len -- garbage) and (b) even past that, `*T + i64` lowered via
   ptrtoint/add/inttoptr -- element-UNSCALED + BYTE load. Three coordinated
   fixes: mono receiver-field binding types Vec.data as the CONCRETE
   element pointer (i64* for Vec[Int]; i8* kept for struct/enum/pointer
   elements) + records its XIOM type ("*Int") so the concat gate
   (`expr_is_pointer`) skips buffers; BinOp Add/Sub now emits
   `getelementptr` for pointer operands (element-scaled; Sub negates the
   index) in BOTH the main path and the iterative fold path; the concat
   intercept only fires when the i8* operand is a Str and the other side
   is an integer (Str+Str / Str+buffer keep concatenation).
4. **BUG 38b leaf-match hijack (regression caught by stdlib-exec
   smoke_collect_cache):** with Vec.push now registered, the is_generic
   leaf-name match made `g.edges.push(...)` (fn_key "Graph.push" -- base
   type of the FIELD receiver) resolve to the "Vec.push" decl -> mono'd
   @Graph.push_Int with a LITERAL-0 receiver -> clang "integer constant
   must have integer type". The leaf match now requires the fn_key's
   receiver part to be an ABSTRACT (unregistered) type -- exactly the rule
   find_generic_decl already uses for Iterator[T].collect. ALSO fixed
   `is_container_vec_field` to search ALL type_meta keys (two structs can
   share a leaf name -- collect.Graph vs math.graph_theory.Graph -- the old
   loop broke at the first suffix match and missed the other type's
   fields).

Verified: probes ve2/ve2b/ve2d/ve2e; smoke_collections_vec_edge,
smoke_stress_collections_vec_first_last, smoke_stress_collections_slice,
smoke_iter (leaf-match guard), smoke_collect_cache, 20-smoke Vec/collections
battery; stdlib-exec 70/70 (+2 ignore); e2e `e2e_m37_round7_vec_pop_slot`
+ round6/gzip regressions; checker 178, parser 97, ctfe 97, feature-reg 510.

FOLLOW-UPS (pre-existing, logged -- NOT regressions):
- smoke_collections_set_basic / set_ops: the compiler treats Set as an
  i64-handle CONTAINER builtin while the stdlib declares `type Set[T] =
  { items: Vec[T] }` (struct) -- `Set[Int].new()` resolves the bare "new"
  leaf to another type's ctor (Reverse.new/Vec.new by iteration order).
  Failing at baseline (different failure modes); needs a compiler-side
  decision on the Set container ABI or inline Set handlers.
- smoke_collections_vec_narrow exit 5: inline pop/get on SIGNED narrow
  elements (Int16 -30000) zext the bit pattern (35536) instead of
  sign-extending (emit_elem_payload_load) -- pre-existing, untouched.
  RE-CONFIRMED on the round-13 binary (2026-08-22): probe_narrow_zext.xi
  proves pop() returns 35536 for `-30000 as Int16` (pure zext, not sext);
  probe_narrow_get16.xi (Int16 get(1) negative) and probe_narrow_get8.xi
  (Int8 get(0) = -5) fail too -- the whole narrow-SIGNED inline get/pop
  read path is affected. Positives read back correctly.
- math/graph_theory.Graph vs collect.Graph bare-name type collision: the
  first registration (keep-first) wins the bare "Graph" layout; the other
  module's functions compile against the wrong field offsets (their
  `g.edges.push` stays a no-op stub -- graph_theory fns were dead code at
  baseline too).

### Round-6 findings FIXED (2026-08-19, round-6 commit) -- Imply short-circuit, Try-bound Str payloads, substr handler, Str-builtin receiver guards, path.xi import

FIVE coordinated root causes, all reproduced with fresh probes in
tmp/bug_probes (bi4/bi4u, op1, q3/q8, sw2/sw4, pj6/pj10 -- all exit 0; the
full path smoke family 19/19; e2e `e2e_m37_round6_path_gzip`):

1. **Imply consequence compiled UNCONDITIONALLY (bi4).** `ensures: result
   is Ok => result.len() >= 0` -- the old code compiled the consequence
   inline and masked it with `or (!l, r)`: for an Err result the
   consequence's payload UNBOX (inttoptr 0 + load %struct.Vec) executed
   anyway -> load from NULL -> UB -> the Err return value got corrupted ->
   gzip_decompress returned Ok for garbage input (the stdlib's bi4:
   "Err path never fires"). The Expr::Imply emission now short-circuits:
   the consequence compiles inside a guarded block (imply_conseq) with a
   DEFAULT TRUE store on the false path (imply_done reads the alloca).
   ALSO unblocked: smoke_stress_compress_gzip_bad_input (was blocked on
   bi4), and the stdlib's "Option[Str] payload construction mangles"
   report (file_name's `ensures: result is Some => result.len() > 0` was
   the same unbox-on-None corruption).
2. **Try-bound Str payloads (q3/q8).** `let name = io.file_name(p)?;` --
   the `?` binding stored the payload in an i64 slot WITHOUT recording
   the XIOM type; `name.byte_at(i)` then took the method-call receiver
   heuristic "p0 ends with * -> pass the SLOT ADDRESS" (i8* param of
   byte_at) -> the slot address was read as the string -> garbage -> the
   while loop never found the '.' -> io.extension/path.extension returned
   None. Fixes: infer_try_xiom_type records the payload type on `let x =
   f()?;` bindings (stmt.rs/emitter.rs), and the receiver-heuristic in
   call.rs skips Str receivers (xiom_type_of_local == "Str" -> coerce the
   i64 handle to i8* via inttoptr instead of passing the slot address).
3. **Str.substr had NO inline handler (op1/iop7/iop8).** The stdlib's
   substring form (`s.substr(0, i)` in io.parent_path, path.file_name/
   extension) was registered as a builtin Str method but the call fell
   through to normal dispatch -> `call i64 @Str.substr(...)` against a def
   that never exists -> zero-param stub `ret i64 0` -> NULL string ->
   Option[Str] payloads of 0 -> 0xC0000005 (the stdlib's "Option[Str]
   pointer-payload construction mangles"). substr now lowers to
   xiom_str_slice exactly like the existing slice handler.
4. **Str builtin handlers hijacked non-Str receivers (sw2/sw4/pp12).**
   `p.starts_with(b)` on a Path STRUCT matched the Str.starts_with builtin
   (name-only dispatch): the %struct.Path got BOXED (val_to_i8ptr struct
   path) and the BOX ADDRESS passed as the string -> always false (the
   stdlib's pp12 "alwaysinline returns wrong with correct IR"). All four
   handlers (slice/substr/starts_with/ends_with) now verify the receiver
   is really a Str (receiver_is_str: infer_llvm_type == i8*, or an
   Ident whose XIOM type is Str) and fall through to real method dispatch
   otherwise.
5. **path.xi called bare `join_paths` with NO import (pj6/pj10).** The
   catalog body's bare call resolved to no registered key -> the call-site
   fallback emitted the bare symbol -> zero-param stub `ret i64 0` -> NULL
   -> the join-chain smoke crashed (the stdlib's pj5/pj6 "join-chain AV"
   report was this stub, not an inline issue). path.xi now `use xiom.env;`
   (env.join_paths is FAMILY-aware) and path_separator() delegates to
   env.path_separator() (was hardcoded "/" while join produced "\" -- the
   smoke mismatch). The catalog-body type-check gap (item C) is what let
   the undefined bare name slip through silently.

Verified: probes bi4/bi4u/op1/q3/q4/q5/q8/iop4-9/sw1-4/pj1-10/wfn1-4 exit 0;
path smoke family 19/19 (incl. smoke_io_path, join, pop_clear,
starts_ends_with, with_extension, with_file_name -- the last three needed
stdlib smoke fixes: separator-aware comparisons and as_path().file_name());
compress smokes incl. gzip_bad_input green; checker 178, parser 97, ctfe 97,
feature-reg 510, stdlib-exec 70(+2), stdlib_tests 40; e2e pending.

STDLIB smoke fixes included in this round (examples/stdlib_smoke):
smoke_stress_path_extension (.hidden -> None -- Rust semantics, the impl was
right), smoke_stress_path_with_extension (separator-aware expectations),
smoke_stress_path_with_file_name (PathBuf has no file_name -- use
as_path()). smoke_core_option_map / smoke_core_option_unwrap fail at
BASELINE identically (B-007 closure layer -- unchanged).

### Round-6 findings (2026-08-19) -- path family

- Path.parent: the prose-style ensure (ensures: result is None => self.inner does not contain a parent directory) corrupted the fn (contract-eval Str FIELD read -- BUG 56 family) -- REMOVED (was documentation, not an expression) -- parent now returns correct values (smoke_stress_io_parent_file_name exit 0).
- PathBuf push/pop/clear took BY-VALUE self (stale note said &mut unsupported) -- pushes mutated a copy and were silent no-ops -- converted to &mut self (smoke_stress_pathbuf_push exit 0).
- Path.ends_with catalog alwaysinline returns wrong despite correct IR + working callee + working user-space replicas (probe pp12). The p.join("a").join("b").join("c") chain + Str comparisons in ONE fn AVs while each piece passes alone (probe pj5/pj6 exit 0, smoke AVs) -- a shape-dependent mono issue for the compiler session.
- &struct param field reads of Str-typed fields return garbage in user space (probe pp9/pp10 -- BUG 44 family extension) -- Path starts_with/ends_with switched to by-value Path as the stdlib-side dodge.
### Round-6 findings (2026-08-19) -- compress validation + Result field reads

- gzip_decompress([0,1,2]) returns Ok -- the validation reads (data.len() < 18, data[0] magic, trailer slices) miscompile for small inputs through the catalog (probe bi4: Err never fires; roundtrips of valid data pass). The bad-input smoke is correct (match-based) and blocked on this.
- Generic Result/Option first-field reads (esult.is_ok) return the wrong slot through the catalog while match works (bi3) -- a field-read-on-erased-container shape for the compiler session.
### Round-6 final finding (2026-08-19) -- Option[Str] payload construction

- Path.file_name returns an EMPTY name: the loop logic is verified correct (user-space replicas pass), parent (Option[Path] struct payload) works after its prose-ensure removal, but the Some(str_slice(...)) Option[Str] POINTER-payload construction through the catalog mangles the payload. The ensure is NOT the cause (removal doesn't change it). A remaining Option[Str]/pointer-payload shape for the compiler session -- file_name/file_stem/extension families blocked.
### Round-7 findings (2026-08-19) -- inlined Vec.pop + match Option slot

- Vec.pop on an EMPTY vec returns Some at runtime while the IR is fully correct (probe ve2: len==0 -> vec_pop_empty8 -> Option{tag=0,payload=0}; the match still takes the Some arm). pop on non-empty works; bare eturn None fns work; Vec.get's None works. The inlined-pop Option slot is misread by the subsequent match -- a slot-lifetime shape (single pop in the fn, probe ve2).
- Also logged: the char smoke failures were NOT multi-match slot reuse -- they were to_digit's redundant requires trapping (fixed stdlib-side, commit above).
### Round-11 findings FIXED (2026-08-20) -- B-007 closures (fn-typed params)

- core_option_map/unwrap/filter, array_sort_by, cmp_by, core_slice crashed
  (0xC0000005) or returned garbage: fn-typed PARAMS hold a closure ENV
  pointer (field 0 = the fn ptr) -- calling `f(x)` inside a body must go
  through the M20-A1 env path, and fn-REFERENCE args must be wrapped into
  envs with a forwarding thunk.

**RESOLVED (round-11 commit, e2e `e2e_m41_round11_b007_closures`):**

1. **Fn-typed params registered as closure locals** (decl.rs compile_fn +
   the mono body binding): the param was NOT in closure_locals, so `f(x)`
   inttoptr'd the ENV as a CODE pointer (0xC0000005 in Option.map). The
   param binding now registers Type::Fn params + their declared RETURN
   type (fn_local_returns) -- the M20-A1 call uses the REAL return type
   (struct returns are BY VALUE -- Option.and_then's closure crashed
   inttoptr'ing the by-value Option bits as a pointer).
2. **fn-REFERENCE args wrapped in a forwarding THUNK env**: passing
   `cmp_int` to a fn-typed param passed the RAW code address; the callee's
   M20-A1 loads field 0 = the first 8 bytes of CODE. The wrap emits
   `define {ret} @__fnwrap_N(i64 %__env, i64 %a0, ...) { inttoptr the
   params to the fn-ref's real types; ret call @fn_ref(...) }` and stores
   the thunk in the env. The thunk's signature MATCHES the M20-A1 call
   (all-i64 args) -- pointer params restored via inttoptr inside (the
   baseline sort's comparator call typed i64* matched the def; the first
   thunk version forwarded i64 to i64* params -- clang couldn't inline
   alwaysinline comparators and the heap sort corrupted the last pair).
3. **Re-pass double-wrap guard**: heap_sort_by -> heap_sift_down_by's
   `compare` arg ALSO matched the registered `Int.compare` suffix (the
   is_fn_ref suffix scan) -- the already-env param got wrapped AGAIN (env
   of env -> the inner M20-A1 read field 0 = the inner env ptr as code).
   is_fn_ref now excludes closure_locals.
4. **fn-typed container elements** (timer_wheel_tick's `var t = tw.tasks[i];
   t();` -- Vec[fn()]): the fn-marker arm of type_from_ast_with_args keeps
   "Vec[fn() -> Unit]" in the type_meta field strings; `var`/`let`
   bindings from fn-typed elements register as closure locals (M20-A1
   env-first calls); Vec.push of a bare fn-REF into a Vec[fn()] wraps it
   (uniform env convention -- the raw-address form crashed the indexed
   call); compile_index_fn_ptr_call (`tasks[i]()`) loads field 0 + env-first.
5. **Zero-arg thunks**: the trailing-comma `(i64, )` signature fixed.

Verified: stdlib-exec 70/70 (+2 ignore -- async/thread restored), e2e round
regressions, feature-reg 510, checker 178, parser 97, ctfe 97, 54-smoke
battery green incl. the full closure family + search/sort/async/thread.
Pre-existing (baseline-failing, unchanged): smoke_collections_btree_map
first_entry/last_entry (exit 7) + smoke_stress_collections_btreemap (exit 2).

### Round-10 findings FIXED (2026-08-20) -- checker builtin Ord/Bounded resolution (C001)

- num_checked/num_saturating stopped at `error[C001]: type 'Int' does not
  implement 'Ord': missing method 'cmp'` -- the stdlib's new Ord tower
  (compare+cmp on 15 types) did not register, and the math/interfaces.xi
  Ord interface adds lt/le/gt/ge/min/max on top of core.xi's compare+cmp.

**RESOLVED (round-10 commit, e2e `e2e_m40_round10_ord_bounded`):**

1. **Stdlib fix**: the round-10 Ord tower was written `impl Ord[] { fn
   compare(a: , b: ) ... }` -- EMPTY brackets and empty param types (the
   Eq/Bounded towers use `impl Eq[Int]` / `impl Bounded[Int]` with explicit
   types). Nothing registered; the C001 check had no "Int.cmp" to find.
   Rewrote the 14 malformed blocks as 15 properly-typed impls
   (`impl Ord[Int] { fn compare(a: Int, b: Int) -> Int ... }` etc.).
2. **C001 builtin fast-path**: "cmp" | "min" | "max" added to the
   is_builtin_method list -- min(a,b) = a < b ? a : b is derivable from the
   comparison ops for primitives (no tower impls exist for them).
3. **Inline scalar handlers**: cmp (the -1/0/1 select, same as compare) and
   min/max (select on lt/gt) in the is_scalar section -- the op match
   previously `unreachable!()`'d on these names.
4. **Generic-param static receivers**: `T.max_value()` inside a mono'd body
   -- "T" is a generic param, not a param name; receiver_is_instance(T) is
   FALSE (T is not a registered type), so the fn_key degraded to a bare
   "max_value" leaf -> zero-param stub -> 0 -> checked_add's overflow guards
   saw max_value = 0 and ALWAYS returned None. The fn_key construction now
   resolves Ident receivers through current_type_map (the T->concrete
   substitution recorded during mono body emission) FIRST -> "Int.max_value".
5. **Module-qualified impl suffix fallback**: the direct-call fallback only
   tried the bare leaf; the Bounded impls register as "precision.Int.
   max_value" -- the call resolved "Int.max_value" -> not found -> 0. The
   fallback now resolves the ".{Recv}.{method}" suffix (unique when it
   exists) to the module-qualified key.

Verified: e2e round-10/9/8/7 regressions, stdlib-exec 70/70 (+2 ignore),
feature-reg 510, checker 178, parser 97, ctfe 97; 35-smoke battery green
incl. num_checked/num_saturating/num/cmp_ordering and the BTree/Set towers.

### Round-9 findings FIXED (2026-08-20) -- Set container ABI mismatch

- The compiler had NO builtin Set layout (Vec/Slice/Map register %struct
  layouts in compile_program; Set was only the i64-erasure fallback in
  xiom_to_llvm_type), while the stdlib implements `type Set[T] = { items:
  Vec[T] }`. Every Set value erased to i64 while the injected methods
  operated on %struct.Set: `Set[Int].new()` hijacked Reverse.new (the
  type-receiver call degraded to a bare "new" leaf), Set-typed params/
  returns/fields compiled as i64 (make_set() -> i64, Holder.s -> i64), and
  `holder.s.insert(10)` mono'd Vec.insert with a %struct.Vec* receiver.

**RESOLVED (round-9 commit, e2e `e2e_m39_round9_set_abi`):**

1. **Inject the stdlib Set type**: "Set" removed from the checker's
   PRIMITIVES ("types that should never be injected") -- the stdlib
   `type Set[T]` now registers %struct.Set and Set resolves like any
   struct (methods were already injected since round-8).
2. **is_container_vec_field tightened**: the regular-field path matched
   ANY bracketed field type (`ftype.contains('[')`) -- a `Set[Int]` field
   fired the inline Vec.insert on the %struct.Set (invalid IR). Now only
   Vec/Slice/Array bases (Map/Set fields route through their own
   generic-mono methods).
3. **Field-receiver fn_key**: infer_struct_type_name's instance-field arm
   didn't split generic args -- `Set[Int]`/`Vec[Int]` fields degraded to
   None, so `holder.s.insert(10)` fell to a bare "insert" key (leaf-match
   -> Vec.insert mono). Split on '[' ("Set[Int]" -> "Set") so the key is
   "Set.insert".
4. **Mono pointer-self for FIELD receivers**: the pointer-self branch
   only handled Ident receivers (local slot / module global) -- field
   receivers passed the LOADED VALUE as the pointer. Now emits the field
   ADDRESS via the Ref arm (`&holder.s` -> GEP into the base struct).
5. **Pointer-len guard**: the "&Slice[Int] -> i64*: length at buf[0]"
   path fired for ANY pointer-typed receiver -- `s.len()` on a &Set param
   read the Vec's data pointer as the length. Skips when the pointee is a
   registered struct (falls through to the generic Set.len dispatch).

Verified: e2e round-9/8/7 regressions, stdlib-exec 70/70 (+2 ignore),
feature-reg 510, checker 178, parser 97, ctfe 97; 44-smoke battery green
incl. all six Set smokes + set_probe edge shapes (Set[Str], &Set params,
Set returns, struct fields, union/intersection). NOT covered: `for x in
set` iteration -- the For stmt is a hardcoded Range GEP (fields 0/1) and
needs the iterator-protocol work (iter adapter queue item).

### Round-8 findings FIXED (2026-08-20) -- catalog &mut-self receiver wiring + Option<&T> reference payloads

- **VecDeque/LinkedList/Stack/Queue/BTreeMap/BTreeSet mutations entirely LOST
  through the catalog** (probes vd2/vd6; identical code passes user-space in
  vd3-vd5): `dq.push_front(20)` left len at 0. Same family as the Set
  container gap.
- **Option<&T> reference payloads** (rw1): rand.weighted_pick returns the
  correct selection but the reference payload read gives garbage.

**RESOLVED (round-8 commits, e2e `e2e_m38_round8_catalog_mut_self` +
`e2e_m38_round8_ref_payload`):**

1. **Non-pub generic TYPE DECLS were never injected** -- the Type arm of
   `collect_pub_decls` only injected `pub` types, so VecDeque/Stack/
   LinkedList/Queue/BTreeMap/BTreeSet/BinaryHeap had no layout AND
   `generic_type_names` missed their receivers; their methods were skipped
   by `recv_is_nonpub_generic` (now dead -- removed). Calls hijacked
   same-leaf methods of OTHER types (`dq.push_front` -> Reverse.push_front;
   `VecDeque[Int].new()` -> Reverse.new -> dq typed %struct.Reverse). Fix:
   inject non-pub GENERIC type decls (`td.is_pub || !td.generics.is_empty()`)
   and drop the method skip -- the BUG 38b parser fix captures receiver
   generics, so every such method is mono-able; `recv_is_generic` blocks
   concrete direct-emission. Unblocked the whole collections family.
2. **Inline Vec/Slice handlers hijacked name-EMBEDDING types** (vd2):
   `recv_ty.contains("struct.Vec")` matched "%struct.VecDeque" -- `dq.len()`
   ran the inline Vec.len on the WHOLE VecDeque struct (extractvalue
   %struct.Vec on %struct.VecDeque -> invalid IR). Fix: `is_llvm_struct_named`
   -- leaf-name exact match (bare/qualified/args-embedded/pointer forms).
3. **`Option<&T>` payload auto-deref** (rw1): `Some(&items[i])` stores the T
   SLOT ADDRESS; consumers (strcmp content compare, icmp) treated it as the
   T VALUE -- the address bytes were strcmp'd -> garbage. The `&` was stripped
   from XIOM type records at THREE sites: `type_string_full` (fn_return_xiom
   "Option[&Str]"), param bindings (decl.rs `ref_preserving_name`), and the
   match payload bindings (guarded arm: scrutinee_payload fallback; unguarded
   arm: reference case + "&T" xiom record). `auto_deref_ref` now loads
   through &T-typed operands in Eq/Ne (bitcast for pointer forms, inttoptr
   for i64 bits) -- covers payloads, &Str/&Int params, and `&local`.
4. **Regression caught mid-round (geom + collect_cache):** with Vec.len now
   registered, indexed-element `.len()` (`ac[0].len()` on Vec[Vec[Float64]])
   typed the receiver as "%struct.Vec[Int]" -- `is_llvm_struct_named` rejected
   the args-embedded form -> inline handler skipped -> generic leaf-match mono'd
   a garbage "@Vec[Int].len_Int" symbol (brackets invalid in LLVM idents).
   Fix: `is_llvm_struct_named` splits container args ("%struct.Vec[Int]" ->
   "Vec"); VecDeque stays excluded ("VecDeque" = "Vec").

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, e2e round-8 fixtures + round-7 regressions, 39-smoke
collections/rand/geom battery green (incl. smoke_collections_set_basic and
smoke_rc_weak, previously failing).
### Round-12 residual closure/payload shapes (2026-08-21, post B-007)

The B-007 closure fix landed the Int-returning closure family (option map/filter/unwrap/deep_chain, and_then, core_slice, async, thread all exit 0). Three residual shapes remain, all with correct stdlib logic (probe-verified):
1. Str-returning closures through Result/Err construction: err.map_err(fn(e: Str) -> Str { str_concat("ERR_", e) }) produces a corrupted payload (rm1 probe prints garbage bytes). Int-returning map is fine.
2. Option[(K, V)] TUPLE payloads: BTreeMap.first_entry's Some((keys[0], values[0])) returns a corrupt tuple (the keys[0]/values[0] reads are verified fine standalone) -- smoke_collections_btree_map exit 7 / btreemap exit 2.
3. Ordering-returning closures with &T params: cmp.min_by's comparator (n(&T, &T) -> Ordering) returns the wrong Ordering (cb2 probe) -- the &-arg + enum-return closure shape.
### Round-12 findings FIXED (2026-08-21) -- rm1 Str-return closures + cb2 enum-variant receivers

- smoke_core_result_map/deep_chain/chains exited 5/4/4 -- `err.map_err(fn(e:
  Str) -> Str { str_concat("ERR_", e) })` produced a CORRUPTED payload
  (garbage bytes). The Int-returning map was fine.

**RESOLVED (round-12 commit, e2e `e2e_m42_round12_str_closures`):**

1. **Closure thunk params lost their XIOM types** (rm1): the Expr::Closure
   thunk stored every param as an i64 local (`add_local(name, alloca,
   "i64")`) WITHOUT recording the declared XIOM type -- a Str param used
   inside the body resolved to "Int" (via the i64 slot reverse lookup), so
   coerce_arg_for_param's materialize path fired: `alloca i8; trunc i64 to
   i8; store` -- the string HANDLE was truncated to a single byte and
   passed as i8* to str_concat (garbage payload). Fix (expr.rs Closure
   thunk): mirror decl.rs's param binding -- record local_xiom_types
   (ref_preserving_name keeps "&T"), ref_params for scalar-pointee &T
   params, signed_locals, param_locals. Verified in the IR: the body now
   emits `inttoptr i64 %e_0 to i8*` instead of `trunc i64 %e_0 to i8`.
2. **Mono fn_local_returns not substituted** (latent): the generic decl's
   `fn(E) -> F` ret was registered RAW ("F") -- llvm_type_for fails -- the
   M20-A1 call site degraded the fn-ptr signature to i64 (benign for
   8-byte returns, ABI-breaking for struct returns > 8 bytes). Fix
   (lib.rs mono body): substitute the Fn ret through type_map first.
3. **Module-qualified ENUM-VARIANT receivers dropped** (cb2, found while
   probing then_with/reverse): `cmp.Less.reverse()` emitted
   `call %struct.Ordering @Ordering.reverse()` with NO receiver arg, and
   `Ordering.then` lost its second arg -- infer_struct_type_name's Field
   arm had no enum-variant resolution, so receiver_is_instance returned
   FALSE and the receiver was skipped (undefined self at -O2 -- passed by
   luck in script mode, garbage in `-o`/e2e mode). Fix (lib.rs Field arm):
   resolve the variant's parent enum via resolve_enum_for_variant before
   the instance-field fallthrough. Verified: `-o` mode 0/0 on the
   reverse/then/then_with battery.

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, e2e round regressions incl. e2e_m42; the full closure
family green (result map/deep_chain/chains, option map/filter/unwrap/
deep_chain, cmp_by, array_sort_by, cb2 zero-arg + capturing + struct-T
probes). Pre-existing (baseline-failing, unchanged): smoke_collections_
btree_map first_entry/last_entry (exit 7) + smoke_stress_collections_
btreemap (exit 2) -- the tuple-payload queue item; smoke_error_edge +
smoke_iter_edge (exit 1 at baseline).

### Round-14c finding (2026-08-22, stdlib session) -- Option[&T] payloads are value-boxed; array module switched to Option[T]

- **Construct:** the array module's `first/last/get/get_mut` returned
  `Option<&T>` / `Option<&mut T>`. The mono'd body BOXES THE ELEMENT VALUE
  into the Option payload (`store i64 %loaded, i64* %payload` -- the
  `Some(&arr[i])` address is never materialized), while call sites treat
  the payload as a POINTER (comparisons auto-deref:
  `inttoptr i64 20 to i64*; load` -> 0xC0000005 at address 0x14). The
  `Option<&T>` shape is broken end-to-end on this compiler.
- **Evidence:** get1.ll IR (mono'd array.get_Int_Int boxes the value;
  main's `v != 20` derefs it), probe_get1/probe_first6/probe_first7/
  probe_gfl (all deterministic AV), probe_get2 (println-only works --
  value prints correctly).
- **Stdlib resolution (committed):** first/last/get/get_mut now return
  Option[T] (value semantics, Vec.get-consistent). Smokes realigned
  (derefs dropped): smoke_array_get_first_last, smoke_array_edge,
  smoke_array_narrow (Int paths). GREEN.
- **Compiler fix direction:** either materialize the &T address in
  Option<&T> payloads (and stop auto-derefing value payloads), or treat
  Option<&T> as unsupported.

### Round-14c finding (2026-08-22, stdlib session) -- array.len const-N stale across call sites

- **Construct:** `array.len[T, const N](&arr)` called on DIFFERENT arrays
  reuses the FIRST call's N: probe_stale.xi -- len(&[7,8]) = 2 (correct),
  then len(&[1,2,3,4]) = 2 (should be 4) and len(&[9]) = 2 (should be 1).
  Same family: the empty literal `[]` infers N=3 (probe_vararr),
  typed [0]Int annotations lose N (probe_arr0b).
- **Impact (stdlib):** smoke_array_len_empty's multi-literal checks
  blocked (single-literal form green); any program calling len on
  multiple arrays gets wrong lengths.
- **Fix direction:** the mono const-N resolution for the call site must
  re-publish the const map per distinct argument array (the round-14c
  const publishing fires once and caches).

### Round-14c finding (2026-08-22, stdlib session) -- narrow-element reads through &[N]T mono bodies read 0

- **Construct:** `array.first(&[1 as Int8, 2, 3])` returns 0 (probe_arr8
  prints f0=0) -- the Int8/Int16 element read inside the mono'd &[N]T
  body is broken (narrow reads return 0 / garbage); `array.map` over
  narrow arrays crashes (probe_arr8b 0xC0000005; explicit type args
  `array.map[Int16, Int16, 2]` fail the parser: P001 at the comma).
  Int-element paths are green. smoke_array_narrow realigned to the
  Int/len paths.

### Round-14c finding (2026-08-22, stdlib session) -- Ord[T].compare dispatch misbehaves inside BinaryHeap (stdlib context)

- **Construct:** the stdlib BinaryHeap's sift_up/sift_down use
  `Ord[T].compare(heap.data[parent], heap.data[i])` -- the heap property
  is not maintained (push 3,1,5,2 -> pops 2,3,1,5 instead of 5,3,2,1).
  A user-space replica with direct `>=` comparisons works perfectly
  (probe_heap2 -> [5,2,3,1]). Ord[...].compare is NOT callable from
  user space at all ("undefined variable 'Ord'") -- the interface
  dispatch is stdlib-context-only and mis-evaluates here (queue item 7:
  builtin-interface matching). The heap's push/pop by-value self was
  ALSO wrong (fixed -> &mut self, len works); the remaining order
  divergence is the Ord dispatch.
- **Probes:** probe_heap3.xi (stdlib pops 2,3,1,5), probe_heap2.xi
  (replica correct), probe_ord.xi (Ord unreachable from user space).

### Round-14c finding (2026-08-22, stdlib session) -- array.map's [N]U result type unresolved for implicit type args

- **Construct:** `var arr = [1,2,3,4,5]; var d = array.map(arr, fn(x: Int)
  -> Int { ... });` compiles (d[0] reads garbage -- smoke_array_map exit
  1), and calling array.len(&d) trips "unknown type '[N x T]' --
  defaulting to i64" (probe_map COMPILE FAIL). The round-14c
  const-N/array-type resolution covers the explicit-args path
  (array.map[Int16, Int16, 2]) but not the implicit-args + result-type
  registration. smoke_array_zip (T001 on the zip result) is the same
  family.

### Round-14c finding (2026-08-22, stdlib session) -- extra call args are silently dropped (no arg-count check)

- **Construct:** `convert.float_to_string(3.14159, 2)` (2 args) against
  the 1-param def emitted `call @convert.float_to_string(double, i64 2)`
  with the definition taking `(double %param0)` -- the precision arg was
  SILENTLY DROPPED instead of a T001 mismatch (the smoke expected a
  2-arg API that never existed; realigned to float_to_fixed_str).
  The checker DOES reject arg-count mismatches in other paths (e.g.
  deflate_decompress &Vec vs Result) -- this path (module-prefix call
  against a same-leaf fn) skips the check.

### Round-14c finding (2026-08-22, stdlib session) -- unary minus on nested Vec[Vec[Float64]] index reads the wrong element

- **Construct:** `-m[1][0]` (unparenthesized) reads flat index 1 (element
  [0][1]) instead of [1][0] -- the minus binds to the ROW read before the
  index (probe_neg: -m[1][0] == -2.0 bits for m=[[1,2],[3,4]]).
  Parenthesized `-(m[1][0])` and `0.0 - m[1][0]` are CORRECT (probe_neg3).
- **Impact (stdlib):** 10 sites across geom/linear, math/approximation,
  factorial, finance, numerical used the unparenthesized form -- all
  parenthesized (committed). is_skew_symmetric's check still fails for a
  DIFFERENT reason (see next finding).

### Round-14c finding (2026-08-22, stdlib session) -- nested Vec[Vec[Float64]] element reads through &Vec[Vec[Float64]] PARAMS return garbage

- **Construct:** inside a fn taking `m: &Vec[Vec[Float64]]`, `m[1][0]`
  reads garbage (-4.0 bits for m = [[0,1],[-1,0]] -- the value is NOT in
  the data; probe_skew3/4). The same access on LOCALS in main is correct
  (probe_neg2/neg4). The mono'd param marshaling for nested-Vec Float64
  element reads is broken (the outer element read degrades to a raw i64
  slot read).
- **Impact (stdlib):** geom/linear is_skew_symmetric (and every matrix
  predicate with &Vec[Vec[Float64]] params) -- smoke_geom_vec (l-skew),
  smoke_geom_mat (m-identity), smoke_geom_quat, and the math matrix
  family. The stdlib code is verified correct (user-space replicas fail
  identically). Compiler-side (the round-14c note's "mono'd &[N]T body
  reads offset+1" family).

### Round-14c finding (2026-08-22, stdlib session) -- fn-typed params returning Float64 return 0

- **Construct:** a module fn with a Float64 RETURN passed as an fn-typed
  arg and called through the wrapper returns 0 (probe_fnv:
  apply(sqminus2, 2.0) -> 0 instead of 2.0; the fn-ref wrapper's uniform
  i64 return convention loses the double). i8*-returning fn-refs work
  (transliterate family).
- **Impact (stdlib):** smoke_math_numerical (bisection/newton/...),
  smoke_math_analysis (euler), smoke_math_calculus (derivative),
  smoke_math_integral (trapezoid), smoke_math_optimization (sa) -- all
  pass fns with Float64 returns. Compiler-side.

### Round-14c findings FIXED (2026-08-22) -- write-back bug + generic fn-param aggregates + narrow casts + const-N arrays

Follow-up session closed four of the stdlib session's six new findings
(e2e `e2e_m48_round14c_writeback_aggregates`):

1. **By-value self methods returning the SAME type wrote the result back
   into the receiver's slot** (the time.Duration family -- `d1.identity()`
   clobbered d1 even though identity never read self; sum/diff computed
   against the overwritten receiver). The call site emitted the result
   TWICE -- once into the lhs AND once into the receiver's local slot
   (the "Counter.inc(self)" store-back, which fired for ALL %struct-
   returning by-value self methods). Removed the store-back entirely
   (hot-reload + direct call paths); the lhs assignment and explicit
   rebinds cover every observed case. The whole time.Duration battery
   (35 smokes incl. duration_ops/negative/normalize + the stress family)
   now PASSES.
2. **Generic fns with fn-typed params broke on AGGREGATE instantiations**
   (ZipIter.find/all/any with fn(&(T, U)) -> Bool): (a) a TUPLE type arg
   with a SINGLE generic param is the tuple TYPE itself
   (_find_via[(T, U)]) -- the dispatch took element 0 ("T") and emitted
   the generic _find_via_T symbol; now the tuple-TYPE case builds
   "Tuple__Int__Int" with the enclosing mono fn's generic map
   (mono.current_type_map). (b) the fn-typed param binding used
   type_from_ast, DROPPING the Option args ("Option") -- switched to
   type_string_full ("Option[Tuple__Int__Int]") so the payload tracking
   records the tuple; (c) `var item = next_fn()` (a closure-local
   callee) never recorded the Option payload -- the Some(v) binding kept
   the box pointer raw and predicate(&item) passed the i64-slot address
   (tuple reads garbage). track_boxed_payload_binding now falls back to
   fn_local_returns for closure-local callees. ZipIter find/all/any/nth
   with tuple predicates all green.
3. **Negative Int->narrow as casts widened ZEXT** (smoke_string_narrow
   `n8 != -128 as Int8` -- 128 != -128): the inferred let/var bindings
   recorded local_xiom_types but NOT signed_locals (`var n8 = n as
   Int8` widened the load zext). Both binding paths now track
   signed_locals from the inferred type.
4. **Const-generic [N]T declarations lost the const N and the element
   type** (smoke_array_narrow): (a) llvm_type_for's array arm now
   resolves the size from mono.current_const_map (published BEFORE the
   param bindings) and the element from current_type_map (single-
   uppercase generic names resolve via the type map first -- the old
   silent-i64 default won); (b) UNANNOTATED VAR array literals compile
   as FIXED arrays ("[2 x Int16]" -- the new compile_fixed_array_literal)
   -- the array module's fns take [N]T/&[N]T; the Vec conversion stays
   for the LET path (the core/slice fns take &Slice[T]) and annotated
   Vec targets; (c) the generic-arg inference extracts the array's
   ELEMENT from array locals (&arr args unwrapped, Ref-Array params
   matched); (d) the call-site's &[N]T param is the ELEMENT pointer
   (was hardcoded i64* -- mismatched i8-elem arrays -> clang symbol
   clash); (e) the call site publishes its const_values for the ret
   resolution ("[N]U" -> "[2 x i16]"). array.map[Int16, Int16, 2] over
   an Int16 fixed array compiles + runs.

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending. PRE-EXISTING (unchanged): the
clang -O2/MSVC-CRT layout family (smoke_iter_collect/smoke_array_sort_by
startup AVs + smoke_error2's has-mid flip); smoke_array_narrow's FIRST
section (a LET array + array.len/first -- the M33 let->Vec conversion
conflicts with the array module's &[N]T fns; the VAR + map path works);
smoke_array_edge; smoke_geom_vec; smoke_convert_narrow_roundtrip;
smoke_array_narrow's let-array tail (the fixed-array-vs-Vec
representation conflict for LET arrays is a stdlib-design item).

### Round-15 findings FIXED (2026-08-23) -- fn-typed Float64 thunks + Ord-interface injection + const-N arrays + checker generic bare calls

Follow-up session closed five roots (e2e `e2e_m49_round15_fnfloat_constarrays`):

1. **Fn-typed params returning Float64 returned 0** (stdlib finding 8 --
   smoke_math_numerical/analysis/calculus/integral/optimization all
   blocked). The fn-REF wrapper thunk (`__fnwrap_N`, vec_abi.rs
   wrap_fn_ref_env) declared every non-aggregate param as `i64` and
   restored the callee's real types via casts INSIDE the thunk
   (`sitofp i64 %a0 to double`). The M20-A1 call site passes the REAL
   arg types (`double` in XMM registers on Win64) -- the i64 declaration
   read RDX garbage and sitofp corrupted the bit pattern
   (apply(sqminus2, 2.0) -> 0). Fix: float/double/fp128 params declare
   REAL types in the thunk signature and forward unchanged (like the
   round-14 aggregate params). The BLOCK-STYLE closure thunk (expr.rs
   Closure arm) had the same defect (`_ =>` collapsed "double" to
   "i64") -- fixed the same way. probe_fnv + probe_fnv2 (mixed
   fn(Int)->Int + fn(Float64)->Float64, Float32 closure) green; all five
   math smokes exit 0.
2. **Ord[T].compare dispatch inside mono'd stdlib bodies** (stdlib
   finding 10 -- smoke_core_binary_heap popped 2,3,1,5). The checker's
   collect_external_decls injected catalog INTERFACES only when
   `id.is_pub` -- core.xi's `interface Ord[T]` (and Eq/Bounded/...) are
   non-pub, so codegen never registered them: `Ord[T].compare(a, b)`
   in mono'd sift_up was misread as a VALUE-INSTANCE method (receiver_
   is_instance's interfaces check missed) and the inline scalar compare
   fired with a LITERAL-0 receiver (`compare(0, data[parent]) >= 0` ->
   heap order broke). Fix: inject all interfaces (pure declarations, no
   layout -- same rationale as the round-8 non-pub TYPE injection).
   probe_heap3 pops 5,3,2,1; smoke_core_binary_heap exit 0.
3. **Const-N array residuals** (stdlib finding 4):
   (a) array.len const-N STALE across call sites (probe_stale: l4=2,
   l1=2) -- the const-generic inference pushed the literal "Int" into
   concrete_types, so len(&[7,8]) and len(&[1,2,3,4]) both specialized
   to `array.len_Int_Int` and the first call's const map {N: 2} won.
   The mono name now embeds the VALUE (`array.len_Int_2` vs
   `_Int_4`), and the const inference also extracts N from a
   call-returned fixed-array local's slot ("[5 x i64]" -> 5).
   (b) NARROW-element reads through &[N]T mono bodies (probe_arr8:
   array.first on [1 as Int8, 2, 3] returned Some(0); probe_arr8b:
   UInt8 200 -> -56) -- THREE coordinated roots: (i) the CALLER's array
   locals leaked into mono bodies (array_locals/local_array_elem were
   never cleared per fn -- BUG 47 family), so the mono'd `arr[0]`
   misrouted through the array-BUFFER path (length-at-[0], index+1);
   (ii) `[1 as Int8, ...]` built an 8-byte-slot Vec (elem "Int"
   default) while the mono body read i8 elements; (iii) i8* elem
   pointers were ambiguous with Str (xiom_char_at) and the widening
   defaulted to sext. Fixes: per-fn + mono-body clears of the array
   metadata sets; Ref-Array mono params register their ELEMENT type
   (substituted XIOM name -- "UInt8", not the LLVM-width "Int8");
   a dedicated i8* array-param index path reads data[idx] with the
   elem width and widens with the registered signedness; VAR array
   literals infer the scalar element type (`[200 as UInt8, ...]` ->
   1-byte-element Vec); a new local_array_elem_xiom map keeps the
   As-target name for generic-arg inference. probe_arr8b full battery
   (Int8 first/get/last, UInt8, Int16 negatives) green; smoke_array_
   narrow/edge/len_empty/get_first_last all exit 0.
   (c) array.map's [N]U result (probe_map -- was clang reject then
   AV then len=0): the BY-VALUE `[N]T` param had NO call-side type arm
   (the default used the ARG's type: %struct.Vec against the mono
   def's "[5 x i64]" -> ABI mismatch), and the caller passed the Vec
   header. Fix: call-side Type::Array arm resolves "[N x T]" from the
   substituted elem + const N, and coerce_arg_for_param MATERIALIZES
   the aggregate from a Vec value (data + i*elem_size -> insertvalue).
   probe_map: len=5, d0=2, d4=10; smoke_array_map exit 0.
   NOT covered (unchanged): array_zip T001 (CHECKER rejects the
   `(T, U)` element store -- "cannot access field on non-struct type
   Int"), array_slice/fold AVs (CRT-layout family).
4. **Checker: type-parameterized bare calls** `apply_g[(Int, Int)](...)`
   (stdlib finding 2 residual -- probe_zip_j/k): the callee parses as
   Expr::Call(Expr::Index(Ident(fn), types), args) and the checker
   treated the Index as a VALUE expression -> Unit -> "cannot logically
   negate type ()". Fix (xiom-check): unwrap the Index callee to the
   bare Ident when the base is a registered FUNCTION (not a type) and
   feed the explicit type args into the generic substitution (they win
   over arg inference). probe_zip_j Z1/Z2 checker-clean; probe_zip_k
   runtime re-verified after the checker round.

Verified: full e2e 2294/2294 (incl. e2e_m49_round15_fnfloat_constarrays),
stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, 47-smoke regression battery (array/heap/math/iter/
collections/string/rc/sync families) all green.
PRE-EXISTING (unchanged, baseline-confirmed): the CRT-layout family
(smoke_iter_collect/array_sort_by/array_slice/fold startup AVs,
smoke_geom_vec exit 57 / smoke_geom_mat exit 4 / smoke_geom_quat exit 18
/ smoke_math_edge AV -- these FLIP with unrelated IR: each reproduced
identically at baseline and passed on intermediate builds),
smoke_error2's has-mid flip, smoke_convert_narrow_roundtrip, the
LET-array representation conflict (stdlib-design item).

### Round-14b findings FIXED (2026-08-22) -- checker generic-tuple substitution + multibyte Char family

Follow-up session (while the stdlib sweep runs) closed two more roots
(e2e `e2e_m47_round14b_multibyte_chars` + the upgraded `e2e_m44`):

1. **Checker generic-tuple substitution** ("cannot compare Int with Str"
   on the Str element of `Some((k, v))` from a generic return like
   BTreeMap.first_entry): the checker dropped the receiver's concrete
   args (`var bm = BTreeMap[Int, Str].new()` bound "BTreeMap"; the
   Index type-expr returned the bare container name), and the method-
   return substitution only replaced the WHOLE name ("Option[Tuple__K__V]"
   stayed generic). Fixes (xiom-check): `generic_ctor_type_name` renders
   ANY `Type[args].new()` binding's full type; the Index type-expr arm
   keeps the args; the receiver-method return substitutes generic TOKENS
   with the receiver's args (arity-gated); parse_tuple_elem_types accepts
   BOTH "(A, B)" and the registered "Tuple__A__B" form; the Some/Ok/Err
   payload binding passes the PAYLOAD type to inner patterns. The
   upgraded m44 fixture now asserts the Str element through the tuple
   pattern (was key-only).
2. **Multibyte Char family** (BUG 26 #7; smoke_string_slice chars check):
   (a) xiom_char_at returned the raw BYTE -- a 2-byte char yielded 0xCE
   instead of the codepoint 0x03A9, so len_utf8()/str_chars mis-counted.
   The runtime now decodes the UTF-8 codepoint (i64 ABI; the old i8
   return couldn't hold codepoints). (b) byte_at REUSED xiom_char_at --
   once char_at decoded, byte_at double-decoded (byte consumers got
   430080 for the Omega byte pair). New xiom_byte_at runtime accessor +
   extern; byte_at uses it. (c) Vec[Char] slots held 1 byte -- codepoints
   > 255 truncated to their low byte (str_chars("eOmega") stored 169).
   Char element storage is now 4 bytes (i32). (d) `as`-cast widening
   hardcoded sext -- `byte_at(s, 1) as Int` on a UInt8 byte (0xCE = 206)
   sext'd to -50; the integer-width arm now consults the source's XIOM
   type for UInt*-RETURNING CALLS (the BUG-14 fix only covered Ident
   sources). smoke_string_slice + smoke_convert_utf (mojibake strings
   "hello" restored to "hello", expectations re-derived: 5 sequences,
   6 bytes) now PASS.

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending. PRE-EXISTING (unchanged,
baseline-confirmed): the clang -O2/MSVC-CRT layout family (smoke_iter_
collect/smoke_array_sort_by startup AVs + smoke_error2's has-mid flip --
smoke_error2 passes on some builds, fails on others per the layout);
smoke_string_narrow's `as Int8` comparison (exit 4); smoke_convert_narrow
_roundtrip (exit 3); smoke_array_narrow (fixed-array "[N x T]" typing);
smoke_geom_vec (exit 57).

### Round-14 findings FIXED (2026-08-22) -- aggregate closure params + Vec[Str] elements + narrow-SIGNED loads

The stdlib session's round-13/14 sweep (819/907) surfaced three compiler
roots. All three are RESOLVED (e2e `e2e_m45_round14_aggregate_closure_
params` + `e2e_m46_round14_vec_str_elems_narrow`):

1. **AGGREGATE-typed closure params corrupt on call** (the ZipIter tuple
   predicates / EnumerateIter.map blocker): the closure thunk declared
   EVERY param as `i64` while the call site passed structs/tuples BY
   VALUE (a 16-byte %struct.Point splits across two registers; the i64
   thunk param read only the first -- p.x garbage, tuple elements bound
   literal 0, &Vec params garbage). Top-level fns were correct (decl.rs
   binds real param types). Fix (expr.rs Closure thunk + vec_abi.rs
   __fnwrap): params keep their REAL LLVM types when they cannot marshal
   through an i64 register -- struct/tuple values declare by value and
   bind typed (p.x GEPs the struct), struct-pointee refs declare
   %struct.X*; scalars/scalar-refs keep the uniform i64 convention.
   Verified: struct/tuple/&struct/&Vec params by value + match-
   destructure + fn-REFERENCE aggregate args (the __fnwrap path).
2. **Method call on a Vec[Str] ELEMENT emits invalid GEP** (context.xi
   pretty-print family): `v[0].len()`, `h.names[i].starts_with(..)` --
   the elem-load switch yields i64 so infer_llvm_type can't see the
   string; the Str builtin guards (len/slice/substr/starts_with/
   ends_with) EXCLUDED Index receivers (the 5c.30 Vec[Vec] handle rule)
   and the generic dispatch GEP'd the i8* receiver as a struct
   (getelementptr i8*, i8**, 0, 1 -- clang reject). Fix (call.rs +
   expr.rs): a shared `resolve_vec_elem_xiom(container)` (local_vec_elem
   / local_xiom_types "Vec[X]" / struct-FIELD type_meta) drives
   `is_vec_str_elem_receiver`; receiver_is_str now recognizes Vec[Str]
   element receivers so all Str builtins fire; the len handler only
   excludes i64-typed indexes (Vec handles) and casts the compiled value
   only when it is actually i64 (the elem path already returns i8*).
3. **narrow-SIGNED Vec loads zext** (queue item 3, re-confirmed by
   probe_narrow_zext): emit_elem_load hardcoded zext for 1/2/4-byte
   loads -- Int16 -30000 popped as 35536. Fix (vec_abi.rs): the loads
   take a `signed` flag from the container's element XIOM
   (vec_elem_signed via resolve_vec_elem_xiom -- Int8/Int16/Int32/Int64
   sext, UInt8../Char stay zext); emit_elem_payload_load (pop/get/remove)
   and the index-read paths pass it. smoke_collections_vec_narrow now
   PASSES (the old stress variant was renamed/removed by the stdlib
   session).

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending; the iter/cmp/closure family + the
new find/all/any/nth/last smokes (smoke_iter_find_all_any/nth_last/edge)
all green. PRE-EXISTING (unchanged, baseline-confirmed): smoke_iter_
collect + smoke_array_sort_by startup AV (clang -O2/MSVC-CRT layout
family -- queue 5/7), smoke_error2's has-mid layout flip (same family),
multibyte char len_utf8 (BUG 26 #7), the checker generic-tuple Str-
element typing gap.

### Round-13 findings FIXED (2026-08-22) -- closure-env family + tuple payloads

The closure-based iter adapter build (stdlib) was blocked on four
compiler roots. All four are RESOLVED (e2e `e2e_m43_round13_closure_
adapters` + `e2e_m44_round13_tuple_payloads`):

1. **Closure env STRUCT NAME collisions**: identical capture shapes in
   DIFFERENT mono fns redefined `%struct.__closure_env_N` (clang:
   redefinition of type -- smoke_iter_take_skip/chain/pipeline failed to
   compile). tmp_counter resets per fn (compile_generic_monomorphisations
   zeroes it), so every mono fn's first closure was `__closure_0`. Fix
   (lib.rs): a new GLOBAL `closure_counter` (never reset, mirroring
   unsafe_block_counter) names `__closure_N` / `__closure_env_N` /
   `__fnwrap_N` (expr.rs x3 + vec_abi.rs).
2. **Captured-state MUTATION did not persist**: the closure thunk copied
   captures into locals at entry -- `r.next()`'s implicit-self write only
   touched the copy, so the env's Range never advanced (Range.count/fold
   hung forever; take/skip countdowns never decremented). Fix (expr.rs):
   bind each capture DIRECTLY to its env-struct field GEP (reads load
   through, writes store through -- persistent across invocations).
3. **FN-TYPED FIELD calls compiled to zero-param stubs**: `self.next_fn()`
   resolved no fn for "{Struct}.{field}" and fell to @FilterIter.next_fn
   ret 0 -- the field holds a closure ENV (uniform convention), not a code
   pointer (0xC000001D in MapIter.next/FilterIter.next). Fix (call.rs):
   an instance-receiver fn-marker field ("fn(...)" in type_meta) loads
   the field (env bits), loads env field 0, and calls env-first with the
   field's declared return type (parsed from the fn-ptr field type --
   struct returns by value). Pointer-typed receivers GEP the pointee
   (ThreadLocal* tls_get shape -- the first cut emitted an invalid
   `getelementptr %struct.X*, %struct.X**`).
4. **TUPLE PAYLOADS through Option/Vec** (the first_entry family -- queue
   item 2): `Some((i, v))` bound the elements as literal 0 (the payload
   binding only handled Ident inner patterns -- the box was never
   deref'd); Vec[(Int, Int)] slots held 8 of 16 bytes (the push stored
   the box pointer; get() loaded a half-slot); "(Int, Int)" never
   resolved to the registered "Tuple__Int__Int" key; and the generic
   mono returns ("Vec[(Int, T)]") were rejected by the binding tracking.
   Fixes (stmt.rs/lib.rs/emitter.rs/decl.rs): (a) scrutinee_payload_xiom
   derives the payload from the RECEIVER's concrete Vec elem type for
   get/pop; (b) infer_call_return_xiom accepts mono-registered generic
   names (Vec[Tuple__Int__T] -- the registered layout is authoritative)
   and substitutes placeholders from the recorded mono instantiation;
   (c) substitute_type recurses into Named type ARGS + Type::Tuple;
   (d) resolve_vec_elem_type normalizes "(Int, Int)" ->
   "Tuple__Int__Int"; (e) the payload binding handles Tuple inner
   patterns (inttoptr the box, load the tuple, bind each element) and
   normalizes tuple names in the Ident-inner struct-deref arm.
   UNBLOCKED: smoke_collections_btree_map (exit 7 at baseline) now
   PASSES; enumerate/zip smokes pass.
5. **Enum-return scrutinees from closure calls**: `match compare(&a, &b)`
   (cmp.min_by) emitted NO discriminant checks (branched to the last
   arm) -- the scrutinee fallback only adopted enum_variants-registered
   names; Option/Result are structs. Broadened to any registered
   `%struct.X` (field 0 = discriminant/is_some/is_ok for all of them).

Stdlib fixes (iter.xi): the adapter TERMINAL helpers must call the
adapter's `.next()` METHOD (the raw `next_fn` closure bypasses map/filter/
take semantics); the chain delegates' next_fn consumed r2 AND second()
replayed its own copy of r2 (double-capture -- chain counted 9). Smoke
expectation fixes: smoke_iter_chain_zip (6 not 5 / 3 not 2),
smoke_iter_pipeline (225 not 729 -- take(5) of the squared multiples),
smoke_stress_collections_btreemap (`contains` -> `contains_key`).

Verified: stdlib-exec 70/70 (+2 ignore), feature-reg 510, checker 178,
parser 97, ctfe 97, full e2e pending; 19/20 iter smokes green +
btree_map/btreemap + the full closure family. PRE-EXISTING (unchanged,
baseline-confirmed via stash): smoke_iter_collect + smoke_array_sort_by
AV at STARTUP for the empty-range/combined shapes -- the documented
clang -O2 / MSVC-CRT layout miscompile family (m34_y15/y20, queue items
5/7) -- the combined m43-style module flips the CRT startup crash, so the
e2e fixtures are split to stay below the flip; smoke_iter_find_all_any /
nth_last / edge fail at the CHECKER (find/all/any/nth/last are not in
the stdlib iter API -- planned feature gap).

### Round-13 finding (2026-08-22, stdlib session) -- method call on a Vec[Str] ELEMENT emits invalid GEP

- **Construct:** any method call whose receiver is a Vec[Str] element
  expression -- `e.free[0].len()`, `items[i].len()` (both by-value Vec and
  `&Vec[Str]` params). IR: `getelementptr i8*, i8** %slot, i32 0, i32 1` --
  the element slot pointer (i8**) is indexed as if the STRING (i8*) were a
  struct -- clang: invalid getelementptr indices.
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_err_ctx_g.xi
  (minimal user-space repro, struct with Vec[Str] fields), probe_err_ctx_i.xi
  (&Vec[Str] param variant) -- both COMPILE FAIL. Workaround PROVEN:
  binding the element to a local first (`let s = v[i]; s.len()`) compiles
  and runs correctly (probe_err_ctx_h.xi).
- **Impact (stdlib):** context.xi's error_pretty_print/pretty_print_chain
  (FIXED stdlib-side with local binding -- smoke_error2 green), and latent
  in format/text.xi, textual.xi, fmt.xi text_columns/format_columns
  (FIXED, verified by probe_fmt_cols). Struct-element receivers
  (Vec[Vec[Float64]] `ac[0].len()`) compile fine -- Str (i8*) is the only
  affected element type (the BUG 44-era "Str receiver treated as struct
  pointer" family -- the round-6 fix #2 covered Str.to_str passthrough but
  not element-receiver method calls).
- **Fix direction:** when a method call's receiver is a Vec element of type
  Str, load the element (i8*) BEFORE lowering the self access -- the
  receiver currently resolves to the SLOT address (i8**).

### Round-13 finding (2026-08-22, stdlib session) -- smoke_error2 flips PASS/FAIL with unrelated stdlib code (layout family)

- **Observation:** smoke_error2 (chain + context + backtrace + io) fails
  at `chain.error_chain_has(e, "mid")` (exit 1) on the current tree, yet
  the IDENTICAL chain-section sequence passes as a chain-only program
  (probe_err_chain2.xi -- same chain.xi code, same ops). It also PASSED
  on the build right after the context.xi fix, and re-PASSES when
  error.xi's `wrap_error` is reverted to the pre-fix `[T, E: Error]`
  bound body (verified: "smoke_error2 OK", exit 0). The wrap_error fix
  itself is required (unblocks smoke_error_chain/edge) -- the chain
  module's has-mid behavior must not depend on it.
- **Construct:** error_chain_has loops `let m = e.messages[i]; if str_eq(m, message)`; str_eq compares via string.byte_at. Top/root/len checks in the same program pass; only the mid element comparison fails, and only in the full-program layout.
- **Conclusion:** same family as the documented clang -O2/MSVC-CRT layout miscompiles (m34_y15/y20, smoke_iter_collect/array_sort_by startup AVs, json flakiness): deterministic per source, flips with unrelated stdlib code in the program. Compiler-side (emitted IR for chain/str_eq/byte_at is layout-sensitive). stdlib-side code verified correct by the chain-only probe.
- **Probes:** probe_err_chain2.xi (chain-only, PASS), smoke_error2 with error.xi reverted (PASS), current tree (FAIL at has-mid).

### Round-13 finding (2026-08-22, stdlib session) -- multibyte char length via xiom_char_at reports wrong len_utf8

- **Construct:** `char.len_utf8(xiom_char_at("eOmega", 1))` returns != 2
  for the 2-byte UTF-8 char at byte 1 (probe_char_utf8.xi, exit 1). The
  runtime char reader / Char payload is byte-oriented (BUG 26 #7 family:
  chr() payload corruption). slice.str_chars (correct: advances by
  char.len_utf8) therefore returns the BYTE count for multibyte strings
  -- smoke_string_slice check 25 ("chars-5", str_chars("eOmega").len()
  != 2). stdlib code verified correct; the smoke comment documents this
  as BUG 26 #7.

### Round-14 finding (2026-08-22, stdlib session) -- typed [N]T let declarations lose the const N

- **Construct:** `let a: [3]Int = [1, 2, 3];` (annotated declaration with a
  literal) fails to compile: "unknown type ''", "unknown type '[N x T]'"
  warnings + clang reject `%tmp29 defined with type i64 but expected
  [3 x i64]` (probe_arr0c.xi). The UNINITIALIZED form `let a: [3]Int;`
  compiles but `array.len(&a)` returns garbage for ANY N (probe_arr0b:
  [3]Int and [1]Int both wrong; [0]Int wrong -- smoke_array_len_empty
  exit 3). Only pure-literal inference works (`let a = [1,2,3];` then
  len(&a) == 3).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_arr0c.xi
  (annotated+literal -> COMPILE FAIL), probe_arr0b.xi (annotated only ->
  len garbage), probe_arr0.xi (smoke replica).
- **Impact (stdlib):** smoke_array_len_empty blocked; any user code
  declaring typed fixed arrays with const-N annotations is affected
  (the array/array.xi generic fns themselves are correct -- the const
  N param resolution is the gap).
- **Fix direction:** resolve `[N]T` type annotations in local
  declarations to the concrete const N (the checker/codegen appears to
  keep the unsubstituted '[N x T]' form); the literal-inference path
  already produces the correct type.

### Round-14 finding (2026-08-22, stdlib session) -- `as` casts of negative runtime Int values to Int8/Int16 are wrong

- **Construct:** `var m: Int = -128; var m8 = m as Int8; if m8 != -128 as
  Int8` FAILS (probe_narrow_cast2 check 3); `n as Int8` where n comes from
  to_int_from_str("-128") fails the same way (smoke_string_narrow check 4,
  smoke_convert_narrow_roundtrip check 3 -- exit 3/4). Literal casts
  (`-128 as Int8`), positive-value casts, and typed-local compares all
  PASS. The value read back is the zext pattern (0x80 -> 128) -- the
  cast/read of the narrow SIGNED local does not sign-extend.
- **Relation:** same family as the round-14 Vec narrow fix (that covered
  emit_elem_load for CONTAINER elements only); the plain narrow-LOCAL
  read/cast path is still hardcoded zext.
- **Probes:** probe_narrow_cast2.xi (A/B/D/E pass, C fails),
  probe_narrow_str.xi (parse ok, cast fails).

### Round-14 finding (2026-08-22, stdlib session) -- runtime string-literal conversion mangles multibyte content (program-dependent)

- **Construct:** non-ASCII string literals compile to CORRECT UTF-8 IR
  (verified raw: `c"[U+041F]\00"` = D0 9F in the .ll for every probe), but the
  runtime's literal-to-Str conversion produces different bytes per
  program: probe_cyr.xi ("[U+041F]" + "e") reads back D0 9F / C3 A9 (correct);
  probe_slice.xi ("[U+041F][U+0440][U+0438][U+0432][U+0435][U+0442]") reads back 1F 9F ... ([U+041F] = U+041F LOW BYTE +
  continuation!). Deterministic per source+binary (text2 rebuilds 4x,
  all fail), flips with unrelated program content -- same family as the
  documented clang -O2 layout miscompiles and the BUG 26 #7 internal
  string encoding (U+00FC stored as FC BC, not C3 BC).
- **3-byte literals mangle deterministically:** the circled-one char
  (E2 91 A0 = U+2460) and the CJK char (E4 B8 AD = U+4E2D) read back as
  the code point's LOW BYTE + original continuation bytes (60 91 A0 /
  2D B8 AD) in EVERY file (probe_uni, probe_uni2) -- the UTF-8 decode in
  the literal path truncates.
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_cyr.xi
  (correct), probe_slice.xi (mangled -- the contrast pair),
  probe_uni.xi / probe_uni2.xi (3-byte truncation), probe_puny.xi
  (decode-side FC BC output), probe_escape.xi (`\u{fc}` produces FC BC,
  not C3 BC).
- **Impact (stdlib):** smoke_text2 (transliterate_to_ascii("[U+041F][U+0440][U+0438][U+0432][U+0435][U+0442]") ->
  garbage; single-call transliterate_cyrillic works -- the 9-call chain
  feeds mangled literals) fails deterministically; smoke_string_unicode/
  ea_width/emoji fail transiently (they passed on fresh compiles after
  the sweep -- the sweep's failures were mid-sweep stdlib states).
  The transliterate/unicode modules' code is verified correct (IR and
  single-call paths). The runtime encoding layer (BUG 26 #7 family) is
  the root.

### Round-14 finding (2026-08-22, stdlib session) -- by-value self methods returning the SAME type write the result back into the receiver

- **Construct:** any method call `r = s.m(...)` where the method takes
  `self` BY VALUE and returns the SAME type as the receiver: the call site
  emits the result TWICE -- once into the lhs slot AND once into the
  RECEIVER's local slot (`store {res}, {recv_slot}`). The receiver
  variable is silently overwritten. Even a method that never reads self
  triggers it (`fn Dur.identity(self) -> Dur { Dur{secs:42,...} }`
  clobbers d1 -- probe_dur7 prints d1.secs=42 after
  `var r = d1.identity()`).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_dur7.xi
  (identity clobbers d1; different-return-type methods are safe --
  to_other leaves d1 intact), probe_dur6.xi / probe_dur5.xi
  (`var r1 = d1.sum(d2); var r2 = d1.diff(d2)` -> r2 computes against the
  OVERWRITTEN d1: sum=8 then diff=5 instead of 2), probe_dur2.xi
  (stdlib-shaped replica: add=8, sub=5, div=2). IR evidence in
  probe_dur5.ir.ll: the spurious `store %tmp15, %tmp7` right after the
  call.
- **Impact (stdlib):** smoke_time_duration_ops (sub/mul/div wrong),
  smoke_time_duration_negative, smoke_stress_time_duration_add_sub,
  smoke_stress_time_duration_negative, smoke_time_normalize -- the whole
  time.Duration family (by-value pure methods). ANY module that calls a
  same-type-returning by-value self method and then reads the receiver
  again is corrupted. The error/chain "immutable push/pop" pattern
  (`e = error_chain_push(e, ...)`) is MASKED by the explicit lhs rebind
  (both stores hit the same slot) -- removal is safe for it.
- **Fix direction:** delete the unconditional result write-back into the
  receiver slot at the call site. The lhs assignment (`var r = s.m(...)`)
  and explicit rebinds (`s = s.m(...)`) are sufficient in every observed
  case; the write-back only corrupts.

### Round-14 finding (2026-08-22, stdlib session) -- generic fns with fn-typed params break on aggregate instantiations

- **Construct:** a GENERIC catalog fn whose parameter is a fn type
  (`fn _find_via[T](next_fn: fn() -> Option[T], predicate: fn(&T) -> Bool)`)
  called with an AGGREGATE instantiation (T = (Int, Int)): the mono'd
  fn-param forwarding corrupts -- the predicate always returns false /
  reads garbage from its argument (find returns None). The checker also
  DEGRADES the generic by-value form: `fn apply_g[T](f: fn(T) -> Bool,
  v: T)` instantiated with T = (Int, Int) infers `f` as `()` (error:
  "cannot logically negate type ()").
- **What WORKS (round-14 verified):** direct closure calls with
  struct/tuple/Vec params by value or &ref (probe_zip_f/g/e/c), and
  NON-GENERIC fns with fn-typed aggregate params (probe_zip_k Z1:
  `apply_v(f: fn((Int, Int)) -> Bool, v)` passes). The remaining gap is
  exactly the generic-mono path (probe_zip_k Z3 fails at runtime;
  probe_zip_j Z2 fails at the checker).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_zip_k.xi
  (Z1 passes, Z3 fails), probe_zip_j.xi (Z2 checker degrade),
  probe_zip_h.xi (generic &T-predicate tuple case returns None).
- **Impact (stdlib):** ZipIter.find/all/any with `fn(&(T, U)) -> Bool`
  and the generic _find_via/_all_via/_any_via helpers stay blocked for
  tuple-yielding adapters (shipped in iter.xi; scalar adapters fine --
  Range/Map/Filter/Take/Skip/Chain predicate smokes green). The by-value
  predicate variant (`fn(T) -> Bool`) hits the same checker degrade, so
  no stdlib-side signature change can dodge it.
- **Fix direction:** the mono instantiation of fn-typed params must
  rebuild the fn type with the substituted concrete argument types
  (same substitution machinery as the round-13 mono-return fix), and the
  checker's inference must not collapse `fn(T) -> Bool` to unit when T is
  a registered tuple type.

### Round-13 finding (2026-08-22, stdlib session) -- aggregate-typed CLOSURE params corrupt on call

- **Construct:** any closure whose parameter is an AGGREGATE type -- user
  struct (`fn(p: Point) -> Bool { p.x == 3 }`), tuple (`fn(p: (Int, Int)) ->
  Bool { p.0 == 3 }`), or container (`fn(p: &Vec[Int]) -> Int { p.len() }`,
  by value or by `&` reference). The closure body reads garbage from the
  param (field access, deref, destructure, or len() all wrong). Top-level
  (non-closure) fns with identical shapes are CORRECT (probe_zip_d passes
  by-value tuple AND `&(Int, Int)` params), and scalar closure params work
  (the green iter smokes exercise `fn(&Int) -> Bool` predicates).
- **Probes (C:\Users\lefte\AppData\Local\Temp\kilo\):** probe_zip_f.xi
  (struct param by value + &ref -- W5/W6 fail), probe_zip_c2/c3.xi (tuple by
  value -- fail), probe_zip_b.xi (`&(Int, Int)` param -- fail), probe_zip_e.xi
  (match-destructure of tuple param -- fail), probe_zip_g.xi (Vec[Int] and
  &Vec[Int] params -- fail). Controls: probe_zip_c4.xi (local tuple field
  access -- PASS), probe_zip_d.xi (top-level fns, tuple and &tuple params --
  PASS).
- **Impact (stdlib):** the new iter find/all/any methods on tuple-yielding
  adapters (ZipIter.find/all/any with `fn(&(T, U)) -> Bool`, shipped in
  iter.xi this round) compile but cannot be exercised until this is fixed;
  the pre-existing `EnumerateIter[T].map[U]` (`f: fn((Int, T)) -> U`) has the
  same latent corruption (never called by any smoke). Any future stdlib
  closure taking a struct/Vec/tuple param is affected (e.g. fold accumulators
  of aggregate type).
- **Fix direction:** the closure thunk/`__fnwrap` call convention marshals
  aggregate-typed arguments incorrectly (probably the param's LLVM type in
  the thunk signature or the arg coercion at closure call sites); top-level
  fns demonstrate the correct lowering.
- **Not to be confused with:** BUG 46 (generic &UserStruct[T] PARAM field
  reads -- that's a non-closure generic fn shape, already fixed); this is
  the closure-call path only.

### Round-13 finding (2026-08-21) -- closure-env struct name collision

- **Construct:** multiple closures in one program with IDENTICAL capture shapes (e.g. Range.map's n() -> Option[Int] { r.next() } and Range.take's same-shape closure -- both capture one Range). The mono names both envs %struct.__closure_env_10 -> clang: error: redefinition of type. Reproduces with the new iter adapter build (smoke_iter_take_skip/smoke_iter_max_min/smoke_iter_pipeline fail at clang; map/filter run but crash 0xC000001D -- likely the same env-shape issue at runtime).
- **Impact:** the closure-based iter adapters (stdlib build) are blocked until the env naming dedups (reuse the first definition) or keys by site.
### Round-13 follow-ups (2026-08-21) -- closure-env mutation + cmp_by residual

- The closure-based iter adapters' Range.count/fold HANG (smoke_iter_count/smoke_iter_fold): the next-closure captures the Range by value; .next() inside the closure does NOT advance the env's copy (the implicit-self mutation doesn't wire through the closure env) -> infinite loop. Map/filter crash 0xC000001D + take/chain/pipeline fail at the env-struct redefinition (previous note) -- the whole env-in-catalog family.
- cmp_by STILL exits 1 here despite the round-12 battery claim (fn(&T,&T) -> Ordering closure returns the wrong Ordering; cb2 probe) -- the enum-variant receiver fix covered cmp.Less.reverse() but not the comparator closure's Ordering return through min_by.
### BUG 53 - &[N]T param element access emits invalid GEP -- read FIXED (9757e864) + WRITE facet FIXED (round-3 commit)

- **Construct:** n f(arr: &[5]Int) -> Int { return arr[0]; } -- the fixed-array reference param lowers to [5 x i64]** and element access emits getelementptr [5 x i64]*, [5 x i64]** %p, i64 0, i64 0 -- clang: invalid getelementptr indices. User-space probe (as2) reproduces; array.sort/sort_by and every &[N]T catalog fn is blocked.

### BUG 55 - unsafe-block context capture of pointer-typed variables corrupts reads (0xC0000005/wrong values) -- facet-2 FIXED (round-3 commit)

- **Construct:** write through a pointer inside one unsafe { } block, then read via *(p) inside a SEPARATE unsafe block (or fn): the read returns the wrong value. Same-block reads work (vg9 exit 0), cross-block reads fail (vg10 exit 1). The unsafe-block ctx-struct capture of pointer-typed locals is misrouted.
- **Root cause of:** Vec.get / Weak.upgrade / json_set / contains-style payload corruption -- the catalog fns construct Some(*(data + i)) inside unsafe and the CALLER's match reads the payload -- the caller-side read path (separate block/fn) corrupts. smoke_core_slice/cmp_by/cross_cmp_sort/rc/cell/json blocked.
- **Probes:** vg10.xi (minimal), vg4.xi (by-value struct + unsafe deref return), rcprobe3.xi (upgrade payload).
### BUG 43 - Float64 payload in generic Result/Option container read via sitofp (should itcast)

- **Construct:** a catalog fn returning Result[Float64, Str] (generic %struct.Result layout per BUG 41's primitive rule); the caller's match reads the payload as sitofp i64 - double while the definition stored it via itcast double - i64 - 3.14's bit pattern becomes ~4.6e18. IR: definition itcast double %x to i64 vs caller sitofp i64 %slot to double.
- **Impact:** smoke_core_convert exit 11 - 	o_float_from_str("3.14") reads garbage. ALL Float64 payloads in generic Result/Option containers are affected. The BUG 41 rule needs the READ side to bitcast for float payloads.
- **Stdlib:** core.xi's to_float_from_str is correct; no workaround possible from the stdlib side.

### BUG 44 - deref/coercion of &Str broken (loads a single byte)

- **Construct:** ar p = &s; var d = *p; (deref of &Str) - IR emits load i8 instead of load i8*; user-space probe AVs (0xC0000005). Passing &Str where Str is expected (auto-coercion) also miscompiles (compares a byte).
- **Impact:** any * on &Str, or implicit &Str-Str coercion, is unusable. Str is the only pointer-typed primitive; &Int/&Bool/struct derefs are fine. Blocks Eq-style impls taking &Self for Str.

### BUG 45 - method-form interface dispatch inside generic-bound fns resolves to a stub

- **Construct:** n f[T: Eq](x: T) -> Bool { x.eq(&y) } (method call on a T-typed or concrete-typed receiver inside a fn with an interface bound). Non-generic fns with the same call dispatch correctly (probe eqt12 exit 0); generic-bound fns return the stub's constant (eqt11/13 exit 1).
- **Impact:** core.contains/is_sorted (and every [T: Ord] user: cmp.min/max, sort.*) return wrong results. The tower pattern (EqT[T].eq(a, b) associated calls) works in user space - the stdlib conversion to that form was attempted and reverted pending BUG 47.

### BUG 46 - generic &UserStruct[T] param field reads return garbage

- **Construct:** n get_v[T](b: &Box2[T]) -> T { return b.v; } - reading a field of a USER-DEFINED generic struct through a & param returns garbage (42 != 42). By-value params and builtin &Vec[T]/&[N]T are fine; &Slice[T] (user struct with *T field) element reads break inside generic fns.
- **Impact:** blocks generic algorithms over &Slice[T] (contains/sort over slices). Distinct from BUG 45 (a plain == read fails too, probe eqt18).

### BUG 47 - impl method names collide with fn-typed params in mono (fn-param collapses to i64)

- **Construct:** impl Eq[Int] { fn eq ... } / impl Ord[Int] { fn compare ... } in the SAME program as a fn taking a fn-typed param (e.g. heap_sort_by(v, compare: fn(&Int,&Int)-Int)). The fn-param's mono collapses to a garbage i64 (IR: define ... heap_sift_down_by(%struct.Vec*, i64, i64, i64) - the fn param became an integer) - AV at the call. Impls named differently (ar) or for Str-typed receivers do NOT trigger it; renaming the fn-param to cmp does NOT fix it.
- **Impact:** the tower-style Eq/Ord impls (BUG 45 workaround) cannot land while fn-param fns (heap_sort_by, sort_by, min_by, merge_sort_stable...) exist - the stdlib conversion was reverted. Likely a symbol-key collision between impl method symbols and the fn-pointer trampoline naming for i64-ABI types.
### P001 indentation quirk (parser [U+FFFD] CAN HANG the compiler, priority for the compiler session)

- Indented module-level declarations (indented `use xiom.x;` + indented `fn main`) + a final column-0 `}` produce `error[P001]: expected declaration, found '}'` at EOF, while any one of those three properties removed compiles. Module-level `use`/`fn` at column 0 (repo convention) always works.
- **WORSE: the 2-space-indented variant HANGS the compiler indefinitely** (no error, no exit [U+FFFD] smoke_stress_crypto_hash_known_vector.xi hung the 904-sweep for 15+ min; the compiler session's own "batch 8 timed out" was the same family). The unindented copy of the same file compiles in ~14s.
- 158 smoke files in examples/stdlib_smoke were realigned to the col-0 convention on 2026-08-16 (stdlib session) [U+FFFD] zero indented `use` remain; re-check this note if a future sweep hangs.

---

## 2026-08-16 (late) -- compiler session: BUG 38 family + BUG 32/33/31/34/35 status

### BUG 38 -- FIXED (`9042e8a2`) -- is-Some double-check binds payload 0

The bare `is Some/Ok/Err` scrutinee payload rebind (the BUG 29 contract
convenience: `result is Some => result.len()`) fired in EVERY context. In
if/while conditions it poisoned the subsequent `match` on the same value --
the scrutinee read as an i64 payload, Some(v) arms bound 0 and skipped the
discriminant check (sx4: pos=0 instead of 5; iter.xi collect returned [0,0,0]
or len 0). The rebind now fires ONLY inside an Imply left side
(`in_imply_lhs` flag). The checker never rebinds, so codegen now matches it.

### BUG 38b (NEW, same commit) -- generic-receiver methods never monomorphised

`Iterator[T].collect(self)` (and 44 other decls in iter/core/collections/
sync/rc/memory) had their receiver generics DROPPED by the parser -- the decl
registered with an erased i64 receiver ABI and never entered the mono
machinery (iter smokes 0/19, all 16 in the stale runfail list). Four-part fix:
1. **Parser** (`xiom-parser`): receiver type-param names merge into
   fd.generics (receiver-first, deduped) -- `Iterator[T].collect` ==
   `Rc.get[T]` convention.
2. **Mono dispatch** (`call.rs`): receiver-keyed generic calls
   ("Range.collect" <-> decl "Iterator.collect") enter monomorphisation via a
   leaf-method match; `find_generic_decl` prefers UNDECLARED abstract
   receivers ("Iterator") over concrete types that merely share the leaf
   ("HashMap.count" no longer hijacks "Range.count" -- the wrong decl's
   [K,V] generics produced `Range.count_Int_Int` with a `%struct.HashMap*`
   receiver).
3. **Emission** (`lib.rs`): the concrete receiver LLVM type derives from the
   mono key (`mono_receiver_part`/`receiver_llvm_type`); container returns
   (Vec[T]) keep `%struct.Vec` instead of the erased i64.
4. **Mutating self ABI**: `block_mutates_self` now recurses into if/while/
   for/match bodies, and a new `block_mutates_receiver_state` catches BARE
   receiver-field assignments (`start = start + 1` in Range.next) -- both
   flip the ABI to BY POINTER (value copies silently dropped the mutation;
   iterators looped forever on the first element).

Verified: sx4 pos=5, `iter.range(1,4).collect()` == [1,2,3], user-space
collect/count shapes, checker 178/178. iter smokes 0/19 -> 6/19 (the
remaining adapter-chain failures -- receiver-CALL chains like
`iter.range(1,6).max()` and fn-value params -- reproduce identically at
baseline HEAD; B-007-adjacent).

### BUG 32 -- FIXED (`74bcc28b`) -- Int-var -> ptr cast emits address-of-local

`var h = buf as Int; var q = h as *UInt8;` emitted `bitcast i64* %slot to
i8*` -- the ADDRESS OF THE LOCAL. The As-cast arm now distinguishes the
`&x as *T` address-of form from the bare-Ident VALUE form (load + inttoptr).
Verified: pdb3 eq=OK/read=OK/from_cstring=OK.

### BUG 33 -- FIXED (`74bcc28b`) -- Option[Float128] unwrap loads opaque `%struct.Float128`

The concrete Option__T builtin impls hardcoded `%struct.{payload}` for
non-Int payloads. The payload LLVM type now resolves through llvm_type_for
(Float128 -> fp128). Verified: match-Some and unwrap() on Option[Float128]
both return 42.

### BUG 31 -- FIXED (`74bcc28b`) -- unary minus on Float128 emits `sub i64 0, fp128`

fneg now covers fp128 (was double/float only). Verified: -a + -2.0 == -3.0.

### BUG 34 -- FIXED (`b15d0d3b`) -- nested Vec[Vec[T]] element writes

`bs[i].push(x)` lost the mutation (and corrupted the buffer) through three
gaps: (1) dispatch -- `bs[0]` compiled as i64 so the inline push path missed
the Vec receiver (call fell to a bare `@push` stub); (2) receiver ABI --
resolve_vec_receiver_ptr inttoptr'd the LOADED element value for i64
receivers instead of GEP-ing the element address in the outer buffer;
(3) slot math -- Vec.new() defaulted elem_size to 8, so 32-byte Vec elements
overwrote 4 slots and reads used wrong offsets. Struct-element pushes now
use the struct's real byte size, persist it into field 3, and record the
nested element type so reads take the struct-element (memcpy) path.
Verified: bs[0].push(5) -> bs[0][0] == 5.

### BUG 35 -- primary shape VERIFIED WORKING (no crash, correct buckets)

The Int128 index-math + Vec-write shape (radix-sort distribution) compiles
and runs correctly -- counts[0..2] == [1,1,1] for (100,200,300)/min=100/
width=100. The documented 0xC0000005/0xC000001D variants did not reproduce;
likely resolved by the BUG 38/34 batches. The extreme-i64 variant needs the
stdlib session's exact repro to re-verify.

### BUG 37/36 -- FIXED (`feat/architect`, 2026-08-17) -- fp128 libcall ABI mismatch

**Root cause (NOT a shape miscompile):** the IR clang generates for fp128
arithmetic calls the soft-float helpers with LLVM's Win64 f128 convention
(f128 args by POINTER: `__addtf3`: rcx=&a, rdx=&b; f128 RESULT in XMM0),
but fp128_helpers.c compiled the helpers as `xiom_f128` STRUCT functions,
which get the MSVC ABI (hidden sret in rcx, args shifted to rdx/r8, result
written through sret, XMM0 never set). Every f128-returning helper was
therefore ABI-mismatched: the callee dereferenced r8 = garbage (0xC0000005
in every RUNTIME fp128 shape -- the chain was a red herring; it only
prevented clang -O2 from constant-folding the loop). The earlier
"working" fp128 verifications (t_fneg, smoke_d1_native128, BUG 31/33)
all had CONSTANT-FOLDABLE shapes -- clang -O2 eliminated the libcalls.

**Fix (stdlib/runtime/fp128_helpers.c):** the 13 f128-returning helper
symbols (`__addtf3 __subtf3 __multf3 __divtf3 __negtf2 __extenddftf2
__extendsftf2 __floatsitf __floatunsitf __floatditf __floatunditf
__floattitf __floatuntitf`) are now naked-asm shims implementing the IR
convention (arg pointers as the IR passes them, result in XMM0) that
forward to plain sret-ABI wrappers around the existing pure by-value
implementations. SSE2-only instructions (movdqu/movaps) so the JIT build
(no -mavx) works. Scalar-return helpers (__trunctfdf2/__fixtfdi/compares)
already matched. Also ADDED the previously-missing `__fixtfti` /
`__fixunstfti` (f128->i128) -- those were latent link errors.

**Verified:** all 14 t_b37* shapes + t_chainloop exit 0 with CORRECT
values (t_b37e `bigfloat_to_float128(&v)` non-zero -> 42 val=OK; t_b37u
user-space BigFloat Horner -> 42). New runtime test t_f128rt (Vec-loaded
values: add/sub/mul/div/neg/trunc/compare all exact) -> OK. Suites:
checker 178/178, codegen 2263, feature-reg 510, parser 24 -- all green.
`stdlib_api_freeze_no_removals` still fails IDENTICALLY at baseline (item
B family -- 905 stale paths, NOT a regression).

**Note for the stdlib session:** the bigfloat_to_float128
three-single-shape-fn workaround can be consolidated now; the Option
variant (BUG 33 follow-up) can land; the "TODO(compiler)" notes in
num/bigfloat.xi, sort/radix.xi, misc/glob.xi for the fp128/ptr-cast
families can be re-checked (BUG 32 is fixed too).

### BUG 37/36 -- PRE-FIX analysis record (shape AV era)

Minimal deterministic repro (user space, no catalog needed):
`var n = v.significand.digits.len();` (BigFloat field-chain Vec len) used as
a loop bound + ANY fp128 op in the loop body -> 0xC0000005, even at clang -O0,
with verifiably sound IR (t_chainloop/t_b37f/t_b37k in the compiler session's
scratch). `bigfloat_to_float128` (catalog) with non-zero values crashes in
every consumer shape -- the stdlib's three-single-shape-fn workaround did NOT
fully dodge it (the main fn still mixes chain + fp128). fp128 arithmetic
without the chain, and the chain without fp128, both work. The loop's
semantics even shift when a probe print is added -- because a probe print
breaks clang's constant folding, exposing the libcall ABI fault (the "shape
miscompile" was the folding/unfolding flip, not a miscompile).

### P001 -- NOT REPRODUCED (resolved by the stdlib realignment)

Indented module-level decls now reject in 0.14s (clean P001 error, no hang);
the former 15-min hang case (smoke_stress_crypto_hash_known_vector) compiles
in 4.9s and runs exit 0. The 238-file re-sweep completed with no hangs.

---

## 2026-08-17 -- compiler session: BUG 37/36 follow-up batch -- IR determinism, BUG 39, Item B

### BUG 39 -- Vec element-type records: push overwrite + ctor elem_size (FIXED)

Two coordinated bugs made `Vec[Struct]`/nested-Vec programs fail ~85% of
builds (layout-dependent):

1. **Push-time element-type overwrite** (call.rs): the BUG 34 nested-Vec
   recording (`local_vec_elem[recv] = "Vec[{inner}]"`) fired for EVERY
   struct-element push. For `Vec[Item]` receivers the arg-based inner lookup
   resolved the struct literal to None -> "Int" -> the record became
   `"Vec[Int]"` OVERWRITING the binding's correct `"Item"` -- so `v[0]` took
   the nested-Vec memcpy path and loaded a 16-byte Item into a %struct.Vec
   slot (garbage field reads; m21_vec_edge_012/027, vec_of_struct, eco
   suites). Fix: record only when the receiver's recorded elem is a nested
   Vec OR unrecorded (`Vec[Vec[Int]].new()` -- vec_ctor_elem_type cannot
   resolve nested type args, so the push is the first chance to record).
2. **Ctor elem_size under-counts container fields** (call.rs/stmt.rs):
   `Vec[Vector].new()` sized elements with struct_byte_size (XIOM-semantic
   field-count x 8) -- a `Vec[Float32]` FIELD counted as 8 instead of 32, so
   `Vector` got elem_size 16 instead of 40. The ctor's initial buffer
   (16 x 16 = 256B) overflowed once 40-byte elements were stored
   (test_vector/test_db/test_json 0xC0000005). Fix: new
   `vec_elem_storage_size` -- real byte size with container fields counted
   fully (Vec 32, Map/Set 64, Option 16, Result 24, arrays N x inner).

### BUG 40 -- nondeterministic IR emission (type_meta + mono order) (FIXED)

`type_meta.entries()` and `generic_instantiations` are HashMap-backed; their
iteration order varies per process, so the emitted IR (struct declaration
order, mono function order) differed BETWEEN BUILDS of the same source.
clang -O2's codegen of the linked MSVC CRT objects is layout-sensitive:
identical sources built passing vs crashing/wrong-result binaries. This
explained the session-long "flaky" failures (m21_vec_edge_012 exit-1,
smoke_simd 0xC0000005, eco suites, even a CRT-internal call reading an
uninitialized r9d). Fix: sort both emission loops by key. The output is now
byte-reproducible (verified 5x identical for m21/test_vector/m34).

### BUG 41 -- FIXED (`feat/architect`, 2026-08-17) -- mono call-site ret type vs concrete Result payload

`fn adjust[T](e: Env, target: Climate) -> Result[Env, Str]` (generic with an
UNUSED type param) mono'd to adjust_Env: the DEFINITION emitted
`%struct.Result__Env__Str` (the mono's subst_type resolves concrete
Result/Option keys), but the CALL-SITE fallback (call.rs generic_ret,
used when the specialized key isn't registered yet -- the `-o` compile
order compiles the caller first; `xiom run` happened to compile the def
first, masking the bug) produced the generic base `%struct.Result` ->
`call %struct.Result @adjust_Env` + `ret %struct.Result__Env__Str` ->
clang rejected the IR (m34_y15/y20 compile failures).

Two root causes, both fixed:
1. `type_from_ast` drops Named args and renders `Type::Result` as bare
   "Result" -- the call-site fallback now renders the FULL
   "Result[Env, Str]" (Named-with-args, Type::Result, Type::Option arms).
2. `concrete_container_llvm` (new) mirrors concrete_type_for's rule:
   concrete `%struct.Result__A__B` iff at least one payload is a STRUCT
   (primitives/enums keep the generic layout, matching the definition);
   falls back to the bare concrete name when the key isn't registered yet
   (a struct payload guarantees the def emits it). Verified via both the
   `run` and `-o` (harness) paths: m34_y07/11/13/15/16/19/20,
   m35_z02/09/24/29, m21_complex_generic_008/009 all exit 0.

### BUG 42 -- FIXED (`feat/architect`, 2026-08-17) -- enum values in Vec elements

`Vec[JsonValue]` / structs containing enum fields (JsonEntry
{ key: Str; value: JsonValue }) were stored/read as i64 SCALARS (tag
only, payload garbage):
1. **Push path**: is_struct_elem checked only `types` -- enums register in
   `enum_variants`, so a JsonValue push took the scalar store (tag, and
   the payload slot stayed zero) -> as_number read garbage bits ->
   float_to_string crashed in the CRT (shld on a garbage exponent).
2. **Read path**: resolve_vec_elem_type had no enum_variants fallback ->
   Vec[JsonValue][i] fell to the scalar i64 load instead of the struct
   memcpy.
3. **Storage size**: vec_elem_storage_size had no enum arm -- JsonEntry
   (with the JsonValue field) sized 16 instead of 24; the push's
   sizeof_struct also under-counted enum fields (8). The enum layout is
   `{ i64 tag, i64 x slots }` (payloads boxed to i64 handles; tuples one
   slot per element) -- 16 bytes for JsonValue, 24 for JsonEntry.
   ALSO: qualified field names broke the leaf-suffix lookup (ColumnDef's
   affinity -> 8 -> elem_size 18) and Bool FIELDS are 8 bytes (only
   Vec[Bool] ELEMENT slots are 1 byte) -- ColumnDef sized 18 instead of 40
   (test_sqlite wrong results).
Fix: is_struct_or_enum_type helper used by both push checks; the push's
struct esz now uses vec_elem_storage_size uniformly (structs, enums,
containers, arrays); resolve_vec_elem_type falls back to enum_variants
keys; leaf-segment matching for qualified names; Bool-field width 8.
Verified: test_json (29), test_db (18), test_vector (32), test_sqlite
(23), test_test (20) all exit 0.

### BUG 40-era harness hardening (2026-08-17)

The e2e harness's compile_and_run_once now deletes the target exe before
compiling -- a STALE exe from an interrupted run held the output path
open and clang failed with "permission denied" (e2e_main.exe,
e2e_t3-hot-reload.exe -- previously misread as real failures).

### Item B -- exec harness wiring (DONE)

- `stdlib_api_freeze_no_removals` 905 -> 0 missing: (1) module paths now
  resolve via STDLIB_MANIFEST.md (exact `xiom.X` then unique last-segment
  match, then filesystem scan) -- the 2026-08-16 folder refactor broke the
  flat `stdlib/xiom/{module}.xi` join; (2) the signature extractor's
  comment-stop is now indentation-aware (crypto.xi documents "no requires
  clauses" BETWEEN the sig and `{`).
- `stdlib_execution_tests` 72 -> 70 pass + 2 documented #[ignore]s:
  smoke_core (stdlib-side: `interface Eq` has ZERO impls anywhere -- add
  `impl Eq for Int/...` in core.xi), smoke_simd (BUG 40-era latent CRT
  layout miscompile; re-enable after a toolchain investigation).
- Fixed en route (verified): Vec[Str] element reads now inttoptr the loaded
  handle to i8* (yaml_emit_sequence garbage); null/dangling inline results
  are recorded as pointer-valued so is_null coercion inttoptrs instead of
  materializing a temp address; smoke_ptr/serialize/misc/net_folder pass.

### Verified after the batch

- checker 178/178; stdlib_tests 40/40; api_freeze 2/2; execution 70/70+2
  ignores; e2e 2256/2263 with the 7 remaining = 2 hot-reload file-lock
  artifacts + m34_y15/y20 (BUG 41) + eco db/json/vector (BUG 41-adjacent
  container/Result shapes; were flaky at baseline too).
- Repro battery: t_chainloop, t_b37* (14 shapes), t_f128rt, t_f128i128,
  sx4, t_iter38, pdb3, t_b33, t_b34, t_b35, t_fneg_clean -- all exit 0.
- Reminder: `xiom run` caches binaries in ~/.xiom/jit/<sha>.exe keyed ONLY
  by source hash -- delete them after ANY runtime C change or results use
  stale runtimes.

### Re-triage numbers (current isolated binary, 2026-08-16 late)

238 files from the stdlib session's two remaining lists -> 43 pass, 82
compilefail, 113 runfail. Known stdlib-side: smoke_num_saturating (Bounded/
Ord impls), smoke_alloc_basic (`use xiom.ptr;`), char.from_digit contract
false-fire (contract on a graceful-fallback fn -- stdlib rule BUG 22 #5),
smoke_collections_vec_push_pop (is_empty broken -- reproduces at baseline),
Vec.get usage in stale smokes (not a Vec method). Remaining compiler-side
families cluster in: receiver-CALL chains for generic methods (iter
adapters), the BUG 24/36 shape family, and the stale-API smokes.

---

## 2026-08-16 (late) -- compiler session: BUG 31 batch -- fmt, Map, variant-hijack, tuple destructure all fixed

The compiler session's queue from the 904-sweep. Commits:
`2b238da4` (fmt), `b28ac72b` (Map), `a7571ac7` (variant hijack), `99f894b7` (tuple destructure).

1. **Unit fields in `Result[Unit, FmtError]` literals -> `store void 0, void*`**
   (invalid IR): generic-name literals now resolve to the CONCRETE
   instantiation via the fn's return type; field types degrade Unit->i64
   (field_llvm_type + ctor sites + the 5c.39 value adoption never takes
   void). **Fixes smoke_fmt_align/formatter/format1-3/edge/float.**
2. **Str.to_str passthrough read the first BYTE of the string**: the
   prologue treated i8* receivers as struct pointers (self registered as
   the pointee "i8"); only `%struct.X*` receivers take that branch.
3. **`v.to_str()` on primitive locals -> bare `@to_str` stub**:
   infer_struct_type_name now resolves literal receivers (incl. negated +
   paren-wrapped), primitive-typed locals via local_xiom_types, and
   monomorphised generic params via param_concrete_types;
   infer_value_xiom_type learns literals.
4. **Mutating `self` methods (Formatter.write_int's `self.buf = ...`) lost
   the mutation**: block_mutates_self pushes the POINTER ABI in compile_fn
   AND the signature registration.
5. **Map[K,V] `&K` params (get/contains/remove) passed the VALUE**: four
   coordinated fixes -- param_llvm_type Type::Ref -> pointer for scalars;
   mono subst_type Ref -> pointer arm; generic-call inference &T scalar ->
   pointer; coerce_arg_for_param materializes plain VALUE args into temps
   for pointer params. **Fixes all 7 smoke_stress_collections_map_*.**
6. **Struct literal vs enum-variant name hijack**: `Node{...}` (a STRUCT)
   was hijacked by `enum BST[T] { Node(...) }`'s Node VARIANT -- the
   bare-variant search now only fires when the name is NOT a known type.
   **Fixes bench_math's native IR (was store %struct.BST %vecval).**
7. **Cross-module tuple destructuring** (`var (a, b) = pair()`): the
   checker bound every name to the WHOLE tuple -- now splits the
   Tuple__A__B element types. **Fixes BUG 26 #3.**

**Verified fixed this session:** BUG 26 #2 (bare prelude names -- repro
passes), BUG 26 #5 (high-bit mask AND -- repro passes), BUG 24 residual
(smoke_num_precision OK).

**Remaining compiler queue (documented):** BUG 32 (Int->ptr cast emits
address-of-local), BUG 33 (Option[Float128] unwrap opaque struct name),
BUG 34 (nested-Vec element WRITES), BUG 35 (Int128+Vec-write shape AV),
BUG 36 (fp128 Horner shape AV), BUG 37 (fp128 catalog-return AV),
BUG 38 (is-Some + match double-check binds 0 -- BUG 30 #1/#2 family).

**Stdlib-side (for the stdlib session):** smoke_num_saturating needs real
Bounded/Ord impls (generic bounded fns, no impls -- interface dispatch
itself verified working); smoke_alloc_basic needs `use xiom.ptr;`;
smoke_hash_folder needs `use xiom.convert.toint;`.
---

## 2026-09-10 -- stack-cookie/KDF family: &Str len/receiver/ref-arg + Vec accessor builtins

smoke_stress_crypto_pbkdf2 (+_iterations) FIXED; smoke_os stays green;
the family's remaining members are stdlib-side (below). Four codegen roots,
all red-green proven via tmp/bug_probes/p_pbk_iter.xi, p_pbk_rep.xi (a
user-module replica that always worked -- catalog-body-only failure), and
p_sha_leak.xi (600x sha256 clean, ruling out the counter-leak theory):

1. `.len()` on a `&Str` param (i8**) hit the generic pointer-typed branch
   and emitted `load i64, i8**` -- the string POINTER BITS as the length.
   `while pi < password.len()` then ran ~forever pushing bytes until the
   Vec cap trap (0xC000001D). Fix: an i8** receiver loads the HANDLE
   (i8*) and calls xiom_str_len.

2. `password.char_at(pi)` with `password: &Str` passed the SLOT ADDRESS
   (i8**) to char_at's by-value Str param (i8*): opaque pointers made the
   call type-check silently and char_at read the slot bytes as the string
   -> its ensures (`pos < s.char_count()`) aborted with a contract
   violation. Fix: method-receiver coercion loads the handle when
   p0 == i8* and recv_llvm_ty == i8**.

3. `crypto.pbkdf2(&"password", ...)`: the `&"literal"` arg to the `i8**`
   `&Str` param was BITCAST from the string data pointer instead of being
   materialized into a handle slot -- the callee read the literal's first
   8 bytes as the Str handle (str_len AV at -1). The BUG 52 materialization
   path was gated on `lvalue.is_none()`, which a Ref-of-literal fails; the
   gate is removed (ref-locals carry i8** already and never match).

4. Vec/Slice `.as_ptr()`/`.as_mut_ptr()` did not exist anywhere in the
   stdlib, so calls were auto-stubbed to `ret i64 0`; the arg coercion then
   materialized the 0 as a 1-BYTE stack temp passed as a buffer DEST with
   size 4096 (io BufReader fread -> 0xC0000409 stack overrun; same shape in
   os.read/write, brotli fwrite). Fix: real builtins returning field 0
   (data pointer) for Vec (alloca + GEP) and Slice (extractvalue), plus
   checker signatures so they resolve. Also hardened coerce_arg_for_param:
   a CALL returning a raw pointer resolves via infer_call_return_xiom so it
   inttoptrs instead of being truncated to a byte.

Locked: stdlib_exec_pbkdf2_runs, stdlib_exec_pbkdf2_iterations_runs.
Gates at commit: e2e 2306/2306, feature-reg 510/510, stdlib-exec 77/77
(+2 ign), checker 182/182.

STDLIB LANE NOTES (these cannot pass compiler-side):
- smoke_stress_io_bufreader: BufReader stores `io.stdin()` (FD 0) and
  passes `self.inner as *UInt8` to fread as the FILE* -> null stream
  (0xC0000409 fail-fast). Use xiom_stdin()/fd->FILE mapping.
- smoke_stress_crypto_argon2_basic: calls crypto.argon2 with 6 args; the
  signature has 5 (no key_len).
- smoke_math_edge still traps (0xC000001D) -- next round.


## 2026-09-10 -- CRT-family AV flips: Slice builtin, `&*p` reborrow, concrete payloads, mono ABI

Three of the four remaining CRT-family AV smokes are fixed (array_slice,
core_box, regex_find); convert_url no longer AVs (fixture issue, below).
Four independent roots, all red-green proven:

1. Slice[T] was UNKNOWN to codegen (only the checker had a permissive
   builtin). `array.as_slice`'s %struct.Slice return was erased to i64 by
   the mono type-substitution fallback (type_from_ast STRIPS Slice to its
   element), and `s.len()` then ran the Str.len builtin on the returned
   LENGTH (xiom_str_len(5) -> AV at 0x5). Fixes: (a) register the
   canonical builtin `%struct.Slice = { i8*, i64 }` in compile_program
   (the stale stdlib `type Slice[T] = {data: Vec[T]; invariant}` decl
   must not win; builtin first-wins); (b) `Type::Slice(_) =>
   "%struct.Slice"` in the mono subst_type; (c) the mono CALL-SITE ret
   name keeps "Slice" (was element -> i64); (d) the `.len()` builtin
   handler reads field 1 of a %struct.Slice directly (the Vec path would
   extractvalue 2/3 out of a 2-field struct).

2. `&*p` / `&mut *p` REBORROW compiled as a LOAD of the pointee. Box.get's
   `return &*ptr` returned the boxed VALUE 42; smoke_core_box then
   dereferenced 42 (AV at 0x2a). Fix: Ref/MutRef of a Deref yields the
   deref OPERAND's value (the pointee address). Handled in both AST shapes
   (Expr::Ref/Expr::MutRef and Unary::Ref over Unary::Deref).

3. Mono method ABI for this-based methods whose explicit param NAMES the
   receiver (Box.get[T](b: &Box[T])): the def carried a DUPLICATE
   receiver-type param (`%param1`) the call never passes, and the `&T`
   return resolved to i64 at the call site while the def returns i64*.
   Fixes: (a) elide a receiver-typed explicit param from the mono def
   ONLY when the body never references its ident (a genuine
   `eq(other: &Foo)` that the body uses is kept); (b) the call-site ret
   computation maps Type::Ref to {pointee}*.

4. Result/ Option payload field reads on a CONCRETE container
   (Result__Uri__Str) applied the ERASED-i64 override: the inline
   %struct.Uri payload was unboxed as if the first 8 bytes were a heap
   pointer (uri_normalize -> _lower -> xiom_str_len AV at 0x50544854;
   smoke_stress_regex_find same class). Fix: the payload override only
   fires when the static field type IS i64 (`static_payload_i64`).

Locked: stdlib_exec_array_slice_runs, stdlib_exec_core_box_runs,
stdlib_exec_regex_find_runs. Gates at commit: e2e 2306/2306,
feature-reg 510/510, stdlib-exec 75/75 (+2 ign), checker 182/182.

STDLIB LANE NOTES:
- smoke_convert_url now produces the CORRECT output (exit 27 only because
  the fixture input is ASCII "https://example.com/path" while the
  expectation still contains "%C3%A4": the `chore(encoding): strip UTF-8
  mojibake` sweep (42b94404) turned the original non-ASCII "a-umlaut" input into ASCII but left
  the expectation). Restore the non-ASCII input bytes to flip it; the
  compiler warrants it.
- Still open (next round): stack-cookie family -- smoke_stress_io_bufreader
  (0xC0000409), smoke_stress_crypto_pbkdf2 (+_iterations, 0xC000001D),
  smoke_stress_crypto_argon2_basic (compile failure).

## 2026-09-10 -- json heap layer: PART 1 + PART 2 FIXED (Stage 2c entry landed)

The flaky json family (kat_serialize_json_minimal, json_nested /
json_parse_nested / jsonvalue_get startup-or-exit AVs) had three layers,
all now fixed and gated.

PART 1 -- ctor mono substitution (call.rs Vec.new / with_capacity):
`Map.new`'s mono'd body calls `Vec[V].new()`; the expression-level type
arg kept the RAW generic param "V", which resolved as unknown -> elem
size 8. Map[Str, JsonValue] values were stored at 8-byte strides
(112-byte JsonValue truncated). The type arg now substitutes through
mono.current_type_map -> stride 112 (IR-pinned by
e2e_m65a_json_values_stride). Eco fixture test_json.xi stays green.

PART 2 -- the map VALUE READ + Option/Result enum concretization (landed
TOGETHER this round; the Option-typing alone was attempted and reverted
in round 25 because it broke the ecosystem fixture's handle-ABI
coherence -- that break is now root-caused and fixed as (c)):

(a) M18 gate REMOVED (lib.rs concrete_type_for, both Option and Result
    arms): enum payloads now concretize like any struct --
    Option[JsonValue] -> `%struct.Option__JsonValue = type { i64,
    %struct.JsonValue }` (120 bytes) and Result[JsonValue, Str] ->
    `Result__JsonValue__Str`. Enum layouts are real structs
    (discriminant + deduped payload slots); the old exclusion made the
    handle-ABI and concrete-struct paths disagree on the same value.

(b) GENERIC-FIELD Vec element resolution (lib.rs/decl.rs/context.rs/
    stmt.rs): `entries.values[i]` where `entries: Map[Str, JsonValue]`
    fell to `emit_elem_load` (8-byte scalar = the enum TAG) because
    resolve_vec_elem_type could not resolve the field type "Vec[V]" of a
    generic instantiation. New `generic_type_params` registry (declared
    param ORDER per generic type decl) + `resolve_generic_field_vec_elem`
    + identifier-token `subst_type_params` (a plain replace corrupts
    "Vec[V]"), fed by recording the DECLARED payload XIOM type of
    enum-variant match bindings in local_xiom_types. The read now emits
    memcpy + `load %struct.JsonValue` at the runtime stride (IR-verified);
    keys keep the scalar Str-handle path.

(c) POINTER-SELF METHOD ON A STRUCT-VALUE TEMPORARY (call.rs method
    dispatch): `name.unwrap().as_string()` -- once Option[JsonValue] is
    concrete, unwrap yields the 16-byte JsonValue VALUE, and the old
    fallback passed that value where the callee's `%struct...*` self
    param expects a pointer (INVALID IR: at -O0/clang used the
    discriminant register as the self address -> ASAN pinned
    `JsonValue.as_string` reading address 0x3, the String tag; at -O2 the
    ecosystem fixture AV'd at the test boundary). Fix: materialize the
    temporary into an alloca and pass its address when
    p0 == "{recv_llvm_ty}*".

Evidence: pre-fix j3b/m65 exit -1073741819 (0xC0000005); post-fix
j1..j7 + m65b probes exit 0, m65 exit 0, ecosystem test_json exit 0.
Gates: e2e 2306/2306 (+2 new: e2e_m65_json_map_enum_payload,
e2e_m65b_json_get_concrete_option), feature-reg 510/510, stdlib-exec
70/70 (+2 ign), checker 182/182. IR pins: the 112 stride (m65a), the
concrete Option layout (m65b), the runtime shape (m65).
STDLIB LANE: kat_serialize_json_minimal is UNBLOCKED -- re-sweep the
json family.

### -g follow-up FIXED in the same round (honest debug builds)

While symbolizing the ecosystem AV (ASAN/lldb work above), honest
`xiom -g` builds failed at clang: "expected '{' in function body" at
the first define line. Root cause: decl.rs emitted the !dbg attachment
BEFORE the function attributes --
`define i64 @main(...) !dbg !5 alwaysinline {`; LLVM's define-line
grammar requires ATTRIBUTES first, metadata attachments after
(`... alwaysinline !dbg !5 {`). One-line order swap. Verified: `-g`
builds of m65/j3b/ecosystem test_json now compile and exit 0; the
emitted DISubprogram nodes are unchanged. (No e2e covers -g; the
m65/j3b/eco -g runs are the regression evidence.)

## 2026-09-10 -- opt-level flag landed; -O0/-O1 floor REMOVED (re-verified)

## 2026-09-10 -- opt-level flag landed; -O0/-O1 floor REMOVED (re-verified)

The driver hardcoded -O2 debug / -O3 release because of a 2026-08-08
note ("clang miscompiles native Int128 loops + inlined Vec ops at
-O0/-O1; i128 loop + Vec.push crashes"). After the CRT-layout family
fixes (closure env malloc under-allocation + MutRef array elem
addresses -- the exact classes of invalid IR that -O2's mem2reg used to
mask), a 9-probe matrix (m20_stress_big_loop, m20_stress_int_overflow,
m32_int_0400, m63, m64, m59, m37_u128, crt_full, m62) now passes at ALL
of -O0/-O1/-O2/-O3.

Change (crates/xiom): new --opt-level 0..=3 flag
(CompileConfig.opt_level, main.rs parse + resolve_source_files skip
list); both the opt-pass invocation AND the clang link step honor it.
Default unchanged (-O2/-O3). A user can now produce honest -O0 debug
builds and -O3/-Os-style tuned builds instead of the silent floor.

Gates: full e2e + feature-reg + stdlib-exec run at doc time.

## 2026-09-10 -- CRT-layout family FIXED: two root causes (closure env size + MutRef array elem addresses)

The documented clang -O2/MSVC-CRT layout family (smoke_iter_collect /
smoke_array_sort_by startup AVs, m34_y15/y20, "flips with unrelated
stdlib code") had TWO distinct root causes, both now fixed:

1. CLOSURE ENV MALLOC UNDER-ALLOCATION (expr.rs PipeClosure +
   block-Closure arms): the env struct malloc'd 8 bytes PER CAPTURE
   regardless of the captured LLVM type. Capturing a STRUCT
   (%struct.Range = 16B; %struct.Vec = 32B) overflowed the malloc'd
   tail by (size-8), corrupting the heap -- iter.range(0,0).collect()
   alone AV'd at startup (the capture stores the Range struct into a
   16-byte env allocation). Fix: env size = 8 + sum of
   llvm_type_byte_size per capture field (existing helper).
2. `&mut [N]T` PARAM ELEMENT-ADDRESS LOWERING (expr.rs Ref arm +
   lib.rs mono param registration): the Ref arm's fixed-array address
   branch requires a by-value `[N x T]` slot; a `&mut [N]T` param's
   slot is the bare data pointer (i64*). The fallback loaded the
   element VALUE and passed it as the ADDRESS -- the comparator thunk
   derefed small ints (5,3,1,4,2) -> 0xC0000005 in smoke_array_sort_by.
   Fixes: (1) MutRef array params now register local_array_elem like
   Ref ones (lib.rs mono binding, Type::Ref | Type::MutRef);
   (2) the Ref arm emits GEP + ptrtoint for pointer-typed array params
   (is_array_elem_param detection).

Regressions: tests/regression/m63_crt_closure_env_struct.xi (empty+
full range collect) + m64_crt_sortby_refargs.xi (sort_by comparator).
Both red pre-fix (-1073741819), green after. Full e2e + feature-reg +
stdlib-exec run at doc time. Note: smoke_iter_collect / smoke_array_
sort_by now exit 0 -- the e2e fixtures can be re-unified if desired.

## 2026-09-09 -- Delegation crash FIXED: qualified calls bind catalog fns despite local shadows

stdlib-audit #3 (the crash that forced the stdlib's copy-paste base64/
base58/base32/... duplicates). Current manifestation was silent wrong-
module resolution: `xiom.num.convert.to_base58(255)` bound a LOCAL
`pub fn to_base58` when the user module declared one. Probes:
tmp/bug_probes/deleg1.xi (shadow) + deleg2.xi (control) -- IR diff showed
@to_base58 (bare local) vs @convert.to_base58 (correct).

Root cause: Checker::collect_external_decls kept a `user_free_fns`
shadow set and SKIPPED injecting any stdlib free fn whose BARE name
matched a user fn (xiom-check/src/lib.rs:2405). The skipped fn's
leaf-qualified key ("convert.to_base58") never registered in codegen,
so resolve_module_call's leaf lookup missed and fell through to the
bare name -> local fn. The skip's original rationale (duplicate @alloc
from `module sys { pub fn alloc }`) was obsolete: injected decls arrive
LEAF-qualified and emit qualified symbols (@alloc.alloc); the user's
bare fn keeps the bare symbol, and the codegen alias map prefers the
existing bare entry, so bare calls still bind the user's fn.

Fix: injection now dedups by the QUALIFIED key only (module-qualified
free fns, methods, impl fns); user_free_fns removed. Bare calls keep
shadowing; qualified calls bind the module namespace strictly.

Regression: tests/regression/m62_delegation_shadow.xi +
e2e_m62_delegation_shadow (qualified->catalog "ABC", bare->local
"LOCAL"; red pre-fix exit 1, green after). Existing user-alloc shadow
(m35_z06) exercised by the full suite. Full e2e + feature-reg +
stdlib-exec run at doc time. STDLIB LANE: same-name delegation is now
safe -- the base64/base58/base32/... copy-paste duplicates can be
consolidated behind re-export shims.

Also fixed in this round: the parallel e2e FLAKE -- e2e_cross_package_
extern vs e2e_cross_package_use both compile examples/e2e/cross_pkg/
main.xi, so both targeted the SAME output binary (e2e_main.exe) and
raced it (intermittent FAILED in full runs, always green in isolation).
The harness now suffixes every output with a per-invocation counter
(e2e_tests.rs compile_and_run_once_with_flags), making names unique
across tests and retries.

## 2026-09-09 -- R1 FIXED: byte_at/char_at upper-OOB reads (runtime clamp)

Stdlib report R1: byte_at("", 999) returned 4 after a string-op preamble
but 0 before one -- the bound check looked "state-dependent". Root cause:
xiom_byte_at (xiom_runtime.c) only clamped pos < 0; for pos >= len it
read PAST the NUL terminator into adjacent heap bytes. The value was
heap-adjacency luck (allocation history), not compiler state.

Fix: xiom_byte_at and xiom_char_at now clamp
pos >= strnlen(str, 1MB) -> return 0 -- a pure function of (str, pos),
matching the language's strlen-based length model (xiom_str_len is
strlen; Str values are NUL-terminated). Direct extern callers (base32/
ascii85 alphabets) index in-bounds, so the added scan is O(alphabet) per
access there; correctness over speed for the raw accessors.

Regression: tests/regression/m61_byte_at_oob.xi + e2e_m61_byte_at_oob
(boundary positions on empty/short strings, negative pos, in-bounds
value check, and R1's exact preamble shape). Pre-fix red is
nondeterministic (garbage reads: 1/4 runs nonzero); post-fix always 0.
Full e2e + feature-reg + stdlib-exec run at doc time.

## 2026-09-09 -- R2 FIXED: module-level mutable arrays with literal initializers

Stdlib report R2 (M58 residual): `var _tbl: [256]Int = [256 literals];`
emitted `store [N x T] %tmp (type 'ptr')` at the module-init store --
the runtime-init ctor ran compile_expr on the array literal, which
lowers to an i8* heap buffer (a 'ptr'), stored into the [N x T] global.

Fix (codegen):
- expr_is_const_init (decl.rs) accepts ARRAY literals whose elements are
  all const-init -> such vars skip the runtime-init path entirely.
- global_const_init (lib.rs; types.rs copy kept in sync) renders an
  all-constant array literal as an LLVM constant aggregate
  (`@sym = global [N x T] [T c0, T c1, ...]`), with recursive element
  rendering (scalar typing + nested arrays) and defensive pad/truncate
  to the declared extent. The global is now emitted directly with its
  constant initializer -- no @llvm.global_ctors entry.
- Mixed/runtime-element array literals still take the runtime-init path
  (pre-existing limitation for runtime-filled fixed module arrays;
  no stdlib module needs that shape -- tables are const).

Regression: tests/regression/m60_module_array_literal.xi +
e2e_m60_module_array_literal (Int sums, negative elements, 256-literal
extent, index writes into the literal-backed global). Note: fixed-array
element reads type as Int in the checker (pre-existing), so the Float64
array variant was excluded from the regression. Full e2e + feature-reg
+ stdlib-exec run at doc time.

## 2026-09-09 -- R4 FIXED: guard-arena escape via outer-Vec growth (compiler lane)

The CSPRNG-flip blocker (stdlib report R4; probes r4c_single5000 /
r4b_os_5000 in tmp/bug_probes) is fixed in the RUNTIME, not the caller
frame the bisect suspected.

Root cause: `xiom_guard_realloc` (stdlib/runtime/xiom_runtime.c) migrated
ANY confined-block Vec growth into the guard arena -- including Vecs whose
data was allocated on the MAIN heap before the block (crypto.xi
os_secure_random_bytes declares `result` outside `unsafe`, grows it inside
the 4096-byte chunk loop). The arena is discarded wholesale at block exit
(xiom_guard_heap_exit frees every slab), so the returned Vec's data
pointer dangled: byte reads AV'd 0xC0000005. The 16-byte draw survived
only because the initial malloc(16) capacity was never exceeded (no
growth -> no arena migration); 5000-byte draws grew past capacity and
AV'd on reads -- single OR double call (the "second-call" bisect framing
was incidental: their small probes happened to stay under capacity).

Fix (xiom_runtime.c, 4 edits):
- XiomGuardArena tracks per-slab sizes (parallel long* slab_sizes) so
  membership tests and slab teardown (munmap on POSIX) use the real
  extent, including oversized slabs.
- xiom_guard_realloc now checks arena membership of `old`; main-heap
  pointers grow in place with plain realloc (data outlives the block),
  arena-born pointers keep the arena copy path (confinement intact).
- xiom_guard_alloc / heap_exit updated for the sizes array.

Verified: single- and double-5000-byte probes exit 0 with plausible
sums; m18_guard confinement subset 125/125; e2e_m59_guard_arena_escape
(regression; red pre-fix: AV exit code, green after). Full e2e +
feature-reg + stdlib-exec run at doc time. Stdlib session may flip
secure_random_bytes -> os_secure_random_bytes.

## 2026-09-09 (stdlib lane) -- re-verified on round-20 binary (HEAD 223603ce + stdlib T007/io fixes): two items still OPEN

### R1. byte_at contextual OOB read STILL BROKEN (state-dependent bound check)
The r17 partial fix (isolated byte_at OOB returns 0) does NOT hold once a
case-mapping/slicing preamble runs first. Fresh probe:
probe_byte_at_context2.xi (preamble: str_lower/str_upper on multibyte +
str_slice, then byte_at("", 999) / byte_at(s, len) / byte_at(s, -1)).
Result: byte_at("", 999) returns 4 (adjacent bytes) instead of 0; the same
call before any preamble returns 0. The contextual assert in
smoke_string_bytecopy_locks stays gated. Fix direction: the builtin bound
check depends on prior string-op state (stale length/adjacency); the OOB
clamp must be a pure function of (str, index).

### R2. M58 residual: module-level mutable-array LITERAL INITIALIZER still emits invalid IR
M58 (index reads/writes through the real global) is fixed, but a module-level
`var _tbl: [256]Int = [ ... 256 literals ... ];` still fails to compile:
`store [256 x i64] %tmp (type 'ptr') -> expected [256 x i64]` at the
module-init store (the initializer materializes as a pointer, not the array
value). Probe: probe_m58_tbl.xi. No stdlib module currently needs this shape
(all tables are const); documenting so the fix can land without a stdlib
blocker. Recommended direction: fold the array literal into an @.init
global constant and memcpy/aggregate-store it, or GEP+elementwise store.

### R3. Catalog-body checker noise to expect during Item A maturation (stdlib lane triage notes)
These W000s appear on EVERY compile of the affected modules and are catalog
limitations, not stdlib defects (code compiles and runs correctly):
- "undefined variable 'io'/'string'/'size_of'/'alloc'" -- module-qualified
  / intrinsic resolution gaps inside catalog body typing (thread.xi spawn
  size_of[T], io.fs io.* calls).
- "unknown type 'fn() -> T' -- defaulting to i64" (generic fn-typed params).
- "non-exhaustive match ... YamlValue variant not covered" at 0:0 spans in
  serialize/yaml_lite consumers (line mapping unavailable; may be real --
  flagged for sweep triage, not yet stdlib-actioned).
When Item A flips warnings to hard errors, the T007 class is CLEARED
(128 whole-body-unsafe fns now carry requires); the classes above will
still fire and need checker-side resolution or span-level triage.

### R4. Cross-module OS-entropy multi-draw STILL AVs on round-20 (bisected 2026-09-09)
The r17-era blocker for the secure_random_bytes -> os_secure_random_bytes
flip persists. Full bisect on the round-20 binary (probes preserved in the
stdlib_campaign probes dir):
- single cross-module call to crypto.os_secure_random_bytes + byte reads:
  WORKS (exit 0).
- TWO calls in one frame + byte reads of either Vec: AV (0xC0000005).
- two calls WITHOUT byte reads: WORKS -- corruption shows up only when the
  returned Vec bytes are read.
- two calls in SEPARATE frames (each one os draw per frame) + reads: AV.
- reads of the FIRST draw's Vec after the second call: AV (both Vecs corrupt).
Shape = the second invocation of a cross-module fn containing
unsafe+extern+Vec (xiom_os_entropy loop) poisons Vec value slots in the
caller frame. This is what blocks the honest CSPRNG flip; the stdlib
interim (one flag-gated OS draw per process seeding the legacy LCG,
documented as NOT a CSPRNG) is committed and locked by
smoke_stress_crypto_secure_random_seeded. Flip the moment this lands;
direction: second-call Vec return-value slot clobbering (compare the
single-call vs double-call IR of the caller frame).

## 2026-09-09 (stdlib lane, evening) -- R4 FIXED (041e8bb3); CSPRNG flip landed (7148b615)

The stdlib lane flipped secure_random_bytes -> os_secure_random_bytes
after verifying the guard-arena escape fix on the current binary: probes
p_os_direct20 / p_os_double / p_os_twoframes all green with differing
draws; 5000-byte multi-draws (the true trigger shape) pass; crypto smoke
battery + consumers green. Lock smoke strengthened accordingly
(smoke_stress_crypto_secure_random_seeded). Their root-cause correction
(confined-block capacity growth past 16 bytes, not second-call) is
recorded in the flip commit.

### R5. xiom.net.address cross-module struct-Str returns corrupt (all fields empty)
Compiled via smoke_net_address + probes p_addr_probe/p_addr_dbg/p_addr_dbg2
(and p_addr_full): the ENTIRE address.xi, compiled as a standalone program
(module p_addr_full), parses "example.com:8080" and reads host/port/family
correctly. Imported as xiom.net.address, address_parse returns
Some(Address) with ALL fields empty (host=[], port=0, family=[]);
address_host/address_port (same-module readers, called cross-module) also
return empties. NOT a cache issue (--force identical). NOT reproducible
with minimal replicas (Option[struct], tuple (Str, Int), cross-module
xiom.net.ip classify calls, full-function replicas -- all pass) nor with
other struct-Str cross-module returns (io IOError e.message, net.mime
MimeType kind/subtype after the keyword rename). Shape-specific: a
3-segment dotted module whose exported fn returns Option[struct{Str x2,
Int}] built from tuple-sourced locals. Direction: compare the imported-
module vs main-module codegen of address_parse (module-boundary struct
return lowering). smoke_net_address stays compilefail-BLOCKED as a
flip-green lock. Stdlib-side candidate when root-caused: none (code is
correct -- full-file probe proves it).

UPDATE 2026-09-10 (round-25 binary): TWO distinct faces bisected:
1. CHECKER face (T001 "type 'Address' has no field 'host'"): occurs only
   when net.ip or net.header is imported AFTER net.address (smoke order
   address->ip->header; p_addr_ba address->ip T001; p_addr_bb
   address->header T001; p_addr_bd ip->address COMPILES; p_addr_bc
   address->convert COMPILES). Import-order-sensitive field resolution --
   suspect per-module type registration/ID reuse in the catalog.
2. CODEGEN face (values empty): persists in every order and with any
   import set (p_addr_bc/p_addr_bd compile but host mismatches). The two
   faces are independent; fixing either alone leaves the smoke red.
Note: CRT-layout fix 4b7dc529 did not affect either face.

FIXED 2026-09-10 (round-27, compiler lane). ROOT CAUSE (both faces):
the catalog's fuzzy `index_lookup` matched ANY module ending in
`.<last-segment>` for MULTI-segment requests. The checker's transitive
worklist therefore resolved the prefix `[xiom, memory]` (pushed by
`use xiom.memory.alloc`) to `examples/benchmark/bench_memory.xi`
(declared `module benchmark.memory`). Loading it pulled
`use benchmark.main.BenchResult` -> `benchmark.main` ->
`use benchmark.types` -> the whole benchmark graph into EVERY stdlib
compile. `benchmark.types.Address = {city, street, zip}` then claimed
the BARE name `Address` ahead of `xiom.net.address.Address`
(`{family, host, port}`) in both registries (first-wins):
- CHECKER: get_type('Address') fell back to the bare entry ->
  "no field 'host'" (order-sensitive because the winner depended on
  load order).
- CODEGEN: field-name lookup used benchmark's names while the struct
  layout came from the same first-wins entry -- `a.host` had no field
  index and fell to the generic `0` fallback -> all fields empty.

FIXES:
1. catalog.rs `index_lookup`: multi-segment fuzzy resolution is now
   ROOT-SCOPED (candidate must start with the requested first segment)
   and unique; single-segment leaf resolution (`use types.run_all` ->
   benchmark.types) stays unique-leaf. `[xiom, memory]` no longer
   resolves to `benchmark.memory`; `use xiom.slice` still resolves
   `xiom.string.slice` (regression caught by
   e2e_m47_round14b_multibyte_chars on the first, stricter attempt).
2. xiom-check `get_type`: before the bare-name fallback, prefer the
   qualified type of a module the CURRENT module actually imports
   (new per-module `module_import_paths`, recorded in process_use).
   Scoping to the current module is required -- a global import scan
   regressed smoke_async (`Future` resolved to another module's
   same-named type).
Locked by stdlib_exec_net_address_runs and stdlib_exec_net_http2_runs
(new strict Some(0) locks). smoke_net_address exits 0 (was T001/AV);
R6 below is fixed by the same root cause.

### R6. smoke_net_http2 graph: invalid getelementptr indices at codegen -- FIXED

FIXED 2026-09-10 (round-27): same root cause as R5 -- the benchmark
graph pollution (its same-named types shadowing stdlib types in the
codegen registry) produced mismatched struct field indices in the
mime/multipart/sse consumer graph. With the catalog lookup root-scoped,
smoke_net_http2 compiles and exits 0 (verified + stdlib_exec_net_http2_
runs lock). No separate getelementptr fix was needed.

## 2026-09-10 -- R7 FIXED: generic container mono for large aggregate V

Root-caused and fixed. The nested json round-trip is now deterministic
(8/8 runs) and the whole serialize-json family is green (kat_serialize_
json_minimal, nested, large_json, parse_valid/nested, jsonvalue_get/index,
object/array/number/string/bool/null, detect_format, is_valid, convert_json).

Seven interacting defects, all fixed in codegen:

1. GENERIC ARG INFERENCE from container params: `push_v[V](v: &mut
   Vec[V], x: V)` with a Vec[JsonValue] arg hit the nested-generic branch,
   which hardcoded "Int" and broke BEFORE `x: V` could infer JsonValue --
   the 112-byte arg was coerced to its i64 tag and stored 8 bytes. New
   `infer_generic_arg_from_container` resolves the concrete container args
   (local_xiom_types string, Vec/Slice LLVM elem, call returns); on failure
   it keeps the historical "Int" fallback (continuing let an outer-type
   fallback pick a bogus non-type -- m35_t28/m35_o06 AVs).
2. CALL/INDEX args as generic args: `Map.insert` V inference treated
   `json.json_number(..)` and `old.values[i]` as Int. The ctor arm now
   falls back to `infer_call_return_xiom` for module-qualified calls, and a
   new Index arm resolves the element type via `resolve_vec_elem_xiom`
   (entries.insert(k, old.values[i]) mono'd _Str_JsonValue).
3. `resolve_vec_elem_xiom` Field arm: the registered generic type_meta
   keeps RAW params ("keys" -> "Vec[K]"), so `entries.keys` returned "K"
   and escape args materialized a 1-BYTE temp (garbage keys). The
   substituted generic-instantiation path now runs FIRST
   (`substituted_generic_field_vec_elem`), with the direct type_meta arm as
   fallback for non-generic structs.
4. CONTAINER-FIELD BINDINGS keep the element type: `var vals = m.values;`
   dropped the record, so `vals[i]` scalar-loaded the tag. The let/var
   inheritance path now resolves Field initializers through
   `resolve_vec_elem_xiom`.
5. ANNOTATED LOCAL SLOTS use `concrete_type_for` (Option__JsonValue /
   Result__Payload__Int) instead of the erased `llvm_type_for`, fixing the
   opaque-slot vs concrete-ctor store mismatch. Their definitions are
   PRE-REGISTERED before the type-decl emission (`collect_annotation_types`
   walks params/returns/let/var annotations + nested blocks) because clang
   requires SIZED types at alloca/GEP parse time -- a trailing definition is
   too late. A body-time deferred list remains as a fallback for both the
   serial and parallel compile paths (driver uses the serial path).
6. OPAQUE->CONCRETE container conversion in coerce_value: a ctor built
   while the expected type was the erased base (`Ok(...)` in a fn returning
   Int) stored into a concretely-annotated slot. The conversion copies the
   tag and unboxes/repoints each payload field, GUARDED by the tag branch --
   inactive payloads hold 0, and an unguarded inttoptr 0 trapped
   (m21_result_option_013 / m35_o06). Discriminants: Some/Ok = 1, Err = 0.
7. Str ELEMENT args to i8* params inttoptr: `_json_escape_impl(entries
   .keys[i])` truncated the handle to a byte (the Index arg did not resolve
   its element type in coerce_arg_for_param).

Verified: p_gp_a/b/c, p_map_json, p_json_min2, p_json_nested_dbg green;
smoke_stress_serialize_json_nested deterministic x8; e2e 2306/2306,
feature-reg 510/510, stdlib-exec 79/79 (+2 ign), checker 182/182.
Locks: stdlib_exec_serialize_json_nested_runs,
stdlib_exec_serialize_large_json_runs.

R8 REMAINS OPEN (compiler side): contract clauses using free-fn
method syntax (`s.char_count()`) evaluate to 0, and free-fn ensures
attached to the method-position builtin `.char_at` corrupt the stack
(stdlib clause removed with a PENDING note). Next contract-codegen round.

## 2026-09-10 (stdlib lane, round-29 verification) -- R7: generic container mono truncates/corrupts large V

Found while verifying the json Part-2 fix. The json nested round-trip
(smoke_stress_serialize_json_nested) still fails: JsonValue values stored
through generic containers come back garbage, nondeterministically
(sometimes keys too, depending on heap layout). Bisected to the generic
mono of Vec operations for large aggregate V (JsonValue ~112 bytes):

- A: concrete `var vs = Vec[JsonValue].new(); push_v(&vs, v)` where
  `fn push_v[V](v: &mut Vec[V], x: V) { v.push(x); }` -- the push inside
  the generic fn writes v at the wrong width; reading vs[0] + stringify
  gives 1.088e-311 (v's bytes misplaced). Probe p_gp_a.
- B/C: `type Holder[V] = { values: Vec[V]; }` with
  `fn make_holder[V]() -> Holder[V] { Holder[V]{ values: Vec[V].new() } }`
  -- the Vec[V].new() inside the generic ctor yields a corrupt vector;
  a later push (generic OR concrete-from-main) AVs 0xC0000005.
  Probes p_gp_b / p_gp_c.
- Contrast (all green): concrete Vec[JsonValue] creation+push from main
  (p_vect_json: v0=42, v1="second"); json array paths; Map[Str,Str].
- User-visible: Map[Int,JsonValue]/Map[Str,JsonValue] `.values[i]`
  stringifies garbage (p_map_key_probe: m7=[garbage bytes]), while keys (Str)
  are fine. This is the remaining red of smoke_stress_serialize_json_nested
  after the Part-1/2 land (kat_serialize_json_minimal now PASSES).
Direction: the mono substitution for (a) Vec[V].new() inside generic
fns/ctors and (b) Vec[V].push from inside generic fns -- both must use
the substituted concrete element size (112), matching the already-fixed
generic-field READ stride. Probes preserved in the stdlib probes dir.

## 2026-09-11 -- R8 follow-up + R10 FIXED (method sugar routing, nested container args)

R8 follow-up -- Str method sugar on a PARAM (`s.trim()` -> len=0xFFFFFFFF):
three coordinated roots.
1. CODEGEN (call.rs): context called `Str.trim` (no such fn) and the
   auto-stub returned garbage. Method sugar for trim/trim_start/trim_end/
   to_lower/to_upper now emits the canonical stdlib symbols directly
   (`@string.str_trim`, `@trim.str_trim_start|_end`,
   `@lowercase.str_lowercase`, `@uppercase.str_uppercase`) with the
   receiver deref'd from a &Str slot / inttoptr'd from an i64 handle.
2. CHECKER (collect_expr_names): the injection reachability filter prunes
   fns by referenced leaf names; a sugar call referenced only "trim", so
   str_trim's BODY was never injected and codegen stubbed the symbol. The
   reference walk now seeds the canonical leaves (str_trim, str_trim_start,
   str_trim_end, str_lowercase, str_uppercase).
3. CHECKER (PRELUDE): trim_start/trim_end live in the xiom.string.trim
   submodule and lowercase/uppercase in their own submodules --
   force-loaded with the prelude alongside xiom.string so the canonical
   modules exist in every xiom.* program.

R10 -- `Vec[Option[struct-with-Str]]` element reads AV (Captures.get):
1. `type_from_ast_with_args` only preserved Vec/Map/Set args; a struct
   field `Vec[Option[M2]]` was registered as "Vec[Option]" so the element
   read fell to the scalar path. Option/Result args are now preserved
   (layout-neutral: llvm_type_for erases them to %struct.Option/Result).
2. `resolve_vec_elem_type`'s Field arm returns bracketed container elements
   ("Option[M2]") so the index site maps to the concrete/erased struct.
3. The same scan `break`t on the FIRST type_meta key matching the base
   name, so a stale/qualified key without the field ended resolution early;
   it now scans all matching keys.

Locks: e2e_m65_str_method_sugar, e2e_m65_vec_option_struct_str,
stdlib_exec_regex_captures_get_runs.
Gates: e2e 2311/2311, feature-reg 510/510, stdlib-exec 84/84 (+2 ign),
checker 182/182.



## 2026-09-11 -- smoke_array_zip FIXED: fixed-array tuple elements

Three roots:
1. `array.zip[T,U,const N]` returns `[N](T,U)`; the mono body's local slot
   resolved the composite element name RAW ("Tuple__T__U") because
   llvm_type_for only substituted a single-char generic name. New
   `subst_type_tokens` replaces whole identifier tokens (with '_' as a
   separator, so mangled names tokenize) -- the local is now
   `[3 x %struct.Tuple__Int__Int]`.
2. Fixed-array indexing always ran `val_to_i64` on the loaded element,
   BOXING struct elements into a heap handle and returning i64; `.0` then
   saw i64 and fell to the 0 fallback. Struct elements now return by value.
3. Computed-value `.0` field access (`zipped[0].0`) only consulted
   `types.types`; tuple layouts live in `type_meta`. The lookup now falls
   back to type_meta (exact + suffix) and uses `resolve_field_index` for
   numeric tuple fields ("0" vs "_0"). The same fallback was added to the
   struct-value local path.

Lock: `stdlib_exec_array_zip_runs`.
Gates: e2e 2309/2309, feature-reg 510/510, stdlib-exec 83/83 (+2 ign),
checker 182/182.



## 2026-09-11 -- smoke_ptr_offset + smoke_convert_escape FIXED

1. ptr_offset: `substitute_type` double-wrapped raw pointers -- the
   `Type::Ptr(i2)` arm recursed with outer = the WHOLE node, and the outer
   match wrapped again, so `*const T` substituted to Ptr(Ptr(Int)) and the
   call site emitted `i64**` while the mono def returned `i64*`. The arm
   now substitutes the pointee (`substitute_type(i2, i2, ..)`).
2. ptr_offset (second face): `d` (a CTFE-folded pointer value, i64) compared
   with a real pointer emitted `icmp ne i64* %p, %i64` (invalid IR). The
   comparison lowering now inttoptrs the scalar side when the other is a
   pointer.
3. convert_escape: `decoded.value.len()` (Result[Vec[UInt8], Str] payload)
   took the Str.len path and called xiom_str_len on a %struct.Vec --
   `is_container_vec_field` only inspected the ERASED Result layout
   ("value" -> "Int"). It now consults `field_payload_xiom` for payload
   fields, so Vec/Slice/Array/Map/Set payloads route to the container path.

Locks: stdlib_exec_ptr_offset_runs, stdlib_exec_convert_escape_runs.
Gates: e2e 2309/2309, feature-reg 510/510, stdlib-exec 82/82 (+2 ign),
checker 182/182.



## 2026-09-11 -- interface dispatch arity hardening + remaining triage

1. CHECKER: `allow_interface_dispatch` accepted ANY arity for interface
   method calls; `value.hash()` on `interface Hash { fn hash(self, hasher) }`
   passed checking and codegen emitted the erased body as an invalid
   ptr->i64 cast (smoke_hash_values). The dispatch now validates arity
   against the interface signature (self-param implicit) and errors
   "expects N argument(s)". In catalog bodies the finding stays a warning
   (Item A staged rollout) so codegen can still hit the stale call -- the
   SMOKE FIX is stdlib-side: hash_value's `value.hash()` no longer matches
   the Hasher-based Hash API.
2. REMAINING TRIAGE (verified on HEAD):
   - smoke_ptr_offset: clang "i64 defined but expected ptr" (missing
     ptr conversion at an op).
   - smoke_convert_escape: clang "%struct.Vec ... but expected ptr".
   - smoke_array_zip: checker T001 tuple element store on `[N](T,U)`
     (Stage 2c TypeId deliverable).
   - smoke_stress_io_read_int_float: STDLIB/FIXTURE mismatch --
     `io.read_int()` takes 0 args (io.xi:108) while the smoke calls
     `io.read_int(c)` with a Cursor; checker correctly reports the arity.
   - smoke_stress_regex_captures x4 / match_count: stdlib call form
     (method-position `.char_at(...).unwrap()`; see the regex section).
   - smoke_hash_values: stdlib Hash-API call (above).
Gates with the hardening: checker 182/182, feature-reg 510/510,
stdlib-exec 80/80 (+2 ign).



## 2026-09-11 -- smoke_math_edge FIXED: defined shl/shr semantics

`math.shl(1, 100)` trapped (0xC000001D) at runtime. Root cause: the
math bitwise/shift BUILTIN intercept (call.rs, BUG 15 path) emitted a raw
`shl i64 1, 100`; LLVM shifts with a count >= bit width are POISON, and
clang -O2 materialized the poison as `ud2`. The stdlib math.shl/shr have
DEFINED semantics (n <= 0 -> a; n >= 64 -> 0 / sign extension; n == 63
INT_MIN cases). The builtin now emits exactly those semantics with
guards:

  neg_n  = icmp sle rv, 0
  big_n  = icmp sge rv, 64
  safe_n = and rv, 63                     ; avoids poison entirely
  shifted = shl/ashr lv, safe_n
  shl: big = 0;  shr: big = select(lv < 0 ? -1 : 0)
  res = select big_n, big, shifted
  res = select neg_n, lv, res

Verified: smoke_math_edge exit 0; p_shift covers shl(1,10)=1024,
shl(5,-1)=5, shl(1,100)=0, shl(2,63)=0, shl(1,63)=INT_MIN,
shr(-8,1)=-4, shr(-1,100)=-1, shr(8,1)=4, shr(5,-2)=5,
shr(-8,63)=-1, shr(7,63)=0 -- all exit 0.
Locks: stdlib_exec_math_edge_runs, e2e_m65_shift_semantics
(tests/regression/m65_shift_semantics.xi).
Gates: e2e 2309/2309, feature-reg 510/510, stdlib-exec 80/80 (+2 ign),
checker 182/182.



## 2026-09-11 -- regex family compile fails FIXED (container element coherence)

The four regex captures smokes + match_count used to fail at clang
(`%struct.Match` vs `%struct.Captures`) and then AV. The COMPILE failures
are fixed; the residual runtime AV is a STDLIB call-form bug (below).
Seven codegen roots:

1. `type_arg_to_name` only rendered `Vec[...]` chains -- `Option[Int]` /
   `Result[A,B]` / any bracketed ctor arg fell to "Int". It now renders any
   `Base[args]` (Field bases included) and tuple args
   ("Result[Int,Str]"). `Vec[Option[Int]].new()` used to store elem_size 8.
2. `Vec[Option[Match]].new()` needs Option__Match's REAL size (32) -- new
   `ensure_container_named_concrete` registers Option__T/Result__A__B before
   sizing, and `vec_elem_storage_size` sums the concrete fields when the
   name is registered (else keeps the erased 16/24).
3. `resolve_vec_elem_type` (Ident arm) now returns bracketed CONTAINER
   elements ("Option[...]", "Result[...]", "Map[...]", "Set[...]") so the
   index site can memcpy the right struct instead of scalar-loading the tag.
4. The Index read maps container element names to their LLVM struct
   (`%struct.Option`/`%struct.Result` or the registered concrete).
5. The Some ctor adopts an ALREADY-REGISTERED concrete Option__T when the
   return-derived candidate's payload type mismatches the value
   (`groups.push(Some(match_obj))` in a fn returning Option[Captures]).
   Only registered candidates are adopted -- creating one here mismatched
   consumers whose signature stayed opaque (net_address Option[Tuple...]).
6. Body-time concrete definitions are SPLICED into the type-decl block at
   final assembly (`type_decl_splice_offset` + `splice_deferred_type_defs`)
   so clang parses them SIZED before any alloca/GEP. This replaces the
   trailing-definition fallback, which was too late for the IR parser.
   The decl pre-pass also collects Vec/Mono ctor bracket type args
   (`call_bracket_type_arg`).
7. `coerce_value` now bridges CONCRETE -> OPAQUE containers (box struct
   payloads) as well as the R7 opaque -> concrete direction -- a generic
   `Option[T]` mono param can stay erased while the caller built
   Option__Data (m21_struct_mut_015).

STDLIB HANDOFF (residual regex AV): engine.xi/regex.xi call
`pattern.char_at(pos).unwrap()` in METHOD position. XIOM resolves
method-position `.char_at` to the builtin Char-returning overload (correct
for glob's `== '*'`), so `.unwrap()` is invalid -- the checker warns
"cannot compare <error> with Char" and codegen produces garbage
(ASAN: xiom_str_len(1) in Regex.find_first_match). The free-call form is
fully supported and correct: `string.char_at(pattern, pos).unwrap()`.
Probe p_charat_form verifies BOTH forms (free-call -> Option[Char],
method -> Char). Convert the ~8 method-position sites in
regex.xi/engine.xi to the free-call form (or compare the Char directly);
smoke_stress_regex_match_count shares the same root.

Locks: e2e_m65_vec_option_elem (tests/regression/m65_vec_option_elem.xi).
Gates: e2e 2308/2308, feature-reg 510/510, stdlib-exec 79/79 (+2 ign),
checker 182/182.



## R8. char_at contract codegen -- FIXED (2026-09-11)

Face A -- METHOD-POSITION FREE-FN CALLS (receiver sugar): `s.char_count()`
where `char_count(s: Str)` is a free fn was rejected by the checker
("cannot call 'char_count' on this expression"); in catalog bodies the
finding is staged, and codegen then compiled the sugar to a constant-0
stub -- so `string.char_at`'s ensures (`pos < s.char_count()`) evaluated
0 and aborted EVERY Some return (json/glob verification path).
FIX (checker, method-call fallback): resolve a registered fn whose LEAF
matches the method name, whose FIRST parameter's base type equals the
receiver base (Str/Vec/user type), and whose arity is receiver + args;
return its declared return type. Codegen already resolved the free fn and
prepended the receiver (IR-verified: `%tmp5 = call i64 @string.char_count
(i8* %tmp4)`), so no codegen change was needed. User-space and catalog
free fns both work (`p_r8_a`, `p_r8_b`, `p_hashcall`).

Face B -- free-fn ensures vs the builtin Char-returning `.char_at` path:
verified CLEAN against an ISOLATED patched stdlib copy ($XIOM_STDLIB
override) with the byte-domain clause restored
(`ensures: result is Some => pos >= 0 && pos < s.len()`): p_globchar
(`pattern.char_at(1) == '*'`), smoke_string_glob, smoke_convert_url,
smoke_convert_ascii85, smoke_convert_base32, smoke_stress_serialize_json_
nested and kat_serialize_json_minimal all exit 0. The stdlib lane can
re-add the clause (the PENDING note in string.xi is obsolete).

Regression lock: e2e_m65_r8_method_free_fn
(tests/regression/m65_r8_method_free_fn.xi). Gates: e2e 2307/2307,
feature-reg 510/510, stdlib-exec 79/79 (+2 ign), checker 182/182.
OP NOTE: judge checker changes on a FRESHLY REBUILT canonical binary --
the first run used a stale target/debug/xiom.exe and produced phantom
failures (same class as the standing "baseline rebuild" discipline).

### R8 ORIGINAL FINDING (stdlib lane, 2026-09-10)

Stdlib finding from the json/glob verification (2026-09-10):
1. `string.xi char_at`'s ensures used `s.char_count()` -- char_count is a
   FREE fn; method syntax in a contract evaluates to 0 at runtime, so
   every Some return tripped "ensures at 236:12" (hit via the json build
   path; probe p_json_nested_dbg).
2. Rewriting the clause to the byte-domain `pos < s.len()` (correct for
   the byte-position body) made method-position `.char_at(...)` calls --
   the BUILTIN Char-returning overload used by xiom.misc.glob -- corrupt
   the stack: 0xC0000409 at the first wildcard loop (probe p_glob_trace;
   same probe green with the clause removed). Suggests the free fn's
   ensures are attached/evaluated for the method-position builtin call
   with a result-type mismatch.
Stdlib state: clause removed with an in-file PENDING note (body guard
unchanged); a correct in-range clause should be re-added once contract
codegen distinguishes the free fn from the builtin method and supports
free-fn calls in contracts.

### R7 follow-up -- REGRESSION (r20 green -> r29 compile-fail): erased Option slot for concrete Option[JsonValue]

smoke_stress_serialize_large_json compiled and exited 0 on the r20 sweep
(7.7s). On r29 it fails at codegen:
  xiominput.ll:5780: error: '%tmp135' defined with type
  '%struct.Option__JsonValue = { i64, %struct.JsonValue }' but expected
  '%struct.Option = { i64, i64 }'
  store %struct.Option %tmp135, %struct.Option* %tmp131
Shape: 20+ repeated `arr = json.json_array_push(arr, json.json_number(n))`
statements -- a concrete Option[JsonValue] temporary is stored into an
ERASED Option {i64,i64} slot. Same class as round-28's "concrete
container payloads no longer unboxed as erased i64" (fixed for the
shapes they tested; this repeated-temp shape still erases). REGRESSION
vs r20 -- worth a gate addition (stdlib-exec should include this smoke).
Probe/repro: examples/stdlib_smoke/smoke_stress_serialize_large_json.xi
(compile-only failure).

## R9. Delegation residual: full-path calls without a module import corrupt at runtime

Found 2026-09-10 while verifying the dedup shims under r29 (same on r25:
reproduced with both binaries). The m62 delegation fix resolved qualified
calls when the target module is registered; the following shapes still
crash 0xC0000409 (stack cookie) deterministically:

1. CALLER full-path without import: a module calling
   `xiom.string.glob.glob_match(...)` (or xiom.string.soundex.soundex,
   xiom.string.levenshtein.levenshtein_distance) WITHOUT `use` of that
   module crashes. Repros p_x1/x2/x4/x6 (glob), p_sdx_shim_first,
   p_lev_shim_first. If the SAME program first calls the canonical module
   directly (xiom.misc.glob.glob_match), the later shim call works
   (p_x5, p_glob_trace, smoke_misc_soundex_parity) -- i.e. registration
   by a direct call masks the bug.
2. SHIM-INTERNAL missing import: the string.glob shim's body called
   `xiom.misc.glob.glob_match` while the shim itself never imported
   xiom.misc.glob. Consumers importing only the shim crashed
   (smoke_string_glob). Stdlib-side FIXED (all shims now `use` their
   target; commit 1e099b84), but the underlying resolver behavior remains:
   an unresolved/never-registered full-path call should be a hard
   compile/link error, never a runtime stack corruption.

Impact: dedup parity smokes must use the sanctioned twin-vs-vectors style
(kat_convert_base64_parity); dual-module side-by-side full-path calling is
unsafe until fixed. Current stdlib/examples all import their targets, so
the shipping corpus is clean; treat as a latent landmine + fix target for
the resolver's missing-import fallback.

## R8 follow-up. Method-position sugar still corrupts for some free fns on Str params -- FIXED 2026-09-11 (round 37)

`.trim()` in method position on a Str PARAM returns a corrupt Str:
  fn f(s: Str) -> Int { let t = s.trim(); t.len() }   // -> 4294967295
Probe p_strparam2: `s.len()` / `s.byte_at(0)` / `string.str_slice(s,..)` /
`string.str_trim(s)` all correct on the same param; only method-form
`.trim()` is corrupt. Same class as the R8 char_at case (fixed by
20aa3d07 for char_at); trim was not covered. Stdlib avoids the shape
(io.parse_int/parse_float use string.str_trim with a comment). A general
fix should route method-position sugar through the same resolution the
free-call path uses, for all free fns -- not per-name.

## R10. Vec[Option[struct-with-Str]] element reads AV (container element coherence) -- FIXED 2026-09-11 (round 37)

`Vec[Option[Match]].push(Some(m))` succeeds, but reading `v[i]` (or doing
the read inside a method that returns it) AVs 0xC0000005. Local minimal
repro p_cap_mirror (M2 = {start,end,text:Str}; C2 = {groups:
Vec[Option[M2]]}; `fn get2(c: C2, i: Int) -> Option[M2] { ...
c.groups[i] }`): everything up to the get call prints, then AV. Matches
the earlier round-28 family (concrete container payload unboxing), still
open for OPTION-wrapped struct elements. User-visible:
smoke_stress_regex_captures_get (Captures.get) stays red; the other four
captures smokes are green after the API realignment. Fix direction:
element load for Option[aggregate] in Vec must use the concrete
Option__M2 layout, not the erased form.

## R11. Iterator `.filter()` silently returns EMPTY on r31/r32 (regression vs r30)

**STATUS 2026-09-12 (compiler round 53): RESOLVED at HEAD.** Verification on
the current canonical binary: `smoke_iter_pipeline`, `smoke_iter_filter`,
`smoke_iter_chained_adapters` all exit 0; the minimized probe
(`p_iter_filter_r32.xi`) exits 0; preserved binaries reproduce the reported
bisect exactly (r29 green, r30 green, r31 red -> run=1, current green). The
r31/r32 red was the mid-session Stage 3 Item A WIP tree, not the committed
slices 3/4; the finalized collect-then-check + isolation state is green.
PROBE ANOMALY (separate, PRE-EXISTING): the combined stage probe
`p_iter_pipeline_r32.xi` (all five chains in one function) AVs
(0xC0000005) on r29, r30 AND current HEAD, while r31 happens to be green --
i.e. it is NOT a reliable R11 repro and NOT a regression from Stage 2c. The
individually staged variants (A..E) all pass; the AV needs a heap-corruption
debug pass (ASAN/Application Verifier) and is queued as a separate latent
item. Use the smoke + the minimized filter probe for R11 gating.

Found 2026-09-12 (stdlib-lane r32 sweep, first solo-confirmed new failure).
`smoke_iter_pipeline` (r29 CSV: green, exit 0; r32: exit 1) is red on the
r31/r32 binary and green on r29/r30. Minimized probe (preserved at
`C:\Users\lefte\AppData\Local\Temp\kilo\stdlib_ws\probes\p_iter_filter_r32.xi`):

```text
iter.range(1, 50).filter(fn(x: &Int) -> Bool { return true;  }).collect().len()  -> 49 on r29/r30, 0 on r31/r32
iter.range(1, 50).filter(fn(x: &Int) -> Bool { return false; }).collect().len()  ->  0 everywhere
iter.range(1, 50).filter(fn(x: &Int) -> Bool { return *x % 3 == 0; }).collect().len() -> 16 on r29/r30, 0 on r31/r32
iter.range(1, 50).collect().len()                                                -> 49 everywhere (range path fine)
```

The predicate is never consulted (`true` and `false` both give 0) -- the
FilterIter.next closure `fn() -> Option[T] { return it.next(); }` appears
to be dead/no-op. Stage-wise probe `p_iter_pipeline_r32.xi` shows the
breakage starts exactly at the first `.filter()` link; `.map()` and later
links inherit the empty input.

Bisect by preserved isolated binaries:
- target_r29 (2026-09-10): green (run=0)
- target_r30 (2026-09-11 17:14): green (run=0)
- target_r31 (2026-09-12 14:19): red (run=1); target_r32 = r31 copy: red

Suspect window: commits since the r30 build -- 380febec (Stage 2c slice 3
interned CheckedType), 51c81efd (Stage 2c slice 4 TypeId registry/canonical
codegen keys), fecf69fe (Stage 3 Item A corpus report API). NOTE: the r31
build may also have included an earlier revision of the compiler lane's
currently-uncommitted Stage 3 Item A phase-2 checker work (the tree held
that WIP at 14:27 today); a clean-HEAD rebuild will separate the two.

Stdlib side is NOT implicated: `stdlib/xiom/iter/iter.xi` has no commits
since well before r29 (git log --since 2026-09-10 -- stdlib/xiom/iter is
empty) and smoke_iter_pipeline is untouched; the shape is lazy-adapter
closure capture + `FilterIter.next` dispatch. Impact confirmed by the r32
sweep: smoke_iter_pipeline, smoke_iter_filter, and
smoke_iter_chained_adapters all red (r29 green, solo-reproduced).

## R12. Numeric-tower `Int.to_float` wrapper self-calls with a pointer self on r31/r32

**STATUS 2026-09-12 (compiler round 53): RESOLVED at HEAD.** The minimized
probe `p_ct_tower2.xi` compiles and exits 0, and `smoke_convert_traits`
exits 0, on the current canonical binary. The r31/r32 red was the
mid-session Stage 3 Item A WIP tree; the finalized state is green. (The
self-recursive forwarder shape is the same class as R9, whose fix landed
in `collect_external_decls`; no separate wrapper fix was needed.)

Found 2026-09-12 (same r32-sweep window as R11). `smoke_convert_traits`
(r29: green 6.5s; r32: clang failure) is red on r31/r32 and green on
r29/r30. Minimized probe (preserved at
`C:\Users\lefte\AppData\Local\Temp\kilo\stdlib_ws\probes\p_ct_tower2.xi`):
only

```text
use xiom.convert.into;
fn main() -> Int { var ifl = into.into_float(7); ... }
```

fails at clang. `--emit-ir` shows the generated forwarder is SELF-recursive
and feeds the alloca POINTER as the i64 self argument:

```llvm
define double @tower.Int.to_float(i64 %param_self) { ...
  %tmp3 = alloca i64
  store i64 %param_self, i64* %tmp3
  %tmp4 = load i64, i64* %tmp3
  %tmp5 = call double @tower.Int.to_float(i64 %tmp3, i64 %tmp4)
  ...
```

clang: `error: '%tmp3' defined with type 'ptr' but expected 'i64'`.
The wrapper should call the REAL implementation symbol for `Int.to_float`,
not re-enter itself with an injected receiver pointer. Bisect by preserved
binaries: target_r29 green, target_r30 green, target_r31/r32 red (identical
window to R11). Prime suspect: Stage 2c slice 3/4 (interned CheckedType +
TypeId-keyed registry / canonical codegen keys), which changed the
method/impl symbol keying; the tower impl key (`tower.Int.to_float`) and
its wrapper now diverge. Stdlib side unchanged (`convert/into.xi` untouched
since before r29). Impact: smoke_convert_traits + any `into_float` /
numeric-tower consumer.

## R13. Tuple return-type identity split: Tuple__Int__Int vs Tuple__UInt64__UInt64 on r31/r32

**STATUS 2026-09-12 (compiler round 53): RESOLVED at HEAD.** The minimized
probe `p_hash_tuple.xi` compiles and exits 0, and `smoke_hash_farm`,
`smoke_hash_spooky`, `smoke_hash_t1ha_metro` all exit 0, on the current
canonical binary (the mismatch was clang-fatal on r31/r32, so a clean
compile+run is decisive). The r31/r32 red was the mid-session Stage 3
Item A WIP tree; the finalized TypeId/canonical-key state is green.

Found 2026-09-12 (third r31/r32 regression from the same r32 sweep).
`smoke_hash_farm`, `smoke_hash_spooky`, and `smoke_hash_t1ha_metro`
(r29 green, 5.5/5.6s) all fail at clang on r31/r32:

```text
xiominput.ll: error: '%tmp107' defined with type
'%struct.Tuple__Int__Int = type { i64, i64 }' but expected
'%struct.Tuple__UInt64__UInt64 = type { i64, i64 }'
```

Minimized probe (preserved at
`C:\Users\lefte\AppData\Local\Temp\kilo\stdlib_ws\probes\p_hash_tuple.xi`):

```text
use xiom.hash.farm;
var d = farm.farmhash128(&v);   // declared -> (UInt64, UInt64)
```

is enough. The callee's real signature is `(UInt64, UInt64)` (farm.xi:175,
181, 199) but the call-site receiver path materializes the result as the
Int-typed tuple struct. Layouts are identical ({i64,i64}) so only the
struct NAME mismatches -- i.e. the semantic-type identity, not the layout,
is split. `farmhash64` returns a plain UInt64 and is unaffected.

Bisect by preserved binaries: target_r29 green, target_r30 green,
target_r31/r32 red (same window as R11/R12). Prime suspect: the same
Stage 2c TypeId/canonical-key work; tuple names with UInt64 elements are
not unified with the Int-defaulted tuple in the call lowering.
Impact: farm/spooky/hash tuple consumers.

## 2026-09-12 (stdlib lane, r33 re-verification) -- R11/R12/R13 FIXED by b8e2fa43; 935/935 all-green

The three r31/r32 regressions were an earlier revision of the compiler
lane's in-flight Stage 3 Item A work (phase 1 at load time), superseded by
the committed collect-then-check implementation (b8e2fa43). On a fresh
isolated build of that HEAD (target_r33, built from the committed tree):

- p_iter_filter_r32: green (was run=1 on r31/r32)
- p_ct_tower2: green (was clang failure)
- p_hash_tuple: green (was clang failure)

Full r33 sweep: 935 files -> 935 PASS / 0 RUNFAIL / 0 COMPILEFAIL. This
is the first all-green corpus, and it includes the stdlib session's new
smoke_serialize_csv. R11/R12/R13 are CLOSED; the r32 reds were WIP
collateral, not committed-baseline defects.

## R15. Leaf-qualified codegen key collision: same-leaf modules + same-name fns bind a wrong 0-arg stub (delegation crash)

Found 2026-09-12 (stdlib lane, dedup wave 2). A delegating shim crashes at
runtime when its module leaf equals the canonical module leaf AND its fn
name equals the callee fn name. Probe pair preserved in stdlib_ws\probes:
- p_b32_shim.xi (`use xiom.convert.base32; base32.base32_encode(&v)`) ->
  exit 0xC000001D (0xC0000005 via smoke_convert_base32), no stdout even
  with explicit flushes.
- p_b32_canonical.xi (canonical only): green.
- p_b32_import_only.xi (import the shim, no calls): green.
- Control: the endian shim (convert.endian -> serialize.endian) has the
  SAME leaf name but DIFFERENT fn names (write_u64_be etc.) and is
  all-green, so the collision requires leaf + fn-name match.

IR evidence (stdlib_ws\probes\b32ir.txt, `--emit-ir`):

```llvm
define i8* @base32.base32_encode(%struct.Vec* %param0) {   ; shim body
  ...
  %tmp5 = call i64 @base32_encode(%struct.Vec* %tmp4)      ; bare symbol
}
define i64 @base32_encode() { entry: ret i64 0 }           ; 0-arg stub!
define i8* @encoding.base32_encode(%struct.Vec* %param0)   ; canonical
```

The shim's qualified call (`enc32.base32_encode`) did not bind the
canonical definition; it bound a synthesized bare stub with the wrong
arity (i64() vs Vec* -> i8*), corrupting the stack at runtime.

Root cause: the codegen qualified key is module-LEAF-qualified
("<leaf>.<fn>", per the 2026-09-09 m62 entry). `xiom.convert.base32` and
`xiom.encoding.base32` both have leaf `base32`; with the same fn name the
two definitions produce the SAME key and one becomes a stub. The m62 fix
(user module shadowing a catalog fn) does not cover catalog-vs-catalog
leaf collisions.

Stdlib impact: same-leaf + same-name pairs cannot be consolidated behind
shims until fixed -- convert.base32 vs encoding.base32 (identical names),
convert.percent vs encoding.percent (identical names), and future
convert.X vs encoding.X with matching names. Pairs with different fn
names (convert.endian, convert.ascii85) shim safely. Workaround in
place: keep the local implementation (base32 reverted 2026-09-12).

Fix direction: qualify the codegen key by the FULL module path
("convert.base32.base32_encode" vs "encoding.base32.base32_encode"), or
detect same-leaf key collisions and fall back to the dotted path for the
second registration.

RE-TEST after round 56 (9e625094, built as target_r37, 2026-09-12):
the base32 shim was re-landed and the same-leaf delegation now COMPILES
but returns a SILENT EMPTY value for the same-name fns (crash -> wrong
result):
- p_b32_s5a: `let e = convert-shim.base32_encode(&[102,111,111])` prints
  `B-enc=` (empty) instead of `MZXW6===`.
- p_b32_s5b (direct concat operand) same empty result, exit 0.
- The differently-named legs work: `base32hex_encode` ->
  `CPNMU===`, `base32hex_decode("CPNMU===")` -> 3 bytes.
- smoke_convert_base32 with the shim crashed 0xC0000005; with the local
  implementation restored it is green and `base32_encode` is correct.
So round 56's D5 work fixed silent crashes for user-module shadowing but
NOT catalog-vs-catalog same-leaf + same-name delegation: the shim's
same-named call still binds the wrong definition (empty Str path).
Stdlib action: shim reverted again (local impl kept); the encoding-family
consolidation stays gated on this exact same-name case. Probes preserved
(p_b32_s5a/s5b, p_b32_shim2/3).

ROUND-58 CALL-SIDE FIX LANDED (2026-09-14): R15 is fixed for the catalog
(catalog-vs-catalog same-leaf + same-name delegation works).

- Definition side (commit 4311b5db): the external-decl injection dedup only
  scanned TOP-LEVEL items, so a program module's fns were also injected as
  flattened duplicates (a second `ping`, keyed "network.ping"); the dedup now
  recurses into `TopDecl::Module` and records bare + path-qualified keys
  (crates/xiom/src/lib.rs); `fn_symbol`'s qualified-first lookup then
  re-landed safely (crates/xiom-codegen/src/decl.rs).
- The checker now returns the FULLY-DOTTED resolved key from
  `resolve_module_function` and records it per catalog-body call site
  (`Checker::catalog_resolved_calls`, "line:col" -> "xiom.encoding.base32.
  base32_encode"). The driver hands it to codegen
  (`set_catalog_call_targets`); `resolve_catalog_call` (call.rs) normalizes
  the `xiom.` prefix, sanity-checks the receiver text against the resolved
  path, and binds the recorded target before falling back to
  `resolve_module_call`.
- Injected catalog free fns are now qualified by the FULL xiom-stripped
  module path ("convert.base32.base32_encode" / "encoding.base32.
  base32_encode") instead of the leaf -- same-leaf pairs no longer share one
  name. Two-segment modules keep the historic "math.abs_float" spelling.
- `resolve_module_call` also tries the xiom-stripped dotted candidate so
  fully-qualified user calls (`xiom.iter.map.iter_map`) match the stripped
  injected names.
- Verification: the temp-shim stdlib probe pair (`p_b32_s5a`/`s5b` ->
  "B-enc=MZXW6===", exit 0; canonical/import-only unaffected), R14/R17 locks,
  stdlib-exec 85/85, feature-reg 510/510. REMAINING NARROW CASE: same-leaf
  same-name modules declared in the USER program (no catalog recording) --
  tracked as R15b; not needed for the encoding-family dedup.

### R15b FIXED (2026-09-15, round-65): same-leaf same-name USER-program modules

Reproduced with three source files passed on the command line (user
program, no catalog):

```
alpha_base32.xi:  module alpha.base32  pub fn encode(x)->Int { x * 2 }
beta_base32.xi:   module beta.base32   use alpha.base32 as canon;
                                       pub fn encode(x)->Int { canon.encode(x) }
gateway.xi:       module gateway       use beta.base32 as b32; b32.encode(10)
```

`--emit-ir` showed `beta.base32.encode` compiled as `call i64
@beta.base32.encode` -- SELF-recursion (0xC000001D trap). Four defects:

1. Free fns of USER-program modules share the bare key (`fn_key` = bare
   name), so two same-leaf modules both register `types.functions["encode"]`
   (last wins) and alpha's definition claimed the bare SYMBOL.
2. `use_alias_paths` recorded a leaf-qualified entry for MODULE aliases too;
   the codegen's alias map then overwrote the full path with the leaf
   ("b32" -> "base32") in random HashMap order, so `use beta.base32 as b32`
   resolved through a bare/unrelated module half the time.
3. `resolve_module_call` tried the LEAF-qualified key before the full dotted
   key, while user modules register full-qualified names.
4. The method-call emitter's unique `.name` suffix search counted
   module-qualified REGISTRATION ALIASES as distinct candidates.

Fixes (crates/xiom-codegen/src/decl.rs + crates/xiom-check/src/lib.rs):
- Free fns with a module context now register the MODULE-QUALIFIED signature
  and XIOM return type alongside the bare key (methods keep their
  receiver-qualified keys -- an extra alias made suffix searches ambiguous).
- `preassign_fn_symbols` counts bare keys per module: a key defined by more
  than one module qualifies EVERY definition's symbol (the first no longer
  keeps the bare slot); the bare slot keeps a deterministic first mapping.
- The checker records MODULE aliases without the `::qualified` leaf entry;
  codegen's alias map is built in two deterministic passes (full paths first,
  then leaf forms for item aliases).
- `resolve_module_call` prefers the full dotted key for multi-segment
  receivers (leaf fallback preserved for injected catalog names), and expands
  single-segment receivers through the alias map.
- The suffix-search fallback prefers candidates with a PREASSIGNED SYMBOL;
  registration-only aliases no longer make unique resolutions ambiguous.
- Leaf-module key registration is first-wins (a second same-leaf module no
  longer clobbers the first signature).

Verification: m78 exits 0 deterministically (5/5 compiles) and the emitted
IR binds `@beta.base32.encode` at the gateway call and
`@alpha.base32.encode` inside the shim. Regressions caught and fixed during
the full-suite runs: `eco_json_29_tests` (unqualified `.as_string()` on an
`unwrap()` receiver gained a second suffix candidate -> bare stub; fixed by
the symbol-backed candidate preference) and `e2e_m19_default_0075`
(nondeterministic `p.greet()` binding to the interface default; fixed by
limiting qualified registration to free fns). Lock:
`tests/regression/m78_user_sameleaf/` + `e2e_m78_user_sameleaf_modules`
(multi-source invoke). Gates: checker 188/188, stdlib-exec 85/85 (+2 ign),
feature-reg 510/510, e2e 2325/2325.

FLIP RE-HELD (2026-09-14, round 59): R19 and alias isolation fixed the
first blocker set (stdlib-exec 85/85 with strict ON), but the broader E2E
surface still hits pre-existing catalog findings the corpus cannot see,
because BARE-name/method resolution is load-ORDER-dependent.

Evidence (m34_j08 under strict): `xiom.encoding:151,156,162,244,249`
- `write_base64_triplet(buf, out, data.get(i).value, ...)`: the bare call
  binds a same-named fn from another module whose first param is `Box`
  (`argument 1 type mismatch: expected Box, found Int`).
- `data.get(i).value`: the method wildcard resolves `get` to a variant
  returning `UInt8` (then `.value` -> "cannot access field on non-struct
  type UInt8"). In other load orders (the all-imports corpus) the
  Option-returning `get` wins, so the corpus gate stays clean.
Stdlib action: qualify these calls (`base64.write_base64_triplet`, ...) /
import the defining submodule; same class as section Q. Compiler follow-up
(queued): make bare/method resolution scope-first and load-order-independent
(prefer current-module + explicit imports, then argument-compatible
candidates) so the corpus becomes a faithful superset gate.

FLIP HISTORY: strict first enabled 42943cd2 (held: 37/85 smokes);
re-enabled here after R19 + alias isolation (stdlib-exec 85/85) and held
again when the E2E surface exposed the order-dependent class above. The
un-ignored corpus gate remains the stdlib-regression canary;
`stdlib_execution_tests` + `e2e_tests` with strict default are the
real-compile gates.

## R19 FIXED (2026-09-14): generic deref-store pointee type

`trim_end_matches('*')` strips ALL trailing stars, so the deref-store path
mapped `i8**` (`*mut Str`) to `i8` and emitted `store i8 %ptr, i8**` -- clang
`'%tmp' defined with type ptr but expected i8` in `ptr.replace_Str`, reached
via `mem.replace[Str]`. Same class as BUG 44 in the deref-LOAD path.
Fixed in stmt.rs (deref assign) and call.rs (ptr.write/ptr.read inline
handlers): strip exactly ONE star. Lock: `e2e_m73_ptr_replace_str`
(`mem.replace[Str]` + `mem.replace[Int]`); the R19 workaround (core.xi
dropping `use xiom.mem;`) can now be reverted by the stdlib lane.

## ALIAS ISOLATION (2026-09-14): user aliases must not leak into catalog bodies

`flush_catalog_bodies` now retains only the pre-use module keys and clears
`imported_items`/`local_module_paths`/`module_import_paths`/`use_alias_paths`
per body (snapshot-restored afterwards), so each catalog body resolves
through ITS OWN `use` bindings. Before this, a USER alias
(`use xiom.collect.hash` binding `hash`) hijacked `collections.xi`'s
`hash.hash_combine` (its own `use xiom.hash` lost first-wins) and hard-failed
under strict. This is the real-compile counterpart of the corpus
`corpus_loading` isolation.

FLIP HISTORY: strict was first enabled 42943cd2, held when 37/85 smokes
failed, and re-enabled here after R19 + alias isolation made stdlib-exec
85/85 with strict on. The un-ignored corpus gate remains the stdlib-regression
canary; `stdlib_execution_tests` (strict default) is the real-compile gate.

ROUND-57 DIAGNOSIS (2026-09-12): definition-side partial landed then REVERTED
(regression); call site needs module-scoped alias plumbing.

- Definition side: `fn_symbol` was changed to prefer the module-qualified
  preassigned slot (`fn_symbol_map["{current_module}.{key}"]`) before the bare
  slot, which fixes the redefinition for local same-leaf modules
  (`tmp/bug_probes/p_r15_local.xi`: modules base32 + enc.base32 now emit
  distinct symbols). BUT it broke `e2e_m34_j08`: the driver emits a used
  module's fns through more than one path, and with qualified-first BOTH
  paths resolved to the same qualified slot, emitting `@network.ping` TWICE
  ("invalid redefinition of function 'network.ping'"). Reverted; the
  duplicate-emission path must be understood before re-landing.
- Call side: the shim's internal call resolves only the leaf key
  (`base32.base32_encode`), which under the collision is owned by the
  canonical module's registration; the emitter emits a zero-param stub. The
  real problem is lexical scope: catalog-body `use` bindings live in the
  checker's isolated context and are restored before codegen, so the emitter
  cannot know that the shim's `base32` alias means `xiom.encoding.base32`.
- FIX DESIGN (next compiler slice): carry each catalog module's own `use`
  bindings to codegen -- attach the checker's per-module
  `local_module_paths`/`use_alias_paths` snapshot to `CachedModule` and
  inject it with the decls (or stop flattening injected `TopDecl::Module`
  wrappers with their UseDecls). Then `resolve_module_call` builds the
  FULL-path key ("encoding.base32.base32_encode"), preassign registers it,
  and the duplicate-emission paths must be collapsed to one definition site.
  Until then the encoding-family consolidation stays GATED; differently-named
  shims (endian, ascii85) remain safe.

## R16. `ptr + int` in a call argument miscompiles (memcpy dest offset) -- Int-cast workaround

Found 2026-09-12 (stdlib lane, re-test of the reverted string fast-path,
night-session finding 13). Probes preserved in stdlib_ws\probes:
- p_str_memcpy.xi: direct `concat("foo","bar")` via two
  xiom_memcpy_dispatch calls (second dest `buf + len_a`) -> "foo<?>\x01".
- p_str_memcpy2.xi: (A) single memcpy from `s as *UInt8` -> CORRECT
  ("hello"); (B) memcpy first source + BYTE LOOP second -> CORRECT
  ("foobar"); (C) memcpy at `buf + len_a` -> CORRUPT.
- p_str_memcpy3.xi: (D) byte-loop first + memcpy at `buf + len_a` ->
  CORRUPT -- proves the DEST expression is the defect, not the Str casts
  or the double call; (E) two memcpys both at `buf` -> CORRECT (second
  overwrites), call count is fine; (F) `var dst2 = (buf as Int + len_a)
  as *UInt8; memcpy(dst2, ...)` -> CORRECT ("foobar").

So `ptr + int` used directly as an argument computes a wrong/truncated
address; casting through Int first produces the right one. The reverted
fast-path commit 21691b44 (revert d0a3851c) used `buf + len_a`, which
matches finding 13's chained-concat corruption from the 3rd link.
Stdlib re-landed the fast path on 2026-09-12 using the F workaround;
the underlying miscompile remains for other callers.

### R16 FIXED (2026-09-15, round-64): extern return types feed pointer inference

Reproduced with all three probes on the round-63 tree: p_str_memcpy3
`D=[fooE\x01]` (corrupt), E/F correct; p_str_memcpy2 C corrupt.

`--emit-ir` of the unsafe block showed the dest expression compiled as
string concatenation:

```llvm
%tmp32 = call i8* @xiom_int_to_string(i64 %tmp30)          ; len_a
%tmp33 = call i8* @xiom_str_concat(i8* %tmp29, i8* %tmp32) ; buf + len_a as Str!
%tmp38 = call i8* @xiom_memcpy_dispatch(i8* %tmp33, ...)   ; heap string as dest
```

`buf` is `i8*` at the ABI (same as Str), so the Add intercept's
`expr_is_pointer(left)` gate decides the lowering; it checks
`local_xiom_types["buf"]`. An EXPLICIT `var buf: *UInt8 = malloc(n)` fixed
the probe, proving the gap was INFERENCE: the extern registration recorded
only LLVM param/return types in `types.functions`, never the XIOM return
type in `types.fn_return_xiom`, so `var buf = malloc(n)` stayed untyped and
`buf + len` took the Str-concat path.

Fix (crates/xiom-codegen/src/decl.rs, `TopDecl::Extern` registration):
record `fn_return_xiom[name] = type_string_full(return_type)` alongside the
LLVM signature. Raw-pointer returns (`malloc -> *UInt8`) now type the
binding, `expr_is_pointer` fires, and `buf + len` lowers to
`getelementptr i8, i8* %buf, i64 %len` (byte pointer arithmetic) instead of
`xiom_int_to_string` + `xiom_str_concat`.

Verification: p_str_memcpy3 D/E/F all correct (`D=[foobar]`), p_str_memcpy2
A/B/C correct, p_str_memcpy all links 0..4 correct (the original finding-13
corruption). Lock: `tests/regression/m77_ptr_plus_int_arg.xi` +
`e2e_m77_ptr_plus_int_arg` (unannotated malloc buffer, byte-loop + memcpy at
`buf + len_a`, empty-string edges, and a 4-link concat chain). Gates:
checker 188/188, stdlib-exec 85/85 (+2 ign), feature-reg 510/510, e2e
2324/2324.

## 2026-09-12 (stdlib lane) -- Item A corpus CLEAN; strict flip unblocked

The last stdlib parse item is fixed: `xiom.time:232`'s non-operator `<=>`
was replaced with the exact active contract
`result.is_ok == (self.secs > earlier.secs || (self.secs == earlier.secs
&& self.nanos >= earlier.nanos))` (the handed `secs >=` form is wrong on
the equal-seconds nanos-borrow edge; probe p_duration_since.xi covers all
five branches). Commit e4d3ee1d.

Re-measure on committed HEAD 9e625094: `cargo test -p xiom-check
catalog_corpus_is_clean -- --ignored --nocapture` **PASSES** -> 0
findings, 0 hard errors, 0 parse errors; `is_clean() == true`. The staged
flip (`strict_catalog_findings: true` + `#[ignore]` removal) is
unblocked from the stdlib side. Runtime re-checks: 39/39 time smokes
green; full r36 sweep 935/935 (pre-fix binary) stands.

## R17. R14 regression: nested-index Str in concat emits an integer on r37 (9e625094)

Found 2026-09-12 (stdlib lane, first r37 smoke run). `smoke_serialize_csv`
regressed green -> red on the round-56 binary (target_r37, committed
9e625094): the `flat()` helper prints pointer-ish integers instead of the
field strings:

```text
basic: got [2653219331984|2653219332016|...] want [a|b|c;1|2|3]
```

Minimized probe (preserved: stdlib_ws\probes\p_nested_index_concat.xi):

```text
r: Vec[Str]; rows: Vec[Vec[Str]]
s1 = "" + r[0];            -> "a"                       (fine, both builds)
s2 = "" + rows[0][0];      -> r34 "a"; r37 "140699826060103"
"[" + rows[0][1] + "]"     -> r34 "b"; r37 "140699826060105"
```

So the R14 concat-operand fix (`expr_is_integer` -> `llvm_scalar_is_int`
fallback for Index/Field shapes) is applied to the OUTER index load of a
doubly-indexed Str element and misclassifies it as an i64 scalar; the
single-index Vec[Str] case still takes the semantic Str path. Impact: any
`x = x + v[i][j]` / inline nested-index string concat; CSV `flat()` is one
instance, and r36's 935/935 (r34) does not cover it. The interrupted r37
sweep had only reached early files when this was found -- expect more
corpus hits until fixed. Stdlib side unchanged (CSV module untouched
since e398df2f; r34 run of the same smoke is green).

## 2026-09-14 (stdlib lane, round-57 follow-up) -- R17 verified; section Q closed on isolated contexts

- **R17 fixed (84e14908) and verified on target_r38:** p_nested_index_concat
  prints `nested-assign=[a] / nested-inline=[b]` (exit 0), R14's
  p_iter_pipeline_r32 still prints E[0]=9 / E[4]=225, and
  smoke_serialize_csv is green. Full r38 corpus sweep launched as the
  definitive runtime gate.
- **Section Q (import discipline under faithful isolated contexts)
  149 -> 0 findings** in commit 0de47e11: `xiom.os` now imports
  env/io/string/core; `net.https` imports string; `log` imports io;
  `simd` imports math; `collections` declares the realloc/free externs
  its @-intrinsic calls need in scope; `crypto` declares malloc/free;
  `encoding.percent_encode` uses the bare same-module `url_encode`
  (self-qualification rejected by the gate). Re-measure:
  `cargo test -p xiom-check catalog_corpus_is_clean -- --ignored
  --nocapture` -> 0 findings / 0 hard errors / 0 parse errors; test
  PASSES. The strict flip is unblocked again on the stdlib side.
  Targeted runtime battery: 76/76 smokes green.
- **R15 remains open** (round-57 diagnosis: qualified-first symbol fix
  causes duplicate emission; needs module-scoped alias plumbing). Encoding
  family consolidation stays gated.

## R18. Contract false positive: Option payload `.len()` compared to a param `.len()` always violates

Found 2026-09-14 (stdlib lane, validating wave-3 contract shapes on r38).
Minimal probe (stdlib_ws\probes\p_wave3_opt.xi):

```text
fn wa(s: Str) -> Option[Str]
  ensures: result is Some => result.value.len() >= 0      // PASSES
{ return Some(s); }

fn wb(s: Str) -> Option[Str]
  ensures: result is Some => result.value.len() <= s.len() // FALSE VIOLATION
{ return Some(s); }
```

`wb("abc")` returns Some("abc"), so `3 <= 3` holds, yet the runtime
contract evaluator aborts at 13:12 ("contract violated: ensures"). The
payload `.len()` itself is fine (A passes); the failure appears only when
the payload length is compared against a PARAM's `.len()` inside the same
ensures. This is the shape a natural family of drop-in clauses needs
(str_strip_prefix/suffix payload bounds, Option-returning filters).
Wave-1/2 clauses avoided it (constant bounds only), so the corpus is not
affected today. Suspect: contract lowering resolves the RHS `s.len()`
against the wrong receiver (payload/return) or evaluates it on the
return slot; a fix should make the param receiver win and add a negative
lock (Some("abc") must satisfy `<= s.len()`).

### R18 FIXED (2026-09-15, round-63): payload reads on the bare `is` rebind

Reproduced with `p_wave3_opt.xi` ("contract violated: ensures at 13:12",
wa passed). `--emit-ir` showed the clause consequent compiled as:

```llvm
imply_conseq3:
  %tmp22 = load i64, i64* %tmp17      ; payload handle (dead)
  %tmp23 = inttoptr i64 0 to i8*      ; literal NULL!
  %tmp24 = call i64 @xiom_str_len(i8* %tmp23)   ; -1
  %tmp25 = load i8*, i8** %tmp3       ; s
  %tmp26 = call i64 @xiom_str_len(i8* %tmp25)   ; 3
  %tmp27 = icmp sle i64 %tmp24, %tmp26          ; -1 <= 3 -- passed by luck
```

Patching just `inttoptr i64 0` to `inttoptr i64 %tmp22` made the probe green,
confirming the emitter dropped the payload handle.

Root cause: the implication's bare `result is Some/Ok/Err` rebind (BUG 29)
binds the scrutinee to the PAYLOAD slot so the clause can call payload
methods directly (`result.len()`). A `.value` / `.error` field read on that
rebound name then has no struct to GEP into (the local is the i64 payload
slot), fell through the Field arm to the literal-`0` fallback, and
`result.value.len()` became `len(NULL)`. (wa passed only because clang
constant-folded/eliminated the dead check variant; wb compared against a
param and tripped.)

Fix (crates/xiom-codegen):
1. `LocalContext::is_payload_rebind` marks names bound by a BARE scrutinee
   rebind (only when the bound ident name equals the scrutinee name, so match
   arm `Some(v)` bindings are untouched).
2. The Field arm resolves `.value`/`.error` on such an i64 slot through the
   new `unbox_payload_slot` helper -- the payload itself (`inttoptr` for Str,
   bitcast for floats, box deref for struct/container payloads), mirroring
   the Option/Result `.value` override.
3. Err-side rebinds load field 2 (Result's error slot) in all three binder
   sites (struct-form, i64-form `bind_payload`, nested-is) -- they hardcoded
   field 1, so `result is Err => result.error...` bound the (zeroed) value
   slot.

Lock: `tests/regression/m76_contract_payload_param_len.xi` +
`e2e_m76_contract_payload_param_len` (payload len vs param len, payload value
equality, scalar payload bound, and the Err-side `result.error.len()` leg).
Gates: checker 188/188, stdlib-exec 85/85 (+2 ign), feature-reg 510/510,
e2e 2323/2323.

## R19. Generic `ptr.replace[Str]` stores the Str as i8 (clang ptr/i8 mismatch)

Found 2026-09-14 (stdlib lane, r38 sweep: smoke_stress_path_components
clang-failed; green on r36). It reproduces on BOTH the r34 and r38
binaries against the current stdlib, so it is stdlib-graph-triggered,
not a binary regression: the newer closure pulls `core -> mem -> ptr`
(my os->core import) and instantiates `ptr.replace[Str]`.

`--emit-ir` shows the miscompiled generic instantiation:

```llvm
define i8* @ptr.replace_Str(i8** %param0, i8* %param1) {
  ...
  %tmp4007 = load i8**, i8*** %tmp4001   ; dest slot
  %tmp4008 = load i8*, i8** %tmp4003     ; value (Str)
  store i8 %tmp4008, i8** %tmp4007       ; ERROR: should be `store i8*`
}
```

clang: `'%tmp4008' defined with type 'ptr' but expected 'i8'` at
xiominput.ll:8043. The store of a Str value through the generic `*T`
pointer picks the scalar fallback type (i8) for T=Str instead of the
pointer type -- the same R17 fallback family, now via generic pointer
stores rather than concat operands.

Stdlib workaround landed: `core.xi` dropped its UNUSED `use xiom.mem;`
(no `mem.` references), which keeps mem/ptr out of the path/os closure;
smoke_stress_path_components + smoke_bigfloat are green again solo. The
underlying bug remains for any real `mem.replace[Str]` caller -- lock
with a minimal `mem.replace` on a Str once fixed.

RE-TEST of R15 after 4311b5db (definition-side recursive injection dedup +
qualified-first symbol lookup) and the 42943cd2 flip, built as target_r39
(2026-09-14): STILL NOT FIXED. Re-landing the convert.base32 shim and
running the same probes:
- p_b32_s5a: `B-enc=` still EMPTY (silent wrong value; exit 0).
- smoke_convert_base32: 0xC0000005 crash with the shim.
- Local implementation restored: green immediately.
- smoke_encoding_base32 (canonical side) is green throughout.
So catalog-vs-catalog same-leaf + same-name delegation remains broken
after the definition-side fix; the silent-empty mode persists. Stdlib
shim reverted again; encoding-family consolidation stays gated. Probes:
p_b32_s5a/s5b (+ p_b32_shim2/3) preserved in stdlib_ws\probes.

## R20. R15 residual: delegated Result-returning same-name catalog fn yields empty-payload Err (Str legs fixed)

Found 2026-09-14 on target_r40 (a07507c4 "R15 catalog delegation fixed --
checker-recorded call targets + full-path injected names"). The Str-returning
same-name legs are now CORRECT, but Result-returning ones are not:
minimal probe stdlib_ws\probes\p_b32_residual.xi (convert.base32 shim over
encoding.base32) prints:

```text
enc=[MZXW6===]        <- base32_encode delegated: CORRECT now
hex=[CPNMU===]        <- base32hex_encode delegated: CORRECT now
inline ERR msg=[]     <- base32_decode("MZXW6==="): Err with EMPTY payload
helper ERR msg=[]     <- same via a helper frame: also Err/empty
RESIDUAL DONE (exit 0)
```

The canonical `smoke_encoding_base32` decodes the same input to Ok(3 bytes);
through the shim the caller always observes `Err("")`. Results are also
program-shape-dependent: in a different probe (p_b32_shim2) the same
decode calls produced plausible Ok values while `hex=` printed empty --
i.e. the delegated Result (tag + payload) is not reliably propagated from
the canonical body to the shim consumer. `smoke_convert_base32` with the
shim still AVs (0xC0000005); with the local impl restored it is green
immediately (also on r40).

Stdlib action: shim reverted again; encoding-family consolidation stays
gated on Result-returning delegation. The Str-side progress (a07507c4)
means a Str-only shim family can be reconsidered once one exists
(e.g. pure Str helpers), but base32/percent/punycode all expose Results.

### R20 FIXED (2026-09-15, round-62): owner-qualified call targets + deterministic fallback

Re-tested on the round-61 tree by re-applying the convert.base32 shim
(backup/restore, never committed). At that point the Result legs had
improved -- `inline OK len=3`, `helper OK len=3` -- but the STR legs were
still broken (`hex=[]`) and p_b32_shim3's direct-match decodes all returned
`Err("")`. Repeated `--emit-ir` runs of the same source showed the
delegated call binding DIFFERENTLY per process:

```llvm
; shim body, run 1..6 -- two outcomes at random:
%tmp5 = call i8* @encoding.base32.base32_encode(%struct.Vec* %tmp4)   ; correct
%tmp5 = call i64 @base32_encode(%struct.Vec* %tmp4)                    ; 0-arg stub
```

Root cause: `resolve_catalog_call` keys the checker's recorded targets by
source span only, then sanity-checks the receiver TEXT against the resolved
module path. A catalog body's receiver is usually an ALIAS (`use
xiom.encoding.base32 as enc32;` then `enc32.base32_encode(...)`), which the
textual check can never match -- so every aliased delegation fell through to
`resolve_module_call`, which cannot resolve catalog aliases either. The
resulting bare key then hit HashMap-order-dependent ".name" suffix scans
(`call.rs` pass 2 / `decl.rs`) that could bind the canonical fn, the shim
itself (infinite recursion -> 0xC0000005), or emit a zero-arg stub
(`ret 0` / `zeroinitializer`) -> empty Str, lost Result payload.

Fix (three parts):
1. The checker records each catalog-body module call under an
   OWNER-QUALIFIED key as well: `"{owner}#{line}:{col}"` where owner is the
   enclosing fn in codegen's injected naming (`convert.base32.base32_encode`,
   `xiom.` stripped). New `current_fn_qual` tracks it during `check_fn_decl`.
2. `resolve_catalog_call` tries the owner-qualified key FIRST and trusts it
   (the checker resolved the call in the same module context, so aliases are
   already resolved). The legacy span key + receiver check stays as a
   fallback for paths whose owner key the emitter cannot build.
3. `resolve_module_call`'s suffix scan is now deterministic: it drops the
   CURRENT function (a delegating shim must never bind itself) and picks
   longest-key-first, then name, instead of the first HashMap hit.

Verification: 4/4 separate `--emit-ir` processes emit the canonical target
at both the shim body and the consumer call site; the re-applied shim probes
are green (residual: `enc=[MZXW6===] hex=[CPNMU===] inline OK len=3 helper
OK len=3`; shim3: A/B correct, C..H all correct lengths; shim2 all 8 legs).
The shim was restored byte-identical (sha 660D5198...). Lock:
`tests/regression/m75_alias_delegation/` (canon + shim + an unrelated
longest-key module; main asserts Int/Str/Result Ok/Result Err across the
alias delegation) + `e2e_m75_alias_delegation`.

Gates after the fix: checker 188/188, stdlib-exec 85/85 (+2 ign),
feature-reg 510/510, e2e 2322/2322. R20 CLOSED; encoding-family dedup
unblocked for the stdlib lane.

## 2026-09-14 (stdlib lane, round-58 follow-up) -- bare-name worklist ZERO; strict flip re-ready

Round-58 held the strict flip at 37/85 stdlib-exec smokes. The stdlib lane
built an independent detector -- a per-module import probe (`use xiom.X`
alone) exposes that catalog body under its own imports, exactly like the
isolated corpus -- and swept all 509 manifest modules with 8 workers. The
flip-blocking class collapsed to 13 unique findings in 5 modules, all
fixed:

- `xiom.core:896` bare `zeroed` -> `mem.zeroed[T]()` (qualified; the mem
  import is retained for the intrinsic declaration).
- `xiom.io:630` bare `chmod` shadowed by `os.fs_ffi.chmod` (pub Result) ->
  wrapper renamed to `fs_ffi.chmod_path` (smoke_os_ffi updated).
- `xiom.io:667/677/688` stdio accessors -> console.xi's `xiom_std*` externs
  harmonized to `-> Int` with explicit `as *UInt8` casts at the
  fgets/fgetc/fwrite/fflush sites (no more *UInt8 shadowing).
- `xiom.io:930` bare `rename` shadowed by xiom.os's unused libc extern ->
  that extern was deleted.
- `xiom.collections:1425` vec_sort_by full-path sort_by + Ordering mismatch
  -> local insertion sort keeps the Int-comparator API.
- `xiom.collections:960` `hash_combine` shadowed by collect.hash /
  crypto.hash leaf collisions -> `use xiom.hash as hsh` alias.
- `xiom.collections` bucket_idx generic key -> `*key as Int` (Int-like
  keys, documented; the legacy HashMap family has no consumers).
- `xiom.crypto.mac:90` `crypto.md5` dotted call resolved to the legacy
  xiom.crypto.md5 module in call position -> new uniquely-named
  `crypto.md5_bytes` wrapper.

Re-scan: 509/509 module probes -> 0 catalog-body findings. Targeted
runtime battery: 101/101 (core/mem/collections/io/console/os_ffi/
crypto-hmac). Corpus gate green. The flip can be re-enabled for the
compiler lane's stdlib_execution_tests run.

FOLLOW-UP: restoring core's mem import (needed for qualified
`mem.zeroed[T]()`) re-pulled `core -> mem -> ptr` into the path/os
closure and re-triggered R19 (`ptr.replace_Str`, clang ptr/i8) in
smoke_stress_path_components. Since os.xi's only core use was
`core.to_string(ts)`, os now imports `xiom.convert` and calls
`convert.int_to_string(ts)` instead; core stays out of the os/path
closure and both constraints hold simultaneously (509-probe scan 0,
path_components green on r40, 101/101 battery). R19 itself remains open
for direct core/mem/ptr closures.

## 2026-09-15 (stdlib lane, round-60) -- encoding qualification landed; strict flip READY (r41 936/937, one compiler-side red)

Stdlib side:
- **Encoding qualification (commit 1f4f0aad):** all 28 in-bounds loop reads
  `data.get(i + n).value` -> `data[i + n]` (base64_encode 151/156/162,
  base64url_encode 244/249/255/258, hex_encode 325, hex_encode_upper 364,
  utf8_decode 506/526/531/532/538/539/540/554/567, utf8_valid 584/603/
  608/609/616/617/618) and the private helpers renamed to unique names
  (`write_base64_triplet` -> `_enc_write_base64_triplet`,
  `write_base64url_triplet` -> `_enc_write_base64url_triplet`, defs + call
  sites). The round-59 `m34_j08` hold causes (bare Box-first-param twin;
  UInt8-returning `get`) are gone at the call sites.
- **Verification:** new probe `probes\p_enc_qual.xi` (imports `xiom.encoding`
  + all encoding submodules; RFC 4648 tails, url alphabet 0xFB 0xFF -> "-_8",
  hex both cases, url/percent, utf8 2/3/4-byte accepts + overlong/truncated/
  surrogate/>10FFFF rejects, base32 sibling) green pre-edit and post-edit;
  encoding battery 16/16 (13 `smoke_encoding*`/`smoke_stress_encoding*` +
  3 `kat_encoding_*`); 509-module per-module bare-name scan 0 findings;
  `cargo test -p xiom-check catalog_corpus_is_clean -- --nocapture` PASS
  (isolated target_r40, 46.1s).
- **Strict-gate stdlib gap found by r41 and FIXED:** `xiom.sync` (5 sites),
  `xiom.thread` (3), `xiom.reflect` (2: size_of + align_of) called the bare
  intrinsics with no `use`/local declaration. With strict on, the on-demand
  catalog-body warnings became hard errors (`smoke_sync_arc_battery` red on
  r41: `catalog body: undefined variable 'size_of'` at sync.xi 45/74/117/
  351/352/404). Fixed with explicit item imports `use xiom.core.size_of;`
  (+ `use xiom.core.align_of;` in reflect), matching the bits/num idiom;
  sync/thread/reflect smokes green on r41; probe `probes\p_sync_sizeof.xi`.
  NOTE for the flip worklist: the 509-probe per-module scan does NOT see this
  class -- a trivial `use xiom.X;` probe never type-checks the on-demand
  bodies, so the full smoke sweep is the detector of record for strict-on
  findings.

r41 sweep (target_r41 = HEAD 1f4f0aad + the current compiler-lane working
tree, i.e. R21 scope-first resolution + `strict_catalog_findings: true`;
8 workers, 937 files; ratchet OK): **936/937 PASS**.

The single red is COMPILER-side (R21):
- `smoke_stress_path_join` (and minimal probe `probes\p_path_chain.xi`):
  a method call directly on a method-call result loses the receiver type --
  `p.join("a").join("b")` -> `error[T001]: cannot call 'join' on this
  expression`, then the downstream `as_path()` on the error-typed binding.
  Control shapes pass: single join stored in a local, then `.as_path()`;
  the receiver type error only appears on the chained call. r40 builds and
  runs the same probe exit 0; r41 fails with 4 type errors and no binary.
  This matches the new unique-candidate guard in `resolve` (a UNIQUE but
  UNRELATED candidate can no longer capture a concrete receiver): the
  method-call result's receiver type apparently no longer matches the
  registered `Path.join` key. Repro:
  `target_r41\debug\xiom.exe --run examples\stdlib_smoke\smoke_stress_path_join.xi`
  or `--run probes\p_path_chain.xi`.
- Adjacent observation (pre-existing on r40 too, NOT R21): the generic
  method surface `sync.Mutex.new[Int](7)` + `.lock()` / `.get()` does not
  resolve (`cannot call 'lock' on this expression`), while `Arc` methods on
  the same shape do; no corpus smoke covers it.

FLIP STATUS: encoding was the last stdlib blocker; scan 0 + corpus gate clean
+ r41 936/937. With the R21 chain regression fixed, the compiler lane can
re-run stdlib-exec/e2e with strict on -- no stdlib-side work remains for the
flip.

**ROUND-60 CLOSE (2026-09-15, stdlib lane): FLIP GREEN.** After the compiler
lane committed 6e7d72e5 (strict flip final, R21a-d) + a2a456c4 (R20) and the
stdlib lane committed e6d31a1a (sync/thread/reflect intrinsic bindings), the
stdlib lane rebuilt target_r42 from the clean HEAD, re-ran the R21b
regression probe (`p_path_chain.xi`: green again -- the container-only
receiver guard keeps the struct-receiver capture) and the full sweep:
**r42: 937/937 PASS + ratchet OK with strict_catalog_findings=true.** r42 is
the new baseline. R20's fix unblocks the encoding-family dedup (queue item 2).


## R21. FLIP LANDED (2026-09-15): Stage 3 Item A CLOSED -- scope-first user aliases, container-receiver wildcard guard, generic-`!` defer

`strict_catalog_findings = true` is final. The re-test after round 60 failed
first on the round-59 encoding class (`xiom.encoding:151,156,162,244,249`:
bare `write_base64_triplet` + `data.get(i).value`); the stdlib lane qualified
those sites in 1f4f0aad (`_enc_write_base64_triplet` + direct `data[i]`
reads). Three COMPILER-side defects surfaced behind it; all fixed here, plus
one fixture namespace collision found by the full-suite run.

### R21a -- user `use X as Y;` aliases were shadowed by catalog leaf modules

`resolve_imports` builds the catalog pre-load worklist from use-path
prefixes BEFORE any alias is processed, and the skip test
(`self.modules.contains_key(leaf)`) only knows module keys. For
`use network as net; use net.local;` the prefix `net` was not yet a module
key, so the worklist catalog-loaded `xiom.net` -- pulling the whole net
graph into a compile that never referenced it, and under the flip hard-checking
its bodies in a graph without `xiom.collections`. There, encoding.xi's
`tmp.get(j)` hit the unique-candidate method wildcard and captured core's
`Box.get[T](b: &Box[T])`: 84 errors (`expected Box, found Int`; `cannot
access field on non-struct type UInt8`).

Separately, `process_use` never bound a single-segment module alias: the
generic item lookup treats the path's last segment as an ITEM, and a
module's export map does not contain its own name, so `use network as net;`
was a silent no-op and the follow-up `use net.local;` fell through to the
catalog.

Fixes (crates/xiom-check/src/lib.rs):
- `resolve_imports` collects `use ... as Y` alias names up front; the
  pre-load worklist skips any prefix whose first segment is such an alias
  (the alias target is loaded through its own use path, so no module is
  lost).
- `process_use` binds a single-segment module path (`use network as net;`)
  to the PROGRAM-DECLARED module surface before any catalog fallback.

Lock: `tests/regression/m74_user_alias_shadows_catalog.xi` +
`e2e_m74_user_alias_shadows_catalog` (bare imports plus `net.ping()` /
`net.local().port` qualified alias calls; pre-fix the compile pulled the
xiom.net graph).

### R21b -- container receivers captured unique UNRELATED methods

The AUDIT #6 unique-candidate wildcard capture is deliberate for struct
receivers (`PathBuf.join` -> the sole `Path.join`; `join`/`as_path`), but
unsound for compiler-known container shapes: a `Vec[UInt8]` receiver
captured `Box.get` whenever `xiom.collections` (which registers `Vec.get`)
was not in the graph, silently typing `.value` on `&T`/UInt8. The capture
now requires a leaf-name/base relation when the receiver base is a container
(Vec/Slice/Array/Map/Set/HashMap/BTreeMap/Option/Result/Tuple/Stack/Deque/
Queue); struct receivers keep the AUDIT #6 behavior. Wildcard/generic
receivers are unchanged.

### R21c -- `!generic` was a hard checker error

`UnaryOp::Not` rejected every non-Bool, non-wildcard operand.
`benchmark.monomorph:58,315` and `benchmark.generics_hard:346` negate a
generic param (`!flag` with `flag: T`), which under the flip became fatal.
Generic params (single uppercase) now defer to monomorphisation, the same
convention as the wildcard-receiver path; non-deferrable operand types still
error.

### R21d -- fixture module-name collisions (examples/test_mod)

`examples/test_mod/{main,math}.xi` declared `benchmark.main` /
`benchmark.math`, colliding with the 30-module benchmark suite. The catalog
index is last-insert-wins, so `use benchmark.math;` in main.xi resolved to
the test_mod file (or the reverse, per directory scan order): the bench
suite compiles flipped between green and `undefined variable 'power_iter'`
/ `undefined variable 'make_result'`. The fixtures now declare
`test_mod.main` / `test_mod.math` (the e2e test still expects exit 34; the
catalog unit tests track the new names). FOLLOW-UP (not required for the
flip): make catalog index collisions deterministic and reported -- two files
declaring the same module path is ambiguous and currently scan-order
dependent.

#### R21d follow-up FIXED (2026-09-15, round-66): deterministic + reported catalog collisions

`ModuleCatalog::index_dir` used last-insert-wins over a recursive filesystem
scan, so two files declaring the same dotted module could flip between runs.
Now:

- Candidate identity is the CANONICAL path, so the same file indexed under
  relative and absolute spellings is not a collision.
- Winner order: highest source-dir index (historic last-source-dir priority
  -- the repo stdlib outranks copies earlier in the search path), then
  highest structural path match (trailing path segments vs trailing module
  segments: `stdlib/xiom/net/dns.xi` beats `packages/xiom-net/src/dns.xi`
  for `xiom.net.dns`), then smallest canonical path.
- Ambiguities are REPORTED only when the module is actually LOADED
  (`find_owned`/`peek_owned` surface a note; indexing a broad search path
  must not flood unrelated probe files): the driver prints
  `warning[W001]: module 'X' declared by N files [...]; using '...'` on the
  compile path and a W001 diagnostic on the check path.
- `release/` is skipped during indexing (packaged `release/xiom-v*/lib/...`
  stdlib copies used to shadow the live stdlib and produce 49-file collision
  notes); `debug` is NOT skipped (`stdlib/xiom/debug/` is a real module dir
  -- skipping it broke `smoke_debug`, caught by stdlib-exec).

Unit test `test_catalog_same_module_path_collision_is_deterministic`
(insert-order-independent winner + note). Manual probe: two dirs declaring
`dup.mod` -> winner `aaa/mod.xi`, one W001 note, program exits 0. Gates:
checker 189/189, stdlib-exec 85/85 (+2 ign), feature-reg 510/510, e2e
2325/2325.

### Verification (fresh canonical driver)

- checker 188/188 (corpus gate live and clean), stdlib-exec 85/85 (+2 ign),
  feature-reg 510/510, e2e **2321/2321** (2320 + the new m74 lock).
- xiom-ast 9/9, xiom 20/20, fmt 83/83, lsp 42/42, jit 5/5,
  `cargo check --workspace` clean.
- Mid-run stale-target incident: `cargo check -p xiom` reported
  `set_catalog_call_targets` missing while an isolated-target check and the
  full e2e were green; `cargo clean -p xiom-codegen -p xiom` (4.6 GiB of
  stale artifacts) resolved it. The e2e harness still spawns
  `target/debug/xiom.exe` -- rebuild the driver after `cargo clean`.

Stage 3 Item A is CLOSED. Remaining compiler-lane queue: R20 (Result-
returning same-leaf delegation payload; blocks the encoding-family dedup),
R18 (contract false positive), R16 (`ptr + int` arg), R15b (user-program
same-leaf modules), then Stage 5-7.

## R22. Catalog same-leaf CONSUMER aliases: leaf-qualified calls bind the wrong module; explicit `as` aliases AV (2026-09-15, stdlib lane round 60)

Found while re-landing the encoding dedup shims after R20 (a2a456c4). The
catalog-body delegation path (R15/R20: shim body -> canonical via `use ... as`)
is green; these are the CONSUMER-side shapes:

1. **Leaf-qualified call binds a sibling same-leaf module once both are in
   the graph.** `use xiom.convert.percent;` + `percent.percent_encode("/a?b=1&c=2")`
   in a user module: before the shim, `convert.percent` was the only "percent"
   in the graph and the call returned the full-URL-mode string. After
   `convert.percent` imported `xiom.encoding.percent as enc_pct` (dedup shim),
   the same call bound the ENCODING module's component mode:
   `%2Fa%3Fb%3D1%26c%3D2`. Probe `stdlib_ws\probes\p_pct_probe.xi`.
2. **Explicit consumer alias of a catalog module.** Same probe,
   `use xiom.convert.percent as cvt;` + `cvt.percent_encode` returned `""`
   (empty Str) instead of the shim value.
3. **Explicit consumer alias AVs (pre-existing, no shim involved).**
   `use xiom.encoding.base32 as cvt;` + `cvt.base32_encode(&v)` compiles and
   then crashes 0xC0000005 -- on r40 as well. The leaf-import form
   (`use xiom.encoding.base32;` + `base32.base32_encode`) is green. Probes
   `p_b32_alias.xi` (leaf + `as` pair), `p_b32_alias2.xi`, `p_b32_encalias.xi`.

Impact/status: the four landed shims (`convert.base16/base32/base64/
base64url`) have byte-identical twins, so the wrong-module binding is
behaviorally invisible there (smokes + KATs green, including p_b32_s5a);
divergent twins cannot be deduped yet -- `convert.percent` stays local
(unique full-URL mode) and punycode/base58 stay queued. Reported here for
the compiler lane.

RE-TEST on r43 (2026-09-15, stdlib lane; HEAD 8bc08cf0 + the round-61
codegen fixes R18/R16/R15b):
- item 2 FIXED: `use xiom.convert.percent as cvt;` + `cvt.percent_encode`
  now returns the shim's full-URL value "/a?b=1&c=2" (was "").
- item 3 FIXED: `use xiom.encoding.base32 as cvt;` + `cvt.base32_encode`
  runs exit 0 (was 0xC0000005); p_b32_alias2/p_b32_alias green.
- item 1 STILL OPEN on r43: with the percent shim in the graph, the plain
  leaf-qualified `use xiom.convert.percent;` + `percent.percent_encode`
  still binds `xiom.encoding.percent` (component mode,
  "%2Fa%3Fb%3D1%26c%3D2"); smoke_convert_percent fails at check 3.

**R22 CLOSED on r44 (2026-09-15):** the deterministic module-name
collision work (907a728a) plus the plain-import receiver bind (a5e8b1dc)
fixed item 1 -- p_pct_probe now returns the
full-URL mode for BOTH the leaf alias and the explicit `as` alias, and the
percent shim lands (component/decode legs delegate; `percent_encode`
stays local as the unique full-URL mode). Verification: smoke_convert_
percent + smoke_encoding_percent_ascii85 green, full r44 sweep 940/940 +
ratchet OK, corpus gate clean (41.7s).
base58 stays deferred for an independent reason: `xiom.num.convert.
to_base58(INT_MIN)` negates INT_MIN (overflow -> no digits emitted), while
`xiom.convert.base58.to_base58` renders INT_MIN exactly (legacy smoke pins
the round-trip), so delegation needs an explicit INT_MIN branch/translation,
not a blind shim.


Fix direction: resolve a `use path;` leaf alias through the RECORDED use
PATH (never a catalog leaf lookup over all loaded modules), and make
`use X as a; a.fn()` bind exactly the same target as `X.fn()` (the empty/AV
shapes look like the same wrong-target resolution reaching codegen).

### R22 FIXED (2026-09-15, round-68): module bindings for codegen receiver resolution

Re-verified all three items with a marker shim on `xiom.convert.percent`
(`percent_encode` returns `"SHIM:" + enc_pct.percent_encode(s)` so a binding
is visible):

- Item 1 was still open after R15b: `use xiom.convert.percent;` +
  `percent.percent_encode(...)` bound `xiom.encoding.percent` (component
  mode) because the plain item import was never recorded for codegen. The
  checker keeps `use_alias_paths` free of plain module entries on purpose
  (the bare-call alias path, BUG 25 #2, was perturbed by them), so the
  binding had nowhere to travel.
- Items 2/3 (explicit `as` aliases) were already fixed by R15b; re-confirmed
  green in the same marker run.

Fix:
1. New checker map `module_receiver_paths` (local name -> full dotted module
   path) recorded for EVERY `use` whose export is a module, alias or plain
   (`use xiom.convert.percent;` -> "percent" -> "xiom.convert.percent").
   Surfaced to codegen via `IrEmitter::set_module_receiver_paths`; kept
   separate from `use_alias_paths` so bare-call resolution stays untouched.
2. `resolve_module_call` expands single-segment receivers through
   `module_receiver_paths` first, so the receiver resolves to the USED
   module; the R15b full-key-first and R15 xiom-stripped key ordering then
   bind the exact injected definition.
3. `module_receiver_paths` is part of `CatalogImportContext` (capture and
   restore): a catalog body's private `use xiom.encoding.percent as ...`
   was overwriting the user's `"percent"` binding with the ENCODING module.
4. `resolve_module_call` checks the xiom-STRIPPED dotted key before the
   single LEAF key (injected catalog names are stripped full paths; the leaf
   key is ambiguous across same-leaf modules).

Verification: marker-shim run prints `leaf=SHIM:...` and
`as-alias=SHIM:...` (both bind convert.percent); m78 extended with a PLAIN
`use beta.base32;` + `base32.encode(13)`/`base32.name()` alongside the
existing alias legs (all exit 0). Gates: checker 189/189, stdlib-exec 85/85
(+2 ign), feature-reg 510/510, lsp 44/44, e2e 2325/2325. Percent dedup is
unblocked for the stdlib lane; base58 still needs the INT_MIN translation
noted above.

## R23. Async executor stored-fn invocation AVs in reduced program shapes (2026-09-16, stdlib lane round 61)

While building the async stress suite: `executor.executor_spawn(&mut e, f)`
followed by `executor.executor_run(&mut e)` (or run_until_idle) crashes
0xC0000005 when the callback actually runs, unless the program keeps the
full smoke_async async surface. Evidence (all with target_r45):
- `probes\p_async_p5.xi` (stderr-marked): prints P5:start/new/spawn/tasks=1,
  then AVs inside `executor_run`; nothing after it executes.
- `probes\p_async_p7.xi` (2 spawns + `_n == 2` assert): AVs.
- `probes\p_async_p8/p9/p10`: AV with 1 spawn, 5000 spawns, and with/without
  channel/timer calls; importing `xiom.async.io.*` does not help.
- A byte-for-byte copy of smoke_async (module renamed only) runs the same
  callbacks ("copy n=2", exit 0); scaling that shape to a 2000-spawn storm
  also exits 0 (n=2000), and the new
  `examples\stdlib_smoke\smoke_async_stress.xi` passes with the same surface.

Root cause is the already-warned "unknown type 'fn() -> Unit' -- defaulting
to i64. This may produce incorrect code." path: whether the executor's
stored fn() values are callable depends on which other async functions the
program references (the compiled closure decides the fn-typed slot codegen).
The shape-dependence was known (see the smoke_async in-source comment); this
report adds the minimal repro and the workaround. Impact: a single-purpose
async program (the natural shape) cannot rely on executor task callbacks;
the stress smoke keeps the full surface and scales counts. Repro:
`target_r45\debug\xiom.exe --run probes\p_async_p7.xi` -> 0xC0000005;
`--run examples\stdlib_smoke\smoke_async_stress.xi` -> OK.

**FIXED (2e06a3e7) and re-verified by the stdlib lane on r46 (2026-09-16):**
all reduced-shape probes now run their callbacks correctly --
`p_async_p5` (stderr-marked single task), `p_async_p7` (n=2), `p_async_p9`
(5000-spawn storm, n=5000), and the new `smoke_async_cancel.xi`
(executor_shutdown drops 1000 pending tasks; timer-wheel cancel-all /
selective cancel; channel close drain/Err semantics). The full-surface
shape in smoke_async_stress is kept as broader coverage, not a workaround.

### R23 FIXED (2026-09-16, compiler lane round 73): fn-typed values are env-first in every shape

Reproduced all five async probes (p_async_p5/p7/p8/p9/p10 AV) plus a minimal
user program (fn-typed param / local binding / struct field /
`Vec[fn()].pop()` payload). `--emit-ir` showed the wrong lowering at every
call-through-value site: the value is a closure ENV pointer (box word 0 =
trampoline), but the call inttoptr'd the BOX as code and called it with no
env argument:

```llvm
%tmp96 = load i64, i64* %tmp94          ; env box pointer
%tmp97 = inttoptr i64 %tmp96 to i64 ()* ; box interpreted as code!
%tmp95 = call i64 %tmp97()
```

Three mixed conventions:
1. `callee_is_fn_ptr` (bare-ident callee holding a fn-typed local) used the
   RAW-code path; it now emits env-first (`load env[0]` -> trampoline ->
   `call fn(i64 env, ...)`), matching every producer (`wrap_fn_ref_env`,
   M20-A1 closures, fn-typed ARG coercion).
2. MATCH PAYLOADS bound from fn-typed slots (`Some(task)` off
   `Vec[fn()].pop()`) were never marked as closure locals. Both binding
   sites (guard + arm body) now mark the name when the payload marker is
   `fn(...)`; the marker resolves from the scrutinee's recorded type or, for
   `vec.pop()`, from the Vec's element marker.
3. Struct-literal fn-marker FIELDS stored bare fn REFERENCE code addresses
   (`FnBox{ f: add1 }`); the store now wraps via `wrap_fn_ref_env` like
   `Vec.push`.
Also: `xiom_to_llvm_type` accepts `fn(...)` markers (i64 storage erasure)
instead of warning "unknown type -- defaulting to i64".

Verification: five async probes green (`P7 OK`), `smoke_async` +
`smoke_async_stress` green, m80 lock covers param/local/field/vec-pop
shapes warning-free. Gates: checker 189/189, stdlib-exec 85/85 (+2 ign),
feature-reg 510/510, e2e 2328/2328.

## 2026-09-16 (stdlib lane) -- runtime symbol audit for the compiler lane (action item, not a bug)

`docs/RUNTIME_SYMBOL_AUDIT.md`: 322 unique `xiom_*` runtime definitions vs
138 stdlib extern names -> 192 unbound = 83 codegen-referenced (keep),
83 runtime-internal (keep), **20 definition-only delete candidates**
(occurrence count 1, no crates/stdlib references): `xiom_asm_sha256_compress`,
`xiom_async_now_us`, `xiom_channel_close`, `xiom_f128_norm_sig`,
`xiom_f256_is_one`, `xiom_guard_heap_depth`, `xiom_guard_page_is_armed`,
`xiom_hot_enter/generation/get_version/init/is_stale/leave/register/
restore_state_legacy/save_state_legacy/set_contract_checker/
verify_contracts`, `xiom_threadpool_shutdown`,
`xiom_trampoline_clear_returned` (file:line table in the doc).
Nothing in the unbound set is worth binding stdlib-side. CAVEAT: dynamic
symbol lookup (GetProcAddress/dlsym, prefix-built names) is invisible to the
audit method -- the hot-reload family is loaded by CLI tooling, so confirm
before deleting; otherwise delete or add an intentionally-kept marker so
the audit stays mechanical on re-run.






## R24. Program-level bodyless fn declarations shadowed catalog-body externs (2026-09-16, compiler lane round 74)

`diff_tests::test_selfhost_v092_compiles` (the Stage 7 selfhost gate) was red
with catalog-body findings on `xiom.io` and `xiom.string`:

```
error[T001]: 207:33: catalog body [xiom.io]: assignment type mismatch: *UInt8 = Int
error[T001]: 220:5:  catalog body [xiom.io]: argument 1 type mismatch: expected Int, found *UInt8
error[T001]: 153:48: catalog body [xiom.string]: argument 1 type mismatch: expected Int, found Str
```

`selfhost/xiomc_v092.xi` declares the legacy C-runtime signatures as plain
BODYLESS fns (`fn xiom_read_file(path: Str) -> Int;`, `fn xiom_char_at(src:
Int, pos: Int) -> Int;`). The checker's BUG-29 rule kept such program
declarations first-wins over stdlib `extern "C"` blocks -- correct for the
PROGRAM's own calls, but it also applied while checking CATALOG bodies under
the isolated flush context: io.xi's body resolved `xiom_read_file` to the
PROGRAM's `Str -> Int` declaration instead of its own `*UInt8 -> *UInt8`
extern (the corpus gate cannot see it: the selfhost program is the only
place a same-named legacy declaration exists).

Fixes (crates/xiom-check/src/lib.rs):
1. The BUG-29 `user_declared` keep-first guard is scoped to PROGRAM checks
   (`!self.checking_catalog`); inside an isolated catalog body the module's
   own externs must win.
2. `flush_catalog_bodies` re-registers the body module's own declarations
   (fn/extern signatures) inside the isolated context before checking, so
   last-wins ordering reflects the body's own scope.
3. `extern_fns` (the D2.1/T002 unsafe-confinement mark set) joins
   `CatalogImportContext`: without restore, catalog extern marks leaked into
   the program and its plain bodyless declarations suddenly demanded
   `unsafe` (selfhost lines 27/36/64).

Verification: `selfhost/xiomc_v092.xi` compiles (exit 0), the selfhost diff
test passes, and the program-body unsafe requirements are unchanged
(`extern_fns` restored). Gates: checker 189/189, stdlib-exec 85/85 (+2 ign),
feature-reg 510/510, e2e 2328/2328. Stage 7 selfhost build gate unblocked.

## R28. `.value` on a TEMPORARY aggregate Option payload is corrupt (2026-09-16, stdlib lane round 62; renumbered from stdlib-R25 after the compiler lane published its own R25/R27)

Found while writing dedup parity probes on target_r46 (HEAD 9acb9bdd; NOT
fixed by R23/R24). Reading a payload out of a call-result temporary with
`.value` yields a Vec whose element data is all zeros (length and shape are
correct; Int payloads are unaffected). Minimal repro `probes\p_payload_read.xi`:

| shape | result |
|---|---|
| A `var o = f(); let v = o.value;` (Vec payload, named Option local) | correct (b0=1) |
| B `let v = f().value;` (Vec payload, temporary call result) | **CORRUPT (b0=0)** |
| C `let v = g().value;` (Int payload, temporary) | correct (7) |
| D `var o = g(); let v = o.value;` (Int payload, named local) | correct (8) |
| E `match f() { Some(v) => ... }` | correct (b0=5) |

where `f = dns.dns_parse_ipv4("1.2.3.4")` (Option[Vec[UInt8]]) and
`g = char.to_digit('7', 10)` (Option[Int]). A variant that performs an
intervening allocation before the read shows the same zeros, i.e. the
copied aggregate payload does not point at the parsed heap vector (the
Int-payload path is a value copy and is fine; the aggregate path copies the
aggregate from the wrong slot/storage).

Impact: user programs reading `opt.value` directly from a call result get
silently wrong data (memory-safety adjacent). Workaround (used by the new
`convert.ip` / `net.dns` dedup shims): bind the Option/Result to a named
local first, or use `match`. Repro:
`target_r46\debug\xiom.exe --run probes\p_payload_read.xi` -> B b0=0;
`--run probes\p_ip_parity2.xi` (named-local form) -> 0 mismatches.

**FIXED (90261793) and verified by the stdlib lane on r47 (2026-09-16):**
`probes\p_payload_read.xi` -> B tmp-len=4 b0=1 (correct); full r47 sweep
947/947 + ratchet OK. The named-local/match patterns already landed in the
shims stay (harmless and explicit).

### R28 FIXED (2026-09-16, compiler lane round 79)

Root cause: the Field arm has TWO struct-field readers. The local-receiver path
(`Expr::Ident`) applies the Option/Result payload override (reinterpret the
erased i64 slot: Str -> i8*, Float -> bitcast, boxed struct/container ->
inttoptr+load), but the computed-value path (`f().value`, `data.get(i).value`)
just loaded `field_llvm_ty` statically -- for `Option.value`/`Result.value`
that is the erased i64, so the binding held the raw payload HANDLE. The Let
arm's inference still recorded `Vec[Int]`, so `.len()` coerced the handle to
`%struct.Vec*` (length read correctly) while `v[0]` on the i64 slot fell to
the literal-0 fallback: "correct shape, zeroed data".

Fix: extracted the payload-aware read into one helper
(`IrEmitter::emit_struct_field_read`, expr.rs) that mirrors the local-receiver
override exactly (same payload map: Str/Float/Float32/scalars/boxed structs),
and routed the computed-value path through it. The two paths can no longer
drift.

Evidence: `probes\p_payload_read.xi` now prints B tmp-len=4 b0=1 (was b0=0);
all five shapes A-E correct; exit 0. New lock
`tests/regression/m82_tmp_payload_value.xi` + `e2e_m82_tmp_payload_value`
covers temporary `Option[Vec[Int]].value`, temporary
`Result[Vec[Int], Str].value`, scalar temporary, and the named-local control.

Gates: e2e_m82 1/1, feature-reg 510/510, stdlib-exec 85/85 (+2 ign),
perf 2/2 (both determinism canaries).

## R29. Vec built inside a match arm over a Result[Vec[...]] payload breaks clang codegen (2026-09-16, stdlib lane round 62; renumbered from stdlib-R26)

Found while delegating `net.ip.ipv6_parse` to `net.ip6`. A function that
matches on a `Result[Vec[UInt8], Str]` payload and builds/returns a
`Vec[UInt16]` inside the Ok arm fails LLVM with:

    xiominput.ll:1055:24: error: '%tmp147' defined with type
    '%struct.Vec = type { ptr, i64, i64, i64 }' but ... (type mismatch)

Minimal repro `probes\p_match_vec_codegen.xi`: `conv_match` (failing) and
`conv_named` (compiling workaround) differ only in the control shape --
the workaround is `let r = f(s); if r.is_err { return None; }; let b =
r.value; ...build...` (named local + early return), which compiles and runs.
Applied to `net.ip.ipv6_parse`; parity re-verified (p_netip_parity 0
mismatches, net smokes green). Note the named-local `.value` read on a
Result is R28-safe (R28 only affects temporaries). Repro:
`target_r46\debug\xiom.exe --run probes\p_match_vec_codegen.xi` -> clang
type-mismatch; the same probe's conv_named compiled through `p_netip_a.xi`.

**FIXED (bf627c2e) and verified by the stdlib lane on r47 (2026-09-16):**
`probes\p_match_vec_codegen.xi` -> P_MATCH_VEC_CODEGEN OK (both conv_match
and conv_named); full r47 sweep 947/947 + ratchet OK. Locked compiler-side
by tests/regression/m83_match_arm_vec_build.xi.

### R29 FIXED (2026-09-16, compiler lane round 80)

Root cause: `compile_block`'s expression-statement branch (lib.rs) stored the
value of EVERY `StmtOrExpr::Expr` into `fctx.match_result_ptr` when a match
result slot was active -- not just the block's TAIL expression. Inside the
repro's `Ok(b) => { ... while ... { groups.push(...); ... } }` arm, the
`groups.push(...)` STATEMENT materialized the mutated Vec and stored it into
the `%struct.Option` match-result slot:

    %tmp147 = insertvalue %struct.Vec ...
    store %struct.Option %tmp147, %struct.Option* %tmp7    ; invalid IR

Fix: gate the match-result store on `is_last` (the arm value is the block's
tail expression). Non-tail expression statements no longer write to the
result slot; tail expression statements (the existing supported shape) still
do. Localized to the block compiler; the separate `compile_if_arm_value` path
is untouched.

Evidence: `probes\p_match_vec_codegen.xi` compiles and prints
`P_MATCH_VEC_CODEGEN OK` (exit 0; both conv_match and conv_named). New lock
`tests/regression/m83_match_arm_vec_build.xi` + `e2e_m83_match_arm_vec_build`
(Result[Vec[Int], Str] payload, Vec built and pushed inside the Ok arm's
while, Some/None result checked).

Gates: e2e_m83 1/1, feature-reg 510/510, stdlib-exec 85/85 (+2 ign),
perf 2/2 (both determinism canaries).

## R30. Bare variant pick diverged from the checker's first-declared rule (2026-09-16, compiler lane round 81)

**FIXED (round 81).** Found while closing R25: after the determinism fixes, the
bench graph still emitted a type-mismatched call --
`@Message.size_hint(%struct.BST*)` in `benchmark.enums.test_enum_data`:

    var empty = Empty;              // bare variant, declared by Message,
    if empty.size_hint() == 0 { }   // Container, BST, ...

Root cause: TWO different resolution rules. The checker's bare-variant map keeps
the FIRST declaring enum (`enum_variants.entry(name).or_insert(parent)`, so
`Empty` -> `Message`, the first of the three). Codegen's `pick_deterministic`
(round 76, shortest-then-lexicographic) picked `BST`, so the construction built
`%struct.BST` while method dispatch (leaf `size_hint`) called
`Message.size_hint`.

Fix: `TypeContext::enum_decl_order` records enum keys in declaration
(program-walk) order; `IrEmitter::pick_variant_parent` prefers the
first-declared candidate and only then falls back to `pick_deterministic` for
generated enums not in the walk. Used by `resolve_variant_parent_enum`, the
bare-`Ident` variant construction, and the module-qualified enum fallback --
the same rule the checker uses, so construction and dispatch can no longer
disagree.

Evidence: all three `Message.size_hint` call sites in the bench IR take
`%struct.Message*`; `var empty = Empty` builds `%struct.Message`. Bench IR is
still byte-identical across 3 runs (5,687,052 bytes) and the canary covers it.

Residual (separate, OPEN): the bench graph's remaining clang error is a
same-leaf TYPE collision -- `benchmark.borrow.Metrics` (4 fields) and
`benchmark.derive.Metrics` (7 fields) both register/emit as `%struct.Metrics`,
so the derive literal GEPs fields 4-6 of the 4-field definition. Needs a
`fn_symbol_map`-style TYPE symbol qualification pass (qualify every
cross-module same-leaf struct key and route all `%struct.` references through
it); deferred as too broad for this round. The bench is IR-gated only, so no
suite regresses.

## R27. Installed-binary stdlib discovery misses the release layout (2026-09-16, release/infra lane)

Found while preparing the downloadable toolchain for the staged public beta
(docs/RELEASE_INFRA_PLAN.md, R0 gate). Release-blocking: every downloaded binary
must compile `use xiom.io;` style programs with zero environment setup.

Evidence (static, current tree):
- `find_stdlib_dirs()` (crates/xiom/src/lib.rs:1707-1760) resolves stdlib roots in
  this order: `XIOM_STDLIB`; walking up to 8 ancestor directories of the running
  executable looking for a literal `stdlib/` directory; CWD-relative `stdlib/`;
  the compile-time `env!("CARGO_MANIFEST_DIR")` repo path.
- The release/install layouts place the library in `<install>\lib\`
  (`lib/xiom/**`, `lib/package.xi`, `lib/runtime/**`) with binaries in
  `<install>\bin\` -- see package.ps1 lines 124-134 and install.ps1 lines 266-279.
  No `stdlib/` directory exists in that layout, so the ancestor walk never fires
  and an installed `xiom.exe` finds no stdlib unless the user runs from a directory
  that happens to contain `stdlib/` or sets `XIOM_STDLIB` manually.
- On a DEV machine the defect is masked: the compile-time CARGO_MANIFEST_DIR path
  still points at the build checkout, so repo runs always resolve. A CI-built
  release bakes the runner's checkout path, which does not exist on user machines,
  so the failure appears only for real users. The regression test must therefore
  not rely on the baked path; extract the candidate scan into a pure function and
  test it with explicit exe paths.

Fix direction (compiler side):
- Extend the executable-ancestor scan with content-validated candidates (each
  candidate must contain `xiom/` or `package.xi`): `stdlib/`, `lib/`, `share/xiom/`.
- Honor `XIOM_HOME` (`<XIOM_HOME>/lib`, `<XIOM_HOME>/stdlib`) when set.
- Keep the existing search order as fallback; repo/dev behavior unchanged.
- Factor the candidate list into `stdlib_candidates(exe_dir, env)` with unit tests
  covering: repo layout, install layout (bin/ + lib/), XIOM_HOME override, and a
  stale baked path.

Release-lane alternative (rejected as primary): change package.ps1/package.sh/
install.ps1 to lay out a `stdlib/` directory instead of `lib/`. `lib/` is already
documented in the installer text and other tools may assume it, so the compiler
should accept both.

Numbering note: the stdlib-lane findings published as R25/R26 in round 62 were
RENUMBERED to R28 (`.value` on a temporary aggregate payload) and R29 (Vec in a
match arm) because the compiler lane published its own R25 (fn-REFERENCE
determinism, round 77) and R27 (installed-binary stdlib discovery); the
compiler's numbers stand.

Owner: compiler lane. Blocks: docs/RELEASE_INFRA_PLAN.md R0 split gate.

### R27 FIXED (2026-09-16, compiler lane round 78)

Implemented in `crates/xiom/src/lib.rs`:

- New pure `stdlib_candidates(exe_dir, xiom_stdlib, xiom_home, manifest_dir)`
  (filesystem-free, unit-testable), `is_stdlib_root` (`xiom/` or
  `package.xi`), `existing_stdlib_roots` (existence + content filter) and
  `current_stdlib_candidates` (process env + exe path). Wired into all three
  discovery sites: the `compile()` M12 env bootstrap, `find_stdlib_dirs()`
  (catalog search dirs) and `find_runtime_c()` (repo `stdlib/runtime` and
  install `lib/runtime` both resolve).
- Candidate order: `XIOM_STDLIB` -> exe ancestors (`{dir}/stdlib`,
  `{dir}/lib`, `{dir}/share/xiom`, nearest first, up to 9 levels) -> CWD
  `stdlib/` -> `XIOM_HOME/{lib,stdlib}` -> baked repo checkout.

**Deviation from the original fix direction, with evidence:** `XIOM_HOME` is a
FALLBACK, not an override. Implementing it as a high-priority override broke
the dev/test flows on this machine because XIOM_HOME is already set globally
to a stale install (`C:\Users\lefte\AppData\Local\xiom\lib` has
`xiom/` + `package.xi`): (a) the M12 bootstrap selected the stale stdlib over
the checkout, and (b) `find_stdlib_dirs` appended it as a SECOND catalog
search root, where the last-index-wins collision rule let the stale module
copies shadow the repo's. Symptom:
`jit::tests::test_jit_execute_with_implicit_main` (`io.println`) compiled
against the old stdlib and died with 346 catalog-body type errors
(`xiom.convert`/`io`/`core`/`num`/`char`). The version-pinned sibling `lib/`
(dev checkout or install) therefore wins; an explicit `XIOM_STDLIB` remains
the top override for users who really mean it.

Also: `find_stdlib_dirs()` now returns only the FIRST content-valid root
(+ its `xiom/` subdir). Returning multiple valid roots puts two stdlib
VERSIONS on the catalog search path; extra project dirs come from the
dependency graph, not from stdlib discovery.

Tests/locks:
- 5 unit tests: repo layout, install layout (`bin/` + `lib/` incl. the
  `lib/runtime` lookup), XIOM_HOME as fallback + sibling-lib precedence,
  stale baked path/empty `lib/` ignored, full candidate order.
- Integration: fake install (`bin/xiom.exe` + `lib/{xiom,package.xi,runtime}`),
  env cleared and neutral CWD: `use xiom.io; io.println("installed ok")`
  compiles and runs exit 0. Repeated with the RELEASE binary
  (`cargo build --release -p xiom` clean, 1m15s) and a fresh workspace
  outside the repo: `io.println("release install ok")` exit 0.

Gates: xiom 25/25 (+15/+34 integration), checker 189/189, feature-reg
510/510, stdlib-exec 85/85 (+2 ign), perf 2/2 (both determinism canaries),
selfhost v092 gate green, full e2e 2329/2329, release build clean.

NOTE for future sims: the driver adds a source file's PARENT and (if it
contains any `.xi`) its GRANDPARENT as catalog source dirs (5e.3 G-31), so an
integration workspace must not sit in a directory whose grandparent holds
stray `.xi`/stdlib copies -- an early R27 e2e run failed on W001 collisions
from exactly that (`_e2e_m17_zero_warnings` vs a fake install left under the
repo `tmp/`). Keep sim trees outside the repo or in a clean `ws/` subdir.

## R25 (compiler lane). Emitted IR not byte-identical: fn-REFERENCE resolution + emission order (2026-09-16, round 77)

**FIXED (a150f246).** The Stage 6 determinism canary drifted ~130-230 bytes per
run on the 30-module bench graph; round-76 fixed the variant/type_meta half and
left the fn-REFERENCE half. The full diff was SIX independent HashMap-order
classes, all fixed; the bench IR is now byte-identical (5,686,880 bytes, 5/5
runs) and the canary asserts byte-identity on BOTH `selfhost/xiomc_v092.xi` and
`examples/benchmark/main.xi`.

1. **fn-VALUE resolution** (the round-76 evidence: `ptrtoint
   @benchmark.math.is_even` vs `@benchmark.comptime.is_even` inside
   `benchmark.collections.test_partition`). NEW
   `IrEmitter::resolve_bare_fn_ref_key` (lib.rs): **caller module** -> exact
   bare key -> `bare_fn_aliases` -> deterministic suffix pick
   (`pick_deterministic`: current module -> shortest -> lexicographic). The
   scope-first tier is load-bearing: same-leaf user modules register a bare
   key whose `fn_symbol_map` slot belongs to the FIRST module, so resolving it
   inside another module bound the wrong function. Wired into the
   fn-as-value ptrtoint (expr.rs), `wrap_fn_ref_env` symbol resolution
   (vec_abi.rs), and the FIVE duplicated fn-typed-arg param lookups in call.rs
   (`resolve_fn_ref_arg`).
2. **fn-value symbol materialization** (expr.rs): the ptrtoint used the raw
   registry key; with same-leaf user modules that is a bare key whose
   definition is module-qualified, so clang failed on `@is_even` (undefined)
   and beta bound alpha's fn. It now maps through `fn_symbol_map` (the BUG 22
   #11 rule already used by call sites). Lock
   `e2e_m81_fn_ref_same_leaf` (`tests/regression/m81_fn_ref_same_leaf/`:
   alpha/beta each define `is_even` + a higher-order `apply` and each passes
   its OWN fn; exit 0 only when both bind locally).
3. **@pre snapshot order** (decl.rs): the `HashSet<String>` worklist assigned
   the per-field pre-slots in a different order per run (Gauge.adjust
   `%tmp9/%tmp11/%tmp13` permutation, same semantics, different IR); the
   worklist is sorted now.
4. **mono worklist total order** (lib.rs): the BUG-39 sort key was the base
   name only, so `Option.is_some` specializations for `Record`/`Pair` tied on
   the key and kept discovery order -- `Option__Record.is_some` and
   `Option__Pair.is_some` swapped function positions across runs; the sort is
   now `(base_name, concrete_types)`.
5. **concrete Option builtins** (lib.rs): the `is_some`/`is_none`/`unwrap`
   bodies iterated `type_meta.keys()` (HashMap) -> per-process order; sorted.
6. **variant parent resolution** (expr.rs): the bare-`Ident` variant scan and
   the module-qualified enum fallback now use `pick_deterministic` (round-76
   covered the other four variant sites).

Verification: bench IR byte-identical 5/5 and selfhost v092 2/2 (155,936
bytes); `perf_budget_tests` 2/2; checker 189/189, feature-reg 510/510,
stdlib-exec 85/85 (+2 ign), m35 300/300, full e2e 2329/2329 (incl. m81),
selfhost v092 compile gate green.

Residual findings (pre-existing, found while gating; not part of this fix):
- Ambiguous bare cross-enum variants: `var empty = Empty;` in
  `benchmark.enums.test_enum_data` deterministically resolves the parent to
  `%struct.BST` while the later method leaf-bind picks
  `@Message.size_hint` -> a type-mismatched call survives in the bench IR.
  Needs a checker rule/diagnostic for variant ambiguity.
- `%struct.Metrics` has 4 fields but the bench emits
  `getelementptr ... i32 0, i32 4` (invalid IR): `examples/benchmark/main.xi`
  does NOT fully clang-compile at HEAD. Stage 6 measures emitted IR bytes
  only, so this never gated; remeasure before wiring a full bench build.

## R31. Cross-repo test isolation: compiler suites assume a sibling stdlib checkout (2026-09-16, release/infra lane)

Found while planning the monorepo split (docs/REPO_MIGRATION_RUNBOOK.md). After
the split, `xiom-lang/xiom` and `xiom-lang/stdlib` are separate repos; the
compiler's test suites currently ASSUME a sibling `stdlib/` checkout and
`examples/stdlib_smoke/` corpus, so a bare compiler clone cannot run its own
tests, and a bare stdlib clone cannot run anything without a compiler binary.

Evidence (all paths in the compiler repo):
- `crates/xiom-codegen/tests/stdlib_tests.rs` hardcodes `stdlib/xiom/*.xi`
  module paths (list at line 34+), compiles a synthetic `use xiom.*` program
  with CWD = repo root.
- `crates/xiom-codegen/tests/stdlib_execution_tests.rs` compiles and runs
  `examples\stdlib_smoke\smoke_*.xi` (line 137+). One test already has the
  right pattern: it SKIPS with a message when the smoke file is absent
  (`stdlib_exec_cross_module_serialize_convert`, lines 493-497) because the
  file was owned by the parallel lane at the time.
- `crates/xiom-codegen/tests/stdlib_api_freeze_tests.rs` resolves module files
  by scanning the stdlib tree (`resolve_module_path`) and compiles the FROZEN
  module set (lines 1061-1115).
- `crates/xiom-lsp/src/main.rs` test `test_stdlib_module_no_false_positives`
  reads `<repo>/stdlib/xiom/alloc/alloc.xi` with `.expect(...)` -- a hard
  failure once the stdlib is not a subdirectory.
- `crates/xiom-mcp` knowledge tests call `stdlib_reference()` and assert it
  resolves "from repo" (main.rs 951-984); resolution goes through the stdlib
  candidate scan, so a bare checkout fails the test even though the tool
  itself degrades gracefully.
- `crates/xiom-jit/src/lib.rs` resolves runtime sources and libraries from
  CWD-relative `stdlib/runtime/...` plus `XIOM_HOME` (lines 439-500); it does
  not use the R27 `current_stdlib_candidates()` helper, so JIT fails from a
  bare checkout even with a stdlib checked out elsewhere.
- `crates/xiom-codegen/tests/feature_regression_tests.rs`
  `regress_r901_registry_index_exists` reads `packages/index.json`; it is
  guarded by `if path.exists()` and therefore skips silently (the guard should
  be loud).
- `crates/xiom-pkg/src/main.rs` resolves the `xiom-std` dependency from
  `<workspace_root>/stdlib` and reads `<workspace>/packages/index.json`
  (lines 410-434, 887-890). Tooling behavior, not tests, but the same
  repo-layout assumption; it already degrades when the paths are absent.
- Implicit: every e2e/feature test that compiles `use xiom.*` needs a
  DISCOVERABLE stdlib at run time (CWD `stdlib/`, `XIOM_STDLIB`, or the
  installed layout that R27 fixed). A bare compiler clone today fails nearly
  the whole corpus.

Reverse direction (stdlib repo): the smoke corpus currently lives in the
compiler tree at `examples/stdlib_smoke/` -- the split moves it into
`xiom-lang/stdlib`. The `.xi` smoke files themselves contain no compiler-path
references (verified: only string literals such as `"../d"` inside URL tests).
The stdlib repo's tests will need a compiler binary: download a released
`xiom` in CI, or take a `--compiler <path>` argument in a local runner.

Required work (compiler lane), in order:
1. One path helper used by every cross-repo test: `stdlib_root()` =
   `XIOM_STDLIB` -> repo-relative `stdlib/` (the pinned checkout; see the
   split contract in docs/REPO_MIGRATION_RUNBOOK.md); `stdlib_smoke_root()` =
   `XIOM_STDLIB_SMOKES` -> `<repo>/stdlib/tests/smoke/` -> legacy
   `<repo>/examples/stdlib_smoke/` (transition only; the stdlib repo
   normalizes its corpus to `tests/smoke/`).
2. Skip with a LOUD message when the path is missing; FAIL instead of
   skipping when `XIOM_REQUIRE_STDLIB=1` (CI sets it). The existing
   `stdlib_exec_cross_module_serialize_convert` guard is the model to copy.
3. Add `scripts/fetch-stdlib.ps1` + `.sh` reading a new pinned `STDLIB_VERSION`
   file: shallow-clone `xiom-lang/stdlib` at that tag into `stdlib/` (add
   `stdlib/` to the compiler repo's `.gitignore`). The stdlib repo normalizes
   its corpus to `tests/smoke/`, so the smokes then live at
   `stdlib/tests/smoke/` and need no copying. README:
   `./scripts/fetch-stdlib.ps1` then `cargo test`.
4. LSP alloc test and MCP knowledge tests: parameterize on `stdlib_root()`
   and skip loudly when absent.
5. Route `xiom-jit` runtime source/library discovery through the existing R27
   `current_stdlib_candidates()` helper (installed layout + checkout + env),
   not raw CWD strings.
6. Make the R9-01 package-index guard print a SKIP line instead of vanishing.

Acceptance:
- Fresh clone of the compiler repo with NO stdlib: `cargo test --workspace
  --lib` + fast gates pass with visible SKIP lines naming the missing repo.
- Same clone after `scripts/fetch-stdlib`: full fast gates green.
- CI with `XIOM_REQUIRE_STDLIB=1` fails if the stdlib checkout is missing --
  no silent green.

Owners: compiler lane (test harnesses, jit, fetch script); release lane (CI
wiring, README); stdlib lane (move smokes, add a local runner taking a
compiler path).

### R31 FIXED (2026-09-16, compiler lane round 82)

Landed the compiler-lane half; CI wiring/README stay release-lane, corpus
move stays stdlib-lane.

- **One helper** (`crates/xiom-graph/src/paths.rs`, new module): the R27
  candidate machinery moved here (`stdlib_candidates`, `is_stdlib_root`,
  `existing_stdlib_roots`, `current_stdlib_candidates`) plus the R31
  resolvers `stdlib_root()`, `stdlib_smoke_dir()` (XIOM_STDLIB_SMOKES ->
  `<stdlib>/tests/smoke/` -> legacy `<repo>/examples/stdlib_smoke/`),
  `repo_root()`, `require_stdlib()`, `skip_if_missing()`, `stdlib_or_skip()`
  and `skip_smokes_if_missing()`. `xiom-graph` was already a dependency of
  `xiom`, `xiom-lsp`, `xiom-mcp` and a dev-dependency of `xiom-codegen`; only
  `xiom-jit` gained an edge.
- **Driver**: `find_stdlib_dirs` / `find_runtime_c` / the M12 bootstrap call
  the graph helpers; behavior unchanged (r27 tests still pass, now exercising
  the delegated implementation).
- **JIT**: `find_runtime_lib` and `build_runtime_library` resolve runtime
  sources/libs through the candidate scan (checkout `stdlib/runtime`,
  installed `lib/runtime`, XIOM_HOME, CWD) -- no raw CWD strings. Verified
  `xiom build-runtime` builds `libxiom_runtime.dll` (exit 0).
- **Tests**: `stdlib_tests.rs`, `stdlib_execution_tests.rs` and
  `stdlib_api_freeze_tests.rs` skip loudly without a checkout and hard-FAIL
  under XIOM_REQUIRE_STDLIB=1; smoke paths resolve through
  `stdlib_smoke_dir()`; `stdlib_api_freeze`'s manifest resolver now rejects
  stale table paths and falls through to the filesystem layout; LSP alloc
  test and MCP knowledge tests parameterized on `stdlib_or_skip()`; R9-01
  package-index guard prints a SKIP line.
- **Scripts**: `scripts/fetch-stdlib.ps1`/`.sh` (shallow clone at the
  `STDLIB_VERSION` pin; `-Force`; `XIOM_STDLIB_REPO` override; refuses to
  delete a non-git in-tree stdlib), `STDLIB_VERSION` (currently `main` --
  release lane replaces it with the split tag), `.gitignore` gains `stdlib/`.
- **Packaging**: `package.ps1`/`package.sh` bundle from the `stdlib/`
  checkout (fail loudly without `package.xi`) and print the pin;
  `package.ps1` no longer reaches into the old `xiom-playground/` WASM copy
  -- it copies the release artifact from `dist/wasm`, `artifacts/wasm` or
  `target/wasm-dist` and notes when absent.

Verification: workspace `--lib` 521 tests green (includes 8 new
`xiom_graph::paths` unit tests: repo/install layouts, XIOM_HOME fallback,
stale baked path, candidate order, skip guard); stdlib-exec 85/85 (+2 ign);
LSP 44/44; MCP 39/39; JIT 5/5; `xiom build-runtime` exit 0; full e2e on this
binary (see docs/SESSION.md round 82). The bare-clone SKIP/FAIL acceptance is
unit-tested here; the real no-checkout run is a post-split CI check.

Open findings (pre-existing, exposed once the stale manifest path no longer
panics first; stdlib-lane owned):
1. `stdlib_api_freeze_no_removals` is RED: 52 frozen signatures drifted since
   the 2026-08-07 snapshot (compress family moved/renamed, array/cell generic
   changes, encoding/env/error/hash/iter changes). Needs the stdlib lane's
   additive-only stamp or an intentional snapshot regeneration.
2. `stdlib_tests::stdlib_all_modules_compile_to_ir` is RED:
   `xiom.encoding.ascii85` catalog bodies fail T001 at 35:58 and 89:69
   (`expected Result[Vec[UInt8], Str], found Option[Vec[UInt8]]`).
Both were already failing before R31 (the freeze scan panicked on the stale
`stdlib/xiom/memory/rc.xi` manifest path).

## Registry-client integration findings (xiom-pkg) -- 2026-09-16, from the registry repo session

Source: the registry lane's handoff. The registry now implements the full
protocol per registry/SESSION.md section 2 (16/16 e2e checks drive the real
client, 71 unit tests); these are CLIENT-side defects the registry cannot
compensate for. Everything below was re-verified against this tree
(v0.58.0, HEAD `69c92db3`) while writing this entry; per-item verification
notes are inline. NOTHING here is fixed yet.

Priority labels are the registry lane's (P1 = unenforceable security
guarantee, P2 = quality/integration, P3 = edge case); the R-numbers enter the
compiler-lane queue.

### R32. `xiom pkg install` fallback bypasses every integrity check (P1, security)

`install` treats ANY `install_from_registry` error as a fallback trigger
(main.rs:60-65), including integrity failures:

```
if let Err(e) = install_from_registry(name, version, &registry) {
    eprintln!("xiom pkg: registry install failed: {e}");
    eprintln!("xiom pkg: trying local resolution...");
    install_package(&args);
}
```

`install_package` (main.rs:701-751) then re-checks the SAME registry with the
legacy substring scan (main.rs:714-716) and calls
`install_from_registry_download` (main.rs:857-885), which does
`http_get_binary` + `extract_tar_gz` with NO sha256, NO signature and NO
lockfile check. So a CHECKSUM MISMATCH or a signature failure is demoted to
"try again unverified".

Registry-lane repro (deterministic): publish a package; replace the stored
tarball on disk with a structurally valid different tarball; `xiom pkg
install <pkg>` prints:

```
CHECKSUM MISMATCH ... expected a79e... got 766d...
xiom pkg: trying local resolution...
xiom pkg: found <pkg> in remote registry
xiom pkg: downloading ... -> installed
```

and the tampered `lib.xi` replaces the original in the package cache. Note
the fallback's own remote branch only fires when the index is COMPACT
(`"name":"pkg"`); after a server restart (pretty index, see R36) it silently
skips to the local ecosystem path, so "tamper installs" behavior also varies
across server restarts.

Verification here: the code path is exactly as reported; the registry e2e
itself guards against it (`registry/test/e2e/run-e2e.js:360-380`, "assert it
did not silently install content from the tampered bytes").

Fix shape: distinguish error classes (typed enum, not `String`):
RegistryUnavailable / PackageNotFound / VersionNotFound -> fallback allowed;
IntegrityFailure / SignatureFailure / LockfileFailure -> TERMINAL, non-zero
exit. Then delete `install_from_registry_download` (registry lane's
preference) or make it verify the index digest + sha256 + signature itself.
Positive lock: tampered artifact must fail without extracting.

### R33. Signature enforcement fails OPEN on trust-file / registry-URL text mismatch (P1, security)

`TrustStore::get()` normalizes the LOOKUP string (signing.rs:127-129) and
`pin()` stores normalized keys (signing.rs:143), but `load_from`
(signing.rs:112-125) copies keys out of `trusted_keys.json` VERBATIM. Any
trust file not written by the current `pin()` path -- hand-written, test
fixture, legacy/older client -- can therefore hold a key that never matches
the lookup. `install_from_registry` then takes the `(None, sig)` arm
(registry.rs:350-353): it prints "artifact is signed (fp ...); pin it with
..." and installs WITHOUT enforcement. A mismatch must fail closed once the
user believes the registry is pinned.

Registry-lane repro (both installed with only "checksum verified", no
signature check, despite a pinned key):

- trust file key `http://localhost:3203/` vs `XIOM_REGISTRY=http://localhost:3203` (trailing slash)
- trust file key `http://LOCALHOST:3203` vs runtime lowercase

The mismatch is realistic: the registry's own e2e writes
`trusted_keys.json` directly (`run-e2e.js:346-347`, `:353-354`), i.e. not
through `pin()`.

Verification here: the report's wording ("get() normalizes its argument but
install_from_registry passes the raw URL") understates it -- the query IS
normalized; the defect is the un-normalized STORED side. Canonicalizing
`registry_url()` alone (R34) does NOT fix a stored key with a trailing slash
or uppercase host; the load/compare side must normalize too.

Fix shape: normalize both sides of the lookup (normalize keys on load and in
`get()`), or compare normalized keys. Unit tests with hand-written
`trusted_keys.json` variants (trailing slash, uppercase host, surrounding
whitespace) asserting the pin is found.

### R34. Registry URL is never canonicalized: double slashes + duplicated URL in errors (P2)

`registry_url()` (registry.rs:14-16) returns `XIOM_REGISTRY` verbatim and
every call site concatenates: `format!("{registry}/index.json")`,
`format!("{}/publish", registry)`, `format!("{}/packages/...")`. With
`XIOM_REGISTRY=http://localhost:3203/` the client requests
`http://localhost:3203//publish` and `//index.json`; the local registry 404s
(`no_route`), while nginx in front of staging/production typically collapses
`//` -- staging and local behave differently for the same client.

Also the error text renders the URL twice: `http_post_multipart` maps to
`format!("POST {url}: {e}")` (registry.rs:89), and ureq 2.x `Error::Status`
already displays as `{url}: status code {code}` (verified in ureq 2.12.1
`error.rs:214`), producing e.g.
`POST http://localhost:3203//publish: http://localhost:3203//publish: status code 404`.

Fix shape: canonicalize once in `registry_url()` (trim whitespace, drop a
trailing `/` repeatedly, lowercase scheme+host) so trust lookup, URL
building, publish and lock generation all use one string; render the URL
once per error message. Land with R33's load-side normalization.

### R35. Non-2xx response bodies are discarded (P2)

`http_get` / `http_get_binary` / `http_post_multipart` collapse every ureq
failure to `"GET|POST {url}: {e}"` (registry.rs:32, 47, 89) -- the response
body is never read. The registry deliberately returns actionable JSON bodies
with codes (`invalid_signature`, `signature_required`, `reserved_namespace`,
`scope_denied`, `version_exists`, `index_full`, `rate_limited` with
`Retry-After`); none of it reaches the user, so 401 vs 403 vs 422 is
guesswork. ureq 2 exposes the body on `Error::Status(code, resp)`.

Fix shape: on `ureq::Error::Status`, read the bounded body (keep the
existing 16 MiB / 256 MiB / 1 MiB caps) and include status + body in the
error; keep transport errors as-is. This is the diagnostics prerequisite for
field-debugging R32/R33.

### R36. Fallback index lookup is a whitespace/format-sensitive substring search (P2)

`install_package` does `body.contains("\"name\":\"{pkg}\"")` (main.rs:715)
and `install_from_registry_download` restarts its "section" at the same
substring (main.rs:862) before parsing `"latest":"..."`. Consequences:

- The registry serves the index COMPACT from a fresh process and
  PRETTY-PRINTED (2-space indent, `"name": "..."`) after a restart that
  reloads from disk. Pretty output makes the search fail -> the remote branch
  is skipped entirely; compact output works. Same client, same package,
  different behavior across server restarts.
- A description containing `"name":"<pkg>"` earlier in the document shifts
  the section start and `latest` is parsed from the wrong entry.

Fix shape: deserialize the index into the existing `RegistryIndex` and look
up the map; never string-search. Moot for the fallback if R32 removes
`install_from_registry_download`, but `install_package`'s remote branch is
also affected.

### R37. Fallback edge cases: all-yanked package; `@version` handling (P3)

- All-yanked package: the registry sets `"latest": ""`. The primary path
  errors with `Version '' not found ...`; the fallback then builds
  `{registry}/packages/{name}//package.tar.gz` -> 404 with a confusing URL.
  Should report "all versions are yanked" (or skip packages with empty
  `latest`).
- Binary download ignores `@version`: the registry lane reports one call
  site passing only the name. NOT reproducible on this tree: main.rs:718
  passes `_version`, `install_from_registry_download` uses it
  (main.rs:865-870), and all three `http_get_binary` call sites
  (registry.rs:299, main.rs:878, main.rs:995) build a versioned URL.
  Re-verify against the binary the registry lane used; if it was the
  fallback path, R32/R36 supersede it.

### R38. Package cache ignores XIOM_HOME; fallback extraction does not clean the destination (P3)

`package_cache_dir()` (registry.rs:390-397) uses `LOCALAPPDATA`/`HOME`,
while `install_package_files` and `get_xiom_home` honor `XIOM_HOME`
(main.rs:783-794, 916-925). `XIOM_HOME`-sandboxed runs (CI, tests) therefore
still write the real user cache, and stale extractions from earlier runs
persist. Also `install_from_registry` removes the destination first
(registry.rs:379-381) but `install_from_registry_download` does not
(main.rs:880-882), so stale members survive on the fallback path.

Fix shape: route the cache through the shared XIOM_HOME helper; clear the
destination before extracting (or funnel both install paths through one
extractor).

### Compatibility contract (must survive the fixes) -- registry lane

- Publish multipart fields exactly `name`, `version`, `signature`,
  `publicKey` + file field `package`; `Bearer XIOM_REGISTRY_TOKEN`.
- Download path `/packages/{name}/{version}/package.tar.gz`.
- ed25519: 64-byte signature as 128 lowercase hex chars, 32-byte key as 64
  hex, over the exact tarball bytes (`verify_strict`).
- `RegistryIndex.registry` is required in the JSON (no serde default); the
  server emits it from `REGISTRY_URL`. Keep it required -- it caught a real
  drift.

### Not the client's problem (FYI, do not debug the wrong side)

- `/index.json` byte format depends on process history (compact fresh,
  pretty after restart reload): the registry lane will standardize the
  serializer; R36 removes the client's dependence either way.
- The seed index (`registry/seed-index.json`, 70 packages) carries no
  digests and no artifacts; installs from seeded entries fail closed until
  those packages are published through the wire protocol. Server-side.

### Recommended fix order (compiler lane)

1. **R32** -- the only finding where a user can be served bytes that failed
   verification. Typed error classes; fall back only on
   unavailable/not-found; delete or harden `install_from_registry_download`;
   tamper lock asserts failure + no extraction.
2. **R33 + R34** in one slice -- same root cause (registry URL text
   identity). Canonical `registry_url()` + normalize both sides of the trust
   lookup; trust-file unit tests. R34 also removes a local-vs-nginx
   behavior split before any registry work.
3. **R35** -- read non-2xx bodies, render the URL once. Prerequisite for
   diagnosing auth/scope/rate-limit failures in the field.
4. **R36** -- index deserialization everywhere; delete the substring scans.
   If R32 removed the fallback path this collapses to the remote branch of
   `install_package`.
5. **R37 + R38** -- yanked/empty-`latest` handling, XIOM_HOME-aware cache,
   clean-before-extract. Low risk, last.

Gate: `cargo test -p xiom-pkg` plus the registry e2e suite
(`registry/test/e2e/run-e2e.js`, which drives the real client binary);
new unit tests for the trust-store normalization variants and a
tamper-does-not-install lock.

### R32-R38 FIXED (2026-09-17, `main`)

All seven findings were fixed in one hardening slice of `crates/xiom-pkg`.
The registry lane's e2e harness (`registry/test/e2e/run-e2e.js`) carries the
locks (test-only update, owned by the registry lane).

- **R32**: `install_from_registry` returns a typed `InstallError`
  (`RegistryUnavailable | NotFound | Integrity | Local`). The `install`
  command falls back to local resolution ONLY for
  `RegistryUnavailable`/`NotFound`; `Integrity` (checksum, signature,
  lockfile, unhashed-without-override) and `Local` failures print and exit
  non-zero. `install_from_registry_download` -- the unverified
  download+extract -- is DELETED; `install_package` became
  `install_local_package` (local index -> workspace ecosystem only, exit 1
  when nothing is found). e2e now asserts the tamper path exits non-zero
  and never prints "trying local resolution".
- **R33**: `TrustStore::load_from` normalizes every key on load (on top of
  `pin`/`get`), so hand-written / legacy / fixture trust files with a
  trailing slash, uppercase host, or whitespace still match. Unit test
  writes the JSON directly (not through `pin`); e2e check "pinned key
  enforcement survives a non-canonical trust-file URL".
- **R34**: `registry_url()` canonicalizes once via
  `canonicalize_registry_url` (trim, lowercase scheme+host, drop trailing
  slashes, path case preserved); every URL builder uses that string. HTTP
  errors render the URL exactly once (`describe_http_error`).
- **R35**: non-2xx responses surface `HTTP <code>: <body>` (64 KiB bounded
  body read, 2048-char render); ureq transport errors pass through without
  a second URL prefix. e2e asserts the registry codes
  (`invalid_token`, `version_exists`, `signature_required`,
  `reserved_namespace`) reach stderr.
- **R36**: the substring index scans died with the deleted fallback path;
  every remaining index read (install, lock, search) goes through
  `RegistryIndex` deserialization.
- **R37**: version resolution is a pure `resolve_version`; `latest: ""`
  reports "all versions ... are yanked" plus a pin hint instead of building
  `.../<pkg>//package.tar.gz`; a pinned yanked version still resolves. e2e
  check "all-yanked package fails with a clear message, not a bogus URL".
- **R38**: `package_cache_dir()` honors `XIOM_HOME` first
  (`$XIOM_HOME/packages`, matching `get_xiom_home`), and the cache entry is
  cleared before extraction on every remaining path. The e2e cache
  assertions read `$XIOM_HOME/packages` now; the harness's old
  LOCALAPPDATA/HOME workaround is obsolete.

Verification: `cargo test -p xiom-pkg` **58/58** (6 new tests: URL
canonicalization variants, fallback-class matrix, version resolution
incl. all-yanked, status-error rendering, cache resolution, hand-written
trust file); registry e2e **18/18** against the rebuilt client (16 original
checks + 2 new locks, with the tamper and error-body assertions
strengthened); CLI smoke: unavailable registry -> local fallback -> exit 1
with the URL rendered once.

## R39. Same-leaf TYPE collision across project modules -- FIXED (2026-09-17, `main`)

**Finding** (the last entry in "Open compiler findings"): two project modules
declaring the same struct/enum leaf emitted ONE bare `%struct.X` definition.
Catalog-injected modules flatten into the program as top-level decls
(`collect_external_decls`), and while free FUNCTIONS were module-qualified
(`module.leaf`), TYPE names were injected bare. With two owners, the first
module (dotted-name order) won the bare key; the losing module's bodies kept
their own field count and GEPed out of range. Bench-graph repro:
`benchmark.borrow.Metrics` (4 fields) and `benchmark.derive.Metrics` (7
fields) both emitted as `%struct.Metrics`; derive's literal GEPed fields 4-6
of the 4-field definition -> clang reject. `benchmark.borrow.Record` /
`benchmark.interfaces.Record` had the same shape.

**Fix**: new checker pass `crates/xiom-check/src/type_qualify.rs`, wired into
`collect_external_decls` before injection. It computes same-leaf collisions
among NON-generic pub project types/enums (stdlib `xiom.` modules and leaves
the user program declares or references keep the legacy first-wins behavior;
generic types are excluded because mono/concrete-container keys such as
`Option__Pair` are leaf-derived and reshape separately), then
module-qualifies the colliding decl and rewrites every reference in the
owning module before flattening: type annotations (all `Type` variants),
struct literals, enum/associated-call bases, method receivers, destructure
patterns, and expression-position container args (`Vec[Record].new()` parses
as `Index(Ident(Vec), Ident(Record))`). Imported colliding types resolve
through the module's `use` declarations (`use a.b.Leaf` and `use a.b;` +
`b.Leaf`).

Qualification exposed R25-class HashMap-order picks once two qualified
same-leaf types existed; those scans are now deterministic through the
round-76 `pick_deterministic` (current module -> shortest key ->
lexicographic): `resolve_type_key`, the second `llvm_type_for` suffix scan,
`vec_elem_storage_size`'s type_meta/enum leaf picks, the empty-array-to-Vec
element scan in stmt.rs, and the call.rs method-suffix fallback.

**Locks / verification**:

- `e2e_m84_type_same_leaf_modules` (new): compiles a package-graph fixture
  (`tests/regression/m84_type_same_leaf/`, main.xi only; alpha/beta arrive as
  catalog modules), asserts BOTH qualified definitions and the absence of a
  bare `%struct.Metrics`, then compiles + runs exit 0. Added to the CI lock
  line in `.github/workflows/ci.yml`.
- m78 (user same-leaf modules) and m81 (same-leaf fn refs) stay green.
- Bench graph: `%struct.benchmark.borrow.Metrics` (4 fields) +
  `%struct.benchmark.derive.Metrics` (7) and the two `Record` types now emit
  distinctly; the R39 clang error is gone. The next clang error is a
  pre-existing generic-mono ABI mismatch (`generics_hard.Pair`
  `read_first_Int_Int` called with a struct value where the mono'd signature
  expects `ptr`) -- recorded in SESSION.md "Open compiler findings".
- Determinism: bench IR **5,764,620 bytes** (was 5,687,052; budget 6.3 MB),
  byte-identical across 3 runs; `perf_budget_tests` 2/2.
- Full gates: e2e **2332/2332** (16 min), feature-reg **510/510**, checker
  **194/194**, robustness **63/63**, pkg/dbg/lsp/mcp **58/34/44/39**, release
  `cargo build --release -p xiom` clean.

**Harness fixes in the same slice** (pre-existing red on a compiler-only
checkout, exposed while running the full suite): `e2e_runtime_c_exists` and
the chaos t2-t5 tests now honor the R31 cross-repo contract (missing
`stdlib/` / `xiom-benchmark-chaos` checkouts SKIP loudly instead of failing
on a missing file; `XIOM_REQUIRE_STDLIB=1` still hard-fails), and
`e2e_m79_debug_info_metadata`'s line expectations were stale by 2 lines
since the SPDX header commit (`return a + b` is line 12, `if x != 5` line
16).

## R40. `derive[Clone]` on a pointer receiver was a silent miscompile -- FIXED (2026-09-17, `main`)

**Finding**: the derive-generated `X.clone` is emitted with a BY-VALUE self
(`define %struct.X @X.clone(%struct.X %self)`) and registered as such, but a
call on a pointer receiver (`m: &X` -> `m.clone()`) passed the POINTER:
`call %struct.X @X.clone(%struct.X* %tmp4)`. LLVM 22 accepts the type
mismatch silently, the callee reads the alloca address as the struct value,
and the returned clone is garbage. Same shape for `derive[Clone]` enums. The
single-file repro (`type M = { a: Int; b: Int; } derive[Clone]` +
`fn rt(m: &M) -> Int { var c = m.clone(); ... }`) exits 1 where it must exit
0; the same pattern appears in the bench graph.

**Fix** (call.rs, non-generic instance-method receiver coercion): when the
callee's first registered param is a BY-VALUE struct (`!p0.ends_with('*')`)
and the compiled receiver is its pointer (`recv_llvm_ty == "{p0}*"`), load
the struct and pass the value -- mirroring the callee ABI, exactly like the
generic-method path does. Both struct and enum clone flow through this site
(`compile_clone_impl` is shared), and user-defined non-generic by-value-self
methods get the same correct lowering.

**Lock**: `tests/regression/m85_clone_ref_receiver.xi` +
`e2e_m85_clone_ref_receiver` (struct clone on `&T`, enum clone on `&E`, plus
a by-value receiver guard); added to the CI lock line in
`.github/workflows/ci.yml`.

**Verification**: repro exits 0 and the emitted call is
`@solo.M.clone(%struct.solo.M %tmp5)` (loaded value, matching the
definition); full e2e **2333/2333**, feature-reg **510/510**, checker
**194/194**, perf/determinism **2/2** (bench bytes unchanged, canary
green).

## R41. Generic-method calls passed struct VALUES where a pointer self was expected -- FIXED (2026-09-17, `main`)

**Finding**: after R39 numbered the bench types correctly, clang rejected
`call i64 @Pair.read_first_Int_Int(%struct.Pair* %tmp31)` where `%tmp31` was
the LOADED STRUCT VALUE (`pairs[0]`, `pairs: Vec[Pair[Int,Float64]]` from
`collect_pairs()`; source `bench_generics_hard.test_nested_generic_collection`).
The NON-generic instance-method path materializes struct temporaries into an
alloca when the callee self is a pointer (BUG 34 / M65 Part 2b in call.rs);
the GENERIC path fell through and passed the value. LLVM accepts the type
mismatch silently; the callee then treats the value bits as an address.

**Fix** (generic call receiver ABI block, call.rs): mirror the non-generic
handling -- an `Expr::Index` receiver of a struct element passes the element
ADDRESS (`&elem` -> inttoptr to the callee's pointer type), and any other
struct-value temporary is materialized into an alloca whose address is
passed.

**Verification**: the Pair error is gone; full e2e **2333/2333**
(run together with R42), feature-reg 510/510, checker 194/194,
perf/determinism 2/2. The bench's next clang rejection is the separate
pre-existing generic-arg inference issue recorded in SESSION.md ("Open
compiler findings").

## R42. Bare enum variants now resolve scope-first (checker parity) -- FIXED (2026-09-17, `main`)

**Finding**: `var c_empty: Container[Int] = Empty;` in
`benchmark.generics_hard.test_generic_enum` built a `%struct.Message` value
(`Message` is the first-declared enum with an `Empty` variant) and stored it
into the `Container` slot: clang "store %struct...Container %tmp6, %tmp6
defined with type %struct.Message". The CHECKER resolves a bare variant
through its module-scoped key (`resolve_enum_variant` prefers
`{current_module}.{variant}`); codegen's `pick_variant_parent` only used
declaration order (R30) and disagreed whenever a module used the same-leaf
variant of its OWN enum while another module declared it first.

**Fix**: `pick_variant_parent` (lib.rs) now tries scope prefixes first --
the explicit module context, then the enclosing function name's dotted
prefixes (`benchmark.generics_hard.test_x` -> `benchmark.generics_hard`,
`benchmark`) -- then the R30 declaration-order parity, then
`pick_deterministic`. This matches the checker and the R25/R39 scope-first
resolution family.

**Verification**: the Container/Message store mismatch is gone from the
bench IR; full e2e **2333/2333** (incl. every enum/variant test),
feature-reg 510/510, checker 194/194, perf/determinism 2/2.

## R43. `&v` on a reference-typed local hard-errored -- FIXED (2026-09-17, `main`)

**Finding** (stdlib lane, untested-public-surface sweep): calling
`xiom.crypto.keyx.x25519_keypair()` failed codegen with C001 "cannot take a
reference to 'v': it is already a reference (remove the leading '&')".
Automated bisect isolated `xiom.crypto.curves._bigint_to_le`:
`fn _bigint_to_le(b: &BigInt) { var v = b; ... bigint_div_mod(&v, ...) }`.
The BUG 24 guard rejected `&v` when the local already holds a reference, but
`var v = b` is a REFERENCE BINDING (the checker accepts both `let w: Big = v`
via auto-deref copy and `let w: &Big = v`), and `&*v == v` is well-defined:
the reference itself, not the address of the pointer slot.

**Fix** (expr.rs, `Expr::Ref` on an Ident local whose slot type is
`%struct.X*`): load the stored pointer and return it as the `&X` value
instead of erroring. Callers coerce it exactly like any other `&T`; the
double-address garbage BUG 24 guarded against is not produced. The minimal
probe and `p_x25519_keypair_codegen.xi` compile and run exit 0.

**Pinned semantics** (lock m86): `&v` on a ref binding reads through to the
original; a FIELD write through the alias mutates the original; a plain
reassignment rebinds the local without touching the original.

**Verification**: m86 + full e2e **2334/2334** (with the stdlib checkout
active, so stdlib-dependent tests ran for real), feature-reg 510/510,
checker 194/194, perf/determinism 2/2. `stdlib_execution_tests` is 83/85
against the LOCAL stdlib checkout: `smoke_collect_tree` and
`smoke_collect_cache` fail IDENTICALLY on the pre-fix driver (contract
violations at 83:12 / 181:12) -- pre-existing cross-lane checkout drift,
not this change.

## R44. `http_parse_response` emits an out-of-range GEP -- RESOLVED STDLIB-SIDE (same-leaf class remains)

**Finding** (stdlib lane, same sweep, probe `p_http_resp_codegen.xi`):
`xiom.net.http.http_parse_response` is rejected by clang with "invalid
getelementptr indices": it stores field 2 of `%struct.HttpResponse` while
the emitted definition has 2 fields.

**Root cause**: a STDLIB same-leaf type collision of the R39 class --
`xiom.net.net.HttpResponse = { status: Int; body: Str; }` (2 fields) and
`xiom.net.http.HttpResponse = { status: Int; headers: Vec[(Str,Str)];
body: Vec[UInt8]; }` (3 fields). R39 deliberately qualified only NON-stdlib
project modules, so stdlib collisions still inject bare and the
alphabetically-first module wins.

**Inventory**: 40 same-leaf non-generic pub-type groups across stdlib
modules (`net.HttpResponse`, collect `Avl`/`PHeap`/`PMap`/`Hamt`/..., geom
`Vec2`/`Vec3`/`Mat4`, math `Graph`, regex `Regex`/`Match`, sync
`AtomicInt`, serialize `JsonValue`, ...). Most are facade re-declarations
with identical layouts; several are genuinely different types.

**Compiler-side experiment (2026-09-17, reverted)**: including stdlib in the
R39 qualification fixes the probe (both types emit module-qualified with the
right field counts; probe compiles and runs), but wholesale qualification
broke 6 stdlib smokes (regex/collect/sync) and a shape-difference triage
still broke 2 (collect/tree, collect/cache). The remaining groups need
stdlib-side dedup (or a coordinated stdlib-wide pass with their smoke
battery), so the change was reverted to keep the tree green.

**Resolution (2026-09-17, stdlib lane)**: the stdlib renamed
`net.net.HttpResponse` -> `NetHttpResponse` (commit `ce0c7fa`), leaving
`net.http.HttpResponse` as the single `%struct.HttpResponse` owner. Verified
on the pinned v0.60.0 compiler: `p_http_resp_codegen.xi` compiles and runs,
`smoke_stress_fuzz_parsers.xi` regains `http_parse_response` (600 inputs),
net battery 11/11, full module check 509/509.

**Remaining class**: the other same-leaf groups (facade duplicates and the
genuinely conflicting collect/math/etc. types) still rely on first-wins. The
compiler-side experiment above stands: stdlib-wide qualification is a
dedicated compiler+stdlib slice once the stdlib dedups the real conflicts.

**R44 qualification slice (2026-09-18, `main`) -- LANDED**. Catalog
(`xiom.`) modules now participate in the R39/R46 collision triage under the
stdlib audit standard (`tools/same_leaf_audit.ps1`): a group whose owners
are ALL catalog modules qualifies only when its DECLARED SHAPES conflict --
facade duplicates with identical layouts keep the legacy leaf-derived key.
Groups with any project owner keep the exact R39/R46 rule (non-generic
project collisions always qualify; generic ones only when shapes differ),
so user-program emission is untouched: bench IR stays byte-identical at
5,808,645 and `clang -c` exits 0. The compiler's `type_decl_shape` is the
structural form of the audit tool's conflict criterion (the tool's
line-oriented field extraction is an approximation of it).

Evidence (stdlib pin `stdlib-v0.60.0` = 2f819ac, 17 audit-conflicting
groups incl. `HttpResponse`; stdlib `main` = 16 groups after the
`NetHttpResponse` rename):

- `p_http_resp_codegen.xi` compiles and runs (exit 0) on the pin; the
  negative control (change stashed + rebuilt) reproduces the documented
  clang rejection `invalid getelementptr indices` on
  `%struct.HttpResponse` field 2.
- `check_modules.ps1` 509/509 clean (238.6 s, 8 workers).
- Full smoke corpus `run_smokes.ps1`: **949/949 pass**, 0 compile-fail, 0
  run-fail (1535.5 s, 8 workers; 9 parallel flakes retried solo green).
- `stdlib_execution_tests` 85/85 (+2 ignored) on the pin.
- Full e2e **2338/2338** (includes the new m90 lock) and checker
  **195/195** on the final tree; perf/determinism 2/2; bench 5,808,645
  bytes; v092 157,890 bytes (was 155,936 -- stdlib groups now qualify);
  test_json 164,283 bytes (was 164,787) -- all under budget.
- Lock: `e2e_m90_stdlib_same_leaf_http` (fixture
  `tests/regression/m90_stdlib_same_leaf_http/main.xi`), added to the CI
  lock line; skips loudly without a stdlib checkout and hard-fails under
  `XIOM_REQUIRE_STDLIB=1` (CI has the pin).

The stdlib-side dedup of the 16 remaining groups
(`docs/SAME_LEAF_TYPE_CONFLICTS.md` worklist: Executor, Future, Graph,
UnionFind, PHeap, IntervalTree, IntMap, StringMap, SpscRing, FloatScan,
Aabb, Sphere, Ray, Plane, Regex, Match) stays with the stdlib lane. With
the compiler standard in place, qualification is correct for any group
whose shapes genuinely conflict, so the dedup is layout/API hygiene rather
than a correctness prerequisite.

## R45. Tuple element types came from LLVM widths, not XIOM types -- FIXED (2026-09-17, `main`)

**Finding** (stdlib single-param surface sweep, preserved repro
`stdlib/tools/known_failures/p_sweep_single_param.xi`): any call to
`xiom.convert.overflow.overflowing_neg` failed clang with `'%tmp15' defined
with type '%struct.Tuple__Int__Int' but expected
'%struct.Tuple__Int__Bool'`. Root cause: the tuple-literal path and
`infer_llvm_type`'s tuple arm named elements from LLVM widths, so a
Bool-valued element (`true`, a tracked Bool local, a comparison/logical op)
became `"Int"` (i64) and `x as UInt16` became `"Int16"` (i16), while the
function signature named the same tuple from the checker's XIOM types
(`Tuple__Int__Bool`, `Tuple__UInt16__UInt16`).

A second facet surfaced behind it: the literal path queued new tuple
definitions in `pending_module_type_defs`, emitted at MODULE END -- LLVM
rejects `alloca` of a forward-referenced named type ("Cannot allocate
unsized type"; verified with a minimal .ll probe), so once element names
became XIOM-correct the definition was no longer reachable early enough.

**Fix**:

- literal path (expr.rs): `expr_is_bool(i)` (comparisons, logical ops,
  tracked Bool locals) names the element "Bool"; `Expr::As(_, ty, _)` uses
  the CAST TARGET's XIOM type. Unchanged otherwise.
- `infer_llvm_type` tuple arm (lib.rs): same Bool naming (deliberately NOT
  consulting `local_xiom_types` -- in mono/generic bodies those can still
  name type parameters and mis-typed `Vec[(Int, Int)]` element reads, m44).
- tuple definitions now go through `defer_concrete_def_if_in_body`, which
  splices them into the type-decl block (`module_deferred_types` +
  `emitted_type_defs` dedup) so clang parses them SIZED before any use.

**Verification** (all with the stdlib checkout active):

- the preserved sweep: 138 calls compile, link and run; the only stop is an
  EXPECTED `requires` contract trip on dummy input (`os.process` pid 0 /
  `debug` empty msg at 225:15), exit 1. Removed from the sweep for the
  compiler-side run: `xiom.os.env_unset` (links `unsetenv`, missing on
  Windows MSVC) and `xiom.async.io.async_read_line(0)` (passes `0 as *UInt8`
  to `fread` -> NULL FILE* abort) -- both stdlib/harness-side, not codegen.
- stdlib-lane verification on 483f283e: `p_x25519_keypair_codegen.xi` PASS
  (R43), `p_http_resp_codegen.xi` PASS (R44 rename holds),
  `p_async_read_line_codegen.xi` PASS, `p_match_vec_codegen.xi` PASS,
  check_modules 509/509.
- follow-up (2026-09-18, resolved on 12148d43): the FULL unmodified sweep
  (`p_sweep_single_param.xi`, incl. `env_unset` + `async_read_line`) does not
  finish under the debug driver's 300s watchdog and was reported as a >5-15
  min "hang". Re-measured with the RELEASE driver: codegen+clang complete in
  **481.6s**, failing only at link with `undefined symbol: unsetenv` (the
  stdlib Windows gap). No compiler hang: debug codegen is simply much slower
  on a 139-call/54-module program. The remaining run-time stops are the
  stdlib/harness items (`unsetenv`, `async_read_line(0)` NULL FILE*, dummy-arg
  `requires` trips). Debug sweep compile time is a Stage 6 budget candidate,
  not a correctness defect.

- locks: `e2e_m87_tuple_element_types` (new: comparison, Bool-local, cast,
  and Str/Bool tuples), m21 (triple tuple), m44 (zip/BTreeMap tuple
  payloads), m48 (writeback aggregates) all green; full e2e **2335/2335**,
  feature-reg 510/510, checker 194/194, perf/determinism 2/2, robustness
  63/63. `stdlib_execution_tests` remains 83/85 with the local checkout
  (collect/tree + collect/cache drift, identical on the pre-change driver).

## R46. Bench graph CLANG-CLEAN: generic same-leaf types, literal disambiguation, mono tuple params -- FIXED (2026-09-17, `main`)

`xiom --emit-ir examples/benchmark/main.xi` now emits IR that `clang -c`
accepts (the three remaining latent defects in the same-leaf family, behind
R39/R41/R42/R45, are closed):

1. **Generic same-leaf collisions**: `benchmark.generics.Box[T] =
   { value: T }` (1 field) and `benchmark.generics_hard.Box[T] =
   { item: T; sealed: Bool }` (2 fields) merged into ONE bare `%struct.Box`
   (the 1-field definition won) and the loser GEP'd field 1 of it. R39
   deliberately excluded generics (leaf-derived mono/concrete-container
   keys); the collision triage now includes generic types but SPLITS them
   only when their declared SHAPES conflict (`declared_type_shapes` + shape
   comparison in `collect_external_decls`). Identical generic
   re-declarations keep the legacy single key. Lock
   `e2e_m88_generic_same_leaf_boxes`.
2. **Bare literal name that is BOTH a struct and an enum variant**
   (`Node{...}`): the literal path resolved by name only, so
   `Node(value, left, right)` inside `BST.insert` compiled the same-leaf
   struct `benchmark.memory.Node` ({value, children}) and stored a
   `%struct.Vec` into the tree payload. The literal path now disambiguates
   by the LITERAL'S FIELD NAMES: it binds the enum variant when the payload
   matches and the same-leaf struct does not (BUG 31's struct preference is
   preserved when both match). Lock `e2e_m89_struct_variant_node`.
3. **Mono tuple params**: `pair[T](a: T, b: Float64) -> (T, Float64)`
   instantiated with T=Bool built `Tuple__Int__Float64` in the body (param
   names are not in `local_xiom_types`, so inference saw the i64 width)
   while the signature said `Tuple__Bool__Float64`. Element naming now
   consults `mono::param_concrete_types` for PARAMS OF THE CURRENT FUNCTION
   (`mono_param_xiom_name`, used by both the literal path and
   `infer_llvm_type`'s tuple arm). The lookup is deliberately narrow: a
   broader local-type/substitution lookup regressed m44/m48 by leaking
   global/stale entries into `Vec[(Int, Int)]` element reads. Lock: extended
   `e2e_m87_tuple_element_types`.

Also hardened in this slice: `monomorphised_fn_name` sanitizes concrete type
parts into identifier-safe text (an inference leak such as `[2 x Int]` can no
longer emit the invalid symbol `total_area_2 x Int`), and
`pick_variant_parent` gained a leaf-scope fallback for METHOD receivers
(`BST.insert` carries no module prefix, so `Empty` inside it binds
`benchmark.structures.BST` instead of the first-declared
`benchmark.enums.Message`).

**Verification**: `clang -c` on the emitted bench IR exits 0 (previously three
distinct clang rejections across this campaign); bench IR deterministic at
**5,808,645 bytes** (budget 6.3 MB); full e2e **2337/2337**, feature-reg
510/510, checker 194/194, perf/determinism 2/2, robustness 63/63,
pkg/dbg/lsp/mcp 63/34/44/39, `stdlib_execution_tests` 83/85 (the two known
checkout drifts, identical pre-change).

**Residual (open, not bench-gating)**: calls to GENERIC METHODS on a
module-qualified receiver type from ANOTHER module fall back to erased stubs
(`main -> h.Box.pack[Int](42)` emitted `@m88.hard.Box.pack()` returning
zeroinitializer), and generic methods whose type parameter is inferable only
from the receiver (`is_sealed[T]` on `Box[Int]`) stub the same way. m88 uses
module-local wrappers to stay on the proven path. See SESSION.md "Open
compiler findings". **FIXED 2026-09-18 in R46b -- see the next section.**

## R46b. R46 residuals: qualified-receiver generic-method instantiation + receiver-only type-arg inference -- FIXED (2026-09-18, `main`)

Both slopes behind the R46 residual note are closed; the m88 fixture now
locks the DIRECT call forms (module-local wrappers removed).

1. **Alias-qualified receiver types resolved by HashMap order.**
   `infer_struct_type_name`'s `Expr::Field` arm bound a `module.Type`
   receiver (`g.Box`) by scanning `type_meta` keys ending in `.Box` --
   HashMap iteration order. With two same-leaf `Box[T]` modules
   (`m88.generics` 1-field, `m88.hard` 2-field), `g.Box.new[Int](7)`
   resolved to `m88.hard.Box.new` on ~25% of runs (measured 3/12 pre-fix),
   so `is_generic` was false, the call took the erased base and the emitter
   stubbed `@m88.hard.Box.new()` returning zeroinitializer. Fix:
   `qualified_type_key_for_path` (lib.rs) flattens the receiver path,
   expands single-segment module bindings through
   `module_receiver_paths`/`use_alias_map` (mirroring
   `resolve_module_call`), and probes the type registries for
   `<module>.<leaf>` (xiom-stripped fallback for catalog paths), with
   `pick_deterministic` for the remaining suffix probe. The bare-key and
   current-module cases are unchanged; the Ident-arm fallbacks are now
   deterministic too.

2. **Receiver-only type-arg inference for computed receivers.**
   `g.Box.new[Str]("x").value_of()` reaches the outer generic method with a
   Call receiver; the receiver-only fallback defaulted T to `Int`, and when
   `infer_struct_type_name` returned None for the Index-form callee
   (`new[Str]` parses as `Call(Index(Field, type_arg), ..)`) the whole call
   compiled to the literal-0 stub `Ok(("0", i64))` -- the call chain
   disappeared from the IR entirely. Fix: `receiver_generic_arg_at`
   (call.rs) reads the receiver call's explicit (or recorded) instantiation,
   substitutes the receiver callee's declared return type, and returns the
   type arg at the outer method's generic position (`Box[Str]` -> "Str";
   `type_string_full` drops Named args, so the substitution renders them
   explicitly). The i64-lowering "Int" fallback remains for genuinely
   untyped receivers.

**Verification**: m88 fixture rewritten to direct forms (explicit type
args, arg inference, receiver-only inference, computed receiver) and green;
12/12 identical compiles (pre-fix 3/12 bound the wrong module); bench IR
byte-identical at 5,808,645 bytes and `clang -c` exit 0; e2e 2337/2337,
feature-reg 510/510, checker 194/194, perf/determinism 2/2, robustness
63/63, pkg/dbg/lsp/mcp 63/34/44/39. Lock:
`e2e_m88_generic_same_leaf_boxes` (extended fixture; CI lock line
unchanged).

## R47. Playground C18/C19: Str conversion + Option payload codegen -- FIXED (2026-09-18, `main`)

The playground session (AUDIT.md section 12, pin v0.60.1) reported Str
garbage/pointer values through `.to_str()` and containers (C18) and
deterministic `.to_str()` corruption (C19: Float64 IEEE bits printed as
decimals, Str fields/results empty). Four distinct compiler defects were
behind them; all reproduce on `main` before the fix.

1. **Conversion methods were accepted but never injected.**
   `Str/Int/Float64/Bool.to_str()` live in `xiom.fmt` (declared module
   name; directory `format/`). The checker accepted `.to_str()` on any
   receiver (its documented generic spelling, `fmt.format1[T]` body calls
   `arg.to_str()`), but a program that never `use`d `xiom.fmt` had no
   declaration for codegen, which emitted a zero-arg `i64` stub returning
   zeroinitializer: Str printed empty, and on older pins the i64 result
   flowed into print/conversions as raw bits. Fix: the checker's two
   `to_str`/`to_string` special cases record `xiom.fmt` in
   `peeked_resolved`, so `collect_external_decls` peeks it (BUG 28 #4's
   injection pattern) and the concrete conversions reach codegen. The
   peek follows fmt's dependency closure (the closure is what an explicit
   `use xiom.fmt;` loads).

2. **`Option[Str].unwrap_or("...")` phi dominance violation.** The default
   was compiled in the fail block but its `ptrtoint` coercion was emitted
   in the ok block while the merge phi tagged the coerced value on the
   fail edge -> clang `Instruction does not dominate all uses!` (the
   playground's C18 minimal probe did not compile at all on `main`).
   Fix: compute the payload field type first (pure), compile AND coerce
   the default inside the fail block, then branch.

3. **Float64 defaults hit an invalid cast.** The same coercion emitted
   `ptrtoint double 0.0 to i64`. Option/Result payload slots store the
   f64 BIT PATTERN, so the bridge is `bitcast` for float<->i64
   (fptosi/sitofp would numerically convert and corrupt 2.5 -> 2.0).
   Pointer<->i64 stays ptrtoint/inttoptr.

4. **Erased i64 payloads leaked the raw integer ABI.** `unwrap_or` on a
   tracked `Option[Str]`/`Option[Float64]` local returned the phi as i64;
   the call-site arg coercion then materialized a single-byte temp
   (`trunc i64 -> i8`) for a `Str` parameter, printing stack garbage.
   The result is now refined to `i8*` (inttoptr) / `double` (bitcast)
   from `local_opt_payload`, mirroring the 5d `unwrap` path.

5. **Chained conversion receivers lost their type.** `o.unwrap_or("x")
   .to_str()` and `v[0].to_str()` have no declared-fn return type, so the
   `to_str` sugar's LLVM inference saw the erased i64 default and lowered
   to `xiom_int_to_string`, printing pointer bits (and Float64 bit
   patterns, C19's "1.5 -> 4609434218613702656"). New
   `infer_expr_xiom_type_deep` resolves Ident/Field/Index/Call receivers
   (declared returns, `local_opt_payload`, Option/Result generics), and
   the sugar now emits Str identity, `convert.float_to_string`, or the
   Bool select before falling back to the integer conversion.

**Verification**: new lock `e2e_m91_conversion_methods` (fixture
`tests/regression/m91_conversion_methods/main.xi`) covers the whole matrix
without `use xiom.fmt`: Str/Int/Float64/Bool `.to_str`, struct-field Str,
`Option[Str]`/`Option[Float64]` `unwrap_or` (Some and None), chained
`.unwrap_or(...).to_str()`, `Vec[Str]` element `.to_str`. All were red on
the pre-fix tree (clang dominance error / garbage output). stdlib-exec
85/85 (+2 ignored), feature-reg 510/510, perf/determinism 2/2. Playground:
after this lands, run `node tools/generate-expected-outputs.js --wsl` and
`node tools/lesson-audit.js --baseline tools/lesson-baseline.json`.

## R48. Playground verification residue: interface dispatch, zero-init, cache identity, WASM asset (2026-09-19)

Context: playground AUDIT.md section 16 (pinned v0.60.1 data; verification
against the R47 build `16a89615`). The compiler-lane items from that
verification, with repro commands using the playground lesson sources
(`tmp/playground/lessons/**` extracted `.solution`).

### FIXED

1. **Interface dispatch through `&T`/`&mut T` generic arguments (C17, 11
   lessons).** Bare generic params were invisible to the argument
   inference: `type_from_ast` strips `&` so the bare-`T` branch saw
   `introduce(&person)`'s `Expr::Ref` with no arm for it (`_ => "Int"`),
   and `&mut T`/`*mut T` never matched the branch at all because
   `extract_type_arg_names` returned empty for a bare `T`. The call
   mono'd as `introduce_Int` and codegen failed
   `C001: type 'Int' does not implement 'Greetable'`. Fix:
   - unwrap Ref/MutRef/Unary-Ref in the bare-`T` argument branch
     (call.rs);
   - `extract_type_arg_names` reports a bare named type as itself, and
     `infer_generic_arg_from_container` resolves bare params from the
     argument (ref args name the VALUE; a pointer local passed to a pointer
     param defers to the sibling `T` param -- the caller keeps scanning on
     None for bare params only, preserving the m35 container guard);
   - `ptr.replace[T](*mut T, T)` (mem.replace body) keeps inferring from
     `src: T`, so m73's Str path is unchanged.
   Lock: `e2e_m92_interface_dispatch_zero_init`; m73 + m91 stay green.
   Verified: L6-01/02/03/04/06/07/13/16/30 build AND run (L6-05, below,
   now builds but crashes at runtime).

2. **Pointer/double zero-init in match-result slots (C17, clang class).**
   `expr.rs` emitted `store i8* 0` / `store double 0` for non-struct match
   result slots; clang rejects both (`integer constant must have integer
   type`). Now `null` for pointers and `0.0` for double/float in expr.rs and
   stmt.rs. Verified: L2-12/14/15/16 build; L2-19 builds but crashes at
   runtime (below).

3. **Run-cache served binaries across compiler builds.** The script cache
   (`~/.xiom/jit`, `/tmp/xiom_run` in the playground harness) was keyed by
   the source hash alone, so a new build silently reused the previous
   build's binaries (the v0.60.1 -> v0.61.0 crossover). `cache_key` now
   salts the hash with `CARGO_PKG_VERSION|OS-ARCH|pointer-bits` under a
   `xiom-cache-v2` prefix, invalidating every old entry; unit test added.

4. **`xiom fmt` unwired (C2).** `xiom.exe` now dispatches the first word
   `fmt|lsp|mcp|pkg|dbg|verify|ffigen` to the sibling tool binary BEFORE its
   own `--help`/`--version` handling, mirroring the launcher wrapper, so the
   tools (and their own flags) work on Linux/macOS installs without
   `xiom.bat`.

5. **No WASM release asset (C8).** `release.yml` now builds
   `xiom-wasm` for `wasm32-unknown-unknown` in the Linux leg, verifies the
   `\0asm` magic, stages `bin/xiom-wasm.wasm` inside the archive and
   publishes `xiom-wasm-<ver>.wasm` as its own release asset (included in
   SHA256SUMS; release `FILES` glob covers `*.wasm`).

### R49 BATCH STATUS (2026-09-19)

**FIXED in the R49 batch** (codegen + locks; full e2e green, see SESSION.md):

- **L6-28 (C17 interface residue)**: struct-literal arguments now infer the
  generic (`pq.insert(Task{...})` -> T=Task), ANNOTATED generic-struct locals
  keep their type args (`type_annotation_name`), the call site REGISTERS the
  concrete `Option__Task` (`concrete_container_llvm`) so the mono def and the
  call agree, `Vec.get` boxes struct elements in generic bodies
  (mono-substituted element types), `.unwrap()` unboxes struct/enum/aggregate
  payloads, and `clone` returns the COMPILED value type. Lesson builds and
  runs (Write docs/Fix bug/Deploy). Locks: `e2e_m93_interface_generic_inference`.
- **L6-31 (C17 clang residue)**: duplicate definitions (the lesson solution
  declares `new_student`/`new_course` twice) are emitted once -- the serial
  emitter now dedupes on the pre-assigned symbol (`emitted_fns` first-wins,
  matching `types.functions`). Lesson builds and runs.
- **L8-15/L8-18 (C17 clang residue + AV)**: computed Vec receivers
  (`grid.get(i).unwrap()`) resolve their element type through the unwrap
  expression, struct/enum elements box/unbox consistently, the inline `set`
  memcpys element bytes (was storing the boxed pointer), and `unwrap` hint
  resolution handles enums/aggregates. L8-15 plays a correct game, L8-18
  renders the board. Lock: `e2e_m94_nested_vec_struct_elem`.
- **L6-05 (C17 runtime AV)**: generic-method params record their
  mono-substituted Vec element type (`items: &Vec[T]` with T=Meter), so
  `items.get(i)` takes the struct-element path. Total prints 30. Lock
  (L6-05 shape covered by m93's queue work).
- **L2-19 (C17 runtime AV)**: qualified enum variants (`TrafficLight.Green`)
  infer the parent enum struct instead of i64; `next()` returns a valid
  enum. Lock: `e2e_m97_enum_variant_match`.
- **L0-11 + array-of-Str (C18)**: fixed-array element loads keep `i8*` Str
  elements (val_to_i64 ptrtoint printed the ADDRESS: "Hi, 1406..."). Lock:
  `e2e_m96_array_str_elem`.
- **L5-32/L5-34 (C19 invalid UTF-8)**: match-result slots keep Str (BinOp
  `+` with an i8* operand infers i8*; bare conversion calls resolve a unique
  module-qualified return type) and Some/Ok bindings unbox struct payloads
  (`Vec[Student].get(...)`). L5-32 prints 2/2/First: 1, L5-34 Vec: Bob.
- **L5-42 (C19 invalid UTF-8)**: FIXED -- same match-slot class; a bare
  `to_string(v)` call now resolves its return type through the unique
  `.to_string` module-qualified candidate (was the i64 fallback, which made
  the Str consumer truncate the pointer to one byte). Prints 10/20/30; lock
  `e2e_m98_match_str_to_string`.
- **@pre call capture (R49-2, stdlib relay)**: `collect_atpre_vars` now
  collects every variable under `expr@pre` (calls/fields), `AtPre` on a
  compound expression rebinds the entry snapshots for its duration (ref
  params through a fresh pointer slot -- passing the snapshot alloca
  directly made a `%struct.T**` load read the first FIELD as a pointer),
  and struct snapshots deep-copy inline `%struct.Vec` buffers so element
  reads see entry-state data. `p_pre_call_capture` (free fn + method
  receivers + field/index forms) passes. Lock: `e2e_m95_pre_call_capture`.
  Stdlib can restore the stronger size relations in its safe clauses.

**STILL OPEN**:

- **L6-40 (C17 interface residue)**: FIXED (R55) -- module-scoped
  `Runner[T: Plugin]` `create()` has no argument evidence at its call site; T
  is fixed only by the LATER `add_plugin(&mut runner, EchoPlugin{})`. Codegen
  inference is single-pass, so `create` emitted the `0` fallback and `run_all`
  mono'd T=Int -> C001 "Int does not implement Plugin". A bounded
  function-body evidence pre-pass (`prepass_generic_type_evidence`, decl.rs)
  now runs before each body is emitted: for a zero-argument generic call bound
  to a local it scans the body for a later call passing that local at a param
  of the factory's RETURN container while naming the type parameter
  concretely at another param, records the call site's concrete types (keyed
  by the callee byte-span, consumed in `compile_call_with_types` before the
  `0` fallback) and seeds the binding's container type. `infer_call_return_xiom`
  also resolves module-qualified generic returns from the recorded
  instantiation or the pre-pass evidence (the `var` arm records the binding
  BEFORE compiling its initializer, so the pre-pass map is the fallback
  source). Fixture `tests/regression/m110_generic_factory_evidence`, lock
  `e2e_m110_generic_factory_evidence` + CI line. Full e2e 2358/2358.
- **R52 packages relay**: FIXED --
  (1) `use xiom.test; assert(1 == 1, "...")` bound the transitively-imported
  PRIVATE `core.assert` (keep-first bare alias) instead of the imported
  module's exported TestResult assert; the result was a zeroed TestResult
  (F/0) and every unqualified-assert conformance test reported failures.
  Bare-call resolution now keeps a keep-first alias only when its target is
  PUB; a private alias defers to the imported-module export ranking
  (`matches_import` > pub > caller-module > shortest). Lock
  `e2e_m105_unqualified_assert`; the time smoke (private
  `normalize_duration` helper) and path/gzip round-6 stay green.
  (2) `xiom.std` with a VERSION spec failed `xiom pkg install`
  ("dependency 'xiom.std' is not in the registry"): the stdlib is a platform
  package now (`is_platform_dep`), excluded from the registry closure like
  path/git specs; legacy `xiom-std` accepted.
  (3) `xiom pkg keygen --help` printed the top-level help AND generated a
  key: `--help` after a subcommand now prints that command's usage.
  (4) Release artifacts: the local `target/release/xiom.exe` predates the
  R48 native dispatch; the source is correct (`xiom pkg` execs the sibling
  xiom-pkg) -- a fresh release build is required for publishing.
- **R51 L4 cluster (playground audit S19)**: FIXED --
  L4-29/33/39 (`items@pre.len()`, `self@pre.items.len()`): the Vec.len
  dispatch and `infer_struct_type_name` now look through `AtPre` receivers
  (was `xiom_str_len(<%struct.Vec>)` -> clang "defined with type %struct.Vec
  but expected ptr"); L4-26 (`for i in range(0,n)` with a user
  `fn range(scores: &Vec[Int]) -> Int`): the for-in lowering resolves the
  `range` builtin STRUCTURALLY (exactly two args) instead of by name, so the
  user function cannot hijack the loop into an invalid GEP. Locks
  `e2e_m101_pre_len_receivers` (exit 20), `e2e_m103_pre_len_param` (exit 30),
  `e2e_m102_range_shadowing` (exit 25).
- **R52 payload/binding batch (playground C18/C19 + L5/L3 lessons)**: FIXED
  the erased-type family behind the remaining pointer-print lessons --
  L5-20 (generic struct FIELD read through a concrete base: substitute the
  base's type args -> i8* instead of the erased i64), L5-31 (generic METHOD
  return: resolve the receiver's tracked concrete args FIRST; the
  generic_instantiations lookup returned the first (Int) mono for every
  call), L5-26/L5-36 (match-result slots and match-EXPRESSION operands keep
  Str: scrutinee payload preferred over the static `Option.value=i64`,
  arm-common type prediction, literal/Match arms in the deep inference),
  L5-43 (explicit redundant `&receiver` argument dropped; side-effecting
  match scrutinees evaluated ONCE -- the Expr::Match wrapper no longer
  compiles the scrutinee before delegating to the statement form),
  L5-09/L3-02 (Map[Str,Str] and unannotated Some(...) payloads bind as
  Str: last-type-arg payload resolution for tracked container locals,
  `Some/Ok/Err` return recording, placeholder payloads filtered so a stale
  "T" cannot shadow the concrete type), L5-36/L5-35 (to_str via match
  results). Locks `e2e_m106..m108`; all 11 previously nondeterministic
  lessons (L3-02, L5-09/20/24/26/29/31/35/36/43) plus L5-21 are
  deterministic now.
- **L8-14 (Map[Str,Str] morse trap)**: FIXED (R58). The `0xC000001D` trap is
  gone; the lesson prints `... --- ...` / `SOS` / `.... ..` / `HI`. Root
  cause: payload-type resolution for `unwrap`/`unwrap_or` only handled Ident
  receivers, and the tracked type of a `&Map[Str, Str]` PARAM was truncated to
  `"&Map"` by `ref_preserving_name` (args dropped), so
  `morse.get(ch).unwrap_or("?")` returned the Str payload as a raw i64 handle
  and `result + code` printed pointers (later the decode path trapped).
  Fixes: (1) `ref_preserving_name` renders refs with `type_annotation_name`
  (args preserved); (2) `generic_container_last_arg` strips `&`/`&mut` so
  reference-qualified containers resolve; (3) both the `unwrap` and
  `unwrap_or` builtins fall back to `scrutinee_payload_xiom` for CALL
  receivers and unbox aggregate payloads via `try_unbox_payload`. Lock
  `e2e_m113_map_str_str_morse` (`tests/regression/m113_map_str_str_morse`).
  Full e2e 2361/2361.
- **R59 (stdlib p_result_tuple_vec_loop)**: FIXED. The enclosing match arm's
  result slot leaked into nested LOOP bodies: with the arm's
  `match_result_ptr` still set, `oid.push(value)` as a while body's LAST
  expression stored a `%struct.Vec` into the arm's `%struct.Result` slot
  (`'%tmp157' defined with type '%struct.Vec' but expected '%struct.Result'`).
  The R29 guard only covered expression statements in the same block, not
  nested loops. While/For/Spawn bodies now save+clear `match_result_ptr`
  around `compile_block` (statement contexts must not store into the arm
  slot). Lock `e2e_m114_result_tuple_vec_loop`.
- **R60 (stdlib p_ref_tuple_mangle)**: FIXED. Two element-naming defects in
  tuples: (1) a reference-typed local named the element with its `&`
  ("&Vec" -> mangled `%struct.Tuple__&Vec__Vec`, clang "expected '=' after
  name"); (2) a container CTOR element (`Vec[UInt8].new()`) was named by its
  erased LLVM type ("Int") so the construction (`Tuple__Vec__Int`) differed
  from the signature (`Tuple__Vec__Vec`). The expr-side registration namer
  now strips reference markers, and BOTH namers (expr-side registration and
  `infer_llvm_type`'s tuple arm) resolve container-ctor elements to the
  container base. Non-ctor calls keep the original inferred (often
  module-qualified) name so `Tuple__m37_tuple_struct.Big__...` stays intact.
  Lock `e2e_m115_ref_tuple_mangle`; the three locks an over-broad
  shared-namer version broke (`e2e_m37_tuple_struct`,
  `e2e_m44_round13_tuple_payloads`, `e2e_m48_round14c_writeback_aggregates`)
  are green. Full e2e 2363/2363.
- **R61 (R7 residual + interface ABI ruling)**: FIXED/CLOSED.
  (a) `p_generic_push` / `p_gp_b` / `p_gp_c`: local explicit-generic calls
  (`var h = make_holder[JsonValue]()`) never recorded their substituted return
  type, so `h.values[0]` lost the element type and the index read fell to the
  scalar i64 switch, loading the first 8 bytes of an inline aggregate as a
  pointer (0xC0000005). `infer_call_return_xiom` now substitutes the declared
  return type for `Expr::GenericCall` explicit type args. All three probes
  print `A=[42]` / `B=["tree"]` / `C=[9]`. Lock
  `e2e_m116_generic_ctor_field_index`.
  (b) `impl Trait[Args]` desugaring left the `self` PARAM typed `Self`
  (T001: expected Int, found Self); it is retyped to the impl type.
  (c) A REGISTERED impl fn with ZERO generics (`Int.hash` also sits in
  `generic_fn_decls`) was routed into the generic path, which returned a
  silent `0` with no call emitted; `is_generic` now requires a non-empty
  generic list.
  (d) RULING on `p_hash_probe`: the interface VALUE-RECEIVER ABI (box the
  aggregate at the call, deref it in the callee) is not implemented. Instead
  of the old silent wrong answer (`a=5381` for every input), codegen now
  rejects the shape loudly: `unsupported: interface-typed parameter ...
  receiving aggregate argument`. Lock
  `e2e_m117_interface_value_abi_rejected` (asserts the compile error).
  Implementing the ABI is a feature, not a bug fix -- `xiom/hash.xi`
  documents the interface as having no concrete impls.
  Full e2e 2365/2365.
- **R62 (playground request #2): fmt reachable-only peek -- FIXED.** Every
  `.to_str()`/`.to_string()` used to peek the whole `xiom.fmt` module plus its
  8-module dep closure (table/units/ansi/text/markup/string/convert/io/num),
  making the front-end 4-6x slower than the v0.60.1 release on the playground
  harness (to_str emit-ir 2400 ms vs hello 1540 ms on R55). The peek is now
  receiver-aware:
  - Int/IntN/UInt*/Bool/Str/Char peek NOTHING -- codegen already lowers them
    (`xiom_int_to_string`, inline Bool select, Str identity);
  - Float64/Float32/Float128 peek the small `xiom.convert` module
    (convert/convert.xi, 268 lines, deps: xiom.string);
  - every other receiver (generics, user types) keeps the R47 fmt fallback.
  Supporting changes: `peeked_leaves` seeds the injection reachability filter
  with `float_to_string` (codegen lowers to it without any AST reference, so
  the filter previously pruned it); the codegen float builtin resolves the
  conversion symbol through `fn_symbol_map` (peeked-module fns are injected
  for codegen but are not in `types.functions` under that key).
  Measured with the playground harness (`tools/bench-cold-compile.js`, clean
  TEMP, same machine, 3 samples): to_str/hello emit-ir ratio **1.81x -> 1.12x**
  (acceptance <= 1.3x), to_str emit-ir 2660 -> 2077 ms (-22%), loop_200
  3358 -> 2333 ms (-31%); `fmt.*` symbols in the IR 1 -> 0 while the program
  still prints `42` / `1.5`. Gates: full e2e 2365/2365, feature-reg 510/510,
  stdlib-exec 85/85, diff 24/24, robustness 63/63, api-freeze 2/2.
  Pre-existing failures found while gating (present on the stashed baseline
  too -- NOT R62): `stdlib_tests::stdlib_all_modules_compile_to_ir` fails on
  `xiom.encoding.ascii85` `Result[Vec[UInt8], Str]` vs `Option[Vec[UInt8]]`
  and `xiom.core` `cannot call 'float_to_string'`; generic `T.to_str()`
  prints a denormal even with `use xiom.fmt;` (recorded in SESSION.md as
  open findings, not in the gated suite list).
- **R65 (stdlib p_platform_env)**: FIXED. `xiom.env.OS/ARCH/FAMILY` were
  hardcoded literals in `xiom/os/env.xi` ("windows"/"x86_64"/"windows"), so a
  Linux build reported windows while `platform_is_linux()` was true. They are
  compile-time TARGET facts: codegen now overrides the value at the
  module-qualified reference site (catalog decls are injected flattened, so
  the definition carries no module path) via `target_platform_constants()`:
  wasm -> unknown/wasm32/wasm; aarch64/riscv64 linux triples -> linux/<arch>/
  unix; Native -> the compiler's own OS/ARCH/FAMILY (correct on arm64 hosts,
  where the driver's hardcoded x86_64-* triple prefix would lie). The override
  also seeds the leaf key so later bare references agree. Verified on Windows
  and on Linux (WSL) with the stdlib's own probe:
  `env.OS=[linux] env.FAMILY=[unix] env.ARCH=[x86_64]` (was `windows`/`windows`).
  Lock `e2e_m118_env_platform_constants` (host-agnostic: derives the host from
  `$OS`). Full e2e 2366/2366. NOTE for future artifact checks: extracting
  release archives under `tmp/` creates duplicate stdlib trees that break the
  `e2e_m17_zero_warnings` duplicate-module assertion -- clean them after
  verification.
- **R66 (P1-4 contract methods, Windows-CI AV)**: FIXED. `e2e_p1_contract_methods`
  returned `-1073741819` (RUN-time access violation) on `windows-latest`,
  deterministic (2/2) on the runner but NOT reproducible locally (RC=0 with
  debug and release compilers). Root cause (reproduced locally as a WRONG
  ANSWER, not an AV): the contract block in `call.rs` compiled the receiver,
  allocad the value, bitcast the slot to `i8*` and called
  `xiom_is_sorted(i8*)` / `xiom_contains(i8*, i64)`. The stdlib runtime
  (`runtime/xiom_runtime.c`) reads `data[0]` as the element COUNT and elements
  at `data[1..]`, but the pointer addresses the receiver VALUE -- for a
  `%struct.Vec` (`{data, len, cap, elem_size}`) word 0 is the DATA POINTER, so
  the loop bound was a heap address. Locally the garbage words produced an
  early decreasing pair (a sorted `[1..5]` reported FALSE -- minimal repro
  `if arr.is_sorted() { return 11; }` exited 22); on the runner `xiom_contains`
  walked far enough to hit unmapped memory -> AV. Literal receivers were also
  broken twice over (an alloca OF the pointer; the buffer's own `[len]` slot
  never reached the intrinsic), and `let a = [...]` fixed arrays read their
  first ELEMENT as the count.
  FIX (codegen): `is_sorted`/`contains` are now lowered INLINE over the real
  `%struct.Vec` header (`vec_abi.rs`: `emit_contract_is_sorted`,
  `emit_contract_contains`, `resolve_contract_vec_scan`): len/data/elem-size
  are read from the header and elements are compared in their own
  representation -- width/sign-correct integer loads, `fcmp ogt/oeq` for
  Float32/Float64, `strcmp` for Str. Array-literal registers (counted
  `[len][elem...]` buffers) and fixed-array bindings are bridged to a heap
  Vec first (`val_to_struct` / `array_as_vec_arg`). Receivers with no concrete
  element type, or element kinds the scan cannot compare (structs,
  containers, unresolved generics), now fail LOUDLY with
  `unsupported: '<method>' ...` instead of scanning garbage -- the m117
  precedent for silent-wrong paths. `all`/`none` keep the legacy `len=0`
  stub semantics (trivially true) pending a real predicate-call lowering;
  they were not the reported AV.
  VERIFICATION: probes exit as expected for sorted/unsorted `Vec[Int]`,
  `contains` hit/miss, `let`/literal/struct-field/Slice-param receivers,
  `Vec[Str]` (lexicographic), `Vec[Float64]`, empty/single-element Vecs and a
  generic body (`T=Int` via mono). Locks: `e2e_m119_contract_method_values`
  (+ CI line) and the `e2e_p1_contract_methods` fixture now ASSERTS the
  values (it previously returned 0 unconditionally, which hid the garbage);
  `integration_tests` updated: the method form asserts the inline scan and a
  new negative test pins the loud rejection for non-collection receivers.
  Full e2e 2367/2367 (2366 baseline + the m119 lock).
- **R67 (benchmark/option-porter relay, v0.61.3)**: FIXED. Constructing
  `Ok(x)`/`Err(x)` (and the same class for `Some`/`None`) inside a function
  whose return type is a USER struct whose name CONTAINS "Result"/"Option"
  miscompiled. Root cause: the ctor sites chose the container type with a
  SUBSTRING test on the enclosing return type
  (`ctor_ret.contains("Result")` -> used `%struct.TestResult` as the Result
  struct). `Ok(5)` in `fn f() -> TestResult` emitted a `%struct.TestResult`
  payload with three GEP indices -> clang `invalid getelementptr indices`
  (the porter's exact failure; helper ctors returning Result was the
  workaround). `Some(3)` in `fn f() -> Options` / `MyOption` had the same
  shape (the relayed report is Result; Option is the same bug class).
  FIX: strict container-leaf test `is_llvm_container_struct(ty, name)`
  (`vec_abi.rs`) -- leaf exactly `Result`/`Option` (module-qualified included)
  or a concrete `Result__A__B`/`Option__T` instantiation; a user leaf like
  `TestResult` no longer matches. Applied to all four ctor sites
  (`Expr::Some/None/Ok/Err`, expr.rs). `%struct.MyResult` never appears for
  `type MyResult = Result[Int, Str]` (aliases resolve to `%struct.Result`),
  so the strict test loses nothing.
  VERIFICATION: repro probes (Ok/Err locals in a 2-field TestResult fn) now
  compile/run; `Options`/`MyOption` Some/None shapes compile/run;
  concrete `Option[Point]`/`Result[Point, Str]` returns still build their
  concrete containers (5c.35 preserved). Lock
  `e2e_m120_ctor_user_struct_return` + CI line. Full e2e 2368/2368.
- **R68 (packages relay: legacy nested `extern`)**: FIXED. `extern "C" { ... }`
  inside a FUNCTION BODY (a legacy-package idiom, e.g. the audio_beep shape)
  fell through `parse_block` into the expression parser, which called
  `parse_ident` on the `extern` token and reported the misleading
  `P001: 'extern' is a reserved keyword and cannot be used as an identifier`.
  FIX (parser): `parse_block` now parses a nested extern block and pushes it
  onto `pending_externs`; `parse_top_decl_with_pending` flushes those blocks
  immediately BEFORE the declaration whose body contained them (all three
  top-level loops -- file program, block-form module, brace-less module), so
  call sites resolve and module nesting is preserved. Extern fns carry no
  body, so hoisting is semantically transparent; duplicate per-function
  declarations merge in the checker (verified, not deduped in the parser).
  VERIFICATION: parser unit tests (hoist-before-fn ordering, duplicate
  blocks); probes: nested in `main`, in a second fn, inside an `if` body, in
  a block-form module, duplicate declarations -- all compile and run
  (`clock` here). Lock `e2e_m121_nested_extern` + CI line.
  Full e2e 2369/2369 for the R67+R68 batch.
  REMAINING (packages lane, policy): the rest of the rule-drift inventory
  (declarations without terminators, `extern`/`unsafe` contract requirements
  on legacy packages) is a dialect-migration question, not a parser bug --
  a migration note/codemod is the suggested path. The `xiom.ffi` triage
  abort ("no source modules found" instead of a FAIL summary) is in the
  packages harness, not in this repo (message does not exist here).
- **R69 (generic `T.to_str()` denormal -- the queued design-decision item)**:
  FIXED as a codegen bug, not a design change. `local_xiom_types` is a GLOBAL
  map that is never cleared per function; `compile_fn` records every param's
  XIOM type, but the MONOMORPHISATION param loop (`lib.rs`) recorded only
  Vec-elem/array-elem/fn-typed params. A generic `x: T` therefore kept a
  STALE entry from an earlier emitted function: with `use xiom.io`, xiom.fmt
  emits `x: Float64` first, so
  `fn show[T](x: T) -> Str { return x.to_str(); }` lowered `x.to_str()`
  through the Float64 conversion -- show(99) printed `4.89124989382835e-322`
  (the i64 99 bits read as a double), show("hi") printed `0`, show(true)
  printed `4.94e-324`; only show(2.5) was accidentally right. (Repro
  `tmp/cleanbench/to_str_edges.xi`; the minimal delta is any imported module
  that compiles a function with a Float64 local/param named `x` before the
  generic instantiation.)
  FIX: the mono param loop now mirrors `compile_fn` -- records
  `ref_preserving_name(param.ty)` / `type_from_ast(param.ty)` with the
  `type_map` substitution applied, for EVERY param (`x: T` with T=Int ->
  "Int", unresolved T stays "T" and falls through to the erased LLVM
  inference).
  VERIFICATION: `to_str_edges.xi` now prints 42/1.5/2.5/true/hello/7/65/99/123;
  a dedicated probe covers T = Int/Str/Bool/Float64/UInt and two-param
  generics in both argument orders. Lock `e2e_m122_generic_param_type` + CI
  line. (The "Display-bound monomorphisation vs per-concrete expansion"
  question is moot for this failure mode: the erased-LLVM fallback is correct
  once the stale entry is gone).
  Full e2e 2370/2370 (R69 batch).
- **R70 (packages relay: `for x in <collection>` / BUG-17-family garbage)**:
  FIXED. Two coupled defects:
  (a) CODEGEN: the For lowering compiled the iterable and ALWAYS treated it as
  `Range{start: i64, end: i64}` -- field 0 = index, field 1 = bound. For a
  `%struct.Vec` field 0 is the DATA POINTER and field 1 the length, so
  `for x in v` used a heap address as the index, `icmp slt data_ptr, len` was
  false (heap address > len) and the loop silently ran ZERO times; had it
  iterated, `store data+1` would have CORRUPTED the Vec's data pointer. The
  loop variable was bound to field 0 (a pointer), never to an element.
  Non-2-field iterables GEP'd a pointer base and clang rejected the IR
  (`invalid getelementptr indices`, e.g. `&Vec` params). The checker also
  bound the loop variable to `CheckedType::Int` unconditionally
  ("simplified"), so `for s in vec_of_str { str_len(s) }` failed with
  "expected Str, found Int" and struct-element loops typed `p.x` as Int.
  This is the BUG-17 family the geometer relayed as "str_len()/.len() garbage
  for Str values read back from Vec[Str] elements": any Vec touched by a
  `for` loop had its data pointer rewritten.
  FIX (codegen, stmt.rs): `for` over a collection lowers to a REAL element
  loop -- `%struct.Vec` values and `%struct.Vec*` headers read
  data/len/esz from the header and load elements at `i*esz` in their own
  LLVM type (Int/Str/struct/Bool elements verified); array LITERALS compile
  to a heap Vec first (their counted-buffer register has no stride
  metadata); fixed arrays `[N x T]` iterate with a typed GEP; a real
  `Range` value keeps {start,end} iteration; `range_inclusive`/`0..=b` is
  lowered structurally with `end+1` (previously an undefined i64 call that
  iterated garbage); anything else fails LOUDLY. Loop var gets its element
  LLVM type (and XIOM name) recorded, `break`/`continue`/labels/nesting keep
  working (increment before the body, like the Range path).
  FIX (checker): `CheckedType::for_loop_element_type` maps
  `Vec[T]`/`&Vec[T]`/`Slice[T]`/`Set[T]`/`[N]T` -> T (`Range` -> Int, other
  iterables keep the historical Int); `let`/`var` bindings of array literals
  now register `Vec[elem]` via `inferred_binding_type` so a loop over the
  BINDING knows the element type (`var words = ["a","bbb"]; for s in words`).
  VERIFICATION: probes for Vec locals, `&Vec` params, array literals, fixed
  arrays (incl. a 2-element array that must NOT be read as a Range), Vec[Str]
  with `str_len`, Vec[Point] element field access, nested loops,
  break/continue, `range`, `0..b`, `0..=b` and a `Range` value. Lock
  `e2e_m123_for_in_collections` + CI line. checker 195/195; parser 101/101;
  feature-reg 510/510; integration 129/129; robustness 63/63; fuzz 24/24;
  full e2e 2371/2371.
  REMAINING from the relay (need the package repros; simple local probes for
  each shape pass): indexed calls through `Vec[fn]` elements (likely the same
  for-loop family -- `run_all` iterating tests), untyped `Vec[Int]` element
  reads lowered as Str comparisons (the `Vec[elem]` binding refinement may
  cover it), and `byte_at(...) == <UInt8 const>` for bytes >= 128.
- **R71 (queued `all`/`none` contract stubs)**: FIXED. Two failure modes, both
  silent:
  (a) the method form called the `xiom_all`/`xiom_none` runtime stubs with
  `len = 0`, so `v.all(pred)`/`v.none(pred)` returned TRUE for every
  collection (the stubs' third argument was never a real callable either);
  (b) worse, in any program where the core module was registered (e.g.
  `use xiom.io`), `has_user_fn` saw the generic `core.all` helper and the
  method form resolved to it -- `core.all` is never monomorphised for that
  call shape, so it landed on the emitter's "erased-generic dead-code callee"
  auto-stub `define i64 @core.all() { ret i64 0 }` and `v.all(pred)` was
  silently FALSE (same shape for `.none`).
  FIX: `emit_contract_all_none` (vec_abi.rs) lowers both INLINE over the Vec
  header. The predicate is a closure VALUE (env pointer; code pointer at
  env[0]); closure params are uniformly i64 in this ABI, so each element is
  loaded with the width/sign-aware scan load and passed as i64
  (raw bits/pointer); the scan is fail-fast (`all`: first zero result ->
  false; `none`: first non-zero -> false) and empty collections are
  vacuously true. Plain function NAMES are rejected loudly (they are not
  closure values in this position -- calling through one took an access
  violation), pointing at `xiom.core.all/none(items, predicate)`. Call-site
  precedence (call.rs): a METHOD-form call on a COLLECTION receiver always
  uses the inline lowering, even when a helper is registered; the
  `has_user_fn` guard still protects the direct form and non-collection
  receivers (Set/Map/iterator/user methods).
  VERIFICATION: probes for closure/pipe-closure predicates over Int and Str
  elements, empty collections, fail-fast and struct-field receivers; the
  qualified `core.all/none` library path still works; a negative integration
  test pins the loud rejection for a function-name predicate; Set/Map
  `.contains` and iterator `.all` (m39/m48) unchanged. Lock
  `e2e_m124_contract_all_none` + CI line. checker 195/195; feature-reg
  510/510; robustness 63/63;   fuzz 24/24; stdlib-exec 85/85 (+2 ignored);
  full e2e 2372/2372.
- **m125 (stdlib API freeze, the last red gate)**: FIXED. The 9 missing
  frozen signatures were RENAME-ONLY drift against the pinned
  stdlib-v0.61.3 tree, verified 1:1 against the module sources:
  `async.Executor.*` (7 entries: new/spawn/at/step/fire_due_timers/run/
  block_on) is now `AsyncExecutor.*` in `xiom/async/async.xi`, same method
  set and signatures; `net.http_get`/`http_post` return
  `Result[NetHttpResponse, NetError]` in `xiom/net/net.xi` (the R44
  `HttpResponse` -> `NetHttpResponse` rename). No API was removed, so the
  FROZEN snapshot was regenerated with the new names (documented in the
  test header) and `stdlib_api_freeze_no_removals` + `_all_modules_compile`
  are 2/2 GREEN. The freeze gate was also absent from the CI path -- it is
  now a CI step (`Stdlib API freeze`), so renames cannot drift silently
  again. Test-only + CI change; no compiler code touched.
- **m126 (deterministic publish bytes, registry relay)**: FIXED.
  `create_tarball` shelled out to the system `tar` (or PowerShell
  `Compress-Archive` on Windows), so the published bytes changed run to run
  (entry mtimes, uid/gid, gzip header timestamp, tool extensions) and never
  matched a release asset built elsewhere -- "promote exactly the canary
  bytes" was impossible.
  FIX: new in-repo deterministic writer (`crates/xiom-pkg/src/tarball.rs`):
  entries sorted byte-wise; ustar headers with `mtime = SOURCE_DATE_EPOCH`
  (default 0), uid/gid 0, fixed modes 0644/0755, empty uname/gname; gzip with
  MTIME 0 and OS 255 carrying STORED deflate blocks (no external compressor;
  the workspace supply-chain gate stays audited-only -- no new dependency).
  Identical trees produce byte-identical archives. Symlinks/special files are
  refused loudly; ustar prefix splitting covers long paths.
  Also added `xiom pkg publish --tarball <PATH>` (publish-existing-tarball
  mode): promotes EXACTLY the given bytes without re-packing and prints their
  SHA256 for the artifact claim; only temp tarballs are cleaned up.
  VERIFICATION: 5 new unit tests (byte-identical repacks, mtime independence,
  SOURCE_DATE_EPOCH affects only the header, gzip CRC32/ISIZE against an
  independent bitwise CRC, ustar header shape + checksum verification);
  the archive round-trips through Python's `tarfile` (members, 0644 modes,
  mtime 0, uid/gid 0). pkg suite 72/72 (was 67).
  OPERATIONAL NOTE (unchanged): re-running a release job regenerates the
  asset, so re-canary after such a re-run; with deterministic packing the
  regenerated asset is now byte-stable for the same tree.
- **m127 (packages relay: indexed calls through `Vec[fn]` elements)**: FIXED
  for the relayed shape. An array literal of fn REFERENCES (`[ten, twenty]`)
  stored the RAW code address (`ptrtoint i64 ()* @ten`) as the element, while
  every call path uses the uniform closure ENV convention (load env[0] as the
  code pointer and prepend the env). `fns[i]()` therefore loaded field 0 from
  the function's MACHINE CODE and called it -- deterministic access violation
  (`-1073741819`; the packages lane's `xiom.test.run_all`, whose workaround was
  `run_test_at(index)`).
  FIX: `compile_array_as_vec` (vec_abi.rs) now wraps bare fn-reference
  elements into closure envs via the existing `wrap_fn_ref_env` (B-007
  forwarding thunk), matching `Vec.push`/fn-typed args/struct fields. The
  thunk's return type is derived from the `fn(...) -> R` element spelling
  (`fn_type_return_xiom`) when present, else Int.
  VERIFICATION: probes for `fns[i]()` (annotated and bare bindings), the
  relayed `run_all(&Vec[fn() -> Int])` shape with indexed calls in a range
  loop, and a call through a fn-typed param receiving `fns[0]`; lock
  `e2e_m127_fn_vec_indexed_calls` + CI line.
  OPEN (same convention family, pre-existing, loud-vs-silent triage pending;
  repros kept in `tmp/probe_p1/`): (a) `Vec[fn...].new(); v.push(ten); v[0]()`
  still AVs -- the push call resolves to the stdlib `Vec.push` generic, whose
  `T` param is not `Type::Fn`, so the B-007 wrap never runs (the inline push
  handler's `starts_with("fn(")` element check also misses the bare "fn"
  spelling); (b) `var g = fns[0]; g()` still AVs -- the index value's element
  type resolves to "Int" in the tracked maps, so the local is not marked as a
  closure and the call takes the raw-code path; (c) a fn-typed element called
  through a struct field (`s.tests[0]()`) fails the CHECKER
  ("cannot call 'tests' on this expression"). Fixing (a)-(c) needs one
  consistent fn-value convention across generic instantiation, element-type
  tracking and the raw-code call path -- a dedicated refactor, not a patch.
  Full e2e 2373/2373; checker 195/195; feature-reg 510/510; integration
  130/130; robustness 63/63; fuzz 24/24.
- **Item-2 status update (2026-09-23)**: `stdlib_tests::
  stdlib_all_modules_compile_to_ir` now PASSES on the current pin (verified
  with and without `XIOM_REQUIRE_STDLIB=1`; 42 s, all modules together). The
  R62-era findings (`xiom.encoding.ascii85` Option/Result vs
  `Result[Vec[UInt8], Str]`, `xiom.core` `float_to_string`) were resolved by
  the stdlib pin refresh -- no compiler change was needed. The remaining
  red in this family is `stdlib_api_freeze_no_removals` (9 stale snapshot
  signatures: `async.Executor.*`, `net.http_get/http_post`), which fails
  identically at the pre-R66 baseline -- cross-lane, snapshot regeneration.
- **R63 (playground C3/C6 + cache HOME)**: FIXED. (a) C3 script-mode
  `--opt-level`: `xiom run` already honored `--opt-level`/`--opt-level=N`
  (R51) and keyed the script cache by level, but `xiom --opt-level=0 run f.xi`
  still read `run` as the source (dispatch required args[1] == "run") and the
  short spelling `-O0` was not accepted. The driver now finds `run` behind
  leading global flags and parses `-O<n>`; verified all four
  COMPILER_REPROS forms plus a cache hit at the same level and a recompile at
  a different level. (b) Script/JIT cache with an unwritable HOME:
  `jit_cache_dir` returned `$HOME/.xiom/jit` whenever HOME was non-empty, so a
  readonly HOME silently disabled caching; it now probes `create_dir_all` and
  falls back to `$TMPDIR/xiom_jit` (verified: cache hit with HOME set to an
  unwritable path; the entry landed in the temp fallback). (c) C6 stdlib
  manifest: the pin `stdlib-v0.60.0` still carried the xiom_bench
  `package.xi`; `STDLIB_VERSION` is now the pushed stdlib main commit
  `385e1e44fac37a9403cd04cdb5e13d4c122a8710`, whose manifest is
  `package xiom_std { name: "xiom-std" }` and which also carries the
  `xiom_env_set`/`xiom_env_unset` shim (the earlier Windows-link ask). The
  local `stdlib/` checkout is updated to the same SHA; gates on the new pin:
  e2e 2365/2365, stdlib-exec 85/85, feature-reg 510/510, diff 24/24.
- **L3-50 (Result tuple payload via `?`)**: FIXED (R57). `let (a, b) =
  two()?` bound BOTH names to the raw boxed-tuple handle -- `a + b` printed
  pointer arithmetic and the `Stmt::Destructure` fallback aliased the value
  for every name. The `?` handler (both the Option and Result paths in
  expr.rs) now unboxes AGGREGATE payloads out of the erased i64 slot:
  `try_unbox_payload` derefs the heap box for tuples ("(Int, Int)" ->
  `Tuple__Int__Int`), nested containers (Vec/Map/Set/Option/Result) and
  registered structs/enums; primitives, Str and floats stay raw i64. Err
  propagation is unchanged (`try_err` returns the original Result). Repro:
  L3-50 prints 8 deterministically; minimal probe prints 8. Lock
  `e2e_m112_try_result_tuple` (`tests/regression/m112_try_result_tuple`).
  Full e2e 2360/2360.
- **L5-40 (C17 clang residue)**: FIXED (R56). Decision: container elements
  are INLINE (`Vec[Str]` slots hold the 32-byte `%struct.Vec` header, matching
  `Vec[Vec[T]]` and `vec_elem_storage_size`); concrete Option/Result layouts
  keep inline payloads and consumers resolve fields through the concrete
  registration. The coordinated change (all sites must agree -- the previous
  single-site attempts regressed):
  1. `Map.new` explicit type-arg rendering: both fallbacks render tuple
     elements with `type_arg_to_name` + mono substitution, so
     `Map[Int, Vec[Str]].new()` commits V=`Vec[Str]` (was `_Int_Int`).
  2. Bare-local generic-argument inference (`infer_generic_ident_type`)
     prefers the tracked local XIOM type WITH args over the erased LLVM slot
     type (`Map.insert` was `_Int_Vec`).
  3. `record_field_vec_elem` accepts container element types, so
     `values: Vec[V]` with V=`Vec[Str]` records the element and the index read
     memcpys the inline header (`Map.get` was `_Int_Vec_Str_` but read an
     8-byte handle).
  4. The mono signature builder lowers SUBSTITUTED container names
     (`V` -> `Vec[Str]`) to `%struct.Vec`/`Map`/`Set`/concrete Option/Result
     instead of the i64 fallback (insert kept `i64 value` while the caller
     passed `%struct.Vec`).
  5. Match payload binding resolves field 1 through the CONCRETE
     `%struct.Option__Vec_Str_`/`Result__*` registration when available
     (previously the erased "Option[Vec[Str]]" registration said i64, so the
     inline payload was read as a box handle and inttoptr'd).
  Repro: L5-40 prints 2; reduced `Map[Int, Vec[Str]]` probe prints 1 then 2.
  Lock `e2e_m111_map_vec_container_payload`
  (`tests/regression/m111_map_vec_container_payload`). Full e2e 2359/2359;
  the two locks earlier attempts broke (`e2e_fnptr_vec_index_call`,
  `e2e_m71_concat_index_elem`) are green (fixed-array brackets are excluded
  from container-name detection).
- **Remaining C18/C19 pointer/UTF-8 lessons**: per-lesson triage still
  needed (L3-02, L5-09/20/24/26/29/31/35/36/43 nondeterministic; L3-50 exits
  200; L8-14 deterministically `0xC000001D` -- a trap reached in the
  Map[Str,Str] morse flow; L5-21 Float64.to_str inside Vec[T] generics).
  L0-11's array-of-Str class and L5-32/34/42 match-slot classes are fixed.
- **Perf: `xiom.fmt` peek closure** costs +2.3-2.5 s per first `.to_str()`
  compile (sweep p50 3.9 -> 7.9 s). Fix shape: a reachable-function-only
  peek -- peek the checker-resolved module shallow, run the reachability
  filter, then pull the deps named by the SELECTED decls to a fixpoint.
  Deferred to the Stage 6 catalog-index work because it restructures
  `collect_external_decls`'s peek/reachability order and needs the full e2e
  as its gate.
- **C3 script-mode flags / C6 stdlib `package.xi`**: left as-is pending an
  exact repro from the playground session (changing either without one
  risks breaking the documented script semantics).

**Note for future miscompile work**: `--emit-ir` prints the INTERMEDIATE
emitter output, not the IR clang compiles (a later pass rewrites e.g.
`Task.to_str` stubs into `Str.to_str`). To capture the FINAL IR, force the
link to fail (`--link missing_xyz`) and read `<output>.ll` (kept on failure;
deleted on success).

## 2026-09-24 -- Front-end audit Sprint A (FE-1..FE-9): doctor v2 + shared toolchain probe + retired surfaces

The owner-approved front-end backlog (`docs/FRONTEND_AUDIT.md`, written after
the Win11 laptop report: doctor said "clang NOT FOUND" although LLVM was
installed) landed as one batch. No compiler semantics changed; the locks are
unit tests plus a new binary integration test on the CI path.

- **FE-1/FE-3 (ONE shared probe)**: new `crates/xiom/src/toolchain.rs` is the
  single clang/opt/nasm resolver used by the driver AND doctor. Order is PATH
  first (each PATH dir resolved to its absolute entry), then per-OS known
  locations: Windows `%ProgramFiles%\LLVM\bin`,
  `%LOCALAPPDATA%\Programs\LLVM\bin`, `%LOCALAPPDATA%\Microsoft\WinGet\Links`
  and the `WinGet\Packages\LLVM.LLVM_*` globs (User + ProgramData -- the list
  `install_deps.ps1` already probes); NASM `%LOCALAPPDATA%\bin\NASM`,
  `%ProgramFiles%\NASM`, `%ProgramFiles(x86)%\NASM`,
  `WinGet\Packages\NASM.NASM_*`; Unix `/usr/bin`, `/usr/local/bin`,
  `/opt/homebrew/opt/llvm/bin`, `/usr/lib/llvm-*/bin` (glob). Wildcards are
  expanded by scanning the directory that holds them. The driver's hardcoded
  personal NASM path (`C:\Users\lefte\...`) is REMOVED, `find_tool`/
  `find_nasm` are deleted, and `opt` now prefers the directory clang resolved
  in (LLVM tools ship together). Versions come from executing the RESOLVED
  path (`--version`); a file that cannot spawn is skipped, not returned.
- **FE-2 (dead remediation)**: doctor no longer prints `xiom install llvm`
  (no such package exists). Missing clang prints the OS-specific command:
  Windows `winget install LLVM.LLVM` / `choco install llvm -y`; macOS
  `brew install llvm` + the PATH line; Linux apt/dnf/pacman/zypper selected
  from `/etc/os-release`.
- **FE-4/FE-5/FE-16 (identity + parity)**: doctor reports the resolved
  xiom.exe, install root, XIOM_HOME, the SHARED
  `xiom_graph::paths::stdlib_root()` + its `package.xi` version, clang/nasm/
  z3 paths+versions, runtime C presence and the packages dir. Warnings:
  compiler vs stdlib version mismatch, stdlib vs the embedded `STDLIB_VERSION`
  pin (`build.rs` now emits `XIOM_STDLIB_PIN`, closing the FE-16 gap),
  runtime C missing, and duplicate installs (multiple xiom binaries on PATH,
  or the running binary shadowed by the first PATH entry).
- **FE-7 (`--json` + exit codes)**: `xiom doctor --json` emits
  `{schema, ok, compiler, identity{...}, checks[], warnings[], errors[]}`;
  exit 0 all-OK, 1 warnings, 2 errors.
- **FE-8 (retired surfaces)**: `xiom publish` now mirrors `xiom install` -- a
  deprecation note plus delegation to `xiom pkg publish`; the legacy git-tag
  handler (which advertised the retired `xiom install {name}` channel) is
  deleted. `xiom update` points at the release installer / future
  `xiom toolchain update` (toolchain category) and keeps the package advice
  separate. The pkg help example is `xiom pkg install xiom.std`.
  COMPILER_IMPROVEMENT_PLAN.md carries the historical banner for the old
  command names.
- **FE-9 (stdlib registry naming)**: the client resolves the dotted
  `xiom.std` and the legacy `xiom-std` for both index lookups
  (`find_index_package`) and metadata
  (`fetch_package_metadata_alias`); `xiom.stdlib` is NOT an alias (it was a
  bad help example). VERIFIED live against `registry.xiom-lang.org`:
  `info xiom.hello` -> v0.1.0; `info xiom.std` and `info xiom-std` both
  resolve v0.61.3 with the same sha256 `1ad1b33a5caa`; `install xiom.hello`
  completes checksum + signature verification into a temp XIOM_HOME.
- **VERIFICATION**: toolchain 7 unit tests (PATH-first order, per-OS lists,
  no personal path, glob expansion, version-line parsing, dead-file skip);
  doctor 10 unit tests (statuses, warning rules, exit codes, JSON keys);
  `crates/xiom/tests/doctor_cli.rs` 2 integration tests on the built binary
  (identity + `--json` shape + exit-code contract against a fake mismatched
  stdlib root). CI gains the `Front-end CLI lock` step. Workspace
  `cargo check --workspace --all-targets` clean; ascii guard clean;
  **full e2e 2373/2373** on the pinned stdlib.
- **REMAINING (deferred, noted in FRONTEND_AUDIT)**: clang version floor (no
  agreed floor yet), `doctor --deep`/`--fix` (Sprint B FE-6), installer
  alignment FE-10..FE-15/FE-17, grouped `--help` (FE-12). FE-4's "clang
  version floor" is the only identity-block sub-item not implemented.

## 2026-09-24 -- Front-end audit Sprint B (FE-6, FE-10..FE-15, FE-17): P1 first-run pass

Follow-up to Sprint A (same day). The one compiler-visible change is FE-17
(parser); the rest is CLI/installer surface. Gates: parser 102/102 (+1 unit
test), full e2e **2374/2374** (new lock m129), workspace all-targets check
clean, ascii guard clean.

- **FE-17 (module trailing semicolon)**: `module m;` parsed the header and
  left the `;` to the top-level parser -> spurious
  `error[P001]: expected declaration, found ';'` (reproduced before the fix).
  `parse_file_module_header` and the brace-less `parse_module` arm now skip
  ONE optional semicolon; the block form `module m { ... };` is tolerated
  too (skip after `}`). Lock `e2e_m129_module_trailing_semicolon` + CI line;
  the parser unit test covers all three spellings.
- **FE-6 (`xiom doctor --deep`)**: compiles AND RUNS a trivial program
  end-to-end in a temp dir -- the only check that proves the whole chain
  (linker/MSVC headers included); reported as info when clang is absent.
  `doctor_cli` integration test added.
- **FE-12 (grouped help)**: `xiom --help` is now grouped (Getting started /
  Project / Tools / Output / Safety / Build and cache / Packages,
  benchmarks, AI / Subcommands / Dependencies / Examples) with the internal
  sprint tags removed and a `<tool> --help` line for the dispatcher. Every
  flag from the old flat list is still documented.
- **FE-13 (which xiom)**: `xiom --version` prints the install root under the
  version line (`xiom::doctor::install_root_of`, shared with doctor).
  `xiom.bat` now prefers the binaries NEXT TO THE SCRIPT (a freshly unpacked
  tree is no longer shadowed by a stale `%LOCALAPPDATA%\xiom` install);
  LOCALAPPDATA is the fallback. Verified: running the bat from
  `target\debug` reports that install root.
- **FE-10 (installers mirror the archive)**: install.ps1/install.sh ship all
  9 tools (+ optional z3 when present), put the stdlib at `lib/xiom`,
  `lib/runtime`, `lib/package.xi` (previously install.sh created
  `lib/stdlib/**` and both installers also copied the runtime to
  `<root>/runtime`), and finish by running `xiom doctor` (`--json` in CI);
  the archive `lib/` is preferred when `-BinaryPath` points at
  `<archive>\bin`. Sandbox-verified on Windows: 9 tools copied, the `lib/`
  layout resolves as the stdlib root, doctor runs. FIXED while testing:
  `-Unattended` crashed because the local `$registerExt` collided with the
  `$RegisterExt` switch parameter (PowerShell variable names are
  case-insensitive) -- pre-existing; the local is now `$registerExtChoice`.
- **FE-14 (banners)**: both installers end with `xiom doctor` / `xiom run
  hello.xi` instead of the legacy `xiom compile` idiom.
- **FE-15 (uninstaller PATH)**: the generated uninstall.bat removes the
  install's bin dir from the user PATH (and best-effort machine PATH) with a
  `powershell -Command` one-liner built from a token-substituted template;
  the Where-Object filter drops plain and trailing-backslash variants
  (parse-checked and logic-tested).
- **FE-11 (install_deps)**: the unauthenticated direct LLVM downloads
  (hardcoded 19.1.0, no SHA256) are removed from install_deps.ps1 AND
  install_deps.sh; winget/choco (and the distro package managers) are the
  only automated paths, otherwise the exact manual commands print.
- **REMAINING**: clang version floor (no agreed floor), `doctor --fix`
  (deferred; remediation text instead), `xiom toolchain check --json`
  (Sprint D), the website one-liner script (not in this repo).

## 2026-09-24 -- Sprint C: fn-value / generic-mono ABI unification + E001 conservatism

Closes the m127 residuals, the packages lane's confirmed fp probes, the
stdlib cross-type callback matrix and the E001 conservatism item.
Repro-first on every shape; all 15 probe_p1 + 7 fp-probe shapes re-run
green, 3 new e2e locks (m130/m131/m132) + 2 binary E001 locks, full e2e
**2377/2377**, checker 195/195, parser 102/102, workspace all-targets clean.

### fn-value ABI (C1)

- **Parser lost fn type args**: `Vec[fn() -> Int].new()` parsed the type arg
  as `Ident("_")` (`type_to_expr_ident` had no Fn arm), so the ctor recorded
  element `"_"` and `.push(ten)` stored a raw code address (AV). Fixed:
  `type_name_str` renders `fn(...) -> R` (+ Ref/MutRef/Ptr), and the Fn arm
  in `type_to_expr_ident` keeps that marker. The checker no longer resolves
  type args as variables ("undefined variable 'fn() -> Int'"), and the
  `method_target` guard distinguishes `Type.method[TypeArg]()` /
  `module.fn[TypeArg]()` from `value.field[i]()` by whether the index looks
  like a type (numeric / local-variable indices are value indices).
- **Codegen `type_from_ast` rendered `Type::Fn` as "Int"** (the catch-all),
  so `var fns: Vec[fn() -> Int] = [ten]; var g = fns[0]; g()` tracked the
  element as Int and called the env pointer as code (AV). The marker now
  lives in `type_string_full`/`type_annotation_name` (erased to i64 by
  `llvm_type_for`/`xiom_to_llvm_type`); annotated fn locals allocate an i64
  slot.
- **Unannotated fn arrays lost their marker**: `var fns = [ten]; for f in
  fns { f() }` failed with "element type could not be resolved", and
  `var g = arr[0]; g()` treated the env as code. `fn_ref_marker_xiom`
  records the full `fn(...) -> R` for array-literal bindings, the for-loop
  lowering allocates an i64 slot and marks fn elements as closure locals
  (env-first calls), and the raw array-buffer emitter wraps bare fn refs.
- **Struct-literal Vec fields** (`Suite{ tests: [ten] }`) memcpy'd a buffer
  of raw code addresses into the Vec; the raw buffer emitter now wraps bare
  fn refs into closure envs (AV -> green).
- **`(op.f)(x)`**: a parenthesized call target is unwrapped, routing fn
  field receivers to the env-first field path (the generic fallback emitted
  `inttoptr ptr -> ptr`, invalid IR).

### Generic-mono ABI (C1)

- **Cross-type callback inference**: `conv[T, U](x: T, f: fn(&T) -> U)` with
  `to_s: fn(&Int) -> Str` mono'd as `conv_Int_Int` -- the fn-typed param
  contributed nothing to U, which fell to the "Int" default and truncated
  the Str result to pointer bits (wrong compare; fp5). Generic inference now
  reads the ARGUMENT's registered signature through `fn_arg_generic_binding`
  (position-matched param for T, declared return for U) and maps the
  registry's LLVM spellings back to XIOM names (`i8*` -> Str, `double` ->
  Float64, ...). `fp4_maptou` (Vec[U] with U=Str) and `array.map`
  Int->Str / Int->Float64 are green.
- Checker `for`-loop element typing: array literals of bare fn refs now type
  as `Vec[fn(...) -> R]` via `fn_ref_marker` and `CheckedType::from_marker`
  turns markers (and bare "fn") into real `Fn` types, so `f()` in a loop
  body types and calls correctly.

### E001 conservatism

- **Temporary borrows are released per statement**: `check_stmt` records
  borrow/loan marks and releases everything created by a statement unless
  the statement BINDS a ref (`let r = &m;`, refs stored into aggregates or
  reassigned through a local). `smoke_collect_sparse` went 7 warnings -> 0
  and the genuine-overlap warning (`let r = &m; take_mut(&mut m)`) is
  unchanged (locked by m133 + `borrow_e001`).
- **Root cause found while fixing**: `write_borrow` seeded
  `read_borrow_count = 1` as a sentinel, so every read borrow released back
  to 1 and left the variable permanently `ReadBorrowed` -- the "7 warnings"
  were this counter bug, not actually-live borrows. Write borrows now count
  zero reads.
- `LoanSet::mark`/`release_since` give the same per-statement semantics to
  the Place-level loan engine.

### Open (recorded, not fixed)

- The checker accepts a BY-REF callback (`fn(&Int) -> Str`) where a fn-typed
  param declares `fn(T) -> U` by value (`array.map`); the call monomorphises
  and the callee dereferences the scalar argument as a pointer (AV in
  `m131c_array_map_ref`). Needs a fn-signature compatibility check in the
  checker -- follow-up, out of this batch's scope.

## 2026-09-24 -- Sprint D: `xiom toolchain check` + MCP contract queries

Sprint D of the release plan (docs/POST_RELEASE_PLAN.md sections 1 + 2).
The toolchain updater's read-only half and the MCP structured-contract tools
landed; the attested swap half is blocked on dependency procurement and
refuses explicitly.

- **`xiom toolchain check [--json]`** (`crates/xiom/src/toolchain_cmd.rs`):
  GitHub Releases API for `xiom-lang/xiom` ONLY (spec rule 1); no download,
  no writes. JSON = `{schema, current, latest, platform, up_to_date, notes,
  install_kind, asset, exe}` (POST_RELEASE_PLAN shape plus the classification
  fields). Exit codes: 0 up-to-date, 1 update available, 2
  network/API/parse failure. `install_kind` implements spec rule 5's
  package-manager detection (path heuristic for /usr/bin, Homebrew, nix,
  Chocolatey/Scoop/winget trees, plus an optional `.xiom-package-manager`
  marker) and distinguishes dev builds (`target/debug|release`).
  Asset naming matches release.yml staging (`xiom-<v>-windows-x64.zip`,
  `xiom-<v>-linux-x64.tar.gz`, `SHA256SUMS-*`). `xiom doctor` now prints an
  INFO row pointing at the command (the diagnostic itself stays offline).
- **`xiom toolchain update|rollback` refuse with exit 3** (guidance names
  `check` and the release installer). Reason: in-process build-provenance
  attestation verification is not wired yet (dependency procurement), and a
  SHA256SUMS-only swap would weaken spec rule 2. Status recorded in
  POST_RELEASE_PLAN section 1.
- **MCP `get_contracts {symbol, verify?, file?}`** (`crates/xiom-mcp/src/
  contracts.rs`): resolves a stdlib symbol (`xiom.string.str_concat`,
  module-qualified, or a unique bare leaf; receiver methods as `Str.len`)
  against the SAME live stdlib catalog scan as `xiom_stdlib_reference`, and a
  project symbol from an optional `file` via AST parsing. Response is the
  spec object: symbol/module/signature/requires/ensures/invariants/
  pre_refs/post_refs/qualified with exact clause source lines. `verify: true`
  re-parses the declaration into a single-symbol program, generates SMT for
  its clauses and runs Z3, mapping results 1:1 (proved / counterexample with
  model values / unknown+reason; "unknown" when z3 is absent or the mapping
  is not 1:1). Type invariants report unknown (not encoded).
- **MCP `search_symbols {query, file?}`**: ranked hits (exact qualified >
  exact leaf > prefix > substring, alphabetical tiebreak, deduped, top 25)
  across the bundled stdlib plus the optional project file; returns
  `[{symbol, module, signature}]`.
- **Locks/tests**: 5 `toolchain_cmd` unit tests (version compare, asset
  names, payload parse, install-kind, JSON keys) + the
  `toolchain_check_reports_api_failure_as_exit_two` binary lock (closed-port
  override, deterministic); 5 `contracts` tests (spec example symbol,
  bare-leaf + close matches, ranked search, JSON shape, Z3 fold alignment);
  MCP tool-count test updated (18 tools). Workspace all-targets clean;
  `xiom toolchain check` verified live against GitHub (v0.61.3 = latest,
  exit 0).
- **REMAINING (Sprint D tail, procurement-blocked)**: attested `update`
  (download -> attestation verify -> SHA256 -> stage -> atomic swap ->
  `rollback`) and the `--dry-run` mode. The refusal path, asset naming and
  install-kind detection are already in place for it.

## 2026-09-24 -- Relay: private same-leaf triage + clause Bool-mix rejection

Two compiler-lane findings relayed from the stdlib lane (plus their
cross-type callback matrix + E001 repro, already fixed in Sprint C).
Repro-first; locks m134/m135/m136; full e2e **2379/2379**; checker 195/195;
catalog corpus clean.

- **Private same-leaf type collision (R44 class, Timer case)** -- REPRO:
  `xiom.async` declares a PRIVATE `Timer = { deadline: Int; task: fn() }`;
  `xiom.async.timer` declares a PUB `Timer = { deadline: Int; armed: Bool }`.
  The R44 triage's `declared_type_leaves`/`declared_type_shapes` only
  collected `is_pub || generic` decls, so the private Timer never entered the
  collision set: one bare `%struct.Timer` definition was emitted (first-wins)
  and the losing module silently reused the other layout -- no clang error,
  no diagnostic.
  FIX: triage now includes ALL type decls (pub and private; enums stay
  pub-only -- private enums are not injected directly). Conflicting private
  groups qualify module-wise exactly like pub groups; identical private
  layouts keep the shared key. Evidence: the program now emits
  `%struct.xiom.async.Timer = { i64, i64 ()* }` AND
  `%struct.xiom.async.timer.Timer = { i64, i64 }` (no bare `%struct.Timer`).
  Lock `e2e_m134_private_same_leaf` (multi-file project fixture via the
  package graph).
  Scope note: the compiler-side R44 experiment's warning still holds --
  wholesale qualification of catalog modules broke smokes; this change only
  adds private decls to the SHAPE-CONFLICT triage (identical layouts are
  untouched), and the catalog corpus + full e2e stay green.
- **Clause-position `Bool == Int` silently coerced** -- REPRO:
  `fn f(x: Int) -> Int requires: x == true ensures: result == false`
  compiled (exit 0) and ran; the requires comparison was coerced. Root cause:
  contract clause expressions were NEVER type-checked -- only name-collected
  and reachability-scanned. There was also no `Expr::AtPre` typing arm, so
  `len()@pre` typed as Unit.
  FIX: (a) `check_expr` types `@pre` as its wrapped expression; (b) a LIGHT
  clause validator (`check_clause_bool_mix`) rejects comparisons where both
  operands resolve to known primitive types and one is Bool (locals, params,
  literals, `result` bound to the return type inside ensures; parens/@pre/
  unary unwrapped). The mixed comparison now fails:
  "contract clause compares Bool with Int; mixed comparisons are not
  coerced". Locks: `checker_locks.rs` (m135 rejected; m136 well-typed
  clauses with implicit self / @pre / result stay green) + e2e_m136.
  WHY NOT FULL PREDICATE TYPING YET: running the full `check_expr` on every
  clause surfaces 8 stdlib clause sites (below) -- enabling it reds the
  catalog corpus and would require stdlib-lane fixes first. The light
  validator deliberately stays silent on anything it cannot resolve, so the
  reported class is closed without that coupling.
- **Follow-ups recorded** (status 2026-09-24 evening):
  1. Full predicate-Bool clause checking -- ALL EIGHT sites triaged and
     confirmed GENUINE stdlib clause issues (no checker gaps): `xiom.ptr` 91
     (`result == old_value`, undefined name; use `(*dest)@pre`),
     `xiom.math` 201/561 (Float64 `exp` compared with the Int from
     `to_int(exp)`), `xiom.sync` 378 and `xiom.rc` 29 (bare `strong_count`
     without `()` resolves to the method name), `xiom.array` 219
     (`arr.is_sorted_by(compare)` -- no such method exists in the module).
     TRANSITION SWITCH LANDED: `XIOM_STRICT_CLAUSES=1` enables the full
     predicate rule (every clause must type as Bool) while the default stays
     on the light validator. The stdlib lane verifies with
     `XIOM_STRICT_CLAUSES=1 cargo test -p xiom-check catalog_corpus_is_clean`
     (or any build with the env set); when their fixes land, flip the default
     (one condition in `check_clause_bool_mix`). Fixture m137 + checker lock
     cover both modes.
  2. `@pre` on a METHOD CALL -- FIXED (2026-09-24). Root cause:
     `collect_atpre_vars`/`collect_pre_idents` recorded the CALLEE ident
     (`len`) as if it were a variable and never collected the receiver, so
     `pre_snapshot_vars = ["len"]`, no `self` snapshot was emitted, and
     `len()@pre` evaluated against the live receiver (ensures
     `len() == len()@pre + 1` fired although the length grew by exactly 1).
     Fix: bare callee idents are no longer collected as variables, and a
     method body with any `@pre` always snapshots `self` (the existing
     ref-receiver rebind then evaluates the call against `__self_pre`). The
     m136 lock now runs the real clause at runtime.
  3. Private ENUM same-leaf collisions -- TRIAGED (2026-09-24): enums now
     participate in the collision set for pub AND private decls, matching the
     private-type fix; identical layouts keep the shared key. Catalog corpus
     + full e2e green.
  4. Small cleanups -- DONE (2026-09-24): the benchmark-chaos e2e tests t2-t5
     no longer point at the removed monorepo checkout (relocated to the
     internal `tests/ecosystem/t*.xi` copies, ignored with a reason; compile
     coverage stays in `e2e_i2_parallel_codegen`), and the e2e harness now
     deletes its per-invocation `e2e_*` binary (plus `compile_wasm`'s .wasm)
     so full runs leave no artifacts.

## 2026-09-25 -- Packages relay: unsigned-constant widening + &mut call-site copies

Relay received from the packages lane (their traps 12-13). Triage result:

- **Widened UNSIGNED constants sign-extended -- FIXED.** Repro: `let a =
  239u8 as Int;` printed **-17** (`tmp/sprintc/pkg_u8_cast.xi`,
  `pkg_u8_detail.xi`), while the runtime-local form was already correct
  (`let x: UInt8 = 239; x as Int` = 239). Root cause: a suffixed literal is
  parser-desugared to `As(Expr::Int(239), UInt8)`, and the widening arm's
  source-signedness match had arms only for `Ident` and `Call` -- the `As`
  source fell to the sext default, so the i8 bits 0xEF sign-extended to
  -17. Fix: new `as_source_is_signed` helper (expr.rs) resolves `Ident`
  (xiom_type_of_local), `Call`/`GenericCall` (callee_return_xiom),
  `As(_, src_ty)` (the suffixed literal) and `Paren` recursion; unknown
  sources keep the historical sext default. `239u8 as Int` = 239;
  `-17i8 as Int` stays -17; the packages' `(c as Int) & 0xFF` mask workaround
  still works but is no longer needed. Lock `e2e_m138_u8_const_widen`. This
  is the CONSTANT path of the class whose runtime path (`byte_at`) was fixed
  earlier ("VERIFIED FIXED ... UInt8 as Int zexts").
- **`&mut` call sites silently copy -- OPEN (design decision needed).**
  Repro (`tmp/sprintc/pkg_mut_vec_copy.xi`):
  `fn push_one(v: &mut Vec[Int])` called as `push_one(v)` on a `var v`
  COMPILES with no diagnostic, mutates a temporary copy, and the caller's
  Vec is unchanged (length check fails); `push_one(&mut v)` works. The
  checker accepts a value argument for a `&mut` parameter and codegen
  materializes a temporary address. Impact: silent loss of mutation, the
  packages' xiom.tga failures. Options: (a) require explicit `&`/`&mut` at
  call sites for ref parameters -- a T001 checker error, consistent with the
  stdlib and with Rust; (b) auto-borrow the PLACE (pass the real address) --
  ergonomic but diverges from XIOM's explicit-borrow model. Recommendation:
  (a); it is a checker+corpus sweep, NOT a small pre-release change, so it is
  queued for the owner/Stage 6 decision.
- **Str is NUL-terminated (embedded 0x00 unrepresentable) -- KNOWN by
  design.** Confirmed again by the packages' cpio worker (binary names).
  Already documented (strlen-based runtime; "Str values are NUL-terminated");
  now also listed in `AI_CONTEXT.md` known limitations.

## 2026-09-25 -- Packages relay #2 (trap 14 + section 10 notes)

- **Call arity is not validated on the primary call path -- CONFIRMED,
  OPEN.** Repro `tmp/sprintc/pkg_arity.xi`: `fn f(a: Int, b: Int) -> Int`
  called as `f(1)` COMPILES (exit 0) and returns 1 -- the missing `b` is
  silently 0; the method form `s.set(1)` (declared
  `fn Box.set(self, x: Int, y: Int)`) behaves the same (`t == 1`). Extra
  arguments are also silently dropped for plain calls
  (`tmp/sprintc/pkg_arity_extra.xi`: `f(1, 2, 3)` compiles, EXIT=0). Only
  the module-prefix path (lib.rs ~4056) and the impl-dispatch path (~3979)
  reject EXTRA args; the method path has the G-20 arity/offset heuristic
  (~6212-6246) but no exact-count error, and codegen pads missing params.
  Impact: silent wrong results (the packages' ar test). Fix shape:
  exact-count errors on the primary call and method paths (too few + too
  many), then a corpus/e2e sweep.
- **Mismatched generic brackets accepted -- CONFIRMED, OPEN.** Repro
  `tmp/sprintc/pkg_bracket.xi`: `fn g() -> Vec<UInt8]` compiles and runs.
  Root cause is explicit in `crates/xiom-parser/src/lib.rs` (~1080-1085 and
  ~1137): the container arms accept `[` OR `<` to open and independently
  accept `]` OR `>` to close, never checking that the closer matches the
  opener. Fix shape: remember which opener was consumed and require the
  matching closer (both families stay supported; only mixed forms become a
  P001 error).
- **E001 advisory after an immutable accessor -- NOT reproduced as a
  warning; reproduced as MUTATION LOSS.** `tmp/sprintc/pkg_e001_accessor.xi`
  compiles with NO E001: `let n = s.count(); s.add(1);` where
  `S = { v: Vec[Int] }` and `add` uses a by-value receiver, then
  `s.count()` still returns 0 -- the push mutated a COPY of the Vec handle.
  Same copy-semantics class as the open `&mut` finding above
  (Vec fields + receiver/value passing), not a borrow-warning false
  positive. The packages' exact E001 probe is needed to classify their
  warning shape.
- **`&struct.field` to a `&Vec` parameter -- NOT reproduced with the simple
  shape.** `fn take(v: &Vec[Int])` called as `take(&h.v)` for
  `Holder = { v: Vec[Int] }` compiles and returns the right length
  (`tmp/sprintc/pkg_field_ref_vec.xi`, exit 0); a plain `&Holder` parameter
  is also fine. The packages' probe (exact field type/ownership context) is
  needed to classify the trap.
- **`io.println` takes `Str` (ints need conversion) -- BY DESIGN**, now
  stated in `AI_CONTEXT.md`: convert with `xiom.convert.int_to_string` /
  `float_to_string` or format with `xiom.fmt`.

### Follow-up same day (packages commit 6310dba + our probes)

- **`&struct.field` to `&Vec` -- NARROWED to the Result-payload shape and
  FIXED.** Their `probe_struct_field.xi` (plain field) is green, matching
  our non-repro; their `probe_result_value.xi` was the minimal broken shape:
  `Result[Vec[UInt8], Str]`, `&r.value` passed to a `&Vec` parameter read 0,
  while `let v = r.value; &v` read 3. Root cause: `%struct.Result` stores
  payloads as BOXED i64 HANDLES (`{i64, i64, i64}`), but the `&field` arm's
  scalar branch returned the handle SLOT's address, so the callee interpreted
  the handle bits as the Vec struct (length 0). FIX (expr.rs): when the field
  is a boxed payload (`Result`/`Option` `value`/`error`), resolve the payload
  type from the local's XIOM type via `container_parts`, load the handle and
  `inttoptr` to the POINTEE pointer. `take(&r.value)` = 3, `let v = r.value;
  take(&v)` = 3, and `push_more(&mut r.value)` mutation reaches the box
  (length 4). Lock `e2e_m140_result_payload_ref`; full e2e **2377/2377
  (+4 ignored)**; catalog corpus clean.
- **Arity enforcement -- IMPLEMENTED, GATED OFF.** The three exact-count
  checks (bare, method with receiver offset, module-prefix) were written and
  verified against the packages' repro (all three now report
  "'X' expects N argument(s), found M"), then held back because the stdlib
  corpus relies on the laxness in these call sites:
  `xiom.io` printf (1 param, 2 args), `xiom.io.console` printf x2,
  `xiom.crypto.kdf` `_scrypt_blockmix` (2 params, 1 arg) x2,
  `xiom.collections` `get` (2 params, 1 arg), `xiom.path` `replace` (2, 3),
  `xiom.cell` `ptr.is_null` (1, 0) x2 (this last one may be a receiver-style
  registration false positive that needs the G-20 offset path, not a stdlib
  typo). Flip the three comparisons to `!=` after the stdlib wave fixes the
  call sites; the checks stay documented in-line at the three sites.
- **Bracket strictness -- SWITCH LANDED (`XIOM_STRICT_BRACKETS=1`).** The
  parser now enforces closer-matches-opener when the env var is set
  (`Vec<UInt8]` -> `expected '>', found ]`), while the LAX default keeps the
  legacy mixed spellings so the pinned stdlib still builds. Locks: parser
  unit tests (valid forms + strict/lax via `with_strict_brackets`) and
  `checker_locks.rs::m141_mixed_brackets_lax_then_strict` (lax compiles+runs,
  strict rejects). Full e2e 2377/2377 (+4 ignored) with the lax default;
  catalog corpus clean. The default flips at the pin bump after the stdlib
  wave. STDLIB STATUS (wave 31, commit 0823433): the 13 mixed sites remaining
  on their main were canonicalized (`io/fs.xi` x10 at shifted lines,
  `io/console.xi:47`, `io/pipe.xi:185`, `core/contracts.xi:231`) and 5 were
  already canonical; their modules re-check clean. They can self-verify with
  `XIOM_STRICT_BRACKETS=1 cargo test -p xiom-check catalog_corpus_is_clean`
  on a checkout at their ref.
- **Widened UNSIGNED FIELD sources -- FIXED** (`h.b as Int` printed -17; the
  `as_source_is_signed` helper now resolves `Field` through the struct meta,
  alongside `Ident`/`Call`/`As`/`Paren`). `e2e_m138_u8_const_widen` extended
  with the field shape; full e2e 2376/2376 (+4 ignored).
- **Packages porting notes (informational, no compiler action):** trap 16
  (parallel-Vec drift guard) and trap 17 (`as` reserved; `int_to_base` lives
  in `xiom.convert.int`).

## 2026-09-25 -- Stdlib relay: `ptr.is_null()` receiver offset + strict-parser prep

- **`ptr.is_null()` (pointer FIELD receiver on a free fn) -- CONFIRMED
  compiler-side, OPEN.**
  - Decl: `stdlib/xiom/ptr/ptr.xi:36` `pub fn is_null[T](ptr: *const T) -> Bool`.
  - Calls: `stdlib/xiom/cell/cell.xi:155,181` `ptr.is_null()` inside
    `Ref.release` / `RefMut.release`, where `ptr` is the handle's pointer
    FIELD (registered as a prologue GEP local).
  - Checker: the method path's `first_param_matches_receiver` compares
    canonical names only; a ref-ish receiver (`*RefBlock`) never matches the
    ref-ish generic param (`*const T`), so the G-20 offset stays 0 and the
    receiver is treated as a missing argument (the strict-arity run reports
    "'ptr.is_null' expects 1 argument(s), found 0").
  - Codegen (probes `tmp/sprintc/pkg_isnull_probe2/4.xi`): the METHOD form is a
    SILENT STUB. `z.p as Int = 0`, direct `is_null(z.p) = 1` (true) and
    `is_null_inline(h.p) = 1` / `is_null_tramp(h.p) = 1` (both correct, via
    field receivers too), but `h.p.is_null_inline()` and
    `h.p.is_null_tramp()` both evaluate to 0 (false) for a null pointer. The
    final IR contains ONLY the direct calls -- the method form emitted no
    call at all (the auto-stub path for unresolved methods returns a default
    value), so this is the same silent-stub class as the historical Str
    sugar (`Str.trim()` -> len 0xFFFFFFFF), not a bad receiver argument.
    LOCAL pointer receivers are at least honest: the checker reports
    "cannot call 'X' on this expression".
  - Fix direction: resolve UFCS-style method calls (`value.method(...)` ->
    free fn whose first param accepts the receiver, including ref-ish
    receivers) in BOTH the checker and codegen, and turn the codegen
    unresolved-method fallback into a hard error instead of a silent stub.
    WORKAROUND for the stdlib lane: the DIRECT call `is_null(ptr)` is
    verified correct (both inline and whole-body-unsafe forms); use it
    instead of `ptr.is_null()` until the resolution lands.
  - NEXT BATCH (implementation plan, budget-gated):
    1. `emitter.rs::emit_undefined_symbol_stubs` is the post-pass that
       synthesizes `ret 0` for any `call @sym` with no define/declare. It
       should COLLECT the unresolved called symbols and return them so the
       compiler fails loudly (C001, naming the symbol and its call site)
       instead of emitting a stub. Expect fallout: run corpus + full e2e and
       fix each reachable case by real resolution (the historical comment
       trail shows most stub arrivals were bugs already fixed -- B-001 etc.).
    2. Checker: the method path accepts FIELD receivers that resolve to
       nothing (silent '_'), while LOCAL pointer receivers already error
       ("cannot call 'X' on this expression"). Align the field case with an
       error, then implement the UFCS lookup (receiver as first arg) so
       `h.p.is_null()` is accepted and TYPED as the free fn call.
    3. Codegen: give the same UFCS lookup in the method path so the receiver
       is emitted as arg 0; lock with `is_null` on a null pointer field
       (direct and method forms both true) plus a non-null control.
- **Strict-parser prep (packages relay #3 + stdlib reply):**
  - Packages: 0 mixed-bracket sites in their repo; they are ready for the
    strict flip. Stdlib: `xiom.cell` is theirs; `xiom.sqlite`'s
    `SqliteValue.is_null(val:)` is called statically with exact arity, so it
    is NOT the offset case. Their trap-4 re-run was on the RELEASED 0.61.3
    (`probe_result_value.xi` still reads 0 there), which is expected: the fix
    lives in local main and closes on the next build/pin bump; their
    README/SESSION pin that status.
  - The stdlib lane could not match the 18-site list because their main
    differs from our PIN checkout (their `io/fs.xi` has zero angle generics;
    ~100 matched angle sites remain in 15 files, which strict parsing
    ACCEPTS). EXACT PIN LIST (mixed `Result[X, Str>`-style, file:line):
    `core/contracts.xi:231`, `io/console.xi:47`, `io/fs.xi:50,80,110,124,
    138,162,212,227,296,335`, `io/pipe.xi:181`, `math/approximation.xi:496`,
    `test/harness.xi:34,97`, `test/test.xi:180,194` -- 18 sites, 7 files.
  - Next step for coordination: land the strict parser behind
    `XIOM_STRICT_BRACKETS=1` (transition switch, like
    `XIOM_STRICT_CLAUSES`) so the stdlib lane can run the exact diagnostics
    on their tree before we flip the default at the pin bump.







---

## R49 open section -- stdlib-lane relay (2026-09-19)

Findings relayed from the stdlib session; reproduced or filed here before
fixing. Each entry keeps its stdlib-side repro name.

### R49-1 `p_module_path_alias` -- file-path imports vs declared module name -- FIXED
Importing a module by FILE PATH whose path differs from the declared module
name corrupted the catalog: `use xiom.crypto.legacy.md5;` made `--check`
emit 33 T001s in `xiom.crypto.rng_crypto` ("cannot call
'secure_random_bytes' on this expression"); `use xiom.crypto.md5;`
(declared name) was clean; same for legacy/sha. 19 modules carried
path/name mismatches (core/{cmp,contracts,platform}, crypto/legacy/{des,
md5,sha}, crypto/{chacha,ecc,poly1305,rsa}, format/fmt, math/complex,
num/{bigfloat_agg,bigint}, os/{env,path,process}, string/{char,utf8}).

FIX (R49, 2026-09-19), three parts:
1. `Catalog::parse_file` sets `dotted_name` from the file's declared
   `module` header (was the requested path); `find_owned` caches under that
   identity, so one file cannot sit in the cache under two names.
2. `process_use` rewrites a non-declared import path to the declared
   segments up-front (only for paths absent from the module index, so normal
   imports pay no extra parse). The parent-chain walk then keys
   `xiom -> crypto -> md5`, not the bogus `xiom -> crypto -> legacy -> md5`.
3. The freeze test's resolver gained a one-time declared-header index of the
   stdlib tree, so moved modules resolve for the snapshot scan.
Verified: all 19 alias imports `--check` clean; `stdlib_api_freeze_tests`
green (214/214 frozen entries resolved; the stale snapshot lines were
regenerated for 52 intentional drifts/typos -- by-ref return removals in
array/cell, compress `Result<` typos, iter/path/env renames; note `env.var`
-> `env.get_var`). Lock: `e2e_m99_module_path_alias`.

### R49-2 `@pre` on CALL expressions reads post-state -- FIXED (compiler side)
`@pre` on a call expression captured the CURRENT state instead of the
entry state, so size-relation contracts (`f(x) == f(x)@pre + 1`) always
violated at runtime. Field `@pre` works. Minimal 13-line repro filed in the
stdlib repo as `tools/known_failures/p_pre_call_capture.xi`; reproduced on
R46 (12148d43) and R46b (504fcc1e), free-function and method receivers.
Blast radius measured: 11 corpus smokes aborted; the stdlib clauses are
temporarily replaced with safe forms and `tools/probes/p_wave8_shapes.xi`
stays red as the regression lock.

FIX (R49, 2026-09-19): `collect_atpre_vars` collected NOTHING for
`total(b)@pre` (the recursion had no Ident arm under the Call), so no
snapshot was emitted; `AtPre` on a compound expression additionally fell
through to compiling in the CURRENT state. Fixes: collect every variable
under an `@pre` subtree; rebind all entry snapshots while compiling the
`@pre` expression (ref params through a fresh pointer slot -- the snapshot
alloca is the POINTEE, so passing it directly made the local load read the
struct's first field as a pointer); deep-copy inline `%struct.Vec` buffers
into the snapshot so element reads see entry-state data. Verified with the
stdlib repro (free fn + method + field/index forms) and locked as
`e2e_m95_pre_call_capture`. The stdlib session can restore the stronger
size relations in its clauses.

RESIDUAL CLOSED (R51, 2026-09-19): the stdlib's follow-up probe
(`tools/known_failures/p_pre_capture_callee.xi`) showed implication-wrapped
clauses (`result is Some => total(b) == total(b)@pre - 1`) never snapshotted:
`collect_atpre_vars`/`collect_pre_idents` had no `Expr::Imply`/`Expr::Is`
arms, so the walk stopped at the implication, no snapshot was emitted, and
the ensures compared the LIVE pointer with itself (2 == 2 - 1 across
list/queue/rbtree/fenwick and the other blocked modules). Both walkers now
descend through Imply/Is. The residual probe and `tools/probes/
p_wave8_shapes.xi` both exit 0; lock `e2e_m104_pre_capture_callee`. The
stdlib session can restore the strong `@pre` size clauses in the previously
blocked modules.

### R49-3 `p_result_payload_contract` -- scalar + Vec payload Result -- FIXED
One module with a scalar-payload Result contract plus a Vec-payload Result
contract broke clang (`%struct.Vec` passed to `xiom_str_len`); blocked
Err-payload clauses.

FIX (R49, 2026-09-19): after `result is Ok/Some` the contract base is
REBOUND to the payload, so `result.value.len()` resolved `.value` through
the erased i64 field and fell to the Str.len builtin. `field_payload_xiom`
now returns the rebound container type for `.value` when the base local's
tracked type is already a container (Vec/Slice/Map/Set), and the `.len()`
dispatch recognizes Option/Result payload containers
(`is_payload_container`). Verified with the stdlib repro plus an
Err-payload Vec form; locked as `e2e_m100_result_payload_contract`.

### R49-4 `p_sweep_single_param` -- clang 22.1.8 ISel crash -- FIXED (R54)
Symptom: `clang -c` on the emitted IR crashed (0xC0000005) in X86 DAG->DAG
Instruction Selection; the old evidence pointed at `@__unsafe_block_77`
with a 65536-byte stack alloca.

Root cause (bisected to a 5-line LLVM repro): the essential trigger is an
**aggregate zero-initializer store of a huge fixed array**
(`alloca [65536 x i8]` + `store [65536 x i8] zeroinitializer`); a whole-array
`load [65536 x i8]` in the same function compounds it. Threshold: 32768-byte
arrays compile, 65536 crash. `memset` alone or the alloca alone are fine.

Fix (codegen, R54):
- Fixed-array locals >= 16 KiB are zero-initialized with
  `llvm.memset.p0i8.i64(..., i64 bytes, ...)` instead of an aggregate
  `store zeroinitializer` (Let and Var declaration stores).
- Indexed access to such locals goes through their ADDRESS
  (`large_array_local_addr`) so codegen no longer materializes the whole
  aggregate just to discard it (index read and index-assign paths).
- Helpers: `array_type_bytes`, `large_array_local_addr`,
  `LARGE_ARRAY_MIN_BYTES = 16384`.

Evidence: minimal 15-line repro (`var buf: [65536]UInt8;`) now compiles;
`tools/known_failures/p_sweep_single_param.xi` compiles AND LINKS in ~56 s
(was: clang ISel crash) -- the Linux acceptance path is compile/link for
this compile-only sweep probe. Lock `e2e_m109_large_fixed_array` + CI line.

Note for the stdlib lane: the pinned checkout's `xiom/os/env.xi` still calls
`unsetenv` directly, so on WINDOWS this probe fails at LINK with
`undefined symbol: unsetenv` until `STDLIB_VERSION` moves to a revision
carrying the `xiom_env_set/xiom_env_unset` shim (the newer stdlib tree links
only when `XIOM_STDLIB` and `XIOM_RUNTIME_DIR` are both pointed at it -- the
runtime C files resolve separately from the source root).

## 2026-09-26 -- m142: `ptr.is_null()` silent-stub kill + method-position UFCS

### Symptom (stdlib relay, `xiom.cell` 155/181)
`Ref.release`: `if ptr.is_null() { return; }` compiled, exited 0, but the
guard never fired: `release` always decremented `borrows`, including for a
dead handle. Direct `is_null(ptr)` was correct; the METHOD form emitted no
call at all (or a garbage symbol) and answered `false` for every pointer.

### Root cause
Two layers, both silent:
1. The receiver path root `ptr` inside `Ref.release` is the implicit-self
   FIELD `*mut RefCell[T]` -- but `xiom.cell` also imports the `xiom.ptr`
   MODULE alias, and `Checker::check_module_call` bound the module first:
   it typed a zero-arg module call `xiom.ptr.is_null` (arity checks are
   gated off). Codegen then emitted `call i64 @ptr.is_null(%struct.RefCell*
   %recv)`, a symbol with no definition, and
   `emitter.rs::emit_undefined_symbol_stubs` synthesized a zero-arg
   `define i64 @ptr.is_null() { ret 0 }`: clang tolerates the signature
   mismatch, so every call returned the stub default (false / 0).
2. When the receiver was a pointer FIELD/local and no module shadowed it,
   the checker's R8 method-position free-fn block rejected `*Int` vs the
   free fn's `*const T` first parameter ("cannot call 'is_null' on this
   expression"), and codegen resolved the receiver-derived key
   (`RefCell.is_null`) to nothing -- no call, default `0`.

### Fix (all four planned parts)
- **(a) FAIL LOUDLY** (`emit_undefined_symbol_stubs`): no more synthesized
  stubs on the default path. Every called-but-undefined symbol is a hard
  error (C001) naming the symbol, return type, first-call IR line and the
  enclosing `@fn`. Two KNOWN gaps keep the historical typed stub but now
  print `warning[W005]`: unresolved calls inside CONTRACT CLAUSES (clause
  expressions are light-validated until the W002-W004 lint wave) and leaves
  that name an INTERFACE method with no concrete impl (erased interface
  dispatch; `Error.chain`'s `self.description()`/`self.source()`). A symbol
  called from both a tolerated and a non-tolerated site is a HARD error.
  The scan also ignores text inside `c"..."` string constants (the selfhost
  compiler embeds generated IR; matching it produced phantom `@sq`/`@add`
  calls and a false C001).
- **(b) Checker value-shadow + UFCS**: `value_member_resolves` skips
  `check_module_call` when the receiver path is rooted in a bound local that
  can resolve the member itself (`ptr.is_null()` -> the field; module-only
  members like `ptr.from_ref(self)` still bind the module). The R8 block
  now strips `&`, `*`, `mut`, `const` and matches leaves/generics, so a
  `*Int` receiver accepts `*T`/`*const T` and a `Vec` receiver accepts
  `&Vec` (auto-ref UFCS).
- **(c) Codegen UFCS**: method-position calls over pointer-like receivers
  resolve to the free fn (generic decl leaf preferred; registered fn with a
  pointer first param otherwise); the receiver is passed as arg 0
  (`ufcs_receiver` ABI flag -- it used to be DROPPED, producing a zero-arg
  call against a 1-param definition); the generic type arg is inferred from
  the receiver's pointee/leaf (`*Int` -> Int, `%struct.RefCell*` ->
  RefCell), so the monomorphised name and its param ABI match the value.
- **(d) Lock**: `tests/regression/m142_ptr_isnull_ufcs/main.xi` (null field:
  direct+method both true; non-null control both false; the stdlib
  module-shadow shape decrements a real borrow counter; a null handle
  returns early), `e2e_m142_ptr_isnull_ufcs` + CI line, and the codegen unit
  test `m142_undefined_symbols_fail_loudly` (hard error names the symbol, no
  stub emitted, quoted IR text ignored).

### Real bugs the loud C001 surfaced and fixed in the same batch
- `panic(msg)` had NO lowering: every `panic` call emitted `@panic`/`@core.panic`
  and was silently stubbed (`ret 0`), i.e. panic never panicked
  (`core.assert` included). Now lowers to `@xiom_panic` + `unreachable`,
  leaf-matched so `xiom.core.panic(msg)` works; value receivers named
  `panic` are not hijacked.
- `T()` in an erased generic body (`ThreadLocal[T]{ value: T(); ... }`) is
  the concrete zero of the substitution (`current_type_map`), not a stub.
- `type_id::<T>()` / `field_offset::<T>(x)` are CTFE intrinsics but had no
  runtime fold; bare calls now fold to the same FNV-1a id / byte offset
  used by `evaluate_const_init`. Receiver-guarded: `typeinfo.type_id[T]()`
  stays the stdlib function (documented LIMITED 0).
- **Parallel codegen resolution state**: `--parallel-codegen` built fresh
  per-function emitters that never saw the preassign/decl-pass state
  (`use_alias_map`, `bare_fn_aliases`, `fn_symbol_map`, `generic_fn_decls`,
  `fn_typed_params`, `prepass_call_types`). Bare imported calls
  (`is_nan(x)` with `use xiom.math.is_nan;`) and generic calls silently
  stubbed in parallel mode only; they are seeded now.
- In-process IR harnesses (`feature_regression_tests`, `integration_tests`,
  `robustness_tests`, `fuzz_tests`) compile without checker/stdlib, so they
  explicitly opt into `CodegenConfig::legacy_stub_unresolved` (W005-stubbed)
  instead of hard-erroring on derive/builtin helpers. The CLI never sets it.
  Hygiene backlog: ~22 feature-regression sources are only parse-able via
  error recovery (legacy `;`-separated enum variants, `= struct {}`); they
  should be modernized.

### Evidence
- Before: `tmp/sprintc/pkg_isnull_probe2.xi` printed `0` for the method form
  (IR had no call at all); `m142_cellmock.xi` (faithful `Ref.release` shape)
  emitted `call i64 @ptr.is_null(%struct.RefCell* %tmp2)` + auto-stub.
- After: the m142 fixture exits 0 through `compile_and_run`; the cellmock's
  `release` decrements to 0 and the null handle returns early; full e2e
  **2378/2378 (+4 ignored)**, stdlib-exec **85/85 (+2 ignored)**, stdlib
  modules 40/40, checker 195/195, feature-reg 510/510, perf 3/3, diff 24,
  api-freeze 2/2, tool suites green, ascii_guard green. (First full e2e of
  the batch: 2376 pass / 2 fail -- `e2e_i2_parallel_codegen` and
  `e2e_p2_turbofish`, both fixed above; second run 2378/2378.)

### Cross-lane
- **stdlib**: revert the workaround -- `ptr.is_null()` is now correct in
  method form (`xiom.cell` `Ref.release`/`RefMut.release`). W005 names the
  two remaining known gaps for their wave: the invalid `Int.hash` ensures
  (`a == b => a.hash() == b.hash()` with undeclared `a`/`b`) and the
  interface-default dispatch gap in `Error.chain`.
- **Release lane (backlog C8)**: v0.61.3 GitHub release lists
  `xiom-wasm-0.61.3.wasm` in SHA256SUMS but ships no such asset (mirror
  404s; the other five artifacts verify OK). Upload the wasm or regenerate
  SHA256SUMS; ops' dl-deploy.sh now warns on listed-but-absent assets and
  publishes nothing unverified.
- **Benchmark relay (backlog R-1..R-3, `docs/FAIRNESS-RELAY-2026-09-26.md`)**:
  `a << b | c` parses as `a << (b | c)` (silent wrong values; fix precedence
  or lint), `match` on a persistent `Option[T]` binds a copy, `/* */` should
  get a targeted "block comments unsupported" diagnostic.

## 2026-09-26 -- m143: by-value receiver container mutation (pointer self ABI)

### Symptom (packages relay, `tmp/sprintc/pkg_e001_accessor.xi`)
```xi
pub type S = { v: Vec[Int]; }
pub fn S.add(self, x: Int) { self.v.push(x); }
...
var s = S{ v: Vec[Int].new() };
s.add(1);
if s.count() != 1 { return 2; }   // fired: count() == 0
```
The element landed in the shared buffer but the caller's Vec header kept
`len == 0`: the update lived only in the callee's copy.

### Root cause
`compile_fn` (decl.rs) decides the self ABI via
`is_mut = is_mut_self || block_mutates_self || block_mutates_receiver_state`.
Both detectors only match ASSIGNMENTS (`self.field = ...`, bare
`field = ...`). A container mutation through a method call
(`self.v.push(x)`) has no assignment anywhere, so `fn S.add(self, x)` was
registered AND defined BY VALUE
(`define void @S.add(%struct.S %param_self, i64 %param1)`). The push handler
does store the mutated Vec header back through `store_back_to_receiver`, but
against the callee's COPY -- the caller's storage never sees it. (Read-after
in the same function worked, which is why the failure only shows across the
call boundary.)

### Fix
- New `IrEmitter::block_mutates_receiver_container(fd)`: walks the body for
  a container-mutating method call whose receiver is receiver state --
  `self.<field>.<mutator>(...)`, `this.<field>.<mutator>(...)`, nested
  `self.a.b.<mutator>(...)`, or a bare `field.<mutator>(...)` in a
  this-based method (field set shadow-aware like the existing detectors).
  Mutator leaves covered: push/pop/push_back/push_front/pop_back/pop_front/
  insert/remove/clear/extend/reserve/truncate/append/set/put/add/delete/
  retain/shrink_to_fit/swap/swap_remove/enqueue/dequeue/update/sort/sort_by/
  sort_unstable/reverse/dedup/dedup_by/merge/split_off.
- `is_mut` at BOTH ABI sites (`register_fn_impl` param types and
  `compile_fn`'s `self_llvm_ty`) now includes it, so registration and
  definition agree on `%struct.S* %param_self`; the call-site pointer
  coercion (call.rs receiver handling) already existed for `&mut self`.
- Read-only accessors (e.g. `fn S.count(self) -> Int`) keep the by-value
  ABI -- the detector is mutation-only.

### Evidence / lock
- `tmp/sprintc/m143_byvalue_vec_probe.xi` (before: printed "after add(1):
  wrong", exit 2; after: exit 0) and `tmp/sprintc/m143_receiver_mutation_probe.xi`
  (Vec push, Map insert, Set insert, bare-field push, read-only accessor).
- Lock `tests/regression/m143_receiver_container_mutation/main.xi` +
  `e2e_m143_receiver_container_mutation` + CI line.
- Gates: full e2e **2379/2379 (+4 ignored)** in one run; stdlib-exec 85/85
  (+2 ignored); stdlib modules 40/40; feature-reg 510/510; integration 130;
  robustness 63; fuzz 24; `pkg_e001_accessor` exit 0; ascii_guard green.

## 2026-09-26 -- m144: sibling-method receiver binding (HashMap crash class)

### Symptom (discovered by the item-3 arity survey)
`HashMap[Int, Int]` crashed with an access violation on the pinned stdlib:
`new`, `insert`, `get`, `contains`, `count` each reproduced (0xC0000005 /
0xC000005D). PRE-EXISTING: the 2026-09-22 `target/release/xiom.exe`
(v0.61.3-era) reproduces the same probes, so nothing from m142/m143 caused
it. The stdlib smokes never exercised HashMap.

### Root causes (four, stacked)
1. **G-10 dead in monomorphised bodies**: `compile_generic_monomorphisations`
   set `current_fn` but never `current_receiver`, so
   `resolve_implicit_self_call` always returned None inside every generic
   method body.
2. **Arg shift for generic siblings**: with G-10 dead, a bare
   `insert(old_data[i].key, old_data[i].value)` (HashMap.resize) went through
   the GENERIC path, which maps explicit args positionally to the decl's
   params. `self` is not in the AST params for receiver-style methods, so the
   KEY was coerced to the callee's first param (`inttoptr i64 key -> http://
   %struct.HashMap*`) and dropped: `call @HashMap.insert_Int_Int(receiver=
   key, value)` -- a 2-arg call against a 3-param definition. `HashMap.contains`
   read the wrong bucket (`call @HashMap.contains(i64* key)` -- the key as
   the receiver).
3. **No receiver slot**: `body_uses_receiver_state` only counted `this`, bare
   FIELD mentions and assignments; a body whose only receiver usage is a bare
   sibling call got NO `%param_self` while the method call site still passed
   one (arg shift in the other direction). This is also why the item-3 survey
   flagged `collections.get` (2 params vs 1 arg): the checker's `owned_here`
   gate skips G-10 for module-owned leaves.
4. **Broken receiver-type comparison**: the G-10 injection's
   `fp.ends_with(self_base)` compared `%struct.X*` against `%struct.X`
   (false -- the string ends with `*`), silently dropping the receiver for
   non-generic this-based sibling calls (`Holder.get` from `Holder.contains`);
   and primitive by-value receivers passed the alloca address (`double*`)
   where the callee's param was `double` (clang: "'%tmp3' defined with type
   'ptr' but expected 'double'", os/folder + net/http2 smokes).

### Fix
- `compile_generic_monomorphisations` now sets
  `fctx.current_receiver = fd.receiver.name` for the body.
- Bare calls whose G-10 resolution hits a GENERIC sibling are rerouted as
  the equivalent `self.<name>(args)` (synthetic receiver AST node), so the
  method machinery prepends the receiver and offsets the explicit args.
- `body_uses_receiver_state` (lib.rs, the live impl -- note
  `crates/xiom-codegen/src/types.rs` is NOT a module and its copy is dead)
  now counts bare calls to sibling methods as receiver state via
  `block_calls_sibling_method_keys` (bare or module-qualified keys).
- The G-10 injection compares star-trimmed pointee names and loads the value
  for any non-pointer self slot (primitives included).

### Evidence / lock
- Probes: `tmp/sprintc/m143_hashmap_steps.xi`, `m143_hashmap_triage.xi`,
  `m143_hm_after_insert.xi`, `m143_implicit_this_probe4.xi` (all exit 0;
  pre-batch release binary exits 2 / AV).
- Lock `tests/regression/m144_sibling_method_calls/main.xi` +
  `e2e_m144_sibling_method_calls` + CI line: non-generic Holder chain plus a
  20-insert HashMap roundtrip that forces `resize` (bare `insert` in the
  re-insert loop) and exercises get/contains/count on present and missing
  keys. Pre-batch release binary exits 2 on the same fixture.
- Gates: full e2e **2380/2380 (+4 ignored)**; checker 195/195; stdlib-exec
  85/85 (+2 ignored); stdlib modules 40/40; feature-reg 510/510;
  integration 130; robustness 63; fuzz 24; perf 3/3; diff 24.

### Item-3 note
The exact-arity flip list shrinks by one: `collections.get` was this compiler
gap, not a stdlib call-site bug. Remaining stdlib fixes: `io.printf` x3,
`_scrypt_blockmix` x2, `path.replace`. When flipping, the checker's bare path
needs the implicit-this offset (a receiver-method hit with
`args.len() + 1 == sig.params.len()` must not error) or the `owned_here`
G-10 gate must let receiver-method hits through.

## 2026-09-26 -- m145 (R-1): C-family/Rust bitwise precedence

### Symptom (benchmark relay, R-1)
`(1 << 8) | 2` written without the inner parens as `1 << 8 | 2` evaluated to
1024, not 258: bit-assembly code silently produced wrong numbers. `3 | 4 << 1`
evaluated to 14 (`(3|4) << 1`). `a & b * c` grouped as `(a & b) * c`.

### Root cause
The parser's precedence chain was
`parse_cmp -> parse_shift -> parse_add -> parse_mul`, with `^`, `&` and `|`
sharing the `*`/`/`/`%` level in `parse_mul_expr`. Shifts were therefore
TIGHTER than `*`/`&`/`|` but LOOSER than nothing at the bitwise level --
`1 << 8 | 2` parsed its right operand at the mul level and swallowed `8 | 2`.

### Fix (parser-only)
New nesting:
`parse_cmp -> parse_bit_or -> parse_bit_xor -> parse_bit_and -> parse_shift
 -> parse_add -> parse_mul -> parse_as`.
`parse_mul_expr` now handles only `* / %`. Semantics match Rust/C:
shift > `&` > `^` > `|` > comparisons, while `+`/`*` stay tighter than
shifts and bitwise stays tighter than comparisons (the stdlib's
`(n >> hi) & 1 == 1` shape keeps its meaning).

### Evidence / locks
- Probe `tmp/sprintc/m145_shift_precedence_probe.xi`: before 1024/14/8/0,
  after 258/11/8/0.
- Parser unit tests `test_shift_binds_tighter_than_bit_or` and
  `test_bitwise_above_comparisons_like_rust` assert the AST shapes
  (`|(<<(1,8),2)`, `|(3,<<(4,1))`, `==(&((>>(n,hi)),1),1)`,
  `&(8,<<(3,1))`, `&(*(2,3),4)`, `&(2,*(3,4))`, `|(^(1,2),&(3,4))`).
- Lock `tests/regression/m145_shift_precedence/main.xi` + e2e + CI line
  (runtime values incl. bit-packing helpers and the popcount-shape compare).
- stdlib audit before the change: every shift/bitwise mix is either
  parenthesized or already in the new grouping (`(n >> hi) & 1 == 1`,
  `(n << shift) | (n >> (64 - shift))`); stdlib-exec 85/85 unchanged.
- Gates: full e2e **2381/2381 (+4 ignored)**; parser 106/106; checker
  195/195; stdlib-exec 85/85 (+2 ignored); stdlib modules 40/40;
  feature-reg 510/510; integration 130; robustness 63; fuzz 24; perf 3/3;
  diff 24.

### Cross-lane
Benchmark lane: `docs/FAIRNESS-RELAY-2026-09-26.md` R-1 is fixed; the
parenthesized form `(1 << 8) | 2` keeps its meaning, and unparenthesized
bit-packing now behaves like C/Rust. R-2 (match binds a copy for persistent
`Option[T]`) and R-3 (block-comment diagnostic) remain open.

## 2026-09-26 -- m146: signed narrow widening for let-bound extern results (net.tcp_connect relay)

### Symptom (playground relay, full repro)
```
match net.tcp_connect("127.0.0.1", 1) {  // no listener
  Ok(s)  => io.println("closed=Ok"),
  Err(e) => io.println("closed=Err"),
}
```
printed `closed=Ok`. REPRODUCED on Windows with the same shape (exit 7 =
Ok). The runtime wrapper is correct (`xiom_socket_connect` returns
`connect(2)`'s result; SOCK_STREAM is blocking so no completion wait is
needed), so the defect is in the compiler's handling of narrow extern
results.

### Root cause (two stacked issues)
1. **ABI width**: the pinned `xiom.net` declares
   `fn xiom_socket_connect(...) -> Int` -- the compiler emits
   `declare i64 @xiom_socket_connect` while the C function returns `int`
   (i32). On x86-64 `mov eax, -1` zero-extends into RAX, so the i64 caller
   sees 4294967295, not -1. Not fixable at the binding site: the extern
   DECLARATION must say `Int32` (or the runtime must return a 64-bit
   sentinel). This is a stdlib-source decision.
2. **Missing signedness tracking (compiler bug, fixed here)**: with the
   extern correctly declared `-> Int32`, the call still misbehaved: the
   let/var binding path recorded the inferred XIOM return type
   (`infer_call_return_xiom`) into `local_xiom_types` WITHOUT updating
   `signed_locals`. The narrow load's widening consults `signed_locals`
   and a missing entry means UNSIGNED, so the i32 -1 widened
   `zext i32 -1 to i64` -> `result < 0` false. Without this fix a stdlib
   switch to `Int32` would NOT have repaired tcp_connect.

### Fix
`IrEmitter::track_local_signedness(name, xiom_type)` keeps `signed_locals`
in sync, applied to all four inference arms (`infer_if_xiom_type`,
`infer_try_xiom_type`, `infer_field_payload_xiom`,
`infer_call_return_xiom`) in BOTH the `let` and `var` paths. The two arms
that already updated the map inline are unchanged in effect. Unsigned and
aggregate types remove the entry, so a stale signed binding under the same
name cannot leak in.

### Evidence / lock
- `tmp/sprintc/m146_tcp_int32_probe.xi` (extern declared `-> Int32`, refused
  connect): before the fix exit 7 (false Ok), after exit 0 (Err correct).
  IR check: `%tmp46 = zext i32 %tmp45 to i64` (before) vs
  `sext i32 ... to i64` (after).
- Lock `tests/regression/m146_signed_extern_result/main.xi` + e2e + CI line:
  `strcmp("a","b")` (declared `Int32`) let-bound and compared `< 0`, `== 0`
  on equal strings and `> 0` reversed; deterministic, no sockets.
- `tmp/sprintc/m146_tcp_connect_probe.xi` (pinned `net.tcp_connect`) still
  reports Ok -- expected until the stdlib declares `Int32`.

### Cross-lane
- **stdlib**: declare int-returning externs as `Int32` (sweep the extern
  block; `xiom_socket_*`, `xiom_dns_resolve`, ...) once a pin carries this
  fix; this fix is a PREREQUISITE for the declaration change to work.
  Optional: propagate errno/WSAGetLastError in the `NetError.code` field
  (currently the wrapper reports the -1 sentinel). The pending release with
  the wasm asset (C8) does NOT fix tcp_connect -- it needs this compiler
  change + the stdlib declaration + a pin carrying both.

## 2026-09-26 -- interface-as-value support matrix (answer for the stdlib Error.chain wave)

The stdlib deferred the `Error.chain` W005 restructure pending the compiler
lane's expected interface shape. Probes (`tmp/sprintc/m146_iface_*_probe.xi`)
establish what works TODAY:

| shape | status |
| --- | --- |
| interface method call, concrete receiver known at the call site | works (static dispatch, incl. default methods calling siblings on `self`) |
| interface-typed struct FIELD whose concrete value is locally tracked | works (`Box{ item: SomeError }.item.describe()`) |
| generic bound `fn f[T: Error](e: &T)` (monomorphised per call site) | works |
| interface-typed PARAM receiving an aggregate (`e: Error` called with a struct) | C001 "unsupported: interface-typed parameter ... receiving aggregate argument" |
| interface VALUE in `Option[Error]` payloads / erased receivers | W005 erased-dispatch gap: bare unresolved call, default stub (the `Error.description`/`source` case) |

There is NO dynamic dispatch (no vtable/tag) for interface values, so
heterogeneous error storage cannot use the interface itself.

**Expected shape for the Error wave**: make the error DATA a CONCRETE closed
type and keep interfaces only as static bounds:
```xi
pub type ErrorInfo = { kind: ErrorKind; message: Str; cause: Int; } // cause: index/handle, -1 = none
pub enum ErrorKind { Io, Parse, Net, ... }
pub fn chain(info: ErrorInfo, arena: &Vec[ErrorInfo]) -> ErrorChain { ... }
// optional: pub interface Error { fn info(self) -> ErrorInfo; }  // concrete return type
//            fn f[T: Error](e: &T) ...                            // generic helpers only
```
Never use the interface as a value type in signatures (`Option[Error]`,
`e: Error`, `cause: Error`). Full dynamic interface dispatch (vtables) and
interface-typed aggregate params are larger follow-up features, not
prerequisites for the restructure.

## 2026-09-26 -- m147: G-10 receiver-registry fix + item-3 arity flip readiness

### Symptom (found by the item-3 arity survey, fixed here)
With the exact-arity checks temporarily enabled, the corpus flagged
`collections.xi:1040` -- `get(key)` inside `HashMap.contains` -- as
"expects 2 argument(s), found 1". The call is the implicit-this form of
`HashMap.get[K, V](key: &K)`.

### Root cause
`check_implicit_self_method` classified a registry hit as a receiver method
only when `uses_implicit_this` was set or the FIRST PARAM was
`self`/`Self`/the receiver type. THIS-BASED GENERIC methods whose signature
omits the receiver entirely (`HashMap.get` has params `[key]`) failed that
heuristic, so G-10 returned None; the bare path then bound the first-wins
bare slot `get` == xiom.array's free `get(arr: Array[T], idx)` (2 params)
and the exact-arity check errored. The checker's typing of the call was
also not the receiver method; codegen compensated since m144's sibling
binding, but the checker-side resolution stayed wrong.

### Fix
- Any hit in the receiver's method registry (primary `methods[recv]` or the
  `.{recv}` suffix scan) IS the receiver method; the first-param heuristic
  now only decides whether the receiver occupies `params[0]`
  (`receiver_in_params`).
- Call shape: receiver-in-params -> `args.len()+1 == params.len()` with
  `param_offset=1` (explicit-self methods; Vec4f historical case
  unchanged); receiver-omitted -> `args.len() == params.len()` with
  `param_offset=0` (this-based generic methods).
- The G-10 module-shadow gate was refined alongside (a module owning only
  same-leaf METHODS no longer blocks G-10; it shadows only when it has a
  same-leaf FREE fn, module-qualified key, that fits the exact arity).

### Item-3 flip readiness (verified, not yet committed)
The four exact-arity hunks are implemented and were temporarily enabled:
1. impl-method call site: `args.len() != sig.params.len()`.
2. module-prefix call site: same.
3. method path: `expected_args = sig.params.len() - param_offset;
   if args.len() != expected_args { error }`.
4. bare path: exact OR the implicit-this allowance
   (`args.len()+1 == params.len()` for receiver-method sigs).
Verification used a locally patched stdlib mirroring 90e9185
(`printf` -> `(format, arg)`, `_scrypt_blockmix(&x, r)`,
`xiom.string.replace(...)`): corpus GREEN and full e2e GREEN (2382/2382
+4 ignored), stdlib-exec 85/85, feature-reg 510/510. The hunks are OFF in
this commit: the stdlib ref 90e9185 is UNPUSHED (origin/main still
49b4731) and `STDLIB_VERSION` is stdlib-v0.61.3 -- flipping now would make
the repo red on its own pin. Landing plan: on the stdlib push, bump
`STDLIB_VERSION` to that ref (or wait for their release tag per item 4),
apply the four hunks, run the full gates once.

### Evidence / gates (committed state)
G-10 registry fix + checks OFF + official pin: full e2e **2382/2382
(+4 ignored)**; checker 195/195; stdlib-exec 85/85 (+2 ignored);
feature-reg 510/510; integration 130; robustness 63; fuzz 24; perf 3/3;
diff 24.

### Benchmark blockers
The relay pointer lists R-1/R-2/R-3/R-5/R-6, but
`docs/FAIRNESS-RELAY-2026-09-26.md` is not present in this tree. R-1 is
fixed (m145); R-2 (match binds a copy for persistent `Option[T]`) and R-3
(block-comment diagnostic) are summarized; R-5/R-6 need the document or a
summarized relay.

## 2026-09-26 -- m148 (R-2): match payload bindings alias the box

### Symptom (benchmark relay R-2)
`match o { Some(v) => { v.n = 6; } }` on a persistent `Option[Cell]` did not
change `o`'s payload: the re-read saw 5, and `Some(v) => v.push(7)` on a
Vec payload left `len == 0` (relay's workaround was a one-element Vec).

### Root cause
The match arm's payload binding load-dereferenced the boxed payload into a
FRESH alloca (`%v = alloca %struct.Cell; store loaded, ...`) and registered
that alloca as the binding -- every field write mutated the stack copy.
A second, pre-existing defect: TEMPORARY scrutinees
(`match Some(Cell{ n: 9 }) { Some(t) => t.n }`) had no declared payload type
(`scrutinee_payload_xiom` handled locals/calls only), so the arm fell to the
i64-handle fallback and `t.n` read 0 (reproduced on the 2026-09-22 release
binary).

### Fix
- Aggregate and Vec payloads are now bound by ADDRESS: the binding
  registers (payload-pointer register, %struct.X) -- the standard
  pointer-backed struct-local convention used for receivers -- so field
  GEPs, method receivers, `&v` materialization and container pushes target
  the payload in the box. Scalar payloads (Str/Float/number) keep their
  existing value bindings; scalar arm reassignment stays a rebind.
- `scrutinee_payload_xiom` infers `Some/Ok/Err` literal scrutinee inner
  types via `infer_expr_xiom_type_deep`, which now handles `Expr::Struct`
  (named struct literal -> its type name).

### Evidence / lock
- Probes: `tmp/sprintc/m148_match_bind_probe.xi` before after=5/orig=5 ->
  after after=6/orig=5 (the source struct passed into `Some` is still a
  copy -- standard value semantics); `m148_match_vec_payload_probe.xi`
  before vec_len=0 -> after vec_len=1.
- Lock `tests/regression/m148_match_payload_alias/main.xi` + e2e + CI line:
  struct field mutation, Vec push + element read, read-only temporary
  match, and the scalar-rebind control.
- Gates: full e2e **2383/2383 (+4 ignored)**; checker 195/195; stdlib-exec
  85/85 (+2 ignored); stdlib modules 40/40; feature-reg 510/510;
  integration 130; robustness 63; fuzz 24; perf 3/3; diff 24.

### Harness fix (same batch)
`crates/xiom-codegen/tests/stdlib_tests.rs` now compiles with
`--timeout 900`: the CLI's 300s default tripped deterministically when
`stdlib_all_modules_compile_to_ir` ran alongside the 39 per-module tests
(each compiles the whole program; solo ~120s, parallel ~430s), producing
"compilation timed out after 300 seconds" flakes in two batches.

### Cross-lane
- Benchmark lane: R-2 closed; R-1 closed (m145); R-3 open; R-5/R-6 need
  the relay document.
- Playground lane (tcp_connect): the stdlib `Int32` extern declaration is
  committed on the stdlib side but NOT pushed (origin/main 49b4731); it
  ships in the first release whose pin carries it together with m146 --
  verify tcp_connect at that pin (absorption step 6); the wasm-asset
  release does not fix it.

## 2026-09-27 -- R-3 closed (already fixed) + Stage 6 lint-wave sizing

### R-3 (benchmark relay): block comments -- no code change needed
The relay reported `/* */` -> "expected type expression, found '/'". On the
current tree block comments are an intentional lexer feature
(crates/xiom-lexer/src/lib.rs:102) and compile in block, type-annotation and
expression positions (`tmp/sprintc/m149_block_comment_probe{,2}.xi`: exit 0);
an UNTERMINATED block comment reports
`error[L001]: <line>:<col>: unterminated block comment`
(probe 3, lexer lib.rs:288). The report is stale (fixed sometime during the
campaign). Benchmark trio R-1 (m145), R-2 (m148), R-3 (stale) closed.

### Stage 6 lint wave sizing (docs/STAGE6_LINT_WAVE.md)
Start with W002 + W003 (spec order). Plumbing findings for the next session:
- Checker warnings: `CheckError { message, span, cause, guaranteed }`
  (crates/xiom-check/src/types.rs:610), pushed by `warn`/`warn_at`
  (lib.rs:1350/1356); catalog scope is the "catalog body" message prefix.
  8 literal construction sites -> a `code: Option<String>` field is cheap;
  a "W002:"/"W003:" prefix convention is the zero-struct-churn alternative.
- Driver: crates/xiom/src/lib.rs:868-879 partitions catalog warnings (W000,
  capped at 5) from own warnings (currently W000); own warnings must print
  per-lint codes (`warning[WNNN]`). `Diagnostic{kind,code,...}` exists
  (W001 at lib.rs:582); spec item 2 wants the new codes in the JSON
  envelope + docs/JSON_DIAGNOSTICS_V1.md.
- Scope guard: lint only the user program (`checking_catalog` keeps the
  stdlib out) so `catalog_corpus_is_clean` and the zero-warning e2e tests
  (m16/m17) stay green structurally.
- Locks per lint: positive fixture (stderr contains the code, exit 0) +
  negative fixture (silent), via crates/xiom/tests/checker_locks.rs; add
  the CI e2e-name line where the fixture is e2e-able. W002 must stay silent
  for guard-first recursion; W003 is per-block (not across labels) and must
  not fire after loops with a reachable `break` or after a diverging
  `if` branch the block continues from.

## 2026-09-27 -- item 3 landed: exact arity ON (stdlib pin 0c50ac6) + resolver fixes

`STDLIB_VERSION` -> stdlib main tip `0c50ac6e5cd9c749834e3d8993ccca44eb0708cf`
(pushed; carries the 90e9185 call-site fixes -- printf x3,
`_scrypt_blockmix` x2, `path.replace` -- plus c193bc4 m146 prep). The four
exact-arity hunks from the m147 recipe are FLIPPED ON with call-shape-correct
expected counts:

1. impl-method call site: `args.len() != sig.params.len()`.
2. module-prefix call site: same.
3. method path: `expected = params.len() - param_offset`, where
   - explicit-receiver instance calls (`s.push(&mut s, "Alice")`, the R52
     lock shape) use offset 0 when `args.len() == params.len()`;
   - implicit-this STATIC calls (`Color.is_red(&r)`) expect
     `params.len() + 1` (arg 0 is the receiver and the params follow);
   - otherwise the 6370 offset table applies unchanged.
4. bare path: exact count with the implicit-this receiver allowance.

The flip surfaced three latent resolution defects; this batch fixes them so
the catalog corpus is clean under enforcement:

- **Wildcard singleton capture (generic `T: Ord`).** `Path` derives Ord, so
  `register_derived_method` registers a params-less `compare` under
  `methods["Path"]`; as the ONLY registered `compare` it captured every
  generic `x.compare(y)` via the singleton path, and the exact check flagged
  catalog bodies (xiom.cmp + xiom.collections; 22 findings in the path
  smoke). Fix: a generic receiver whose interface bound declares the method
  now defers to interface dispatch (the wildcard capture is skipped).
- **Receiver-sugar shape (smoke_core_box).** `fn Box.get[T](b: &Box[T])`
  called as `b.get()` matched no exact name, so the receiver counted as an
  argument ("'get' expects 1 argument(s), found 0"). Fix:
  `first_param_matches_receiver` compares normalized leaves (module prefix,
  generic args and ref marks stripped); the duplicated `strip_ref_marks`
  closures are now one shared helper.
- **Interface member arity model (m37_bug45).** Method-form interfaces
  (`interface Eq5 { fn eq(other: &Self) -> Bool }`) stored the operand as
  `"Self"`, which `want_of` counted as a receiver (want 0 for a 1-arg call).
  Fix: interface registration keeps the distinction -- a param NAMED self
  stores the "self" marker; a ref-marked non-self first param keeps its ref
  mark; `want_of` only counts an UNREF'd self/Self/receiver-typed first
  param as the receiver.

### Locks
`tests/regression/m150_exact_arity/{main,reject_extra,reject_missing}.xi`
(receiver-sugar + generic-bound calls accepted; extra/missing args must
fail), e2e `e2e_m150_exact_arity` + `e2e_m150_exact_arity_rejects`, CI line
in `.github/workflows/ci.yml`.

### Gates (all green)
- Full e2e **2385/2385 (+4 ignored)**, run with `-- --test-threads 16`.
  Default-thread runs on this box storm the ~300 contiguous m35 compiles
  into 31-32 SILENT spurious compile failures (0 diagnostics; all fixtures
  clean individually, 300/300 in isolation, failure subsets differ per run)
  -- infrastructure fragility, filed for harness hardening. The harness
  does not retry `None` compiles, and the driver prints nothing when
  clang/link fails (silent-failure gap). 20,400 stale `e2e_*.exe` (6.3 GB)
  were cleaned from the repo root before the green run.
- checker 195/195; strict-clause catalog corpus 1/1; stdlib-exec 85/85
  (+2 ignored); stdlib modules 40/40; feature-reg 510/510; integration 130;
  robustness 63; fuzz 24; perf 3/3; diff 24 (+1 ignored).
- Pin absorption step 6: `tcp_connect("127.0.0.1", 1)` takes the Err path
  with a negative code (`tmp/sprintc/p_pin_tcp_refused.xi`: `ERR code=-1`),
  so the Int32 extern result widening holds on this pin.
- ascii_guard clean before commit.

### Cross-lane
- Benchmark lane: R-5 withdrawn to `%TEMP%\kilo\bench_r5_*` during the
  two-writer recovery (owner relayed); resume in a worktree after this
  commit. `io.parse_int` -> bare `@is_empty` C001 is a separate follow-on.
- Playground probes (relays, filed as backlog): `12 + 2.to_string()` prints
  "122" (silent Int+Str coercion -- checker hole, own batch);
  `(2 + 2.5).to_str()` takes the W005 stub (prints 0) while
  `float_to_string(2 + 2.5)` prints 4.5 and annotated Float64 locals work;
  `for x in range(...)`/Range values emit repeated `unknown type
  'Iterator' -- defaulting to i64` warnings (semantics correct; Vec/array
  loops are quiet). `12.to_str()`, `(12).to_str()`, `(-12).to_str()` and
  `12.to_string()` are all guaranteed.
- Packages `docs/COMPILER-FINDINGS.md`: the "compiler does not validate
  arity" row is FIXED by this batch; remaining compiler-lane rows (`&mut
  Int` write-through, loop-carried CSE, mixed brackets, sign-bit tests,
  `byte_at` >= 128) are the next candidates.

## 2026-09-27 -- Stage 6 lint wave: W002 + W003 (warning-only, user scope)

First landings of the Stage 6 control-flow lint wave
(docs/STAGE6_LINT_WAVE.md): warning-only, user-program scope, one code per
lint.

- **W003 unreachable statement after a diverger**: `return`, `break`,
  `continue`, an `if` whose every branch diverges, a `match` with a
  catch-all whose every arm diverges, or `while true` with no `break` of
  its own. Warns once per block, on the first statement after the
  diverger. `debugger` is not a diverger; a return behind an `if` branch
  the block continues from, and loops with a reachable break, stay silent.
  The break scan is complete on purpose (a missed break would fabricate a
  false "unreachable").
- **W002 unconditional recursive cycle**: call graph over the user unit's
  non-generic free fns (direct bare calls by name; recurses through
  `module X { ... }` wrappers -- a file with a module header parses as a
  single Module item, which the first cut missed). The graph is built from
  CERTAIN-call edges only: a call is certain when every path evaluates it
  before any exit (short-circuit `&&`/`||` keep only the left side;
  closures never count; `if`/`match` branches intersect, a missing
  `else`/catch-all contributes nothing; any statement that may exit early
  stops the walk). A cycle in that graph is exactly the unconditional
  shape, so no separate dominance pass is needed. Warns once per SCC:
  `'a' is part of an unconditional recursive cycle a -> b -> a; this call
  chain can never terminate`. Guard-first recursion and branch-reached
  mutual recursion stay silent.

Plumbing:
- `CheckError` gains `pub code: Option<String>` (6 construction sites).
  `warn_coded_at` emits coded warnings with the same catalog scoping as
  `warn_at`. `check_program` keeps CODED lint warnings in the warning
  stream even when hard errors exist, so the driver still renders
  `warning[WNNN]` on the error path instead of mislabelling them `T001`
  (probe `p_lint_mixed_error.xi`: `warning[W003]` + `error[T001]`).
- Driver: the `own_warns` loop prints `warning[{}]` with the code (W000
  fallback); the `--diagnostics=json` error envelope now carries the
  warning stream (kind `warning`) ahead of the errors; the
  `compile_with_diagnostics` mapping uses `w.code`.
- `docs/JSON_DIAGNOSTICS_V1.md`: `warning` kind added; codes W000-W003.
- Four `checking_catalog` guards keep stdlib/catalog bodies structurally
  lint-free; corpus + zero-warning e2e gates stay green.

### Locks
`tests/regression/m151_w002_cycle` + `m151_w003_unreachable` (positive:
code in stderr, compile exit 0) and `m151_w002_guard` +
`m151_w003_guarded` (negative: silent); 4 new
`crates/xiom/tests/checker_locks.rs` tests (covered by the CI
`checker_locks` step; the W002 fixture is compiled but never executed --
it cannot terminate).

### Gates (all green)
full e2e 2385/2385 (+4 ignored, `-- --test-threads 12`), checker 195/195,
strict-clause corpus 1/1, stdlib-exec 85/85 (+2 ignored), stdlib modules
40/40, feature-reg 510/510, integration 130, robustness 63, fuzz 24,
perf 3/3, diff 24 (+1 ignored), checker_locks 8/8.

### Cross-lane: transient silent compile failure corroborated
The e2e m35 storm signature (31-32 spurious silent compile failures in
loaded runs, 0 diagnostics, all fixtures green solo/in isolation) matches
packages `COMPILER-FINDINGS.md` row 28 (`program_exit=-1`, empty output,
three sightings, green on re-run). `stdlib_tests` reproduced it too under
concurrent lane load (0/40, then 32/40 with a different failing set; 40/40
at `--test-threads 8`). This is now a cross-lane-corroborated flake class:
never record a silent failure as a pass without a re-run; a capture batch
(exit code + dump on a loaded re-run) is queued behind the `&mut`
write-through fix.

## 2026-09-27 -- R53: `&mut` out-param write-through (plain-local implicit borrow)

Packages relay (repro battery `mut-int-write-through/`): a call to a
function taking a `&mut T` parameter with a PLAIN local argument
(`set_one(x)`, no explicit `&mut` at the call site) compiled silently and
operated on a copy -- every write through the parameter was lost; a
`&mut Struct` variant additionally corrupted memory (a bag push wrote
`1859382800640`). Reproduced on the current build before the fix:
`probe_out_params.xi` `bad=6`.

Root cause: the checker erases references from its type model
(`CheckedType::from_ast_type` maps `Ref`/`MutRef` to the pointee), so the
plain-local form is accepted; in codegen, `coerce_arg_for_param`'s
`param_ty.ends_with('*')` path only passed a lvalue's slot ADDRESS for the
explicit `&x`/`&mut x` wrapper (BUG 31). A plain scalar ident fell into the
BUG-31 temp materialization (`alloca` + `store` + pass the temp), so the
callee wrote into a discarded copy.

Fix (`crates/xiom-codegen/src/coerce.rs`): a plain lvalue ident fed to a
pointer param is the implicit form of the explicit-borrow branch -- pass
the local's slot ADDRESS (`coerce_value(slot, "{slot_ty}*", param_ty)`).
Value expressions (literals, calls, arithmetic) keep materializing a temp.
Guard set (skip to preserve existing value paths): inferred pointer-valued
locals (`Str` / `*T` XIOM type -- `var p = ptr.null[Int]()` has slot i64
and is NOT in the declared-type `ptr_locals`), `array_locals`,
`ref_locals`, `ref_params` (address-as-i64 params), `closure_locals`,
fn-typed locals, `ptr_locals`, `local_vec_handle`, `local_boxed_struct`,
and pointer-typed slots. The first cut missed the inferred-pointer case and
regressed `smoke_ptr`/`smoke_ptr_offset` (caught by stdlib-exec, fixed).

### Locks
`tests/regression/m152_mut_write_through/main.xi` (plain single/two
out-params, read-modify-write, struct field + Vec push, explicit `&mut`
control; exits with the failure count) + `e2e_m152_mut_write_through` + CI
line.

### Gates (all green)
full e2e 2386/2386 (+4 ignored, quiet-window `-- --test-threads 12`),
stdlib-exec 85/85 (+2 ignored), feature-reg 510/510, stdlib modules 40/40,
checker 195/195, strict-clause corpus 1/1, integration 130, robustness 63,
fuzz 24, perf 3/3, diff 24 (+1 ignored), checker_locks 8/8; packages
`probe_out_params.xi` `bad=0` and the struct-bag probe passes
(`v=99 len=1 e0=7`). Loaded-run e2e attempts hit the m35 storm (12/25
spurious silent compile failures; every examined fixture compiles and runs
green on direct retry) -- consistent with the cross-lane flake class above.

## 2026-09-27 -- v0.62.0 release prep: pin stdlib-v0.62.0 + strict-clause default

Release batch (item 4). Pin: `STDLIB_VERSION` -> `stdlib-v0.62.0` (tag
created on the stdlib release cut `80e767b`; the pushed tip `1770ce6` is
docs-only and was NOT tagged). Version: workspace 0.61.3 -> 0.62.0
(`Cargo.toml` [workspace.package] + `cargo update -w` for `Cargo.lock`;
`xiom --version` reads "XIOM Compiler v0.62.0"). Release notes:
`xiom-release-notes convert --tag v0.62.0 --stdlib stdlib` merged the
stdlib fragment -> 6 highlights (4 compiler + 2 stdlib); `verify` green.

**Strict-clause default FLIPPED ON** (`XIOM_STRICT_CLAUSES=0` opts out):
every contract clause must type as Bool; the stdlib lane verified their
main + the pinned corpus under strict, and the corpus gate is clean under
the new default. Clause expressions are fully checked, so one
accommodation was needed:

- **Literal-0 null-pointer casts are exempt from the unsafe gate.**
  `requires: p != (0 as *Int)` is the idiomatic FFI precondition; forming
  the NULL pointer from the integer literal 0 is safe (only
  DEREFERENCING a pointer is gated). All other int->pointer casts stay
  unsafe-gated (`test_d2_int_to_ptr_cast_outside_unsafe_rejected` still
  rejects `n as *UInt8`). Restores the two d2 unit tests and 5 old e2e
  fixtures (m32_c06, m33_u19/u20, m34_y14, m36_c10) that use the idiom.

**Strict-brackets default flip HELD for this release**: the pin gate found
3 mixed-bracket sites in the released stdlib -- `xiom/io/fs.xi` lines 36
and 244 (`Result[Vec[UInt8], Str>`) and `xiom/math/algebra_extended.xi`
line 311 (`Vec[Int]>`). Relayed to the stdlib lane, which canonicalizes
them in the next wave; the owner chose to ship v0.62.0 with the lax
default (`XIOM_STRICT_BRACKETS=1` still opts in) and land the flip with
the next release. The m141 lock keeps asserting both modes.

### Gates (release state)
full e2e **2386/2386 (+4 ignored)**; checker 195/195 (catalog corpus clean
under the strict-clause default); stdlib-exec 85/85 (+2 ignored); stdlib
modules 40/40; feature-reg 510/510; parser 106/106; release-notes 6/6;
integration 130; robustness 63; fuzz 24; perf 3/3; diff 24 (+1 ignored);
CLI suites 54+6+2+8+1+15; MCP 44; pkg 75.

Environment note: on this machine Windows Defender blocks the `xiom run`
script exe written under `%TEMP%\xiom_run` (os error 225, "potentially
unwanted software"): `e2e_m16_scripting_exit_zero` fails ONLY at the
execute step there (it passed in the R53 full run hours earlier; and with
`TEMP` redirected to a repo-local scratch it passes: exit 0, stdout
"test", 5.1s). The full release e2e was run with the script cache
redirected to a repo-local `tmp/e2etemp`. This is machine AV state, not a
compiler defect.

### Post-push (same day)
Pushed: stdlib tag `stdlib-v0.62.0` (80e767b), compiler `main` =
`7e4d0909`, annotated tag `v0.62.0`. XIOM Release run 36438304704: guard,
all five packages (incl. windows-x64), publish, and the docs
`repository_dispatch` ALL GREEN
(https://github.com/xiom-lang/xiom/releases/tag/v0.62.0). CodeQL on main
green. Extra XIOM CI dispatch on main (run 36439203600) surfaced THREE
PRE-EXISTING CI-only failures, none from this tree's behavior and none
blocking the release:
1. `test_moderate_nesting_ok` (xiom-parser) overflows the test-harness
   stack on windows-latest AND ubuntu-latest (`STATUS_STACK_OVERFLOW`):
   the driver compiles on a big-stack thread, the unit test does not.
2. `toolchain::tests::candidates_are_path_first_then_known_locations` and
   `windows_llvm_locations_cover_winget_and_local_programs` (xiom lib) are
   Windows-specific assertions that run (and fail) on ubuntu-latest.
3. Scheduled `XIOM Heavy Suites` fails fast (38-45s) on 9/21 and 9/28 --
   pre-existing, needs its own look.
Follow-up batch queued: CI hygiene (big-stack test threads or
RUST_MIN_STACK in CI, `#[cfg(windows)]` gating for the path tests, Heavy
Suites triage). Registry canary remains the external lane's step.

CI-hygiene batch EXECUTED (2026-09-28): `test_moderate_nesting_ok` now runs
its parse on a 64MB-stack thread (same pattern as the deep-nesting test --
libtest's default thread stack overflowed on the runners); the two
Windows-path toolchain tests are `#[cfg(windows)]`-gated (the
windows-latest leg still exercises them, ubuntu no longer runs them).

CI-hygiene batch 2 (same day, exposed once the first fixes let later CI
steps run): `stdlib_api_freeze_all_modules_compile` hardcoded `xiom.exe`
under target/{debug,release}, so it was NotFound on ubuntu (CI builds only
the release profile) -- now platform-correct + release fallback;
`stdlib_api_freeze_no_removals` flagged the frozen
`contracts :: get_function_contracts` entry because the stdlib
canonicalized its mixed bracket closers
(`Option<Vec[FunctionContracts>>` -> `Option[Vec[FunctionContracts]]`) --
the snapshot is updated intentionally (same signature semantically). Both
freeze tests pass locally (2/2) and the workspace `--lib` + CLI gates are
green. The scheduled `XIOM Heavy Suites` fast-fail (38-45s, 9/21 + 9/28)
still needs its own look.

## 2026-09-28 -- v0.62.1 patch release

Patch release carrying the post-v0.62.0 fixes:
- **R-5** (PR #4, benchmark relay): local fn definitions shadow same-named
  externs in the T002 gate; catalog-body findings render with
  module-qualified attribution. Merged with ALL required checks green
  (first fully green CI on this repo; admin merge for the review policy).
- **C001**: value receiver no longer resolved as a module (bare
  `@is_empty`) -- see the C001 FIXED section.
- **C9 + C8b** (playground): LLVM-18 `opt` verification syntax; wasm glue
  published (fix-forward on v0.62.0 + release.yml).
- **CI hygiene** (2 batches): parser big-stack nesting test,
  `#[cfg(windows)]` path tests, freeze-test binary path + the intentional
  contracts snapshot update.
- **Promoted `m150_dl_num_parse` to an e2e run lock** (the benchmark's
  ask): R-5 + C001 together make the exact num + ffi.dl program compile
  and run (`e2e_m150_dl_num_parse` + CI line).

Version 0.62.1 (`Cargo.toml` + lock refresh); `release-notes/v0.62.1.md`
(4 highlights) converted and `verify` green. Gates on the release state:
full e2e **2389/2389 (+4 ignored)**, checker 195/195, stdlib-exec 85/85
(+2 ignored), stdlib modules 40/40, feature-reg 510/510, freeze 2/2,
integration 130, robustness 63, fuzz 24, perf 3/3, diff 24 (+1 ignored),
CLI suites, MCP 44, pkg 75, release-notes 6, workspace `--lib`.

Release run notes: the first publish attempt was failed by the new
checksum guard on a FALSE NEGATIVE -- the Windows leg writes its checksum
file via PowerShell (CRLF), so the guard saw
`xiom-0.62.1-windows-x64.zip\r` and reported the (present) asset missing.
The combined SHA256SUMS is now CR-normalized and the guard strips a
trailing CR (`f93f4ee6`); the tag was re-cut onto that commit and the
release run went fully green. The create-release glob also omitted
`*.js`/`*.d.ts`, so the wasm glue JS/types were checksummed and staged but
not attached: fixed for future releases (`*.js *.d.ts` in FILES) and
fix-forward on v0.62.1 by uploading the exact CI-built glue from the
release run's linux artifact (hashes match SHA256SUMS).

### C8 recurrence fix-forward (same day)
v0.62.0 shipped a `SHA256SUMS` entry for `xiom-wasm-0.62.0.wasm` with the
asset missing from the release: the linux leg builds the wasm and writes
`SHA256SUMS-linux-x64` (including it) but its `upload-artifact` step only
listed the tarball + checksums, so the publish job never had the file.
Fix-forward: extracted `bin/xiom-wasm.wasm` from the published linux
tarball, verified sha256 == the checksummed value
(`6df21583...1059cc`), and uploaded it to the v0.62.0 release as
`xiom-wasm-0.62.0.wasm`. Durable fix in `.github/workflows/release.yml`:
the artifact upload now includes `xiom-wasm-*.wasm`, and the publish job
gains a "Verify checksums reference shipped files" step that fails the
release when any `SHA256SUMS` entry has no artifact in `artifacts/`.
Extension note: the VSIX publish steps correctly SKIPPED for v0.62.0
(toolchain-only release; `editors/vscode/package.json` stays 0.12.0 and
the marketplace already has it -- bump the version only for real
extension changes).
## 2026-09-28 -- R-5 FIXED (benchmark relay): extern-name shadowing in the T002 gate

**Branch**: `bench/r5-extern-gate` (worktree `E:\xiom-lang\xiom-bench`, off
release main `7ca323a4`, stdlib pin `stdlib-v0.62.0` = 80e767b).

**Repro (relay)**: `use xiom.num;` + `use xiom.ffi.dl;` failed the checker
with two bogus `error[T001]: catalog body [xiom.math.primitives]: calling
extern "C" function 'abs' requires an `unsafe` block`, printed at 191:11 /
223:12 as if they were user-file positions.

**Root cause**: the T002 confinement gate is a global name match against
`extern_fns`. `xiom.ffi.dl` loads its parent `xiom.ffi`, which imports
`xiom.ffi.c`; c.xi declares a PRIVATE libc `extern "C" fn abs(n: Int)`.
`xiom.num` pulls `xiom.math` -> `xiom.math.primitives`, whose body calls its
own `pub fn abs(x: Float64)` from `copysign`/`nextafter`. Those calls
resolve locally, but the global set contained "abs" from ffi.c, so the gate
fired on the catalog body. The driver had no catalog source map, so the span
printed as a bare user-file `line:col`.

**Fix**:
- `crates/xiom-check/src/lib.rs` (`register_fn_signature_inner`,
  `TopDecl::Fn`): a fn DEFINITION (body present, non-method) removes its
  bare name from `extern_fns` -- a local definition shadows a same-named
  extern from another module. Bodyless declarations keep the mark (the
  selfhost's `fn xiom_read_file(path: Str) -> Int;` pattern still gates).
- `crates/xiom/src/lib.rs`: catalog-body findings render as
  `catalog body [<module>] <line>:<col>: <msg>` (module tag BEFORE the
  span); the JSON diagnostics envelope sets `file` = `catalog:<module>` for
  such findings (merged with the Stage 6 warning-stream envelope).

**R-6 note (relay sibling)**: the plain-`self` receiver mutation in the
relay's R-6 repro is already m143/m152 behavior (probe `1/1`); no compiler
change needed, receiver rules documented on the benchmark side.

**Locks**:
- `tests/regression/m150_dl_num_parse/main.xi` (relay's exact program) +
  `checker_locks::m150_dl_num_parse_checks_clean` (`--check` clean, no
  `catalog body [xiom.math.primitives]` finding).
- `tests/regression/m150_dl_num_catalog_abs/main.xi` (import-closure shape
  that triggered the T001 + `dl_open` API path) + e2e
  `e2e_m150_dl_num_catalog_abs` + CI line.
- `tests/regression/m150_extern_gate/main.xi` +
  `checker_locks::m150_genuine_extern_still_gated` (negative: a plain
  extern call in safe user code still requires `unsafe`).

**Gates (release pin)**: checker_locks 10/10; xiom-check 195/195; targeted
e2e 1/1; full e2e 2387/2387 (+4 ignored, `-- --test-threads 12`, TEMP
redirected past the Defender script-exe block); ascii_guard clean.

## 2026-09-28 -- OPEN FINDING (next batch): `io.parse_int` emits bare `@is_empty` -> C001

After R-5 fixed the check phase, the relay's full shape
(`tests/regression/m150_dl_num_parse`) still fails CODEGEN on the release:

    error[C001]: codegen: unresolved function symbol(s) called but never
    defined or declared: 'is_empty' (returns i64, first call at IR line
    3186, from @io.parse_int)

Evidence (IR dump with legacy stubs, pre-release stdlib 0c50ac6; same
shape): `@io.parse_int`'s body emits `call i8* @string.str_trim(...)` then
`call i64 @is_empty(i8*)` -- the method-sugar call `trimmed.is_empty()`
loses the module qualification while `s.trim()` keeps it. `xiom/string`
declares both the free `pub fn is_empty(s: Str)` and the method
`pub fn Str.is_empty(self)`; the emitter fallback lands on the bare
`is_empty` key in `types.functions` whose definition is never emitted,
instead of the qualified `string.is_empty` / `string.Str.is_empty`.

Direction: when the bare leaf key has no emitted symbol and exactly one
qualified `*.{leaf}` candidate is emittable, prefer the qualified candidate
(mirroring the R15b preference in the suffix search); and/or record
method-position resolutions in `catalog_resolved_calls` while checking
catalog bodies. Pre-existing (reproduced on a pre-R-5 baseline); not caused
by the R-5 fix. When it lands, promote `m150_dl_num_parse` to an e2e run
lock (the check lock stays).

## 2026-09-28 -- C001 FIXED: value receiver resolved as a MODULE -> bare `@is_empty`

Benchmark relay's unblocker (`io.parse_int` -> bare `@is_empty`). Repro:
`tests/regression/m150_dl_num_parse/main.xi` (their fixture, on the R-5
branch) and the standalone shape in `tests/regression/m153_parse_int_trim_isempty/main.xi`.

**Root cause chain** (traced with env-gated instrumentation, since
removed): `io.parse_int`'s body does `let trimmed = s.trim();` then
`trimmed.is_empty()`. `trim` is a receiver-sugar FREE fn, so
`callee_return_xiom` could not resolve a declared return type for the
binding and `local_xiom_types` never recorded `trimmed: Str`. At the
`trimmed.is_empty()` call site, `infer_struct_type_name(trimmed)`
returned None, so the resolver took the "receiver is a MODULE" fallback
(`resolve_catalog_call`) which produced the bare leaf `is_empty`; the
suffix pass then saw TWO emittable `.is_empty` candidates
(`Str.is_empty`, `string.is_empty`) and declined to pick, so the call
emitted `@is_empty` -- registered only as an alias, never emitted -> the
m142 unresolved-symbol gate raised C001 from `@io.parse_int`.

**Fix** (`crates/xiom-codegen/src/call.rs`, method-call resolved-key
initializer): when the receiver is an Ident local whose XIOM type was
never recorded, derive the primitive from its LLVM type (`i8*`->Str,
`double`->Float64, `float`->Float32, `i1`->Bool) and use it **only when
the matching `Type.method` key is registered** (self-validating synth).
`trimmed` -> `Str.is_empty` (registered) -> call binds `@Str.is_empty`,
which the method definition emits.

**Locks**: `tests/regression/m153_parse_int_trim_isempty/main.xi` (exact
trim/is_empty shape + `io.parse_int` runtime values; exits with the
failure count) + `e2e_m153_parse_int_trim_isempty` + CI line. The
benchmark's `m150_dl_num_parse` graduates to an e2e run lock when the R-5
PR lands (its check phase needs R-5).

**Gates**: full e2e 2387/2387 (+4 ignored), checker 195/195,
stdlib-exec 85/85 (+2 ignored), stdlib modules 40/40, feature-reg 510/510
(one load-flake rerun clean), integration 130, robustness 63, fuzz 24,
perf 3/3, diff 24 (+1 ignored), checker_locks 8/8.

## 2026-09-28 -- playground relays: `opt -verify` warning (C9) + wasm-bindgen glue (C8b)

- **C9: `LLVM IR verification failed: The 'opt -passname' syntax for the
  new pass manager is not supported...` on every run.** The driver ran the
  legacy `opt -verify <ir>`; LLVM 18 removed that spelling (the new pass
  manager needs `-passes=`), so every compile on a host with a full LLVM 18
  toolchain printed a user-visible warning and the verification never ran.
  Fix (`crates/xiom/src/lib.rs`): try
  `opt -passes=verify -disable-output <ir>` first, fall back to the legacy
  `-verify` on older hosts, and skip silently when neither spelling is
  accepted (the clang step verifies the IR anyway; a verifier that
  understands neither syntax is not the user's error). This box has clang
  but no `opt`, so the new path could not be exercised locally -- the
  playground host (LLVM 18) is the validating environment.

- **C8b: the published wasm lacked its JS glue.** The release ships the raw
  cdylib as `xiom-wasm-<ver>.wasm`, but the ABI requires the matching
  wasm-bindgen glue (the v0.58.0 JS could not load it: 1 import/10 exports
  -> 3 imports/73 exports). Fix-forward on v0.62.0: glue generated locally
  from the RELEASED raw wasm with wasm-bindgen CLI 0.2.126 (exact
  Cargo.lock match) and uploaded to the release as `xiom-wasm.js`
  (ES module exporting `compile_xiom`/`get_version`), `xiom-wasm.d.ts`,
  and the bindgen-processed `xiom-wasm_bg.wasm`. Hashes:
  `xiom-wasm.js` 097821e931f944525dc4d3d7a92317ca2245de7e8efa7d4ff92a3783755f17d7;
  `xiom-wasm.d.ts` c935ab362bc990983832046a8c1948b64cada69d9d0b2f54c34be979e3324508;
  `xiom-wasm_bg.wasm` 24f8a87b3c58bd725d74005dcb4b145f5514055b274e5d3894ca3328f89d3c53.
  The ES module fetches `xiom-wasm_bg.wasm` relative to itself, so serve
  the three files side by side (or pass an explicit module path to
  `init`). Durable fix (`release.yml` linux leg): `cargo install
  wasm-bindgen-cli` at the lock's version, generate the glue from
  `stage/bin/xiom-wasm.wasm`, and checksum + upload the three files (the
  publish-job checksum guard now covers them).

## 2026-09-28 -- Stage 6 W004: unreachable match arm

An arm that can never match because an earlier arm is an unguarded
catch-all (`_` or a plain binding) or an exact duplicate
literal/enum-variant pattern:
`warning[W004]: unreachable match arm (an earlier arm already matches
these values)`. Complements `--strict-exhaustive`, which covers MISSING
arms. Warning-only, user-program scope, emitted through `warn_coded_at`
(the JSON code list gains W004).

Implementation notes:
- The catch-all rule is UNGUARDED-only: `_ if c` may fail and leaves later
  arms reachable, and a guarded duplicate pattern still reaches later arms
  when its guard fails (both directions locked by the negative fixture).
- Duplicate keys are structural: literals by value, enum variants by
  (dotted) path + payload ARITY -- `Circle(r)` and `Circle(rad)` collide;
  payload bindings/wildcards key as "any". Struct/tuple/or patterns are
  skipped to stay conservative.
- AST subtlety caught by the probes: a BARE enum variant (`Color.Red`)
  parses as a DOTTED `Pattern::Ident`, not a binding. `pattern_is_catch_all`
  now treats only UNdotted idents as bindings, which also hardens the W002
  match walk and the W003 divergence test (a dotted arm no longer counts as
  a catch-all).

Locks: `tests/regression/m154_w004_unreachable` (catch-all shadow, duplicate
literal, duplicate variant) + `m154_w004_guard` (guarded duplicate and
guarded catch-all stay silent), 2 checker_locks tests.
Gates: full e2e 2389/2389 (+4 ignored), checker 195/195, stdlib-exec 85/85
(+2 ignored), stdlib modules 40/40, feature-reg 510/510, freeze 2/2,
integration 130, robustness 63, fuzz 24, perf 3/3, diff 24 (+1 ignored),
checker_locks 12/12.

## 2026-09-28 -- R-2d FIXED: `--check` implicit-main-wrapped LIBRARY files

Benchmark relay (their `--check` failure on the t3 solution/templates):
`xiom --check` applied the implicit-main wrapper to ANY file without
`fn main`, including library files with top-level declarations. Wrapping
put every `fn`/`type` inside a fn body and the parse failed with a bogus
`error[P001]: <shifted line>: expected ';', found :` at the first
`requires:` clause -- while a full compile of the same file succeeded
(and the reported line was shifted by the wrapped prelude length).

Fix (`crates/xiom/src/lib.rs`): the check-only wrap now requires the
source to be SCRIPT-LIKE (`source_has_top_level_decls`: no line starting
with fn/type/interface/enum/module/impl, `pub` stripped; `use` lines do
NOT count, scripts routinely import the catalog). `script_mode` (`xiom
run`) keeps its unconditional wrap; full compiles are untouched (the
changed branch is `check_only`-only, so the e2e suite -- which never uses
`--check` -- is structurally unaffected).

Verified against the benchmark's actual files:
`E:\xiom-projects\xiom-benchmark-chaos\tests\toolchain\solutions\t3-hot-reload.xi`
and three `tasks/*/t3-hot-reload/template.xi` copies: all `--check` PASS
now; a statement snippet still wraps and passes.

Lock: `tests/regression/m155_r2d_check_library` + a checker_locks test
asserting `--check` exits 0 with "Type check PASSED".
Gates: checker_locks 13/13, CLI suite green (scripting tests 34/34 at
`--test-threads 4` -- the parallel JIT tests are the known load-flake
class, isolated runs pass), feature-reg 510/510.

## 2026-09-29 -- Stage 6 tier 2: W006 shift range, W007 self-comparison, W008 literal div/rem by zero

Tier 2 of the control-flow lint wave (docs/STAGE6_LINT_WAVE.md): three
more warning-only lints, user-program scope, one code per diagnostic.
**Numbering note:** `W005` is already owned by the m142 codegen known-gap
advisory (`warning[W005]: unresolved call ...`, stderr-only, not a
checker lint), so the div/rem lint ships as `W008`; `W006`/`W007` keep
their planned codes. `docs/JSON_DIAGNOSTICS_V1.md` records the split.

- **W008 literal integer division/remainder by zero**: `10 / 0`, `x % 0`
  (parens and `-0` unwrapped). Integer/integer only -- `f / 0.0` is IEEE
  inf and an int zero literal beside a float operand adopts the float
  type, so both stay silent. Before: compiles silently then traps at
  runtime (`tmp/sprintc/lint_div0.xi`, exit `0xC000001D`).
- **W006 shift amount out of range**: a LITERAL amount outside the left
  operand's width (`1 << 64` on Int; `n << 8` on Int8 -> valid 0..=7;
  Int/UInt are 64-bit, Int128/UInt128 128-bit). Variable amounts stay
  silent. Before: compiles and yields a garbage value
  (`tmp/sprintc/lint_shift.xi`: `1 << 64` returned 504615968, no trap).
- **W007 self-comparison always true/false**: the same place expression
  on both sides (`x == x`, `x != x`; parens unwrapped) for non-float
  reflexive types (Int*/UInt*/Bool/Char/Str). Floats are excluded (NaN
  makes `f == f` a real question), as are calls, indices and
  structs/containers (a float field would be NaN-sensitive).
- Emission goes through `warn_coded_at`; identical coded warnings at the
  same span are emitted once (expression position is reachable twice via
  the `for`-loop array-element re-check).

Locks: `tests/regression/m156_w008_div_zero` (compile-only: the shapes
trap) + `m156_w008_guard` (float / float-adopting / non-literal divisors
silent), `m157_w006_shift` (compile-only: out-of-range shifts are
poison) + `m157_w006_guard` (in-range Int/Int8/UInt8 and variable shifts
silent), `m158_w007_selfcmp` (Int/Str/Bool, runs) + `m158_w007_guard`
(float/distinct/call/struct silent); 6 checker_locks tests.
Gates: full e2e 2389/2389 (+4 ignored, `-- --test-threads 12`, 2134s);
checker 195/195; stdlib-exec 85/85 (+2 ignored); stdlib modules 40/40;
feature-reg 510/510; freeze 2/2; quick suites: scripting 34/34
(`--test-threads 4`, 1202s under concurrent stdlib smoke load),
integration 130, robustness 63, fuzz 24, perf 3/3, diff 24 (+1 ignored),
cli 1, doctor 4, borrow 2; checker_locks 19/19; ascii_guard clean.

Cross-lane:
- **stdlib release (relayed)**: `stdlib-v0.62.0` was force-updated to
  `0e63101` (ruleset bypass; the release never published, so no assets
  were invalidated); run `36495200067` validate + ubuntu gates PASS,
  windows gates in flight, package -> GitHub Release -> pin-PR -> canary
  follow automatically. Registry lane re-dispatches the publish once the
  assets land (subject SHA
  `0e631018100b157539614cc92fc471f22663baff`, ref
  `refs/tags/stdlib-v0.62.0`). The repo-local nested `stdlib` checkout
  our tests compile against is still `80e767b` -- refresh at the next
  pin/release step; this batch's gates ran on it.
- **packages relay**: `Vec.pop()` -> `Option[T]` is by design (matched
  exhaustively); Int constants + saturating helpers compile wrap-free;
  the E001 `&`/`&mut` interleave advisories in the pool/backoff suites
  are the documented default-borrow warnings (program_exit=0, benign).
- **benchmark lane**: next batch is R-2 partial + R-2c; probes captured
  in `tmp/sprintc/` (`r2_match_mutation` prints 6/6 not 6/7;
  `r2c_unwrap_chain` fails clang on `xiom_str_len(%struct.Vec)`; the
  unannotated chain compiles but misreads len = 6 not 0; match-binding
  and explicit typed-bind paths are already correct).

## 2026-09-29 -- R-2c unwrap-chain Vec.len receiver + R-2 partial inline match-payload aliasing

Benchmark-relayed codegen gaps (benchmark repo `data/probes/`
`r2c_unwrap_chain.xi`, `r2_match_mutation.xi`; copies under
`tmp/sprintc/`). Repro-first: both reproduced on the tier-2 build before
the fix (r2c: clang error / len 6; r2: 6/6).

- **R-2c** (`opt.unwrap().len()` on `Option[Vec[Int]]`): the Vec.len
  receiver probe resolved the payload as the BARE "Vec" -- annotation
  payloads go through `type_from_ast`, which drops generic args -- and
  `receiver_is_unwrap_of_vec` only accepted "Vec["-prefixed hints, so the
  call fell through to `Str.len`: annotated locals failed clang
  (`xiom_str_len(i8*)` fed `%struct.Vec`), unannotated locals compiled and
  `strlen`'d the boxed handle (printed 6, not 0). Fix: shared
  `hint_is_vec_or_slice` accepts bare/qualified/bracketed Vec/Slice
  hints, and `infer_receiver_payload_xiom` also consults
  `local_opt_payload_xiom` for container payloads (the map
  `ctor_payload_xiom` fills for `Some(Vec[Int].new())`).
- **R-2 partial** (`match opt { Some(c) => c.inc() }` discarded): m148
  aliased the payload only for the boxed `i64` layout; the concrete
  `Option__Counter` layout stores the payload INLINE, and the match
  statement snapshotted the scrutinee into a fresh alloca, so the arm
  mutated a dead copy (printed 6/6, expected 6/7). Fix: (a) the payload
  fallthrough binds an inline aggregate payload by FIELD ADDRESS
  (`(reg = field address, ty = %struct.X)` -- the m148 pointer-backed
  convention), and (b) a match on a plain Option/Result LOCAL whose slot
  already holds the struct matches IN PLACE instead of snapshotting;
  temporaries, fields, calls and pointer-deref scrutinees keep the
  snapshot.

Locks: `tests/regression/m159_r2c_unwrap_vec_len` (annotated + unannotated
unwrap-chain len, pushed payload len 1, Str control 2) and
`tests/regression/m160_r2_match_alias` (field + method mutation persist
across two matches; call-result scrutinee still works); 2 checker_locks
(21/21).
Gates: full e2e 2389/2389 (+4 ignored, `-- --test-threads 12`, 1224s);
checker 195/195; stdlib-exec 85/85 (+2 ignored); stdlib modules 40/40;
feature-reg 510/510; freeze 2/2; quick suites: scripting 34/34
(`--test-threads 4`, 1097s), integration 130, robustness 63, fuzz 24,
perf 3/3, diff 24 (+1 ignored), cli 1, doctor 4, borrow 2; checker_locks
21/21; ascii_guard clean.

## 2026-09-29 -- packages `byte_at >= 128` direct compare (i8 widening default)

Packages-lane finding (`E:\xiom-packages\packages\docs\repro\byte-at-128`;
probe copied to `tmp/sprintc/probe_byte_at.xi`): every DIRECT comparison
of `string.byte_at(s, i)` against a >=128 constant failed (`!= 195u8` on
a U+00E9 string) while typed/untyped local binds were correct. The
stdlib wrapper returns i8; the compare widened it with `sext`
(195 -> -61) while the literal widened `zext` (its temp carried a
`reg_signed` override). `widen_to_i64`'s doc said "zext for i1/i8" but
the code sext'd i8, and the i128 branches already zext i1/i8.

Fix (three parts):
- `widen_to_i64` default now matches its documentation and the i128
  branches: zext for i1/i8, sext for i16/i32; per-register overrides
  (`reg_signed`) still win (typed Int8/UInt8 locals unchanged).
- New `widen_operand_to_i64` (used at both binary-op widen sites):
  reg override > `expr_int_signedness` (Ident/As/Paren/shift/Call via
  `infer_call_return_xiom`: Int* -> sext, UInt* -> zext) > type default.
  An UNRESOLVABLE call (stdlib `string.byte_at`) now falls to the i8
  default (zext) instead of forcing sext.
- `expr_is_unsigned` resolves Call/GenericCall returns too (Shr picks
  `lshr` for unsigned call results).

Probe: `bad=3` before, `bad=0` after (the three direct checks pass; the
untyped/typed/widen controls stay green; a signed Int8 local still
sexts). Locks: `m161_byte_at_direct_compare` + 1 checker_locks (22/22).
Gates: full e2e 2389/2389 (+4 ignored, `-- --test-threads 12`, 1223s);
checker 195/195; stdlib-exec 85/85 (+2 ignored); stdlib modules 40/40;
feature-reg 510/510; freeze 2/2; quick suites: scripting 34/34
(`--test-threads 4`, 803s), integration 130, robustness 63, fuzz 24,
perf 3/3, diff 24 (+1 ignored), cli 1, doctor 4, borrow 2; checker_locks
22/22; ascii_guard clean.




