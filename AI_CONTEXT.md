# XIOM AI Context

Canonical context for AI assistants, agents, and MCP clients. This file ships
inside every release archive as `lib/AI_CONTEXT.md` and is served by the XIOM
MCP server (`xiom_workflow_guide {topic: "context"}`), so it always describes
the toolchain it ships with.

- Compiler version: **0.61.1** (release tags are `v<workspace version>`).
- Stdlib: bundled in `lib/` at the stdlib ref pinned by `STDLIB_VERSION`
  (`package xiom_std { name: "xiom-std" }`); upgrading the toolchain upgrades
  the bundled stdlib in lockstep.
- Docs: `docs/` (language spec, `COMPILER_BUGS.md`, `ROADMAP.md`,
  `POST_RELEASE_PLAN.md`), editors: `editors/README.md`.

## Toolchain layout

```
bin/    xiom xiom-pkg xiom-fmt xiom-doc xiom-lsp xiom-dbg xiom-mcp
        xiom-verify xiom-ffigen z3 (bundled SMT solver; libz3.dylib on macOS)
        xiom-wasm.wasm
lib/    xiom/ (stdlib modules) runtime/ (C runtime) package.xi AI_CONTEXT.md
```

`xiom --version` (or `-V`) prints the true workspace version; every tool
self-reports the same version. Installers: `install.ps1` (Windows),
`install.sh` (macOS/Linux). `xiom doctor` diagnoses the install.

## CLI map

Compile:
```
xiom -o app.exe main.xi          # native binary (multi-file: pass several)
xiom --check main.xi             # type-check only
xiom --emit-ir main.xi           # print LLVM IR
xiom -g --release main.xi        # symbols + optimized
xiom --target wasm|arm|riscv ... # cross-compile
```
Flags: `--opt-level N` (`-O0..-O3`), `--no-contracts`, `--diagnostics=json`,
`--dump-contracts`, `--explain CODE`, `--sandbox[=strict]`,
`--sandbox-report=json`, `--timeout SECS`, `-l LIB`, `-L PATH`,
`--c-source FILE.c`.

Scripts (`xiom run` — implicit `fn main()`, top-level statements allowed):
```
xiom run script.xi               # execute
xiom run -e "io.println(\"hi\")" # inline expression
echo "..." | xiom run -          # stdin
xiom run --watch script.xi       # re-run on change
xiom run --opt-level 0 script.xi # or -O0: accepted before or after `run`
xiom run --no-cache script.xi
```
The script cache is content-hash keyed (source + compiler build identity +
effective opt level) in `$HOME/.xiom/jit`, with a `$TMPDIR/xiom_jit` fallback
when HOME is unset or unwritable.

Other: `xiom doc`, `xiom graph[=mermaid]`, `xiom --standalone script.xi -o out`,
`xiom new NAME`, `xiom init`, `xiom install` (deprecated → `xiom pkg install`),
`xiom update` (RETIRED — it used an unverified channel; a verified updater is
planned, see `docs/POST_RELEASE_PLAN.md`).

## Packages and the registry

Manifest `package.xi` at the project root:
```
package mylib {
  name: "mylib";
  version: "0.1.0";
  description: "What it does";
  authors: ["you"];
  deps: { "xiom-std": "0.61.0"; }
}
```
Registry client (`xiom pkg`, the verified client):
```
xiom pkg search --query matrix --category core --json
xiom pkg info xiom-std --json
xiom pkg install name@version      # verified install
xiom pkg publish                   # signs with $HOME/.xiom/keys/default.key
xiom pkg keygen                    # ephemeral ed25519 signing key
xiom pkg sign <file> [--key PATH]
xiom pkg lock
```
- Production registry: `https://registry.xiom-lang.org`; staging:
  `https://staging.registry.xiom-lang.org` (override with `XIOM_REGISTRY`).
- CI publishing uses **OIDC trusted publishing** (no long-lived token): the
  workflow mints a GitHub OIDC JWT with audience `xiom-registry` and passes it
  as `XIOM_REGISTRY_TOKEN`; trusted first-party tokens still require a valid
  ed25519 signature (`xiom pkg keygen` + `xiom pkg publish` auto-signs).

## Language essentials

- Contracts: `requires:` / `ensures:` clauses and type invariants; runtime
  checks can be stripped with `--no-contracts`; `xiom --dump-contracts` emits
  a JSON contract index; Z3 verification via `xiom-verify` / the MCP
  `verify_contracts` tool.
- Errors: `Option[T]` / `Result[T, E]` with `?` propagation, `match`
  destructuring, `unwrap` / `unwrap_or` / `expect`. Tuple payloads through `?`
  are supported.
- Generics: functions/types with bounds; interfaces via
  `impl Trait[Type] { ... }` (methods are STATIC there — `self` is the impl
  type). Calls on interface-typed VALUES with aggregate arguments are NOT
  implemented and are rejected loudly; function-value identity (`f == g` for
  named functions) is unspecified — do not rely on it.
- Modules: `module a.b.c`, `use a.b;`, `pub` exports; the stdlib resolves as
  `xiom.<module>` from the bundled `lib/`.
- Unsafe/FFI: `unsafe { ... }`, `extern "C"` with confinement gates
  (T002/T003/T005/T006/T007); `xiom --sandbox` scores unsafe blocks.
- Diagnostics: codes `T` type, `C` codegen, `P` parse, `X` contract,
  `E` ownership/borrow, `L` lexer; `xiom --explain <code>` prints the
  reference; `--diagnostics=json` for machine consumption.

## MCP server (`xiom-mcp`)

Tools: `compile_and_analyze`, `compile_and_fix`, `check_xiom_syntax`,
`format_xiom_code`, `explain_error_code`, `get_contract_signature`,
`verify_contracts`, `audit_safety_sandbox`, `ai_diagnose`,
`hot_reload_watch`, `xiom_cheatsheet`, `xiom_stdlib_reference`,
`xiom_language_guide`, `xiom_workflow_guide`, `search_packages`,
`package_info`.

- `xiom_stdlib_reference {module?}` — live-parses the bundled stdlib: module
  list, then public signatures **with contracts**.
- `get_contract_signature {file, function_name?}` — requires/ensures/
  invariants for project code.
- `verify_contracts {file, check?}` — Z3 proof results + counterexamples
  (needs `z3`; it is bundled in `bin/`).
- `search_packages` / `package_info` — registry search/info (read-only, no
  token); staging-verified.
- Guide topics: `xiom_language_guide {topic}` and
  `xiom_workflow_guide {topic}`; use topic **`context`** for this file.

## Known limitations (0.61.1)

- Interface value-receiver ABI not implemented (loud compile error).
- Named function values are not first-class; identity is unspecified.
- Generic `T.to_str()` conversion dispatch prints invalid values — use a
  concrete type or `fmt.format1`.
- `xiom update` retired; toolchain updates are manual until the verified
  updater lands (`docs/POST_RELEASE_PLAN.md`).
- Some `xiom.sort`/`search`/`bits`/`geom` APIs carry contracts; coverage is
  partial and documented in the stdlib repo.
