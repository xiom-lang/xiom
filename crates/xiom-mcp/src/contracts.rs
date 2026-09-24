// XIOM MCP -- structured contract queries (docs/POST_RELEASE_PLAN.md section 2).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `get_contracts {symbol, verify?, file?}` returns ONE structured object per
// symbol (signature + requires/ensures/invariants with source lines) instead
// of prose; `search_symbols {query, file?}` ranks symbol hits across the
// bundled stdlib and the current project. Resolution uses the same catalogs
// the compiler uses (bundled stdlib at lib/); nothing shells out to a network
// service. `verify: true` folds in Z3 results for the symbol's own clauses.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;

use crate::knowledge;

/// One contract clause with its source line.
#[derive(Debug, Clone)]
pub struct ClauseRef {
    pub clause: String,
    pub line: u32,
}

impl ClauseRef {
    fn to_json(&self) -> Value {
        json!({ "clause": self.clause, "line": self.line })
    }
}

/// A resolved symbol and its contract surface.
#[derive(Debug, Clone)]
pub struct SymbolContract {
    pub symbol: String,
    pub module: String,
    pub signature: String,
    pub requires: Vec<ClauseRef>,
    pub ensures: Vec<ClauseRef>,
    pub invariants: Vec<ClauseRef>,
    pub qualified: bool,
    pub file: PathBuf,
    /// The declaration's own contracts, kept for `verify: true`.
    fn_clauses: Vec<(String, u32, bool)>, // (clause, line, is_ensures)
}

impl SymbolContract {
    fn to_json(&self) -> Value {
        json!({
            "symbol": self.symbol,
            "module": self.module,
            "signature": self.signature,
            "requires": self.requires.iter().map(ClauseRef::to_json).collect::<Vec<_>>(),
            "ensures": self.ensures.iter().map(ClauseRef::to_json).collect::<Vec<_>>(),
            "invariants": self.invariants.iter().map(ClauseRef::to_json).collect::<Vec<_>>(),
            "pre_refs": self.requires.iter().map(ClauseRef::to_json).collect::<Vec<_>>(),
            "post_refs": self.ensures.iter().map(ClauseRef::to_json).collect::<Vec<_>>(),
            "qualified": self.qualified,
        })
    }
}

fn parse_file(path: &Path) -> Result<Program, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let tokens = Lexer::new(&source).tokenize();
    Parser::new(tokens)
        .parse_program()
        .map_err(|e| format!("Parse error in {}: {}", path.display(), e.message))
}

/// Signature only -- the contract clauses live in their own arrays.
fn signature_only(f: &FnDecl) -> String {
    let mut sig = String::new();
    if f.is_pub {
        sig.push_str("pub ");
    }
    if f.is_async {
        sig.push_str("async ");
    }
    sig.push_str("fn ");
    if let Some(recv) = &f.receiver {
        sig.push_str(&recv.name);
        sig.push('.');
    }
    sig.push_str(&f.name.name);
    if !f.generics.is_empty() {
        let gens: Vec<String> = f.generics.iter().map(|g| g.name.name.clone()).collect();
        sig.push_str(&format!("[{}]", gens.join(", ")));
    }
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| {
            if p.name.name == "self" {
                if p.is_mut_self {
                    "&mut self".to_string()
                } else {
                    "self".to_string()
                }
            } else {
                format!("{}: {}", p.name.name, knowledge::type_to_str(&p.ty))
            }
        })
        .collect();
    sig.push_str(&format!("({})", params.join(", ")));
    if let Some(ret) = &f.return_type {
        sig.push_str(&format!(" -> {}", knowledge::type_to_str(ret)));
    }
    sig
}

fn fn_contract(module: &str, file: &Path, f: &FnDecl, qualified: bool) -> SymbolContract {
    let mut requires = Vec::new();
    let mut ensures = Vec::new();
    let mut fn_clauses = Vec::new();
    for c in &f.contracts {
        match c {
            ContractClause::Requires(e, span) => {
                let text = knowledge::expr_to_str(e);
                requires.push(ClauseRef { clause: text.clone(), line: span.line });
                fn_clauses.push((text, span.line, false));
            }
            ContractClause::Ensures(e, span) => {
                let text = knowledge::expr_to_str(e);
                ensures.push(ClauseRef { clause: text.clone(), line: span.line });
                fn_clauses.push((text, span.line, true));
            }
        }
    }
    let symbol = match &f.receiver {
        Some(recv) => format!("{module}.{}.{}", recv.name, f.name.name),
        None => format!("{module}.{}", f.name.name),
    };
    SymbolContract {
        symbol,
        module: module.to_string(),
        signature: signature_only(f),
        requires,
        ensures,
        invariants: Vec::new(),
        qualified,
        file: file.to_path_buf(),
        fn_clauses,
    }
}

/// Walk a parsed program (module wrappers included), calling `f` for every
/// declaration with the module's dotted name.
fn walk_decls<'a>(items: &'a [TopDecl], module: &str, f: &mut impl FnMut(&str, &'a TopDecl)) {
    for item in items {
        match item {
            TopDecl::Module(m) => {
                let leaf = m.name.name.as_str();
                let nested = if module.is_empty() {
                    leaf.to_string()
                } else {
                    format!("{module}.{leaf}")
                };
                walk_decls(&m.items, &nested, f);
            }
            other => f(module, other),
        }
    }
}

/// Resolve a symbol against the bundled stdlib.
///
/// Accepted spellings: `xiom.string.str_concat`, `string.str_concat`, and the
/// bare leaf `str_concat` when it resolves uniquely. Receiver methods match as
/// `Str.len` / `string.Str.len`.
pub fn find_stdlib(symbol: &str) -> Result<SymbolContract, String> {
    let modules = knowledge::stdlib_module_files()?;
    let raw = symbol.trim().trim_start_matches("xiom.");
    let raw_no_xiom = raw.to_string();
    let (module_hint, leaf) = match raw_no_xiom.rsplit_once('.') {
        Some((m, l)) => (Some(m.to_string()), l.to_string()),
        None => (None, raw_no_xiom.clone()),
    };

    let mut hits: Vec<SymbolContract> = Vec::new();
    for (module, path) in &modules {
        if let Some(hint) = &module_hint {
            // A module hint matches the module name exactly, or is the
            // RECEIVER of a method (`Str.len` -> receiver "Str").
            let module_matches = module == hint || module.ends_with(&format!(".{hint}"));
            let receiver_matches = {
                let recv = hint.rsplit('.').next().unwrap_or(hint);
                matches!(recv.chars().next(), Some(c) if c.is_ascii_uppercase())
            };
            if !module_matches && !receiver_matches {
                continue;
            }
        }
        let program = match parse_file(path) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let mut found: Vec<SymbolContract> = Vec::new();
        walk_decls(&program.items, "", &mut |_m, decl| {
            if let TopDecl::Fn(f) = decl {
                let recv_qual = match &f.receiver {
                    Some(recv) => format!("{}.{}", recv.name, f.name.name),
                    None => f.name.name.clone(),
                };
                let leaf_matches = f.name.name == leaf
                    || raw_no_xiom == recv_qual
                    || raw_no_xiom.ends_with(&format!(".{}", f.name.name)) && module_hint.is_none();
                let hint_matches = module_hint.is_none()
                    || *module == module_hint.clone().unwrap_or_default()
                    || raw_no_xiom.contains(&recv_qual);
                if leaf_matches && hint_matches {
                    let qualified = f.is_pub;
                    found.push(fn_contract(&format!("xiom.{module}"), path, f, qualified));
                }
            }
        });
        // Prefer public items; private helpers only surface on exact matches.
        if let Some(public) = found.iter().find(|c| c.qualified) {
            hits.push(public.clone());
        } else {
            hits.extend(found);
        }
    }

    if hits.len() == 1 {
        return Ok(hits.remove(0));
    }
    if hits.is_empty() {
        let query_leaf = leaf.to_ascii_lowercase();
        let mut close: Vec<String> = Vec::new();
        for (module, path) in modules.iter().filter(|(n, _)| {
            module_hint.as_deref().map_or(true, |h| n.contains(h))
        }) {
            let Ok(program) = parse_file(path) else { continue };
            walk_decls(&program.items, "", &mut |_m, decl| {
                if let TopDecl::Fn(f) = decl {
                    let name = f.name.name.to_ascii_lowercase();
                    if name.contains(&query_leaf) || query_leaf.contains(&name) {
                        if close.len() < 8 {
                            close.push(format!("xiom.{module}.{}", f.name.name));
                        }
                    }
                }
            });
        }
        let hint = if close.is_empty() {
            String::new()
        } else {
            format!(" Close matches: {}", close.join(", "))
        };
        return Err(format!(
            "symbol '{symbol}' not found in the bundled stdlib.{hint}"
        ));
    }
    let names: Vec<String> = hits.iter().map(|h| h.symbol.clone()).collect();
    Err(format!(
        "symbol '{symbol}' is ambiguous in the bundled stdlib: {}",
        names.join(", ")
    ))
}

/// Resolve a symbol in a project source file (contracts are read from the
/// file's AST, so clause lines are exact).
pub fn find_project(symbol: &str, file: &Path) -> Result<SymbolContract, String> {
    let program = parse_file(file)?;
    let leaf = symbol.trim().rsplit('.').next().unwrap_or(symbol).to_string();
    let mut hits: Vec<SymbolContract> = Vec::new();
    walk_decls(&program.items, "", &mut |_m, decl| match decl {
        TopDecl::Fn(f) => {
            let recv_qual = match &f.receiver {
                Some(recv) => format!("{}.{}", recv.name, f.name.name),
                None => f.name.name.clone(),
            };
            if f.name.name == leaf || recv_qual == leaf || recv_qual == symbol.trim() {
                hits.push(fn_contract("(project)", file, f, true));
            }
        }
        TopDecl::Type(t) => {
            if t.name.name == leaf {
                hits.push(SymbolContract {
                    symbol: t.name.name.clone(),
                    module: "(project)".to_string(),
                    signature: format!("type {}", t.name.name),
                    requires: Vec::new(),
                    ensures: Vec::new(),
                    invariants: t
                        .invariants
                        .iter()
                        .map(|e| ClauseRef {
                            clause: knowledge::expr_to_str(e),
                            line: 0,
                        })
                        .collect(),
                    qualified: true,
                    file: file.to_path_buf(),
                    fn_clauses: Vec::new(),
                });
            }
        }
        _ => {}
    });
    match hits.len() {
        1 => Ok(hits.remove(0)),
        0 => Err(format!("symbol '{symbol}' not found in {}", file.display())),
        _ => Err(format!("symbol '{symbol}' is ambiguous in {}", file.display())),
    }
}

/// `verify: true` -- fold Z3 results for the symbol's requires/ensures
/// clauses. Type invariants are reported as unknown (not encoded).
pub fn verify_contract(contract: &SymbolContract) -> Value {
    if contract.fn_clauses.is_empty() {
        return json!({
            "status": "unknown",
            "reason": "no requires/ensures clauses to verify (or type invariants, not encoded yet)",
        });
    }
    let Ok(program) = parse_file(&contract.file) else {
        return json!({ "status": "unknown", "reason": "cannot re-parse the symbol's file" });
    };
    // Rebuild a program containing ONLY this declaration so the SMT queries
    // map 1:1 (in clause order) to this symbol.
    let leaf = contract
        .symbol
        .rsplit('.')
        .next()
        .unwrap_or(&contract.symbol)
        .to_string();
    let recv_want = contract
        .symbol
        .rsplit('.')
        .nth(1)
        .map(|s| s.to_string());
    let mut target: Option<FnDecl> = None;
    walk_decls(&program.items, "", &mut |_m, decl| {
        if let TopDecl::Fn(f) = decl {
            let receiver_key = f.receiver.as_ref().map(|r| r.name.clone());
            if f.name.name == leaf && (recv_want.is_none() || receiver_key == recv_want) {
                target = Some(f.clone());
            }
        }
    });
    let Some(fd) = target else {
        return json!({ "status": "unknown", "reason": "declaration not found on re-parse" });
    };
    let single = Program {
        items: vec![TopDecl::Fn(fd.clone())],
        source_files: vec![contract.file.to_string_lossy().to_string()],
        root_dir: None,
        span: Span { line: 1, col: 1, byte_start: 0, byte_end: 0 },
    };
    if xiom_verify::Z3Runner::find_z3().is_none() {
        return json!({ "status": "unknown", "reason": "z3 not found on PATH" });
    }
    let mut generator = xiom_verify::SMTGenerator::new();
    let smt = generator.generate(&single);
    let runner = xiom_verify::Z3Runner::new();
    let results = runner.verify(&smt);
    if results.len() != contract.fn_clauses.len() {
        return json!({
            "status": "unknown",
            "reason": format!(
                "z3 returned {} result(s) for {} clause(s); mapping is not 1:1",
                results.len(),
                contract.fn_clauses.len()
            ),
        });
    }
    let clauses: Vec<Value> = contract
        .fn_clauses
        .iter()
        .zip(results.iter())
        .map(|((text, line, is_ensures), result)| {
            let verify = match result {
                xiom_verify::VerifyResult::Proven => json!({ "status": "proved" }),
                xiom_verify::VerifyResult::Violated { message, counterexample, .. } => {
                    let values = counterexample
                        .as_ref()
                        .map(|c| serde_json::to_string(&c.values).unwrap_or_default())
                        .unwrap_or_default();
                    json!({
                        "status": "counterexample",
                        "message": message,
                        "counterexample": values,
                    })
                }
                xiom_verify::VerifyResult::Inconclusive { reason } => {
                    json!({ "status": "unknown", "reason": reason })
                }
                xiom_verify::VerifyResult::Error { message } => {
                    json!({ "status": "unknown", "reason": message })
                }
            };
            json!({
                "clause": text,
                "line": line,
                "kind": if *is_ensures { "ensures" } else { "requires" },
                "verify": verify,
            })
        })
        .collect();
    json!({ "status": "checked", "clauses": clauses })
}

/// Full `get_contracts` response: contract object (+ optional verify fold).
pub fn get_contracts(
    symbol: &str,
    verify: bool,
    file: Option<&Path>,
) -> Result<Value, String> {
    let contract = match file {
        Some(path) => find_project(symbol, path).or_else(|_| find_stdlib(symbol))?,
        None => find_stdlib(symbol)?,
    };
    let mut value = contract.to_json();
    if verify {
        value["verify"] = verify_contract(&contract);
    }
    Ok(value)
}

/// `search_symbols` -- ranked hits across the bundled stdlib and (when a file
/// is given) the project. Ranking: exact qualified > exact leaf > prefix >
/// substring; ties keep catalog order.
pub fn search_symbols(query: &str, file: Option<&Path>) -> Result<Value, String> {
    let q = query.trim();
    if q.is_empty() {
        return Err("Missing required parameter: query".to_string());
    }
    let q_lower = q.to_ascii_lowercase();
    let mut hits: Vec<(u8, String, String, String)> = Vec::new(); // (rank, symbol, module, signature)

    let modules = knowledge::stdlib_module_files()?;
    for (module, path) in &modules {
        let Ok(program) = parse_file(path) else { continue };
        walk_decls(&program.items, "", &mut |_m, decl| {
            match decl {
                TopDecl::Fn(f) if f.is_pub => {
                    let symbol = match &f.receiver {
                        Some(recv) => format!("xiom.{module}.{}.{}", recv.name, f.name.name),
                        None => format!("xiom.{module}.{}", f.name.name),
                    };
                    if let Some(rank) = rank_hit(&q_lower, &symbol, &f.name.name) {
                        hits.push((rank, symbol, format!("xiom.{module}"), signature_only(f)));
                    }
                }
                TopDecl::Type(t) if t.is_pub => {
                    let symbol = format!("xiom.{module}.{}", t.name.name);
                    if let Some(rank) = rank_hit(&q_lower, &symbol, &t.name.name) {
                        hits.push((rank, symbol, format!("xiom.{module}"), format!("type {}", t.name.name)));
                    }
                }
                TopDecl::Enum(e) if e.is_pub => {
                    let symbol = format!("xiom.{module}.{}", e.name.name);
                    if let Some(rank) = rank_hit(&q_lower, &symbol, &e.name.name) {
                        hits.push((rank, symbol, format!("xiom.{module}"), format!("enum {}", e.name.name)));
                    }
                }
                _ => {}
            }
        });
        if hits.len() >= 50 {
            break;
        }
    }

    if let Some(path) = file {
        if let Ok(program) = parse_file(path) {
            walk_decls(&program.items, "", &mut |_m, decl| match decl {
                TopDecl::Fn(f) => {
                    let symbol = match &f.receiver {
                        Some(recv) => format!("{}.{}", recv.name, f.name.name),
                        None => f.name.name.clone(),
                    };
                    if let Some(rank) = rank_hit(&q_lower, &symbol, &f.name.name) {
                        hits.push((rank, symbol, "(project)".to_string(), signature_only(f)));
                    }
                }
                TopDecl::Type(t) => {
                    if let Some(rank) = rank_hit(&q_lower, &t.name.name, &t.name.name) {
                        hits.push((rank, t.name.name.clone(), "(project)".to_string(), format!("type {}", t.name.name)));
                    }
                }
                _ => {}
            });
        }
    }

    hits.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    hits.dedup_by(|a, b| a.1 == b.1);
    let items: Vec<Value> = hits
        .iter()
        .take(25)
        .map(|(_, symbol, module, signature)| {
            json!({ "symbol": symbol, "module": module, "signature": signature })
        })
        .collect();
    if items.is_empty() {
        return Err(format!("no symbol matches '{query}'"));
    }
    Ok(json!({ "query": query, "results": items }))
}

/// 0 exact qualified, 1 exact leaf, 2 prefix, 3 substring; None = no match.
fn rank_hit(query_lower: &str, symbol: &str, leaf: &str) -> Option<u8> {
    let symbol_lower = symbol.to_ascii_lowercase();
    let leaf_lower = leaf.to_ascii_lowercase();
    if symbol_lower == query_lower || symbol_lower.trim_start_matches("xiom.") == query_lower {
        Some(0)
    } else if leaf_lower == query_lower {
        Some(1)
    } else if leaf_lower.starts_with(query_lower) {
        Some(2)
    } else if symbol_lower.contains(query_lower) {
        Some(3)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_spec_example_symbol() {
        let c = find_stdlib("xiom.string.str_concat").expect("stdlib symbol");
        assert_eq!(c.symbol, "xiom.string.str_concat");
        assert_eq!(c.module, "xiom.string");
        assert_eq!(c.signature, "pub fn str_concat(a: Str, b: Str) -> Str");
        assert!(c.requires.is_empty());
        assert!(
            c.ensures.iter().any(|e| e.clause.contains("result.len()")),
            "ensures: {:?}",
            c.ensures
        );
        assert!(c.ensures.iter().all(|e| e.line > 0), "clause lines must be real");
    }

    #[test]
    fn bare_leaf_resolves_when_unique_and_reports_close_matches() {
        let c = find_stdlib("str_concat").expect("unique leaf");
        assert_eq!(c.symbol, "xiom.string.str_concat");
        let err = find_stdlib("str_concat_nope").unwrap_err();
        assert!(err.contains("not found"), "{err}");
        assert!(err.contains("str_concat"), "close matches expected: {err}");
    }

    #[test]
    fn search_ranks_and_filters() {
        let value = search_symbols("str_concat", None).expect("search");
        let results = value["results"].as_array().expect("results");
        assert!(!results.is_empty());
        assert_eq!(results[0]["symbol"], "xiom.string.str_concat");
        assert!(search_symbols("", None).is_err());
        assert!(search_symbols("zzz_no_such_symbol_zzz", None).is_err());
    }

    #[test]
    fn get_contracts_json_shape() {
        let value = get_contracts("xiom.string.str_concat", false, None).expect("contract");
        for key in [
            "symbol", "module", "signature", "requires", "ensures", "invariants",
            "pre_refs", "post_refs", "qualified",
        ] {
            assert!(value.get(key).is_some(), "missing {key}: {value}");
        }
        assert_eq!(value["qualified"], Value::Bool(true));
    }

    #[test]
    fn verify_folds_a_z3_status_for_the_symbol_only() {
        let c = find_stdlib("xiom.string.str_concat").expect("stdlib symbol");
        let v = verify_contract(&c);
        // Either z3 ran (checked) or it is missing (unknown); both shapes are
        // valid, but a "checked" response must align one entry per clause.
        if v["status"] == Value::String("checked".to_string()) {
            let clauses = v["clauses"].as_array().expect("clauses");
            assert_eq!(clauses.len(), c.fn_clauses.len());
            for clause in clauses {
                assert!(clause["verify"]["status"].is_string());
            }
        } else {
            assert_eq!(v["status"], Value::String("unknown".to_string()));
            assert!(v["reason"].is_string());
        }
    }
}
