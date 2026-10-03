# Release gate -- v0.62.3

Correctness patch: Arc strong_count on the trusted sync path, the contracts
index access violation, and the match-pattern checker refinements
(m178 + m181). Release hold lifted; the pin moves to `stdlib-perf3`.

## Preconditions (before the tag)

- [x] Workspace version 0.62.3 (`Cargo.toml`) and
      `selfhost/src/codegen.xi` `SELFHOST_VERSION` = 0.62.3.
- [x] `STDLIB_VERSION` = `stdlib-perf3` (stdlib commit `2429ac3`, contains
      `e997201`/`8b23b79`); repo-local `stdlib/` checkout refreshed.
- [x] `release-notes/v0.62.3.md` + `.json` committed; convert + verify
      green (4 highlights, 0 breaking). No stdlib fragment at the pin.
- [x] README texts + links refreshed (version badge, download/folder names,
      Releases link, version-history row); VS Code extension bumped to
      0.12.2 and its README marked tested against v0.62.3.

## Gate A -- suites at the release state (fresh pin)

- [x] full e2e 2411 passed / 0 failed / 4 ignored (1834 s, 8 threads).
- [x] feature-reg 518/518; parser 107/107; full_diff 4/4.
- [x] checker 195/195 (stale-pin `catalog_corpus_is_clean` red closed);
      checker_locks 28/28; borrow_e001 2/2.
- [x] doctor_cli 4/4; run_script_cli 2/2.
- [x] stdlib_tests 40/40; stdlib_execution_tests 85/85 (+2 ignored);
      stdlib_api_freeze_tests 2/2 (additive pin, no snapshot change).
- [x] `cargo check --workspace --all-targets` clean (warnings only).
- [x] ascii_guard clean; tree committed at the tag.

## Gate P -- benchmark acceptance (benchmark lane)

- [ ] Benchmark lane re-runs t2-queue on this candidate (compiler half
      `185342f4` + `stdlib-perf3` annotations): t2 in the ms range and no
      regression on t1/t3/t4/t5/t8. This is the PERF-1 release acceptance;
      it is run by the benchmark lane, not locally.

## Gate D -- tag + publish

- [ ] Owner: dispatch/push, then `v0.62.3` on the release commit (tag push
      triggers `release.yml`: guard, nine-tool archives, VSIX 0.12.2,
      publish). Push only on the owner's ask.
