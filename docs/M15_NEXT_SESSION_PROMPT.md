Continue XIOM M15 self-hosting hardening from SESSION.md. Branch: feat/architect.

CURRENT STATE:
- v0.51.0 "Production Hardening" -- 1049/1049 ALL TESTS PASS
- Self-hosting readiness: 8/10
- All P0, P1, M1-M12 complete
- M14.3-M14.7 complete
- M14.1 file splits: 3/14 done (fmt, pkg, dbg) -- remaining 11 are cosmetic, DEFERRED

M15 TARGET: Fix 3 codegen bugs to reach 10/10 self-hosting.

B-001: Result struct truncation (2d)
- Root cause: %struct.Result = {i64,i64,i64} only 8 bytes per field, but struct payloads are larger
- Fix approach: concrete monomorphized types (Result__JsonValue__SerializeError) with correct field sizes
- All 4 core pieces are correct (see SESSION.md for details):
  1. ensure_concrete_result/ensure_concrete_option methods (in lib.rs)
  2. type_from_ast generates concrete names for Type::Named("Result", args) (in lib.rs)
  3. Emission loop split: base types first, concrete types second (in lib.rs)
  4. Expr::Ok/Err/Some use fctx.current_return_type (in expr.rs)
- ONE ISSUE REMAINS: type definitions created during compile_fn aren't emitted at LLVM top level
- RECOMMENDED FIX: after register_type_layout completes but BEFORE the main emission loop,
  iterate all structs in type_meta and create all Result__A__B + Option__A combinations.
  This way all concrete types exist before the emission loop runs. Remove preamble + on-demand creation.

B-002: &mut self methods crash (1d)
- STATUS_ACCESS_VIOLATION on fn Counter.inc(&mut self)
- Investigate Expr::MutRef codegen in expr.rs

B-003: Option<Str> from method returns crash (1d)
- STATUS_ILLEGAL_INSTRUCTION on fn Path.file_name(self) -> Option<Str>
- Investigate method return codegen in decl.rs (compile_fn)

VERIFICATION after each fix:
1. parse_json("[1,2,3]") returns Ok (not empty Err)
2. PathBuf.push("foo") actually modifies the buffer
3. Path.file_name() returns correct Some("file.txt")
4. Restore original smoke tests (no workarounds)
5. Full 1049 test suite passes

TARGET: v0.52.0 "Self-Host Ready" at 10/10, ~1055 tests.

KEY FILES: SESSION.md, docs/ROADMAP.md, docs/M15_SELFHOST_AUDIT.md, crates/xiom-codegen/src/lib.rs, crates/xiom-codegen/src/decl.rs, crates/xiom-codegen/src/expr.rs, stdlib/xiom/serialize.xi, stdlib/xiom/path.xi

RULES:
- Production-grade only. No workarounds.
- 1049 baseline must not regress.
- Atomic commits after each logical step.
- Use .\test_summary.ps1 to verify.
- Update SESSION.md after each milestone.
