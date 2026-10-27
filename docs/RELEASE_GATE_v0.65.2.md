# Release gate -- v0.65.2

STATUS: **CANDIDATE READY 2026-10-27** -- `stdlib-v0.65.2` is live
(tag `7d4d48f`, registry `xiom-std 0.65.2` sha256 `9638b576...`, release run
40523878342 green). The amended candidate `8ec0904a5` pins it and includes
the engine borrowed/aggregate Vec reclaim. CI **40559690805 SUCCESS** and the
all-platform dry-run **40559700656 SUCCESS** are green on that commit. Tag
`v0.65.2` is **not cut**; owner GO is the only remaining gate.

## Scope

- **Packed Vec element slots** (`vec_slot_size` + stdlib `elem_size`):
  `Vec[UInt8]`/Bool/Int8 no longer over-allocate 8x. t1 Peak Memory target.
- **arm64 contract payload** (`3e775908a`): `Result[Str, E]` field 1 loads
  its real `i8*` type in the contract evaluator; arm64 floor CI green.
- **1609** (`623afca78`): inline `Result.value[i]` element type comes from
  `field_payload_xiom`, not the erased slot.
- **Str bound-local drop** (`b07137eb1`): fresh concat/numeric-format
  locals are freed at scope exit; first non-Vec retention slice.
- **Borrowed/aggregate-moved Vec reclaim** (`REL-20261027-1717-engine`):
  a non-retaining `&`/`&mut` argument to an ordinary safe call keeps the
  caller's ownership; a local struct literal with a fresh Vec field drops
  the field; returned aggregates are excluded; field data pointers are
  captured at the binding so nested arms do not emit non-dominating IR.
  Editor probes: 429 MB -> 5 MB (borrow), ~1 GB -> 6 MB (aggregate).
- **stdlib-v0.65.2**: packed Vec slots + HTTP/2 primitives (stdlib lane).

Not in this cut: Map/JSON/HTTP retention, unbound Str temporaries, the
XgView/Vec slot-type confusion (needs the engine IR slice), the remaining
correctness rows (playground by-ref fn pointer, Int+Str, W005, macOS memset),
t3 perf, engine C09/Vulkan. No v0.64.9.

## Preconditions

- [x] `stdlib-v0.65.2` tag live and pinned (`STDLIB_VERSION`).
- [x] Workspace version 0.65.2; `Cargo.lock` updated.
- [x] Release notes `v0.65.2.md` converted with the pinned stdlib fragment
      (6 highlights, 0 breaking, 1 docs).
- [x] selfhost/VSIX version pins updated.
- [x] arm64 fix + 1609 + Str drop + borrowed/aggregate reclaim merged to
      main; the ownership branch e2e green.
- [x] Final candidate build + **14** hard-RSS gates on debug and release
      (all green; packed 4 MiB under 24 MB; str 5 MB; borrow 5 MB;
      aggregate 6 MB; return constructor 5 MB).
- [x] Full e2e debug+release on the amended tree: **2649/2649 (4 ignored)**
      on both.
- [x] Push the amended commit `8ec0904a5`; CI **40559690805 SUCCESS**
      (all matrix, arm64 floor, MCU).
- [x] All-platform dry-run **40559700656 SUCCESS** on `8ec0904a5`
      (Windows/Linux x64+arm64, macOS x64+arm64, VSIX, SHA256SUMS).
- [ ] Owner GO; annotated tag `v0.65.2`; release workflow (11 assets).
- [ ] Registry `xiom-std 0.65.2` (already live), docs dispatch, lane
      announcements, holds closed.

## Evidence so far

- Vec/t1 combined branch: `p_wave289_vec8` exit 0; 10/10 gates; 4 MiB
  `with_len` under 24 MB; e2e 2648/2648 (pre-1609/Str).
- 1609: stdlib probe exit 0; `e2e_m1609`; e2e 2649/2649 both.
- Str drop: `str_loop_leak` (2M x 20 B, 24 MB) = 5 MB; 14/14 gates; e2e
  2649/2649 both.
- Engine ownership: `borrow_vec_leak` 5 MB (was 429 MB),
  `aggregate_vec_leak` 6 MB (was ~1 GB), `aggregate_return_leak` 5 MB;
  m206/m283/m318/m37 green.
- arm64: CI `40411862697` green including `ubuntu-22.04-arm`.
- stdlib: full battery 533/533 modules, 963/963 smokes, 411/411 probes,
  barename 0/533; registry signed.

## Known residual (not this cut)

- XgView/Vec slot-type confusion (engine item section 2) -- needs the
  minimal IR slice; blocks the editor buffer-reuse workaround.
- Map/Set container drops, `Vec[Str]` element drops, JSON enum payload drop,
  HTTP request builders, unbound Str temporaries.
- By-value aggregate parameters are caller-retained for now; callee-side
  parameter drops are future work.
- Actually-taken retry rebind leaks only the replaced buffer.
- Engine C09/Vulkan and the remaining correctness rows are carried.

## Release log

- [x] Final candidate commit `8ec0904a5`; CI `40559690805`; dry-run
      `40559700656`.
- [ ] Tag `<annotated>`; release `<id>`; 11 assets + SHA256SUMS.
- [ ] Docs dispatch, announcements, holds closed.
