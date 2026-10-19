# m374 -- Parallel checker/codegen units (design/queue)

Status: QUEUED for design. Implementation waits for selfhost Phase 6
parity; not part of v0.64.8 or the m373 config plumbing.
Owner: compiler lane (architecture + selfhost mirror)
Sources: `docs/SESSION-HANDOFF.md` 2026-10-19 next-wave decision;
`docs/rustc-study/*` parallel notes; 7C/I2 existing paths.

## Goal

Extend parallelism beyond lex/parse and per-function emission:

1. Check modules in dependency-DAG order with independent ready modules
   checking concurrently.
2. Partition codegen into deterministic codegen units (CGUs) with a stable
   symbol ownership map.
3. Keep diagnostics and emitted IR deterministic regardless of worker
   scheduling.

## Design questions to settle first

- What is the module-DAG unit? (file, module path, or catalog item).
- Which checker state is truly per-module and which is global? Identify the
  shared caches (type registry, trait/impl tables, const eval) and their
  synchronization or phase separation.
- What is the CGU partition key (module, function group, or size target)?
  How are cross-CGU calls/internalization handled without LTO regressions?
- Determinism contract: stable ordering for diagnostics, symbol names,
  CGU assignment, and IR emission; a scheduling-independent output hash.
- Selfhost mirror: the same partition and ordering rules must be expressible
  in `selfhost/src` once Phase 6 parity is reached.

## Plan (design/queue only for now)

- [ ] Write the design note with a module-DAG/checker sequence diagram and a
      CGU ownership diagram (Mermaid), plus the determinism contract.
- [ ] Inventory the checker's mutable shared state and classify it:
      per-module, read-only after registration, or global-serial.
- [ ] Prototype the DAG scheduler on a scratch branch with the existing
      checker; measure on engine/xvector/orbitdb, not on toy files.
- [ ] Define the CGU partition and symbol-ownership map; prove a
      single-CGU build is byte-identical to today's output.
- [ ] Add a determinism lock: two runs with different `jobs` values emit the
      same IR/diagnostics (normalize paths and timestamps).
- [ ] Mirror the design in the selfhost checklist after Phase 6 parity; do
      not fork the semantics between Rust and selfhost.

## Acceptance (for the design milestone)

- The design note is reviewed and the checklist is actionable.
- The shared-state inventory names every cache and its synchronization.
- A determinism test definition exists (same inputs, different worker
  counts, identical normalized output).
- No implementation is merged before selfhost Phase 6 parity.

## Non-goals

- Shipping m374 in the next cut.
- Replacing the 7C/I2 paths or changing their defaults (m373).
- Speculative CGUs without a measured product build.
