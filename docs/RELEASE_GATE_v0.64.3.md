# Release gate -- v0.64.3

Correctness + performance patch. Scope: m243-m246, m248, m249 (+ bridge),
m252; m247/m250/m251 were reverted mid-release (four stdlib corpus smokes
regressed vs the v0.64.2 archive; bisected). Pin moves to `stdlib-v0.64.3`
(d052a3c5).

## Preconditions (before the tag)

- [x] Workspace version 0.64.3 (`Cargo.toml`) and `selfhost/src/codegen.xi`
      `SELFHOST_VERSION` = 0.64.3.
- [x] `STDLIB_VERSION` = `stdlib-v0.64.3`; repo-local `stdlib/` refreshed to
      d052a3c5 (mirror of the tag); stdlib suites green on it.
- [ ] `release-notes/v0.64.3.json` committed -- convert+verify BLOCKED on
      the stdlib fragment schema violation (see Blockers).
- [x] VSIX: toolchain-only release -- `editors/vscode/package.json` stays
      0.12.2 (marketplace publish skips cleanly); README pin text -> v0.64.3.

## Gate A -- suites at the release state

- [x] full e2e 2469 passed / 0 failed / 4 ignored (847 s) -- WITHOUT
      XIOM_STDLIB (vendored v0.64.3 tree).
- [x] feature-reg 549/549; parser 108/108.
- [x] checker 199/199 (incl. un-ignored `catalog_corpus_is_clean`).
- [x] checker_locks 30/30; borrow_e001 2/2; doctor_cli 4/4;
      run_script_cli 7/7; scripting_tests 34/34; diff_tests 15/15;
      cli_args 5/5.
- [x] xiom-verify 9/9 + 37/37; xiom-graph 34/34.
- [x] stdlib_tests 40/40; stdlib_execution_tests 85/85 (+2 ignored);
      stdlib_api_freeze_tests 2/2 (four frozen entries repaired for the
      vendored sync: lz4 `_checked` rename, reflect bracket syntax).
- [x] `cargo check --workspace --all-targets` clean (warnings only).
- [x] nine-tool release build green; all tools report v0.64.3.
- [!] full_diff_tests `diff_check` red -- selfhost checker-parity drift,
      PRE-EXISTING (the v0.64.2 archive emits the same second diagnostic;
      the tag's manifest is 1 line). Not part of the compiler release list;
      the selfhost lane's branch already carries the 2-line manifest fix.

## Blockers / notes

- `stdlib-v0.64.3` `release-notes/v0.64.3.md` uses `->` in highlight text,
  which `xiom-release-notes` rejects; the release workflow's
  `verify --tag ... --stdlib ../stdlib` fails until the fragment is
  rephrased at a pinnable ref (relay item filed to stdlib, high).
- Reverted in this batch: m247 (regex smoke AV), m250 (rand payload +
  cell/narrow + btree drift), m251 (same m250 fallout). Their findings
  reopen as known-open; the four smoke repros are the acceptance locks
  (`E:\xiom-lang\stdlib\tests\smoke\smoke_{stress_regex_find,rand_weighted,
  cell_narrow,collections_btree_map}.xi`). All four were open on v0.64.2.

## Gate D -- tag + publish

- [ ] v0.64.3 pushed on the release commit; release run watched.
- [ ] Release assets + the notes JSON present at the tag.
