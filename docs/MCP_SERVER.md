# Phase 5d.1 — XIOM MCP Server (Model Context Protocol)

> **Status:** Planned. Depends on 5c (complete), 5c-S (planned), 5g (planned). MVP buildable NOW with 3 tools.  
> **Target:** Every MCP-compatible agent (Claude, Cursor, Continue, etc.) can natively query the XIOM compiler without parsing CLI output or writing shell scripts. The compiler becomes a structured tool-calling API.

---

## 1. Why MCP?

### The Problem Today
```
Agent: "Compile this file and tell me what's wrong"
  → Agent writes shell script
  → Runs `xiom --diagnostics=json file.xi`
  → Parses JSON from stdout (fragile, version-dependent)
  → Tries to map errors to source lines
  → Guesses at fixes
```

### The MCP Solution
```
Agent: calls tool `compile_and_analyze("file.xi")`
  → MCP server runs `xiom --ai --diagnostics=json file.xi`
  → Parses .xiom_ai.json internally
  → Returns structured result directly in agent's context window
  → Agent has error codes, line numbers, AI hints, and safety scores
  → Agent calls `explain_error("X0100")` for deep reference
```

**The compiler is already a structured data oracle.** MCP just puts a standardized API in front of it so agents don't have to parse CLI output.

### Why Not REST/gRPC?

| Protocol | Advantage | Disadvantage |
|----------|-----------|--------------|
| **MCP** | Natively spoken by Claude, Cursor, Continue, etc. Agent tools are MCP tools. | Anthropic-specific protocol. |
| REST | Universal, any HTTP client can call it. | Agents don't natively speak REST. They'd need wrapper code. |
| gRPC | High performance, typed contracts. | Heavy dependency. Overkill for local tool calling. |

**Verdict: MCP for the agent-facing API. REST as a secondary protocol** (for CI/CD, web dashboards). Same server, two transports.

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────┐
│  AGENT (Claude, Cursor, Aider, etc.)                     │
│  Calls tools via MCP JSON-RPC                            │
└────────────────────┬────────────────────────────────────┘
                     │ MCP protocol (stdio or HTTP)
┌────────────────────▼────────────────────────────────────┐
│  XIOM MCP SERVER  (xiom-mcp crate)                      │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ compile_     │  │ explain_     │  │ audit_safety_ │  │
│  │ and_analyze  │  │ error_code   │  │ sandbox       │  │
│  └──────┬───────┘  └──────┬───────┘  └───────┬───────┘  │
│         │                 │                  │           │
│  ┌──────▼─────────────────▼──────────────────▼───────┐   │
│  │              TOOL DISPATCHER                       │   │
│  │  Routes tool calls → xiom CLI or internal APIs    │   │
│  └──────┬──────────────────────────────────────┬──────┘   │
│         │                                      │          │
│  ┌──────▼──────┐                        ┌─────▼──────┐    │
│  │ xiom CLI   │                        │ libxiomc   │    │
│  │ (subprocess)│                        │ (Rust API)  │    │
│  │ --diagnostics│                       │ compile()   │    │
│  │ --sandbox   │                        │ audit()     │    │
│  │ --explain   │                        │ contracts() │    │
│  │ --dump-contracts│                    │             │    │
│  └─────────────┘                        └─────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Two integration modes:**
1. **Subprocess mode** (MVP): MCP server shells out to `xiom`. Works today, no code changes to the compiler.
2. **Library mode** (Phase 2): MCP server links `xiom` as a Rust library (`libxiomc`). Faster, no process overhead. Requires the 5c-R lib/bin split (already done ✅).

---

## 3. Tool Definitions

### 3.1 `compile_and_analyze`

```
Tool: compile_and_analyze
Description: Compile a XIOM source file and return structured diagnostics with AI-enhanced insights.
Parameters:
  - file (string, required): Path to the .xi source file
  - target (string, optional): "native" | "wasm" | "ir" (default: "native")
  - ai (boolean, optional): Enable AI-enhanced diagnostics (default: false)
  - strict (boolean, optional): Fail on contract violations (default: false)
Returns:
  - success: boolean
  - diagnostics: [{ code, severity, line, column, message, ai_hint?, suggestion? }]
  - contract_violations: [{ contract, line, expression }]
  - exit_code: number
```

### 3.2 `explain_error_code`

```
Tool: explain_error_code
Description: Get the full reference documentation for a XIOM error code.
Parameters:
  - code (string, required): Error code (e.g., "X0100", "X0010")
Returns:
  - code: string
  - title: string
  - description: string
  - example_bad: string
  - example_fix: string
  - notes: string
```

### 3.3 `audit_safety_sandbox`

```
Tool: audit_safety_sandbox
Description: Run the compiler safety audit on a file or directory.
Parameters:
  - path (string, required): Path to .xi file or directory
  - severity (string, optional): Filter by severity ("HIGH" | "MEDIUM" | "LOW" | "ALL")
Returns:
  - summary: { total_unsafe_blocks, high, medium, low, safety_score }
  - findings: [{ severity, category, line, description, suggestion }]
```

### 3.4 `get_contract_signature`

```
Tool: get_contract_signature
Description: Retrieve the requires/ensures/invariant contracts for a specific function or type.
Parameters:
  - function_name (string, optional): Fully qualified function name
  - type_name (string, optional): Fully qualified type name
Returns:
  - requires: [string]
  - ensures: [string]
  - invariants: [string]
```

### 3.5 `get_type_definition`

```
Tool: get_type_definition
Description: Retrieve the full field-level definition of a XIOM type.
Parameters:
  - type_name (string, required): Fully qualified type name
Returns:
  - name: string
  - kind: "struct" | "enum" | "interface"
  - fields: [{ name, type }]
  - variants: [{ name, fields: [{ name, type }] }]
  - derives: [string]
  - generics: [string]
```

### 3.6 `format_xiom_code`

```
Tool: format_xiom_code
Description: Format XIOM source code according to the canonical style.
Parameters:
  - source (string, required): XIOM source code
  - file (string, optional): File path for context
Returns:
  - formatted: string
  - diff: string (optional)
```

### 3.7 `check_xiom_syntax`

```
Tool: check_xiom_syntax
Description: Quick parse-only check (no type checking, no codegen). Fast feedback loop.
Parameters:
  - source (string, required): XIOM source code
Returns:
  - valid: boolean
  - errors: [{ line, column, message }]
```

### 3.8 `get_language_cheatsheet` ⭐ (Friction Point Solver)

```
Tool: get_language_cheatsheet
Description: Return a structural template of XIOM syntax rules for the given pattern.
  Agent calls this BEFORE writing its first line of XIOM code.
  Dramatically reduces failed compilation attempts by teaching the
  structural rules upfront instead of through compiler rejection.
Parameters:
  - pattern (string, required): What the agent wants to write
    Values: "function" | "method" | "struct" | "enum" | "contract"
    | "variable" | "control_flow" | "match" | "extern_c" | "full"
Returns:
  - syntax_example: string (valid XIOM code showing the pattern)
  - rules: [string] (non-negotiable rules for this pattern)
  - common_mistakes: [string] (what untrained models typically get wrong)
  - parser_errors_if_done_wrong: [string] (which error codes trigger)
```

**Example: agent calls `get_language_cheatsheet("function")`**

```json
{
  "pattern": "function",
  "syntax_example": "fn add(a: Int, b: Int) -> Int {\n  return a + b;\n}",
  "rules": [
    "All parameters MUST have type annotations",
    "Return type is required unless function returns nothing",
    "Tail expression (no return keyword, no semicolon) is the return value",
    "Every statement MUST end with ;",
    "Contracts go between signature and body: `requires: b != 0;`"
  ],
  "common_mistakes": [
    "Forgetting return type annotation on fn with return value",
    "Using `else if` instead of `elif`",
    "Putting ; after a tail expression",
    "Using `self.x` inside methods — fields are implicit, just write `x`"
  ],
  "parser_errors_if_done_wrong": "X0001 (unexpected token), X0010 (type mismatch)"
}
```

**Why this tool is critical:** The chat analysis nailed it — "the agent will essentially learn the syntax via the compiler's rejection messages." This tool short-circuits that. The agent learns the syntax BEFORE writing code, not through 5 failed compilations. It turns "trial-and-error learning" into "reference-card programming."

---

## 4. Friction Points — Honest Analysis

The chat identified two real friction points. Here's how each is solved:

### 4.1 Friction: "The Agent Doesn't Know XIOM Syntax"

**Problem:** An untrained model guesses syntax based on Rust/C++. Its first attempt WILL fail. It learns XIOM by being rejected by the compiler.

**How much this matters:** Medium. On a 5-compilation loop, the agent spends 80% of its time fixing syntax errors, 20% on logic. With `get_language_cheatsheet`, this inverts: 20% syntax, 80% logic.

**Solution — `get_language_cheatsheet` tool (3.8):**

Before writing code, the agent calls `get_language_cheatsheet("function")`. It gets:
- A valid example it can pattern-match
- Non-negotiable rules (type annotations required, `elif` not `else if`, no `self.x`)
- Common mistakes untrained models make
- Which error codes trigger if done wrong

This tool is NOT a full language reference — it's a **structural template generator**. It gives the agent the exact shape of what to write, which is what untrained models need most.

**Economic impact:** 5 failed compilations → 1-2 failed compilations. Token savings: ~60% per new function.

### 4.2 Friction: "Error Messages ARE the Training Data"

**Problem:** The agent learns XIOM syntax through compiler rejection. Every error message doubles as a training example. If the message says "expected ';', found '}'" without context, the agent learns nothing. If it says "every statement must end with ';' — the tail expression is the last expression in a block and must NOT have a semicolon" with a pointer to the exact location, the agent learns the rule.

**How much this matters:** CRITICAL. Bad error messages = slow agent learning = more iterations = more tokens = more cost. The compiler's error messages are NOT just diagnostics — they are the training curriculum.

**Solution — Two-tier error system:**

| Tier | What | When | Format |
|------|------|------|--------|
| **Tier 1: Imprecise fallback** | "unexpected token" (current state) | Parser catches bare syntax error | Brief, no context |
| **Tier 2: Training-quality** | "Function `add` is missing a return type annotation after the parameter list. All functions returning a value must declare `-> Type`." | MCP `compile_and_analyze` with AI enhancement | Full sentence, rule citation, line pointer |

**Action items to make error messages training-quality:**
1. Audit every error code (X0001–X7999, L001, P001, T001, C001, E001) for whether it teaches the rule or just states the fact
2. For each "states the fact" error, add a `docs/error_codes/{CODE}.md` entry with the rule, bad example, and fix
3. The MCP `compile_and_analyze` tool always includes the `--explain` text inline
4. The `get_language_cheatsheet` tool pre-loads the agent with the most common rules

**Current state:** 3 codes documented (X0010, X0011, X0100). Compiler emits at least 15+ distinct codes. Gap: ~12+ undocumented codes need `docs/error_codes/` pages. This is a ~1-day documentation task — no compiler changes needed.

### 4.3 Friction: "The Feedback Loop Must Be Fast"

**Problem:** If each compilation takes 2 seconds, a 5-iteration loop takes 10 seconds. For an agent generating 100 functions, that's 200+ seconds of dead time. The agent's context window is burning tokens while waiting.

**Solution — Tiered tool pipeline:**

```
check_xiom_syntax (parse only, ~50ms)
       │
       ▼ FAIL
  fix syntax errors (0-2 iterations)
       │ PASS
       ▼
compile_and_analyze (full compile, ~500ms)
       │
       ▼ FAIL
  fix type/contract errors (0-3 iterations)
       │ PASS
       ▼
audit_safety_sandbox (optional, ~100ms)
       │
       ▼
DONE — code is safe
```

**The key insight:** `check_xiom_syntax` catches 90% of syntax errors in <50ms without invoking the full compiler pipeline. The agent only pays for a full compile when the syntax is already clean. This is the same insight as Rust's `cargo check` vs `cargo build` — fast feedback on syntax, slow feedback on types.

### 4.4 Friction: "What If The Agent Gets Stuck In A Loop?"

**Problem:** The agent writes code, compiler rejects, agent rewrites, compiler rejects again with the SAME error. The agent doesn't realize it's making the same mistake. Token budget burns while the agent spins.

**How much this matters:** HIGH for unsupervised loops. An agent can burn thousands of tokens re-compiling the same broken code.

**Solution — Hash-based loop detection in the MCP server:**

```rust
// The MCP server tracks compilation attempts per session
let hash = sha256(&format!("{}:{}", error_code, function_body));
if attempt_count[hash] >= 3 {
    return ToolResult::intervention(format!(
        "This exact error ({error_code}) has occurred 3 times in this function. \
         The agent may be stuck. Consider:
         1. Run `explain_error_code(\"{error_code}\")` for the full rule
         2. Run `get_language_cheatsheet(pattern)` for a structural template
         3. Ask the user for guidance if the error persists"
    ));
}
```

This is a **circuit breaker**. It detects agent looping and provides an escalation path before token budget is exhausted.

### 4.5 Friction: "The Agent Doesn't Know The Stdlib"

**Problem:** Even with perfect syntax knowledge, the agent doesn't know which stdlib functions exist. It writes `array.len(arr)` when the function is `array.length(arr)` or vice versa.

**Solution — `get_type_definition` + `get_contract_signature`:** These tools let the agent query the stdlib directly. If the agent needs to work with `Vec[T]`, it calls `get_type_definition("Vec")` and sees all available methods. No guessing, no reading docs, no training data needed.

---

## 4.6 The Bounded Search Paradigm

The chat's most important insight: **programming becomes a bounded search problem.**

```
while !compiler.passes():
    error = compiler.compile(code)
    if error is None: break      // SAFE — the compiler guarantees it
    code = agent.fix(code, error) // targeted fix based on precise error
```

This works because:
1. The search space is finite: there's only one correct way to write a function signature
2. Each iteration reduces the error count: errors never increase (no cascading from our ErrorGuaranteed fix)
3. The compiler is deterministic: same code = same error every time (temperature 0 AI hints reinforce this)
4. The exit condition is indisputable: `compiler.passes() → code is safe`

**This is the paradigm shift.** The AI doesn't need to prove correctness — the compiler does. The AI just needs to be compliant enough to pass the gate.

The MVP uses **subprocess mode** and ships with 3 tools that work TODAY:

| Tool | Status | Backend |
|------|--------|---------|
| `explain_error_code` | ✅ Ready | `xiom --explain X0100` (reads docs/error_codes/ |
| `compile_and_analyze` | ✅ Ready | `xiom --diagnostics=json` (parse stdout JSON) |
| `get_contract_signature` | ✅ Ready | `xiom --dump-contracts` (parse stdout JSON) |

### MVP Server Implementation

```rust
// crates/xiom-mcp/src/main.rs (~300 lines)

use mcp_server::{Server, Tool, ToolResult};

fn main() {
    let mut server = Server::new("xiom-mcp", "0.1.0");

    server.register_tool(Tool {
        name: "explain_error_code".into(),
        description: "Get reference docs for a XIOM error code".into(),
        parameters: json!({ "code": { "type": "string", "required": true } }),
        handler: |params| {
            let code = params["code"].as_str().unwrap();
            let output = std::process::Command::new("xiom")
                .args(["--explain", code])
                .output()?;
            Ok(ToolResult::text(String::from_utf8(output.stdout)?))
        },
    });

    server.register_tool(Tool {
        name: "compile_and_analyze".into(),
        description: "Compile XIOM source with structured diagnostics".into(),
        parameters: json!({
            "file": { "type": "string", "required": true },
            "ai": { "type": "boolean", "default": false },
            "strict": { "type": "boolean", "default": false }
        }),
        handler: |params| {
            let file = params["file"].as_str().unwrap();
            let args = vec!["--diagnostics=json", file];
            let output = std::process::Command::new("xiom")
                .args(&args)
                .output()?;
            let diagnostics: serde_json::Value = serde_json::from_slice(&output.stdout)?;
            Ok(ToolResult::json(diagnostics))
        },
    });

    server.run_stdio();  // or server.run_http("127.0.0.1:9300");
}
```

### Phase 2: Library Mode

After MVP validation, switch to library mode:

```rust
// crates/xiom-mcp/src/main.rs (library mode)

use xiom::{CompileConfig, CompileResult, Target};

fn handle_compile(params: &Value) -> ToolResult {
    let config = CompileConfig {
        file: params["file"].as_str().unwrap().into(),
        target: Target::Native,
        diagnostics_json: true,
        ..Default::default()
    };
    let result = xiom::compile(&config)?;  // calls into lib.rs
    Ok(ToolResult::json(serde_json::to_value(&result)?))
}
```

---

## 5. Tool Lifecycle: What Grows With New Phases

| Phase | New Tools | Existing Tools Enhanced |
|-------|-----------|------------------------|
| **5c-S** (Sandbox) | `audit_safety_sandbox` | `compile_and_analyze` adds safety findings |
| **5g.1** (AI MVP) | — | `compile_and_analyze` returns AI hints |
| **5g.5** (Batch) | `compile_project` (whole directory) | `compile_and_analyze` supports batch |
| **5e** (Hot Reload) | `reload_module` | `compile_and_analyze` supports hot reload targets |

---

## 6. Transport Methods

| Method | Use Case | Protocol |
|--------|----------|----------|
| **stdio** | Local agents (Claude Desktop, Cursor, Continue) | JSON-RPC over stdin/stdout |
| **HTTP** | Remote agents, CI/CD, web dashboards | REST + Server-Sent Events |
| **Unix socket** | Local-first OS agents, minimal latency | JSON-RPC over socket |

Default: **stdio** for local agents, **HTTP** on port 9300 for remote.

---

## 7. Security Model

### Risk: Agent can compile arbitrary code

This is the same risk as running `xiom` from the command line. The MCP server doesn't execute the compiled binary — it only compiles. The agent must explicitly request execution.

### Risk: Agent can read arbitrary files

The `compile_and_analyze` tool takes a file path. An agent could pass `/etc/passwd` — the compiler would just produce a parse error. To harden:

```rust
// Restrict file access to the project root
fn validate_path(path: &str, project_root: &Path) -> Result<(), Error> {
    let resolved = project_root.join(path).canonicalize()?;
    if !resolved.starts_with(project_root) {
        return Err(Error::PathTraversal(path.into()));
    }
    Ok(())
}
```

### Risk: Agent exhausts resources

Large files or infinite loops in the compiler could consume CPU/memory. Mitigation: timeout + memory limit per tool call.

```
xiom --timeout=30 --memory-limit=512MB file.xi
```

---

## 8. Implementation Plan

### 8.1 — MCP Server MVP (3–5 Days)

- New crate: `crates/xiom-mcp/`
- 3 tools: `explain_error_code`, `compile_and_analyze`, `get_contract_signature`
- Subprocess mode (shells out to xiom)
- stdio + HTTP transports
- Error handling + timeouts

### 8.2 — Library Mode Migration (1–2 Days)

- Link `xiom` as a library (already lib/bin split ✅)
- Remove subprocess calls
- Performance: ~10x faster (no process spawn)

### 8.3 — Sandbox Tool (1 Day, after 5c-S)

- Add `audit_safety_sandbox` tool
- Parse `.xiom_sandbox.json`

### 8.4 — AI Integration (1 Day, after 5g.1)

- Add `ai: true` parameter to `compile_and_analyze`
- Parse `.xiom_ai.json` and include in response

---

## 9. Agent Configuration Example

```json
{
  "mcpServers": {
    "xiom": {
      "command": "xiom-mcp",
      "args": ["--transport", "stdio"],
      "env": {
        "XIOM_PROJECT_ROOT": "/home/user/myproject",
        "XIOM_AI_KEY": "${XIOM_AI_KEY}"
      }
    }
  }
}
```

With this config, any MCP-compatible agent immediately gains the ability to:
- Compile XIOM code
- Look up error codes
- Query contract signatures
- (After 5c-S) Audit safety
- (After 5g) Get AI hints

**Zero training data required.** The tools ARE the training data.

---

## 10. Honest Assessment

### What This Does Well
- **Unlocks every MCP agent immediately.** No training data, no fine-tuning, no prompt engineering.
- **Compounds with every new compiler phase.** 5c-S adds sandbox tools, 5g adds AI tools, 5f adds verification tools.
- **Subprocess MVP is fast to build.** 3 tools, ~300 lines, works today.
- **Library mode is performant.** After lib/bin split, direct Rust API calls, no process overhead.

### What This Doesn't Do
- **Doesn't replace AI training.** The agent still needs to understand XIOM syntax to WRITE code. MCP helps it UNDERSTAND errors and FIX code. Two different skills.
- **Doesn't replace the LSP.** The MCP server is for agent tool-calling. The LSP is for IDE features (autocomplete, hover, go-to-def). They share data but serve different consumers.
- **Depends on MCP adoption.** If MCP is replaced by a new protocol, the server needs a new transport. The tool logic stays the same.

### Recommendation
**Build the MVP now.** The 3 tools that work today (`explain_error_code`, `compile_and_analyze`, `get_contract_signature`) provide immediate value. Add sandbox and AI tools as those phases land. This is the highest-leverage Phase 5d item because it makes every MCP agent XIOM-aware overnight.
