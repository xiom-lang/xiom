# Release gate -- v0.65.4

STATUS: **PREPARING** -- post-release hotfix after v0.65.3.

## Scope

- `REL-20261030-1540-bindings` (epos): the aggregate drop must register its
  field/data GEPs in the entry block. Fixed `34e42ad87`; epos 29/29;
  witness `aggregate_arm_drop`.
- Engine 200K-LOC build: `prepass_owned_vec_locals` rebuilt/cloned
  program-global catalogs per function (O(functions^2); 12,238 injected
  decls). Fixed `13fc28352` (built once, shared by Arc). Full editor build
  `build: OK -> xiom-editor.exe`, 34,764,288 B, ~29 min.
- stdlib `stdlib-v0.65.4` at `d267317`: int64 socket ABI on every target
  (arm64) + POSIX SIGPIPE guard (`REL-20261030-1628-pulse`,
  `REL-20261030-1645-xvector`).

Not in this cut: Map/JSON/enum retention (REL-1704), Vec[Str] element
strings, removal containers, callee-side by-value parameter drops.

## Preconditions

- [x] epos 29/29 debug + release; witness + e2e lock.
- [x] 20/20 memory gates (debug + release).
- [x] Transport 36/36 x5 debug + release (b4 + u4) with stdlib 0.65.4.
- [x] Engine editor build completes (34,764,288 B, ~29 min).
- [x] stdlib tag `stdlib-v0.65.4` live at `d267317` (package.xi 0.65.4).
- [ ] Full e2e debug + release on the final commit.
- [ ] CI green on the final commit.
- [ ] Version bump 0.65.4 + STDLIB_VERSION pin on `rc/v0.65.4`.
- [ ] Release dry-run; lanes verify the archive.
- [ ] Tag `v0.65.4`; release; announcements.

## Evidence

- Compiler commits: `34e42ad87` (epos GEP dominance), `13fc28352`
  (cached drop-analysis).
- Engine: previous run 2h24m at 100% CPU, no IR; now ~91 MB IR in ~20 min
  and a full run/build in ~29 min.
- e2e and CI recorded below at cut time.
