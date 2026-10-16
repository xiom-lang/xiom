# m339 / XVC-C-16 evidence (linux clang-18 /search miscompile)

Full bundle: `%TEMP%\kilo\xvc-c-16\` (see `m339-fix-evidence.md` there);
WSL working dir `/home/lefteris/m339/`.

## Fix

`crates/xiom/src/lib.rs`: x86_64 native clang invocations add
`-mllvm -x86-disable-avoid-SFB=true` when the clang major version is 18
(the LLVM-18 hidden disable for the X86AvoidStoreForwardingBlocks codegen
pass; declared in `X86AvoidStoreForwardingBlocks.cpp`). Gated by
`clang_major_version()` (+ unit lock `clang_major_version_gate_m339`),
so Windows/clang-22 and other toolchains keep their exact previous argv.
The pass is a store-forwarding-stall performance fixup; no IR, ABI, or
runtime-library change.

## Mechanism (all on the byte-identical v0.64.5 emitted IR, WSL clang 18.1.3)

- `-O0` green; `-O2`/`-O3` red -> optimizer-stage miscompile only.
- `clang-18 -mllvm -opt-bisect-limit=N`: N=98456 green, N=98457 red; the
  one added instance is `X86 Avoid Store Forwarding Blocks` on
  `vector.api.http_service.search_req_from_json` (deterministic reruns).
- MIR before/after: the 32-byte VMOVUPS load/store pair of the 128-byte
  `SearchRequest` materialization copy becomes interleaved 8/4/8/4/8
  scalar copies; ptrace shows src = dst - 0x10 at that point (runtime
  overlap), so the interleaved order reads bytes its own earlier partial
  stores already clobbered -> corrupted request fields -> 404/400 wire
  failures from the service.
- Binary-patch A/B (same build, same size/layout): substituting the
  all-loads-first pair back -> green 8/8; equivalent GP copies -> red.

## Acceptance

- WSL clang 18.1.3: without flag rc 3 (3 failed checks) at -O2 and -O3;
  with flag rc 0 both (3x repeats deterministic).
- Windows (clang 22, gate inactive): fixed compiler builds the repro and
  it exits 0, 9/9 checks PASS.
- `--emit-ir` unfixed vs fixed, same env: byte-identical
  (sha256 5C3908D9386C3012EC194F8D0D08BF8BD1188327458A82E1AA9F6ED8349E839E).
- Battery (Windows fixer worktree, m334 precedent; log
  `m339-battery.log` in the evidence bundle):
  e2e g1 16/16 x2; e2e g2 241/241 x2 (full ci.yml group-2 line);
  codegen lib 12/12; feature_regression 549/549;
  stdlib_execution 85/85 (+2 ignored); integration 130/130.
- Risk probe: clang 18.1.3 accepts `-mllvm -x86-disable-avoid-SFB=true`
  (rc 0, silent); an unknown `-mllvm` option is a hard clang error
  (rc 1, "Unknown command line argument") -- a build without the option
  fails loudly, never silently miscompiles; clang 22.1.8 also still
  accepts the option (unused there: the major==18 gate keeps every other
  toolchain's argv byte-identical).

## Queued follow-up (not part of m339)

Any recompile perturbation (even an inert unused global) re-rolls clang
codegen and exposes a second layout-sensitive corruption in the create
path that is NOT the SFB pass (survives the disable flag; binary diff =
only BB order + rip displacements). Needs its own bisect.
