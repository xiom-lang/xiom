<!--
Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
SPDX-License-Identifier: MIT OR Apache-2.0
-->
# XIOM Error Codes -- Reference

Each XIOM diagnostic carries a code. Codes are stable: once assigned they are
never deleted or reassigned. The same pages are published on the documentation
site: https://docs.xiom-lang.org/latest/error-codes/

## Emitted codes

| Code | Stage | Meaning | Doc |
|------|-------|---------|-----|
| L001 | Lexer | Source cannot be tokenized (bad literal, bad escape) | [L001](L001.md) |
| P001 | Parser | Grammar error, messages shaped `expected X, found Y` | [P001](P001.md) |
| T001 | Type checker | Type and semantic errors (umbrella code) | [T001](T001.md) |
| E001 | Borrow checker | Ownership / borrow rule violation | [E001](E001.md) |
| C001 | Codegen | Internal code generation failure | [C001](C001.md) |
| W000 | Checker | Non-fatal checker warning | [W000](W000.md) |
| W001 | Catalog | Module name collision | [W001](W001.md) |

Codes appear in CLI output as `error[CODE]: line:col: message` and in
`--diagnostics=json` output as the `code` field of each diagnostic.

## Reserved codes (not emitted)

The X family below was reserved in an early design and is **not emitted by the
current compiler**. The numbers stay reserved so they can never be reused.
The pages are kept for continuity: [X0010](X0010.md) type mismatch,
[X0011](X0011.md) undefined variable, [X0100](X0100.md) contract requires
violation. Those situations are reported as `T001` today, and a runtime
contract trap prints `Contract violation: requires: ...` without a code.

## Writing a new error code

1. Create `docs/error_codes/{CODE}.md` in the website repository
2. Include: short description, an example, fix, notes
3. Add the row and link to the emitted table above
4. Codes are never deleted -- mark deprecated codes as such

## Using `--explain`

```bash
xiom --explain T001
# Prints docs/error_codes/T001.md when that directory is present in the
# current working directory, then a pointer to this index.
```

`--explain` first resolves the file relative to the current directory, so a
checkout can add or override pages. If no file is found, the same reference
text embedded in the compiler binary is printed, so installed toolchains and
release archives work without a `docs/error_codes/` directory. The embedded
pages are kept in this directory and updated in lockstep with the codes.
