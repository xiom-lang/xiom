# Release gate -- v0.65.2

STATUS: **CANDIDATE PREPARED 2026-10-27** -- `stdlib-v0.65.2` is live
(tag `7d4d48f`, registry `xiom-std 0.65.2` sha256 `9638b576...`, release run
40523878342 green). The compiler candidate pins it on
`feat/v0.65.2-candidate`. Tag `v0.65.2` is **not cut**; CI, the all-platform
dry-run, and owner GO remain.

## Scope

- **Packed Vec element slots** (`vec_slot_size` + stdlib `elem_size`):
  `Vec[UInt8]`/Bool/Int8 no longer over-allocate 8x. t1 Peak Memory target.
- **arm64 contract payload** (`3e775908a`): `Result[Str, E]` field 1 loads
  its real `i8*` type in the contract evaluator; arm64 floor CI green.
- **1609** (`623afca78`): inline `Result.value[i]` element type comes from
  `field_payload_xiom`, not the erased slot.
- **Str bound-local drop** (`b07137eb1`): fresh concat/numeric-format
  locals are freed at scope exit; first non-Vec retention slice.
- **stdlib-v0.65.2**: packed Vec slots + HTTP/2 primitives (stdlib lane).

Not in this cut: Map/JSON/HTTP retention, unbound Str temporaries, the
remaining correctness rows (playground by-ref fn pointer, Int+Str, W005,
macOS memset), t3 perf, engine C09/Vulkan (awaiting minimal repros). No
v0.64.9.

## Preconditions

- [x] `stdlib-v0.65.2` tag live and pinned (`STDLIB_VERSION`).
- [x] Workspace version 0.65.2; `Cargo.lock` updated.
- [x] Release notes `v0.65.2.md` converted with the pinned stdlib fragment
      (6 highlights, 0 breaking, 1 docs).
- [x] selfhost/VSIX version pins updated.
- [x] arm64 fix + 1609 + Str drop merged to main; 1609/Str branches e2e
      green.
- [x] Final candidate build + 11 hard-RSS gates on debug and release
      (all green; packed 4 MiB under 24 MB; str 5 MB).
- [x] Full e2e debug+release on the final pinned tree: **2649/2649 (4
      ignored)** on both.
- [ ] Push main; CI green (all matrix, arm64 floor, MCU).
- [ ] All-platform dry-run green on the final commit.
- [ ] Owner GO; annotated tag `v0.65.2`; release workflow (11 assets).
- [ ] Registry `xiom-std 0.65.2` (already live), docs dispatch, lane
      announcements, holds closed.

## Evidence so far

- Vec/t1 combined branch: `p_wave289_vec8` exit 0; 10/10 gates; 4 MiB
  `with_len` under 24 MB; e2e 2648/2648 (pre-1609/Str).
- 1609: stdlib probe exit 0; `e2e_m1609`; e2e 2649/2649 both.
- Str drop: `str_loop_leak` (2M x 20 B, 24 MB) = 5 MB; 11/11 gates; e2e
  2649/2649 both; CI `40532172041` fully green (arm64 floor, MCU).
- arm64: CI `40411862697` green including `ubuntu-22.04-arm`.
- stdlib: full battery 533/533 modules, 963/963 smokes, 411/411 probes,
  barename 0/533; registry signed.

## Known residual (not this cut)

- Map/Set container drops, `Vec[Str]` element drops, JSON enum payload drop,
  HTTP request builders, unbound Str temporaries.
- Actually-taken retry rebind leaks only the replaced buffer.
- Engine C09/Vulkan and the remaining correctness rows are carried.

## Release log

- [ ] Final candidate commit `<sha>`; CI `<id>`; dry-run `<id>`.
- [ ] Tag `<annotated>`; release `<id>`; 11 assets + SHA256SUMS.
- [ ] Docs dispatch, announcements, holds closed.
