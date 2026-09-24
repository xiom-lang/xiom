# Front-end / first-run audit + backlog (2026-09-24)

Read-only audit of everything a NEW USER sees from the installed compiler:
`xiom doctor`, the retired/deprecated surfaces still reachable or referenced,
the installers, help output, and the stdlib/registry story. No code was
changed; this is the prioritized backlog requested after the Win11 laptop
report (`xiom doctor` said LLVM was missing although LLVM was installed, and
pointed at a dead `xiom install llvm` command).

Evidence is file:line against `main` @ 902f0452 (R66-R72 + m125/m126 batch).
The registry facts come from a read-only fetch of
`https://registry.xiom-lang.org/index.json` on 2026-09-24.

## 0. What is already production-grade (keep)

* Release archive layout is sane and verified: `bin/` (9 tools + pinned
  `z3`), `lib/xiom`, `lib/runtime`, `lib/package.xi`, licenses; every staged
  tool must run and self-report the version before the archive is sealed
  (`.github/workflows/release.yml:306-324`).
* Asset bytes are deterministic as of m126, so re-running a release job no
  longer invalidates a canary by itself (registry-lane relay 5c1cdcb agrees).
* The stdlib resolver is careful: version-pinned sibling `lib/` beats a stale
  global `XIOM_HOME`, content validation rejects unrelated `lib/`
  (`crates/xiom-graph/src/paths.rs:31-40,88-105,239-241`).
* `xiom install` already delegates to the verified `xiom pkg install` client
  with a deprecation note (`crates/xiom/src/main.rs:600-606`).
* `xiom doctor` uses the same canonical XIOM_HOME resolver as the installers
  for the home path, and its z3 message matches the archive
  (`crates/xiom/src/main.rs:1794-1812`).
* The API-freeze gate now runs in CI (m125).

## 1. Findings -> backlog (prioritized)

P0 = user-visible wrong/broken guidance, fix in the next batch.
P1 = first-run quality / correctness gaps, plan next.
P2 = polish and long-tail.

### FE-1 (P0) `xiom doctor` cannot see an LLVM that is not on PATH

* Evidence: doctor probes only `Command::new("clang").arg("--version")`
  (`crates/xiom/src/main.rs:1799-1800`). The compiler itself falls back to
  `C:\Program Files\LLVM\bin\clang.exe`, `/usr/bin/clang`,
  `/usr/local/bin/clang` via `find_tool`
  (`crates/xiom/src/lib.rs:1218-1222,1981-1992`). `install_deps.ps1` already
  knows the winget locations (`install_deps.ps1:150-155`).
* User impact (the report): LLVM installed at the standard location, compiler
  works, doctor says "clang NOT FOUND". Re-running the docs one-liner does not
  change PATH in the already-open terminal, so doctor keeps failing.
* Fix: one shared toolchain probe used by doctor AND the driver: PATH first,
  then the per-OS candidate list (Windows: Program Files, `%LOCALAPPDATA%`
  winget packages, `%LOCALAPPDATA%\Programs\LLVM`, ProgramData winget; Unix:
  `/usr/bin`, `/usr/local/bin`, `/opt/homebrew/opt/llvm/bin`,
  `/usr/lib/llvm-*/bin`). Report the RESOLVED ABSOLUTE PATH and the version
  string, not just OK/FAIL.
* Acceptance: with LLVM installed but off PATH, doctor prints
  `[OK] clang <version> at <path>`; with LLVM absent, it prints the exact
  install command for the detected OS/package managers.

### FE-2 (P0) doctor's remediation command is dead

* Evidence: `run: xiom install llvm` (`crates/xiom/src/main.rs:1800`);
  `xiom install` is deprecated and forwards to `xiom pkg install llvm`
  (`crates/xiom/src/main.rs:600-606`) -- there is no `llvm` package
  (registry index, 2026-09-24), so the user gets a package-not-found error.
* Fix: remove the string. Print OS-specific remediation instead
  (winget/choco on Windows, apt/dnf/pacman, brew), plus the docs URL, plus
  `xiom doctor --fix` as an opt-in auto-installer (reuse `install_deps`
  logic, do not duplicate it).
* Acceptance: `xiom doctor` output contains no `xiom install` reference
  (grep-able); the suggested command works on each OS.

### FE-3 (P0) doctor ignores NASM and the driver hardcodes a personal path

* Evidence: doctor checks clang, z3, home, stdlib, packages only
  (`crates/xiom/src/main.rs:1794-1814`). The driver probes NASM
  (`crates/xiom/src/lib.rs:1181-1186,1995-2007`) and the Windows candidate
  list contains `C:\Users\lefte\AppData\Local\bin\NASM\nasm.exe` -- a
  developer's home directory in shipped code (`crates/xiom/src/lib.rs:1996`).
* Fix: share the probe; replace the personal path with
  `%LOCALAPPDATA%\bin\NASM`, `%ProgramFiles%\NASM`, `%ProgramFiles(x86)%`,
  PATH. Doctor reports NASM as `[--]`/`[OK]` with the note that it is
  optional (C fallbacks).
* Acceptance: NASM installed via `winget install NASM.NASM` is detected by
  both the driver and doctor without manual PATH edits; no absolute
  user-specific path remains in the sources.

### FE-4 (P1) doctor does not verify version parity

* Missing checks: compiler version vs bundled stdlib version
  (`lib/package.xi`), clang version floor, runtime presence
  (`lib/runtime/xiom_runtime.c`), and duplicate/second installs on PATH
  (the `xiom.bat` launcher hardcodes `%LOCALAPPDATA%\xiom\bin` FIRST, so a
  stale install shadows the binary the user just unpacked --
  `xiom.bat:5-12`).
* Fix: doctor prints one "identity block": resolved `xiom.exe` path, install
  root, `XIOM_HOME`, stdlib root + version, LLVM path + version, NASM path,
  z3 path/version; warns on compiler/stdlib mismatch and on multiple
  installs.
* Acceptance: a deliberately mismatched `lib/package.xi` produces a warning;
  two installs on PATH produce a warning naming both paths.

### FE-5 (P1) doctor uses its own stdlib path instead of the shared resolver

* Evidence: doctor checks `XIOM_HOME/lib/xiom` only
  (`crates/xiom/src/main.rs:1804-1812`); the real resolver prefers an
  exe-adjacent `lib/` before `XIOM_HOME`
  (`crates/xiom-graph/src/paths.rs:31-40,239-241`). A dev checkout or a
  custom `--install-dir` can therefore be reported as "stdlib missing".
* Fix: call `xiom_graph::paths::stdlib_root()` and print the root it
  selected; keep the "searched" list only when it returns None.
* Acceptance: doctor always reports the same root the compiler resolves for
  the same environment.

### FE-6 (P1) doctor does not check the link toolchain or the runtime

* The compiler already produces a good hint when `stdio.h` is missing
  (`crates/xiom/src/lib.rs:1441-1446`, MSVC Build Tools + `install_deps.ps1`)
  but doctor does not check it first, and never checks
  `lib/runtime/xiom_runtime.c` (required to link `--target native`).
* Fix: optional `xiom doctor --deep` that compiles a trivial program
  end-to-end (the only check that proves the whole chain) plus a runtime
  presence check; doctor's default output stays fast.
* Acceptance: on a machine without MSVC, doctor names the exact missing
  component before the user tries to build.

### FE-7 (P1) doctor is not machine-readable

* No `--json`, no documented exit codes (`crates/xiom/src/main.rs:1794`).
  The website/installers/CI cannot verify an install programmatically.
* Fix: `xiom doctor --json` with the same fields as FE-4/FE-5 plus
  `{ ok, warnings[], errors[] }`; exit 0 all-OK, 1 warnings, 2 errors.
* Acceptance: installer one-liners end with `xiom doctor --json` and fail
  loudly on `errors`.

### FE-8 (P0) deprecated command surfaces still reachable or referenced

* `xiom publish` (`crates/xiom/src/main.rs:528,619`) still runs the legacy
  git-tag flow and tells users packages become available "via
  `xiom install {name}`" (`crates/xiom/src/main.rs:1464-1521`, message at
  1520) -- the modern path is `xiom pkg publish` (verified client).
* `xiom update`'s retirement text points at
  `xiom pkg install <package>@<version>` (`crates/xiom/src/main.rs:608-611`)
  -- wrong category: users running it want to update the TOOLCHAIN.
* `xiom pkg --help` example `xiom pkg install xiom.stdlib`
  (`crates/xiom-pkg/src/main.rs:1143`) -- no such name in the registry.
* Docs still document `xiom install`/`xiom publish` as the package commands
  (`docs/COMPILER_IMPROVEMENT_PLAN.md:560-561,853-862`).
* Fix: alias `xiom publish` -> `xiom pkg publish` with a deprecation note
  (mirror `xiom install`), delete the git-tag flow, reword `xiom update`
  to point at the reinstall one-liner (and `xiom toolchain update` when it
  ships), fix the pkg help example, mark the old plan doc as historical.

### FE-9 (P0) registry stdlib naming is inconsistent

* Live registry (2026-09-24): the stdlib is published as `xiom-std` v0.61.3
  (`publisher: xiom-lang/stdlib`, tag `stdlib-v0.61.3`, commit
  `c7b4027...` = the compiler pin). Package manifests depend on `xiom.std`
  (dotted) and the client's canonical name is `xiom.std` with `xiom-std` as
  a legacy alias (`crates/xiom-pkg/src/main.rs:471-478`), while
  `registry::is_platform_dep` deliberately keeps BOTH out of the registry
  closure (`crates/xiom-pkg/src/registry.rs:1-6`).
* Consequence: `xiom pkg install xiom.std` and `... xiom.stdlib` do not
  resolve against the registry; only `xiom pkg install xiom-std` would.
* Fix (registry lane + client): publish/alias the stdlib under the dotted
  `xiom.std` (R50 naming), keep `xiom-std` accepted client-side, and verify
  `xiom pkg install xiom.hello` end-to-end on the pinned registry.

### FE-10 (P1) installers drift from the release layout and tool set

* `install.ps1` copies only 6 of the 9 tools (missing `xiom-mcp`,
  `xiom-verify`, `xiom-dbg`; `install.ps1:139,165-168`) and puts the runtime
  at `$installDir\runtime` (`install.ps1:288-291`) while the archive puts it
  at `lib/runtime` (`.github/workflows/release.yml:310`); `install.sh` has
  the same 6-tool list and `$INSTALL_DIR/runtime`
  (`install.sh:39,80,94,124-131`). The resolver only finds the odd layout
  through the legacy ancestor fallback
  (`crates/xiom/src/lib.rs:1798-1799`).
* Fix: make both installers mirror the archive exactly (all 9 tools, runtime
  under `lib/`), and have install.ps1/sh finish by running
  `xiom doctor` (FE-7 JSON when CI).

### FE-11 (P2) `install_deps.ps1` LLVM fallback is stale and unsigned

* Direct-download fallback pins LLVM 19.1.0
  (`install_deps.ps1:189-196`) and runs the downloaded installer without any
  SHA256 verification -- a supply-chain gap in an otherwise careful project.
* Fix: prefer winget/choco only; if a direct fallback stays, pin the URL and
  SHA256 centrally (e.g. `tools/llvm-pins.json`, like `tools/z3-pins.json`)
  and verify before executing.

### FE-12 (P2) `--help` is developer-flavored and hides the user surface

* `print_usage` is one flat list with sprint tags (`7E.1`, `D2.1`, `5e.5f`)
  and no mention of `doctor`, `pkg`, `fmt`, `lsp`, `mcp`, `dbg`, `verify`
  (`crates/xiom/src/main.rs:1212-1300`).
* Fix: grouped help ("Getting started: `xiom doctor`", "Project", "Tools",
  "Advanced") without internal sprint references; add a `--help` line for
  the tool dispatcher.

### FE-13 (P2) "which xiom am I running" is invisible

* `xiom.bat` prefers `%LOCALAPPDATA%\xiom\bin` over its own directory
  (`xiom.bat:5-12`), so an old install shadows a fresh one; nothing prints
  the resolved binary/root. Covered partially by FE-4.
* Fix: doctor identity block (FE-4) + `xiom --version` including the install
  root; make `xiom.bat` prefer its own directory first (or delete the bat
  once the launcher is unnecessary).

### FE-14 (P2) install success banner teaches legacy usage

* `install.ps1:319-321` suggests `xiom --help` / `xiom compile hello.xi`;
  `install.sh:182-184` suggests `xiom compile file.xi`. The modern entry
  points are `xiom run hello.xi` / `xiom doctor`; `compile` still works via
  the launcher but is not the documented idiom.
* Fix: banner ends with `xiom doctor` + `xiom run hello.xi`.

### FE-15 (P2) uninstall leaves PATH entries behind

* The generated uninstaller deletes the install dir and `.xi` association
  but explicitly tells the user to clean PATH by hand
  (`install.ps1:271-272`).
* Fix: remove the install's bin dir from the user PATH in the uninstaller.

### FE-16 (P2) doctor's stdlib check ignores pin metadata

* `lib/package.xi` (shipped) carries the stdlib version; nothing compares it
  to the compiler's expected pin (`STDLIB_VERSION`) at runtime. Covered by
  FE-4; listed separately because it is the cheapest guard against
  half-updated installs (new `bin/`, old `lib/`).

## 2. Decision briefs (the two questions)

### D-1 Should the stdlib be installed as a registry package (`xiom.std`)?

Current model (verified): the stdlib is a PLATFORM dependency. It ships inside
the release archive (`lib/xiom`, `lib/runtime`, `lib/package.xi`, pinned by
`STDLIB_VERSION` at build time), the resolver prefers the install-adjacent
`lib/`, and `xiom.std`/`xiom-std` are deliberately excluded from registry
dependency resolution (`crates/xiom-pkg/src/registry.rs`, `is_platform_dep`).
The registry does additionally host `xiom-std` v0.61.3 (same commit as the
pin), but nothing in the client consumes it for dependency installs.

Recommendation: KEEP the bundled default (it guarantees the compiler<->stdlib
pin parity every gate in this campaign depends on), and define the registry
artifact's role explicitly instead of making it the default:
1. Rename the registry entry to the dotted `xiom.std` (R50 naming) with
   `xiom-std` accepted as a client alias; fix the `xiom.stdlib` help example.
2. Document `xiom pkg install xiom.std@<ver>` as the ADVANCED path (CI,
   archival pinning, custom layouts) -- not something a beginner needs.
3. Have doctor PRINT the stdlib version and its root (FE-4), so "which
   stdlib am I using" is answerable without installing anything.

### D-2 Should there be an update mechanism?

Yes. It is already spec'd and agreed (`docs/POST_RELEASE_PLAN.md:8-39`):
`xiom toolchain check|update` with GitHub-Releases-only source, SHA256SUMS +
build-provenance attestation verification, atomic swap with one-version
rollback, and a refusal path when the toolchain was installed by a package
manager. Recommendations on top of the spec:
1. Sequence it AFTER the P0 doctor fixes: users meet `doctor` first; it
   should recommend `xiom toolchain update` once the command exists.
2. Do NOT resurrect a repo-local `xiom update` package channel; keep the
   single verified client.
3. The bundled stdlib rides with `lib/` on update (spec rule 4) -- this is
   the payoff of D-1: one command updates compiler + stdlib in lockstep.
4. Before the attribution/attestation verification exists (dependency
   procurement), ship `xiom toolchain check --json` alone: it can compare
   the installed version to the latest tag with no new dependencies and
   tells installers/CI whether an update exists.
5. Interim: doctor prints the exact per-OS reinstall one-liner (FE-2), so
   users are never stuck.

## 3. Suggested sequencing

* Sprint A (P0, small): FE-1, FE-2, FE-3, FE-8, FE-9 (name alignment).
* Sprint B (P1): FE-4, FE-5, FE-6, FE-7, FE-10, then `xiom doctor --json`
  in the installer one-liners.
* Sprint C (P2): FE-11..FE-15 + `xiom toolchain check --json` (D-2 step 4).
* Later: the full verified updater (POST_RELEASE_PLAN 1) and MCP queries.

## 4. Verification notes / limits

* The laptop failure is explained by code inspection (PATH-only probe with no
  fallback) and a local check that the driver DOES find clang at the standard
  Windows location; it was not reproduced on the reporter's machine.
* The registry facts are a single read-only fetch of the live index on
  2026-09-24; the end-to-end `xiom pkg install xiom.hello` name-resolution
  test (FE-9) still needs one run against the pinned registry.
* The public website one-liner script was NOT audited (it is not in this
  repo); the repo installers (`install.ps1`, `install.sh`,
  `install_deps.*`, `xiom.bat`) were.
