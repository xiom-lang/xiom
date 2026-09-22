# Post-release plan (after v0.61.1)

Two agreed features plus the AI-context pipeline. Spec'd here so the website
docs and the next compiler sessions can pick them up without re-deriving.

Status: **v0.61.1 shipped** (2026-09-22). Nothing in this document blocks it.

## 1. Verified toolchain updater (`xiom toolchain check` / `update`)

Problem: `xiom update` is retired (it used an unverified channel), so users
must re-download release archives manually. We want one command with no
weakening of the supply-chain posture.

Commands:
```
xiom toolchain check            # current vs latest release (no changes)
xiom toolchain check --json     # {current, latest, platform, up_to_date, notes}
xiom toolchain update           # verify + stage + atomic swap + rollback
xiom toolchain update --dry-run # download/verify only
```

Rules (non-negotiable):
1. Source is the GitHub Releases API for `xiom-lang/xiom` only; no git
   channel, no third-party mirrors.
2. Verify **SHA256SUMS** and the **build-provenance attestation**
   (`gh attestation verify` equivalent in-process) before unpacking.
3. Swap `bin/` and `lib/` atomically (stage into a sibling dir, rename);
   keep the previous `bin/`/`lib/` under `rollback/` for one version.
4. Never touch user data (`$XIOM_HOME`, projects, the script cache). The
   bundled stdlib upgrades with `lib/`.
5. Refuse when the toolchain was installed by a package manager (marker file
   or path heuristic) and point at that manager instead.
6. `--json` everywhere for agents/CI; exit codes: 0 ok/up-to-date, 1 update
   available (with `check`), 2 verification failed, 3 permission/install-kind.

Acceptance: on a scratch install of v0.61.1, `check` reports v0.61.2 when a
new tag exists; `update` verifies + swaps; `xiom --version` and the bundled
stdlib both change; rollback restores v0.61.1; a tampered archive is refused
with exit 2 and leaves the install untouched.

## 2. MCP contract queries (`get_contracts`, `search_symbols`)

Problem: contracts are XIOM's differentiator, but agents must know which of
`get_contract_signature` (project files) or `xiom_stdlib_reference` (stdlib)
to call, and get prose instead of fields.

`get_contracts`:
```json
{ "symbol": "xiom.string.str_concat" }        // or a project symbol
{ "symbol": "...", "verify": true }           // fold in Z3 results
```
Response (structured, one object per symbol):
```json
{
  "symbol": "xiom.string.str_concat",
  "module": "xiom.string",
  "signature": "str_concat(a: Str, b: Str) -> Str",
  "requires": [], "ensures": ["result.len() == a.len() + b.len()"],
  "invariants": [],
  "pre_refs": [], "post_refs": [{"clause": "...", "line": 42}],
  "qualified": true,
  "verify": { "status": "proved|unknown|counterexample", "counterexample": "..." }
}
```
`search_symbols`: `{ "query": "concat" }` -> ranked `[{symbol, module,
signature}]` across the bundled stdlib **and** the current project; fuzzy on
leaf names, exact on qualified names.

Rules: resolution must use the same catalogs the compiler uses (bundled
stdlib at `lib/`); no shelling out to a network service; project symbols come
from the open workspace; results cached per snapshot.

Acceptance: an agent can go from a task ("concatenate two strings safely") to
the exact signature + pre/postconditions of the stdlib function in one call;
`verify: true` returns counterexamples for a deliberately failing clause;
unknown symbols produce a clear error with close matches.

## 3. AI context pipeline (shipped in R64)

`AI_CONTEXT.md` (repo root) is the single source of truth for toolchain
facts. It is:
- **compiled into** `xiom-mcp` (`include_str!`) and served as
  `xiom_workflow_guide {topic: "context"}` — always matches the build;
- **shipped** in every archive at `lib/AI_CONTEXT.md` (release staging +
  `package.ps1`/`package.sh`);
- the reference the website docs can render for an "AI/agent usage" page.

Maintainers: update `AI_CONTEXT.md` whenever syntax, flags, error codes,
registry behavior, or known limitations change (same rule the ROADMAP lists
for MCP knowledge). The MCP guide constants in
`crates/xiom-mcp/src/guides.rs` should stay a curated subset, not a copy.

Website pickup: render `AI_CONTEXT.md` + this file at build time; the release
payload for the docs dispatch carries `{tag, stdlib_ref, compiler_ref}`.
