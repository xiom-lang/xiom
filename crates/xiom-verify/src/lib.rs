// XIOM -- Contract Verifier (Phase 5f, readiness Stage 1 R16c)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// PRODUCTION REWRITE (2026-08-25). Fixes the audited defect set:
//   - AUDIT #3 (fail-closed-by-false): unsupported expressions no longer
//     emit the literal `false` (a guaranteed spurious VIOLATED -- and a
//     malformed s-expression, since the old code injected an SML-LIB
//     COMMENT into the term). Unsupported obligations are SKIPPED with a
//     recorded reason and reported as UNKNOWN (Inconclusive). A verifier
//     whose default answer was the maximally-alarming one trained users to
//     ignore it; UNKNOWN is now honest.
//   - AUDIT #8 (invalid SMT sorts): ONE numeric story -- every integer
//     width maps to unbounded SMT `Int` and floats map to `Real`, matching
//     the operator vocabulary actually emitted (+ - * div mod <= ...). The
//     old mapping sent Int32/UInt64 to BitVec sorts and Floats to
//     FloatingPoint while emitting Int arithmetic everywhere -- any such
//     operand made the query ill-sorted. Operators are now SORT-AWARE
//     (div vs /), mismatches route to UNKNOWN instead of garbage.
//   - AUDIT #8 (undeclared field functions): XIOM structs with fully
//     mappable fields are emitted as real SMT DATATYPES; field access uses
//     the generated selector (<Type>-<field>) so contracts about fields
//     are well-typed and provable instead of failing on undeclared symbols.
//   - Type invariants: real check-sat VCs (were TODO comments).
//   - Contract-composition axioms: forall binders were malformed (the
//     RETURN SORT string was bound as a variable). Binders are now exactly
//     the parameters plus |result| when needed.
//   - Vacuous-proof hole: if/else branches were encoded as CONJOINED
//     assertions -- contradictory contexts made (not ensures) UNSAT and
//     proved clamp/max-style contracts vacuously. Branches are now guarded
//     implications ((=> cond A)); returns under guards too.
//   - AUDIT #14 (pipe deadlock): z3 reads from a UNIQUE TEMP FILE instead
//     of a fully-written stdin pipe (the classic write-all-then-wait
//     deadlock on large inputs). -T is passed in SECONDS correctly (the old
//     code fed milliseconds into the seconds flag), and the parent polls
//     with try_wait and kills at 2x the budget.
//   - SSA bodies track the LATEST binding per local (reads no longer see
//     stale pre-assignment values).
//
// Honesty contract: Proven/Violated mean what they say; anything the
// encoder cannot faithfully express surfaces as UNKNOWN with a reason --
// never as silent `false`, never as fabricated proof.

use xiom_ast::*;
use std::process::Command;
use std::collections::{HashMap, HashSet};

// =========================================================================
// Error Codes (X7000 series -- verification diagnostics)
// =========================================================================

pub const X7001_ENSURES_VIOLATION: &str = "X7001";
pub const X7002_REQUIRES_UNPROVABLE: &str = "X7002";
pub const X7003_OVERFLOW: &str = "X7003";
pub const X7004_DIV_BY_ZERO: &str = "X7004";
pub const X7005_ARRAY_BOUNDS: &str = "X7005";
pub const X7006_INVARIANT: &str = "X7006";
pub const X7007_UNKNOWN: &str = "X7007";
pub const X7008_TYPE_ERROR: &str = "X7008";

// =========================================================================
// Verification Result
// =========================================================================

#[derive(Debug, Clone)]
pub struct Counterexample {
    pub values: HashMap<String, String>,  // variable -> value
    pub raw_model: String,
}

#[derive(Debug)]
pub enum VerifyResult {
    Proven,
    Violated { code: &'static str, message: String, span: Option<String>, counterexample: Option<Counterexample> },
    Inconclusive { reason: String },
    Error { message: String },
}

/// An obligation the encoder could not faithfully express.
#[derive(Debug, Clone)]
pub struct SkippedObligation {
    pub code: &'static str,
    /// Function/type the obligation belonged to.
    pub owner: String,
    pub reason: String,
}

/// Report of everything generate() had to skip (surfaced as UNKNOWN).
#[derive(Debug, Clone, Default)]
pub struct GenReport {
    pub skipped: Vec<SkippedObligation>,
}

impl GenReport {
    fn skip(&mut self, code: &'static str, owner: &str, reason: String) {
        self.skipped.push(SkippedObligation {
            code,
            owner: owner.to_string(),
            reason,
        });
    }
}

// =========================================================================
// SMT Sort Mapping -- ONE numeric story (audit #8 fix)
// =========================================================================

/// All XIOM integer widths map to unbounded SMT `Int` (matches the
/// checker's coercion semantics and keeps one operator vocabulary).
/// Floats map to `Real` (decimal literals; exact FP bit semantics are NOT
/// claimed -- documented honestly rather than ill-sorted).
fn primitive_sort(ty: &Type) -> Option<String> {
    match ty {
        Type::Named(name, _) => match name.name.as_str() {
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "Int128"
            | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64" => Some("Int".to_string()),
            // Char verifies as its codepoint.
            "Char" => Some("Int".to_string()),
            "Float32" | "Float64" => Some("Real".to_string()),
            "Bool" => Some("Bool".to_string()),
            "Str" => Some("String".to_string()),
            _ => None,
        },
        Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => {
            primitive_sort(inner).map(|_| format!("|xiom_ptr_{}|", sort_key(inner)))
        }
        Type::Fn(..) => Some("|xiom_fn|".to_string()),
        _ => None,
    }
}

fn sort_key(ty: &Type) -> String {
    match ty {
        Type::Named(name, _) => name.name.clone(),
        Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => format!("ptr_{}", sort_key(inner)),
        Type::Fn(..) => "fn".to_string(),
        _ => "_".to_string(),
    }
}

// =========================================================================
// SMT Generator
// =========================================================================

pub struct SMTGenerator {
    buf: String,
    ssa_counter: u64,
    /// base local name -> current SSA constant (latest binding wins)
    latest: HashMap<String, String>,
    /// variable/symbol -> SMT sort string
    var_sort_map: HashMap<String, String>,
    /// declared XIOM struct: type name -> ordered (field, Type)
    structs: Vec<(String, Vec<(String, Type)>)>,
    /// struct names that made it to declare-datatype
    datatype_sorts: HashSet<String>,
    /// all function signatures: bare name -> (param sorts, ret sort)
    fn_sigs: HashMap<String, (Vec<String>, Option<String>)>,
    /// Accumulated side-condition obligations
    side_conditions: Vec<SideCondition>,
    /// First reason the CURRENT translation became inexpressible.
    unsupported: Option<String>,
    /// First reason the FUNCTION BODY could not be fully encoded. When set,
    /// |result| is unconstrained -- every obligation must be skipped
    /// (otherwise ensures checks run against nothing and fire spuriously).
    body_unsupported: Option<String>,
    /// R64 soundness fix: contracted functions and their modular axioms
    /// (`forall ... (requires => ensures)`), in declaration order. The axioms
    /// are assumed per CHECK SCOPE (see `emit_assumed_axioms`), never
    /// globally -- a function must not assume its own contract while its body
    /// is being proved (that proved every `ensures` vacuously).
    contract_axioms: Vec<(String, String)>,
    /// R64 fix: module-level constants (name, type, value). Uncollected,
    /// `MAX_ORDER` in a contract became an "unknown constant" and z3 rejected
    /// the whole script.
    consts: Vec<(String, Type, Expr)>,
    /// Monotonic counter for `:named` labels. Source-derived labels
    /// (fn+line+clause-index) collided when several obligations shared a
    /// line; z3 rejects duplicate named expressions ("named expression
    /// already defined") and the whole script fails.
    obligation_counter: u64,
    /// Dynamic (`|xiom_...|`) sorts already declared via ensure_sort_declared.
    declared_sorts: HashSet<String>,
    report: GenReport,
}

#[derive(Debug, Clone)]
struct SideCondition {
    code: &'static str,
    message: String,
    smt: String,
}

impl SMTGenerator {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            ssa_counter: 0,
            latest: HashMap::new(),
            var_sort_map: HashMap::new(),
            structs: Vec::new(),
            datatype_sorts: HashSet::new(),
            fn_sigs: HashMap::new(),
            side_conditions: Vec::new(),
            unsupported: None,
            body_unsupported: None,
            contract_axioms: Vec::new(),
            consts: Vec::new(),
            obligation_counter: 0,
            declared_sorts: HashSet::new(),
            report: GenReport::default(),
        }
    }

    /// Unique `:named` label (see obligation_counter).
    fn unique_label(&mut self, base: &str) -> String {
        self.obligation_counter += 1;
        format!("{}_{}", base, self.obligation_counter)
    }

    /// Emit `(declare-sort S 0)` once for a dynamic (`|...|`) sort the first
    /// time it is used. Primitives (Int/Bool/Real/String) and datatype names
    /// are not touched. Without this, a let/const whose type fell back to
    /// `|xiom_unknown|` produced `(declare-const x |xiom_unknown|)` with no
    /// sort declaration -- z3 rejected the whole script.
    fn ensure_sort_declared(&mut self, sort: &str) {
        if !sort.starts_with('|') {
            return;
        }
        if self.declared_sorts.insert(sort.to_string()) {
            self.emit(&format!("(declare-sort {} 0)", sort));
        }
    }

    // ---------------------------------------------------------------------
    // Public API
    // ---------------------------------------------------------------------

    /// Backwards-compatible entry point (drops the skip report).
    pub fn generate(&mut self, program: &Program) -> String {
        self.generate_with_report(program).0
    }

    /// Generate SMT-LIB text plus the list of obligations that could not be
    /// faithfully encoded (callers MUST surface these as UNKNOWN).
    pub fn generate_with_report(&mut self, program: &Program) -> (String, GenReport) {
        self.emit("; XIOM Contract Verification -- SMT-LIB 2.6");
        self.emit("; Sort policy: integer widths -> Int, float widths -> Real,");
        self.emit(";              mappable structs -> datatypes (selectors <T>-<f>).");
        self.emit("(set-option :produce-models true)");
        self.emit("");

        // Pass 1: collect structs and function signatures (declaration order).
        self.collect_structs(program);
        self.collect_fns(program);
        self.collect_consts(&program.items);

        // 2026-10-04: declare every dynamic sort at TOP LEVEL first. Lazy
        // in-scope declarations (inside a function `(push)`) were lost at
        // `(pop)` and later uses failed with "unknown sort".
        self.collect_dynamic_sorts(program);

        // Emit struct datatypes BEFORE any function section references them.
        self.emit_datatypes();

        // R64 fix: declare module constants + their defining values so
        // contracts and bodies can refer to them (MAX_ORDER, MIN_BLOCK, ...).
        self.emit_consts();

        // Pass 2: contracted function signatures. Axioms are NOT asserted here
        // -- they are scoped into each check section (R64 soundness fix).
        let contracted = self.collect_contracted_fns(program);
        if !contracted.is_empty() {
            self.emit("; --- contracted function signatures (axioms scoped per check) ---");
            for (name, params, ret, reqs, enss) in &contracted {
                self.emit_fn_decl(name, params, ret.as_ref());
                self.emit_contract_axiom(name, params, ret.as_ref(), reqs, enss);
            }
            self.emit("");
        }

        // Pass 3: per-function verification sections.
        for item in &program.items {
            self.process_top_decl(item);
        }

        (std::mem::take(&mut self.buf), std::mem::take(&mut self.report))
    }

    // ---------------------------------------------------------------------
    // Collection passes
    // ---------------------------------------------------------------------

    fn collect_structs(&mut self, program: &Program) {
        Self::walk_types(&program.items, &mut self.structs);
        // A struct gets a datatype iff every field maps to a known sort
        // (primitive or an earlier-declared datatype).
        let mut done: HashSet<String> = HashSet::new();
        for (name, fields) in self.structs.clone() {
            let mappable = fields.iter().all(|(_, ty)| self.field_sort(ty, &done).is_some());
            if mappable {
                done.insert(name);
            }
        }
        self.datatype_sorts = done;
    }

    fn walk_types(items: &[TopDecl], out: &mut Vec<(String, Vec<(String, Type)>)>) {
        for item in items {
            match item {
                TopDecl::Type(td) if !td.fields.is_empty() => {
                    out.push((td.name.name.clone(), td.fields.iter()
                        .map(|f| (f.name.name.clone(), f.ty.clone())).collect()));
                }
                TopDecl::Module(m) => Self::walk_types(&m.items, out),
                _ => {}
            }
        }
    }

    /// R64: collect module-level constants (including inside modules).
    fn collect_consts(&mut self, items: &[TopDecl]) {
        for item in items {
            match item {
                TopDecl::Const(c) => self.consts.push((c.name.name.clone(), c.ty.clone(), c.value.clone())),
                TopDecl::Module(m) => self.collect_consts(&m.items),
                _ => {}
            }
        }
    }

    /// R64: declare module constants and assert their values. Contract clauses
    /// and bodies can then reason about them instead of z3 rejecting the
    /// script with "unknown constant".
    fn emit_consts(&mut self) {
        if self.consts.is_empty() {
            return;
        }
        self.emit("; --- module constants ---");
        // Declare all first so constants may reference each other.
        for (name, ty, _) in self.consts.clone() {
            let sort = self.sort_for(&ty);
            self.ensure_sort_declared(&sort);
            self.emit(&format!("(declare-const {} {})", smt_escape(&name), sort));
            self.var_sort_map.insert(name.clone(), sort);
        }
        for (name, _, value) in self.consts.clone() {
            let saved = self.unsupported.take();
            let term = self.translate_expr_to_val(&value);
            let unsup = self.unsupported.take();
            self.unsupported = saved;
            match unsup {
                Some(reason) => self.emit(&format!("; const {} value unsupported: {}", name, reason)),
                None => self.emit(&format!("(assert (= {} {}))", smt_escape(&name), term)),
            }
        }
        self.emit("");
    }

    fn field_sort(&self, ty: &Type, declared: &HashSet<String>) -> Option<String> {
        if let Some(s) = primitive_sort(ty) {
            // Ref-to-primitive produces opaque ptr sorts -- treat those as
            // NOT mappable so we never claim semantics we don't have.
            return match s.starts_with("|xiom_") {
                true => None,
                false => Some(s),
            };
        }
        if let Type::Named(n, _) = ty {
            let key = sort_key(ty);
            let _ = n;
            if declared.contains(&key) {
                return Some(key);
            }
        }
        None
    }

    fn collect_fns(&mut self, program: &Program) {
        let mut sigs: HashMap<String, (Vec<String>, Option<String>)> = HashMap::new();
        Self::walk_fns(&program.items, &mut |f: &FnDecl| {
            let params: Vec<String> = f.params.iter()
                .map(|p| self.sort_for(&p.ty)).collect();
            let ret = f.return_type.as_ref().map(|t| self.sort_for(t));
            sigs.insert(f.name.name.clone(), (params, ret));
        });
        self.fn_sigs = sigs;
    }

    fn walk_fns(items: &[TopDecl], f: &mut dyn FnMut(&FnDecl)) {
        for item in items {
            match item {
                TopDecl::Fn(fd) => f(fd),
                TopDecl::Module(m) => Self::walk_fns(&m.items, f),
                _ => {}
            }
        }
    }

    /// Sort for a type under the CURRENT policy (datatypes included).
    fn sort_for(&self, ty: &Type) -> String {
        if let Some(s) = primitive_sort(ty) {
            if !s.starts_with("|xiom_") {
                return s;
            }
        }
        let key = sort_key(ty);
        if self.datatype_sorts.contains(&key) {
            return key;
        }
        format!("|xiom_{}|", key)
    }

    fn emit_datatypes(&mut self) {
        if self.datatype_sorts.is_empty() {
            return;
        }
        self.emit("; --- struct datatypes ---");
        let entries: Vec<(String, Vec<(String, Type)>)> = self.structs.clone();
        for (name, fields) in entries {
            if !self.datatype_sorts.contains(&name) {
                continue;
            }
            // Selector names MUST match the translator's `{Type}-{field}`
            // form (Expr::Field). The old unqualified `field` names made
            // every datatype selector application an "unknown constant".
            let parts: Vec<String> = fields.iter().map(|(fname, fty)| {
                format!("({}-{} {})", name, fname, self.sort_for(fty))
            }).collect();
            self.emit(&format!("(declare-datatype {} ((mk-{} {})))", name, name, parts.join(" ")));
        }
        self.emit("");
    }

    /// Declare every dynamic (`|...|`) sort the program can use -- function
    /// signatures, struct fields, constants, annotated locals, and the
    /// `|xiom_unknown|` inference fallback -- at TOP LEVEL. Lazy emission
    /// inside a function `(push)` scope was lost at `(pop)`, so later uses
    /// failed with "Invalid constant declaration: unknown sort".
    fn collect_dynamic_sorts(&mut self, program: &Program) {
        let mut ann_types: Vec<Type> = Vec::new();
        for (_, fields) in &self.structs {
            for (_, ty) in fields {
                ann_types.push(ty.clone());
            }
        }
        for (_, ty, _) in &self.consts {
            ann_types.push(ty.clone());
        }
        Self::walk_annotated_types(&program.items, &mut ann_types);
        let mut sorts: Vec<String> = vec!["|xiom_unknown|".to_string()];
        for ty in ann_types {
            let s = self.sort_for(&ty);
            if s.starts_with('|') {
                sorts.push(s);
            }
        }
        for (_, (psorts, rsort)) in &self.fn_sigs {
            for s in psorts.iter().chain(rsort.iter()) {
                if s.starts_with('|') {
                    sorts.push(s.clone());
                }
            }
        }
        sorts.sort();
        sorts.dedup();
        self.emit("; --- opaque sorts (non-mappable types + inference fallback) ---");
        for s in sorts {
            if self.declared_sorts.insert(s.clone()) {
                self.emit(&format!("(declare-sort {} 0)", s));
            }
        }
        self.emit("");
    }

    fn walk_annotated_types(items: &[TopDecl], out: &mut Vec<Type>) {
        for item in items {
            match item {
                TopDecl::Fn(f) => {
                    for p in &f.params {
                        out.push(p.ty.clone());
                    }
                    if let Some(r) = &f.return_type {
                        out.push(r.clone());
                    }
                    if let Some(b) = &f.body {
                        Self::walk_block_types(b, out);
                    }
                }
                TopDecl::Const(c) => out.push(c.ty.clone()),
                TopDecl::Module(m) => Self::walk_annotated_types(&m.items, out),
                _ => {}
            }
        }
    }

    fn walk_block_types(b: &Block, out: &mut Vec<Type>) {
        for item in &b.stmts {
            let stmt = match item {
                StmtOrExpr::Stmt(s) => s,
                StmtOrExpr::Expr(..) => continue,
            };
            match stmt {
                Stmt::Let(_, Some(t), ..) | Stmt::Var(_, Some(t), ..) => out.push((**t).clone()),
                Stmt::If(_, then_b, elifs, else_b, _) => {
                    Self::walk_block_types(then_b, out);
                    for (_, b) in elifs {
                        Self::walk_block_types(b, out);
                    }
                    if let Some(b) = else_b {
                        Self::walk_block_types(b, out);
                    }
                }
                Stmt::While(_, body, _, _, _) => Self::walk_block_types(body, out),
                Stmt::For(_, _, body, _, _) => Self::walk_block_types(body, out),
                _ => {}
            }
        }
    }

    fn emit_fn_decl(&mut self, name: &str, params: &[(String, Type)], ret: Option<&Type>) {
        let psorts: Vec<String> = params.iter().map(|(_, t)| self.sort_for(t)).collect();
        let rsort = ret.map(|t| self.sort_for(t)).unwrap_or_else(|| "Bool".to_string());
        // R64 fix: declare-fun takes SORTS only. The previous `(name sort)`
        // pair formatting emitted `(declare-fun |f| ((x Int)) Int)`, which z3
        // rejects with "unknown sort 'x'" -- every contract proof failed.
        // Binder NAMES stay in the contract axiom's `forall`, not here.
        for s in psorts.iter().chain(std::iter::once(&rsort)) {
            self.ensure_sort_declared(s);
        }
        self.emit(&format!("(declare-fun |{}| ({}) {})", smt_escape(name), psorts.join(" "), rsort));
    }

    fn collect_contracted_fns(&self, program: &Program)
        -> Vec<(String, Vec<(String, Type)>, Option<Type>, Vec<Expr>, Vec<Expr>)>
    {
        let mut fns = Vec::new();
        Self::collect_from_items(&program.items, &mut fns);
        fns
    }

    fn collect_from_items(items: &[TopDecl], out: &mut Vec<(String, Vec<(String, Type)>, Option<Type>, Vec<Expr>, Vec<Expr>)>) {
        for item in items {
            match item {
                TopDecl::Fn(f) if !f.contracts.is_empty() => {
                    let params: Vec<(String, Type)> =
                        f.params.iter().map(|p| {
                            // Method receiver: the implicit `self` param is
                            // typed `Self`; bind it at the OWNER type so
                            // field selectors and axioms are well-sorted and
                            // `self` is not redeclared with a different sort.
                            let ty = match (p.name.name.as_str(), &f.receiver) {
                                ("self", Some(recv)) => Type::Named(recv.clone(), Vec::new()),
                                _ => p.ty.clone(),
                            };
                            (p.name.name.clone(), ty)
                        }).collect();
                    let reqs = f.contracts.iter()
                        .filter_map(|c| match c { ContractClause::Requires(e, _) => Some(e.clone()), _ => None })
                        .collect();
                    let enss = f.contracts.iter()
                        .filter_map(|c| match c { ContractClause::Ensures(e, _) => Some(e.clone()), _ => None })
                        .collect();
                    out.push((f.name.name.clone(), params, f.return_type.clone(), reqs, enss));
                }
                TopDecl::Module(m) => Self::collect_from_items(&m.items, out),
                _ => {}
            }
        }
    }

    /// Modular contract axiom: forall params [+ result]. requires => ensures.
    ///
    /// AUDIT FIX: the binder list previously included the RETURN SORT STRING
    /// itself ("(forall ((a Int) Int ...)") -- invalid SMT-LIB. Binders are
    /// now exactly params plus |result| (when the function returns and any
    /// ensures exists). Unsupported clauses skip the whole axiom honestly.
    fn emit_contract_axiom(
        &mut self,
        name: &str,
        params: &[(String, Type)],
        ret: Option<&Type>,
        reqs: &[Expr],
        enss: &[Expr],
    ) {
        if reqs.is_empty() && enss.is_empty() {
            return;
        }
        // Scope: fresh var environment for the quantified binders.
        let saved_latest = self.latest.clone();
        let saved_vars = self.var_sort_map.clone();
        self.latest.clear();

        for (n, t) in params {
            let s = self.sort_for(t);
            self.var_sort_map.insert(n.clone(), s);
        }
        if let Some(rt) = ret {
            let s = self.sort_for(rt);
            self.var_sort_map.insert("result".to_string(), s);
        }

        let saved_unsup = self.unsupported.take();
        let req_terms: Vec<String> = reqs.iter()
            .map(|e| self.translate_expr_to_val(e)).collect();
        let ens_terms: Vec<String> = enss.iter()
            .map(|e| self.translate_expr_to_val(e)).collect();
        let unsup = self.unsupported.take();
        self.unsupported = saved_unsup;

        if let Some(reason) = unsup {
            self.report.skip(X7007_UNKNOWN, name, format!("contract axiom skipped: {}", reason));
            self.emit(&format!("; axiom skipped ({}): {}", name, reason));
        } else {
            let mut binders: Vec<String> = params.iter()
                .map(|(n, t)| format!("({} {})", smt_escape(n), self.sort_for(t)))
                .collect();
            if let Some(rt) = ret {
                if !enss.is_empty() {
                    binders.push(format!("(|result| {})", self.sort_for(rt)));
                }
            }
            let body = if enss.is_empty() {
                format!("(=> (and {}) true)", req_terms.join(" "))
            } else if reqs.is_empty() {
                format!("(and {})", ens_terms.join(" "))
            } else {
                format!("(=> (and {}) (and {}))", req_terms.join(" "), ens_terms.join(" "))
            };
            let axiom = format!("(forall ({}) {})", binders.join(" "), body);
            // Remember the term; the caller scopes it into check sections
            // (all axioms except the function's own, plus all of them for
            // type-invariant checks). Never assert globally: assuming a
            // function's own contract while proving its body is unsound.
            self.contract_axioms.push((name.to_string(), axiom));
        }

        self.latest = saved_latest;
        self.var_sort_map = saved_vars;
    }

    // ---------------------------------------------------------------------
    // Top-level walk
    // ---------------------------------------------------------------------

    fn process_top_decl(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Fn(f) if !f.contracts.is_empty() => {
                self.verify_function(f);
            }
            TopDecl::Type(td) if !td.invariants.is_empty() => {
                self.verify_type_invariants(td);
            }
            TopDecl::Module(m) => {
                for item in &m.items {
                    self.process_top_decl(item);
                }
            }
            _ => {}
        }
    }

    /// R64: assert the modular contract axioms available to the CURRENT check
    /// scope, skipping `skip` (the function whose body is being proved).
    /// Emitted with a readable `; assumes` comment for auditability.
    fn emit_assumed_axioms(&mut self, skip: Option<&str>) {
        for (name, axiom) in self.contract_axioms.clone() {
            if Some(name.as_str()) == skip {
                continue;
            }
            self.emit(&format!("; assumes |contract_{}|", name));
            self.emit(&format!("(assert {})", axiom));
        }
    }

    // =====================================================================
    // Function verification
    // =====================================================================

    fn verify_function(&mut self, f: &FnDecl) {
        let fname = f.name.name.clone();
        self.emit(&format!("; === Function: {} ===", fname));
        self.emit("(push)");
        self.side_conditions.clear();
        self.latest.clear();
        self.ssa_counter = 0;
        self.body_unsupported = None;

        // R64 soundness fix: assume every OTHER function's contract axiom
        // (call-site composition), never this function's own -- assuming its
        // own contract made every `ensures` vacuously "proven"
        // (test_buggy_abs proved despite returning -x).
        self.emit_assumed_axioms(Some(&fname));

        // Register param sorts.
        for param in &f.params {
            // Method receiver `self` (typed `Self`) resolves to the owner
            // type here too (see collect_from_items).
            let sort = match (param.name.name.as_str(), &f.receiver) {
                ("self", Some(recv)) => self.sort_for_named(&recv.name),
                _ => self.sort_for(&param.ty),
            };
            self.ensure_sort_declared(&sort);
            self.emit(&format!("(declare-const {} {})", smt_escape(&param.name.name), sort));
            self.var_sort_map.insert(param.name.name.clone(), sort.clone());
            self.latest.insert(param.name.name.clone(), smt_escape(&param.name.name));
        }
        if let Some(ret) = &f.return_type {
            let sort = self.sort_for(ret);
            self.ensure_sort_declared(&sort);
            self.emit(&format!("(declare-const |result| {})", sort));
            self.var_sort_map.insert("result".to_string(), sort);
        }
        // Method receiver: contracts reference `self` (field selectors on the
        // receiver). Only type invariants used to declare |self|, so every
        // method `ensures self.f` became an undeclared symbol
        // ("unknown constant self").
        if let Some(recv) = &f.receiver {
            if !self.var_sort_map.contains_key("self") {
                let owner_sort = self.sort_for_named(&recv.name);
                self.ensure_sort_declared(&owner_sort);
                self.emit(&format!("(declare-const |self| {})", owner_sort));
                self.var_sort_map.insert("self".to_string(), owner_sort.clone());
                self.latest.insert("self".to_string(), "|self|".to_string());
            }
        }
        self.emit("");

        // Requires assumptions.
        let mut req_unsupported: Option<String> = None;
        let mut has_requires = false;
        for contract in &f.contracts {
            if let ContractClause::Requires(e, span) = contract {
                has_requires = true;
                let label = self.unique_label(&format!("req_{}_{}", smt_escape(&fname), span.line));
                self.emit(&format!("; requires (line {}): {}", span.line, expr_display(e)));
                let saved = self.unsupported.take();
                let term = self.translate_expr_to_val(e);
                let unsup = self.unsupported.take();
                self.unsupported = saved;
                if let Some(reason) = unsup {
                    req_unsupported = Some(format!("requires (line {}) unsupported: {}", span.line, reason));
                    self.emit(&format!("; requires assumption skipped: {}", reason));
                } else {
                    self.emit(&format!("(assert (! {} :named |{}|))", term, label));
                }
                self.emit("");
            }
        }
        if !has_requires {
            self.emit("; (no requires -- assumes true)");
            self.emit("");
        }

        // Body encoding (guarded SSA).
        if let Some(body) = &f.body {
            self.emit("; --- body encoding ---");
            let _ = self.encode_block(&body.stmts, "true");
            self.emit("");
        }

        // When the body could not be fully encoded, |result| is
        // unconstrained: running ensures checks against NOTHING would fire
        // spurious VIOLATED verdicts. Every obligation is gated on this.
        let body_gap = self.body_unsupported.clone();

        // Ensures checks: per-clause push/(assert (not E))/check-sat/pop.
        for (i, contract) in f.contracts.iter().enumerate() {
            if let ContractClause::Ensures(e, span) = contract {
                let label = self.unique_label(&format!("ens_{}_{}_{}", smt_escape(&fname), span.line, i));
                let display = expr_display(e);
                self.emit(&format!("; ensures (line {}): {}", span.line, display));
                if let Some(reason) = &body_gap {
                    self.emit(&format!("; skipped: body incomplete ({})", reason));
                    continue;
                }
                if let Some(reason) = &req_unsupported {
                    self.emit(&format!("; skipped: {}", reason));
                    self.report.skip(X7007_UNKNOWN, &fname, reason.clone());
                    continue;
                }
                let saved = self.unsupported.take();
                let term = self.translate_expr_to_val(e);
                let unsup = self.unsupported.take();
                self.unsupported = saved;
                if let Some(reason) = unsup {
                    self.emit(&format!("; skipped: {}", reason));
                    self.report.skip(X7007_UNKNOWN, &fname,
                        format!("ensures (line {}) unsupported: {}", span.line, reason));
                    continue;
                }
                self.emit("(push)");
                self.emit(&format!("(assert (! (not {})", term));
                self.emit(&format!(" :named |{}|))", label));
                self.emit("(check-sat)");
                self.emit("(get-model)");
                self.emit("(pop)");
                self.emit("");
            }
        }

        // Side-condition obligations (div-by-zero etc.).
        if !self.side_conditions.is_empty() {
            self.emit("; --- side-condition obligations ---");
            for sc in self.side_conditions.clone() {
                if let Some(reason) = &body_gap {
                    self.emit(&format!("; skipped {}: body incomplete ({})", sc.code, reason));
                    continue;
                }
                if let Some(reason) = &req_unsupported {
                    self.emit(&format!("; skipped {}: {}", sc.code, reason));
                    self.report.skip(sc.code, &fname, reason.clone());
                    continue;
                }
                self.emit(&format!("; {}: {}", sc.code, sc.message));
                self.emit("(push)");
                let label = self.unique_label(&format!("obl_{}", sc.code));
                self.emit(&format!("(assert (! (not {}) :named |{}|))", sc.smt, label));
                self.emit("(check-sat)");
                self.emit("(get-model)");
                self.emit("(pop)");
                self.emit("");
            }
        }

        self.emit("(pop)");
        self.emit("");
    }

    /// Encode statements under a boolean GUARD term; every assertion becomes
    /// (=> guard A). Returns the FALL-THROUGH guard: after an `if` whose
    /// body DEFINITELY returns, control reaching later statements implies
    /// the negated condition -- threading this prevents the vacuous-proof
    /// hole where contradictory branch contexts made obligations UNSAT.
    fn encode_block(&mut self, stmts: &[StmtOrExpr], guard: &str) -> String {
        let mut ft = guard.to_string();
        for item in stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => self.encode_stmt(stmt, &mut ft),
                StmtOrExpr::Expr(expr) => {
                    let saved = self.unsupported.take();
                    let _ = self.translate_expr_to_val(expr);
                    let _ = self.unsupported.take();
                    self.unsupported = saved;
                }
            }
        }
        ft
    }

    /// Conservative definite-return analysis: a DIRECT return statement, or
    /// an if/else whose arms all definitely return. Used ONLY to decide
    /// whether a branch's negated condition constrains the fall-through path.
    fn block_always_returns(stmts: &[StmtOrExpr]) -> bool {
        for s in stmts {
            match s {
                StmtOrExpr::Stmt(Stmt::Return(..)) => return true,
                StmtOrExpr::Stmt(Stmt::If(_, then_b, elifs, Some(else_b), _)) => {
                    if Self::block_always_returns(&then_b.stmts)
                        && elifs.iter().all(|(_, b)| Self::block_always_returns(&b.stmts))
                        && Self::block_always_returns(&else_b.stmts)
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn encode_stmt(&mut self, stmt: &Stmt, ft: &mut String) {
        match stmt {
            Stmt::Let(name, ty, init, _) | Stmt::Var(name, ty, init, _) => {
                let sort = match ty {
                    Some(t) => self.sort_for(t),
                    None => self.infer_sort(init)
                        .unwrap_or_else(|| "|xiom_unknown|".to_string()),
                };
                let ssa = self.fresh_ssa(&name.name);
                self.ensure_sort_declared(&sort);
                self.emit(&format!("(declare-const {} {})", ssa, sort));
                self.var_sort_map.insert(ssa.clone(), sort);
                let saved = self.unsupported.take();
                let val = self.translate_expr_to_val(init);
                let unsup = self.unsupported.take();
                self.unsupported = saved;
                match unsup {
                    Some(reason) => {
                        self.note_body_gap(&reason);
                        self.emit(&format!("; let {} skipped: {}", name.name, reason))
                    }
                    None => {
                        if ft == "true" {
                            self.emit(&format!("(assert (= {} {}))", ssa, val));
                        } else {
                            self.emit(&format!("(assert (=> {} (= {} {})))", ft, ssa, val));
                        }
                        self.latest.insert(name.name.clone(), ssa);
                    }
                }
            }
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                let saved = self.unsupported.take();
                let c = self.translate_expr_to_val(cond);
                let unsup = self.unsupported.take();
                self.unsupported = saved;
                if let Some(reason) = &unsup {
                    self.note_body_gap(reason);
                    self.emit("; if condition unsupported -- branch skipped");
                    return;
                }
                let base = ft.clone();
                let g_then = self.fresh_guard(&format!("(and {} {})", base, c));
                self.encode_block(&then_b.stmts, &g_then);
                let mut acc = format!("(not {})", c);
                if Self::block_always_returns(&then_b.stmts) {
                    *ft = format!("(and {} {})", base, acc);
                }
                for (cond_e, body_e) in elifs {
                    let saved = self.unsupported.take();
                    let ce = self.translate_expr_to_val(cond_e);
                    let unsup = self.unsupported.take();
                    self.unsupported = saved;
                    if unsup.is_some() {
                        self.emit("; elif condition unsupported -- branch skipped");
                        return;
                    }
                    let g_elif = self.fresh_guard(&format!(
                        "(and {} (and {} {}))", base, acc, ce));
                    self.encode_block(&body_e.stmts, &g_elif);
                    acc = format!("(and {} (not {}))", acc, ce);
                    if Self::block_always_returns(&body_e.stmts) {
                        *ft = format!("(and {} {})", base, acc);
                    }
                }
                if let Some(else_body) = else_b {
                    let g_else = self.fresh_guard(&format!("(and {} {})", base, acc));
                    self.encode_block(&else_body.stmts, &g_else);
                    if Self::block_always_returns(&else_body.stmts) {
                        *ft = format!("(and {} {})", base, acc);
                    }
                }
            }
            Stmt::While(cond, body, invariant, _span, _) => {
                match invariant {
                    Some(inv) => {
                        // Approximation (documented): assume the invariant on
                        // the loop head and encode ONE iteration under it,
                        // registering the invariant as an explicit obligation.
                        // Full fixpoint VC generation is future work.
                        let saved = self.unsupported.take();
                        let inv_t = self.translate_expr_to_val(inv);
                        let inv_unsup = self.unsupported.take();
                        self.unsupported = saved;
                        if let Some(reason) = inv_unsup {
                            self.note_body_gap(&reason);
                            self.emit(&format!("; loop invariant unsupported: {}", reason));
                            self.report.skip(X7006_INVARIANT, "<loop>",
                                format!("invariant unsupported: {}", reason));
                            return;
                        }
                        self.emit(&format!("; loop invariant: {}", expr_display(inv)));
                        self.emit(&format!("(assert (=> {} {}))", ft, inv_t));

                        let saved = self.unsupported.take();
                        let cond_t = self.translate_expr_to_val(cond);
                        let cond_unsup = self.unsupported.take();
                        self.unsupported = saved;
                        if let Some(reason) = cond_unsup {
                            self.note_body_gap(&reason);
                            self.emit(&format!("; loop condition unsupported: {}", reason));
                            return;
                        }
                        let g_iter = self.fresh_guard(&format!(
                            "(and {} (and {} {}))", ft, inv_t, cond_t));
                        self.encode_block(&body.stmts, &g_iter);
                        self.side_conditions.push(SideCondition {
                            code: X7006_INVARIANT,
                            message: format!("loop invariant must hold: {}", expr_display(inv)),
                            smt: inv_t,
                        });
                    }
                    None => {
                        // Naked loop: one-step encoding cannot summarize it --
                        // honest UNKNOWN instead of fabricated proofs.
                        self.emit("; WARNING: loop without invariant -- obligations become UNKNOWN");
                        self.report.skip(X7007_UNKNOWN, "<loop>",
                            "loop without invariant cannot be verified".to_string());
                        let saved = self.unsupported.take();
                        let _ = self.translate_expr_to_val(cond);
                        let _ = self.unsupported.take();
                        self.unsupported = saved;
                    }
                }
            }
            Stmt::Return(expr_opt, _) => {
                if let Some(expr) = expr_opt {
                    let saved = self.unsupported.take();
                    let val = self.translate_expr_to_val(expr);
                    let unsup = self.unsupported.take();
                    self.unsupported = saved;
                    match unsup {
                        Some(reason) => {
                            self.note_body_gap(&reason);
                            self.emit(&format!("; return skipped: {}", reason))
                        }
                        None => {
                            if ft == "true" {
                                self.emit(&format!("(assert (= |result| {}))", val));
                            } else {
                                self.emit(&format!("(assert (=> {} (= |result| {})))", ft, val));
                            }
                        }
                    }
                }
            }
            Stmt::Assign(target, expr, _) => {
                let base = match target {
                    Expr::Ident(id) => id.name.clone(),
                    Expr::Field(obj, field, _) => match obj.as_ref() {
                        Expr::Ident(id) => format!("{}.{}", id.name, field.name),
                        _ => {
                            self.emit("; WARNING: unsupported assign target");
                            return;
                        }
                    },
                    _ => {
                        self.emit("; WARNING: unsupported assign target");
                        return;
                    }
                };
                let sort = self.var_sort_map.get(&base)
                    .cloned()
                    .or_else(|| self.infer_sort(expr))
                    .unwrap_or_else(|| "|xiom_unknown|".to_string());
                let ssa = self.fresh_ssa(&base);
                self.ensure_sort_declared(&sort);
                self.emit(&format!("(declare-const {} {})", ssa, sort));
                self.var_sort_map.insert(ssa.clone(), sort);
                let saved = self.unsupported.take();
                let val = self.translate_expr_to_val(expr);
                let unsup = self.unsupported.take();
                self.unsupported = saved;
                match unsup {
                    Some(reason) => {
                        self.note_body_gap(&reason);
                        self.emit(&format!("; assign skipped: {}", reason))
                    }
                    None => {
                        if ft == "true" {
                            self.emit(&format!("(assert (= {} {}))", ssa, val));
                        } else {
                            self.emit(&format!("(assert (=> {} (= {} {})))", ft, ssa, val));
                        }
                        self.latest.insert(base, ssa);
                    }
                }
            }
            Stmt::Expr(expr, _) => {
                let saved = self.unsupported.take();
                let _ = self.translate_expr_to_val(expr);
                let _ = self.unsupported.take();
                self.unsupported = saved;
            }
            _ => {
                self.emit("; WARNING: unsupported statement type -- body may be incomplete");
            }
        }
    }
    fn fresh_ssa(&mut self, base: &str) -> String {
        let name = format!("|{}_ssa{}|", smt_escape(base), self.ssa_counter);
        self.ssa_counter += 1;
        name
    }

    fn fresh_guard(&mut self, term: &str) -> String {
        let name = format!("|guard{}|", self.ssa_counter);
        self.ssa_counter += 1;
        self.emit(&format!("(declare-const {} Bool)", name));
        self.emit(&format!("(assert (= {} {}))", name, term));
        name
    }

    /// Infer a term's sort WITHOUT emitting anything (used for lets without
    /// annotations and sort-aware operators). Returns None when unknown.
    fn infer_sort(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Int(..) | Expr::Char(..) => Some("Int".to_string()),
            Expr::Float(..) => Some("Real".to_string()),
            Expr::Bool(..) => Some("Bool".to_string()),
            Expr::Str(..) => Some("String".to_string()),
            Expr::Ident(id) => self.var_sort_map.get(&id.name).cloned(),
            Expr::Unary(op, inner, _) => match op {
                UnaryOp::Not => Some("Bool".to_string()),
                _ => self.infer_sort(inner),
            },
            Expr::Binary(l, op, r, _) => {
                let lt = self.infer_sort(l)?;
                let rt = self.infer_sort(r)?;
                if lt != rt {
                    return None;
                }
                match op {
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt
                    | BinOp::Le | BinOp::Ge | BinOp::And | BinOp::Or => {
                        Some("Bool".to_string())
                    }
                    _ => Some(lt),
                }
            }
            Expr::Call(func, ..) | Expr::GenericCall(func, ..) => {
                let name = call_callee_name(func)?;
                let (_, ret) = self.fn_sigs.get(&name)?;
                ret.clone()
            }
            Expr::Imply(..) => Some("Bool".to_string()),
            Expr::Field(obj, field, _) => {
                let obj_sort = self.infer_sort(obj)?;
                self.selector_sort(&obj_sort, &field.name)
            }
            _ => None,
        }
    }

    /// Selector return sort for `<field>` on datatype sort `<obj_sort>`.
    fn selector_sort(&self, obj_sort: &str, field: &str) -> Option<String> {
        let dt = obj_sort.trim_matches('|');
        for (name, fields) in &self.structs {
            if name != dt || !self.datatype_sorts.contains(name) {
                continue;
            }
            for (fname, fty) in fields {
                if fname == field {
                    return Some(self.sort_for(fty));
                }
            }
        }
        None
    }

    // =====================================================================
    // Type invariants -- REAL VCs now (were TODO comments, audited)
    // =====================================================================

    fn verify_type_invariants(&mut self, td: &TypeDecl) {
        let tname = td.name.name.clone();
        self.emit(&format!("; === Type: {} -- Invariants ===", tname));
        self.emit("(push)");
        // Invariant checks are not body checks -- all callable contracts hold.
        self.emit_assumed_axioms(None);
        let sort = self.sort_for_named(&tname);
        self.ensure_sort_declared(&sort);
        self.emit(&format!("(declare-const |self| {})", sort));
        self.var_sort_map.insert("self".to_string(), sort.clone());
        self.latest.clear();
        self.latest.insert("self".to_string(), "|self|".to_string());

        // Bare field names in invariants (`invariant: lo <= hi`) refer to
        // fields of `self`. Bind them to `(<Type>-<field> |self|)` for
        // datatypes. For non-mappable types (generic structs like
        // Range[T], opaque fields) the fields have no modeled sort -- skip
        // the invariant as UNKNOWN instead of emitting undeclared symbols
        // (2026-10-04: z3 rejected the whole script with "unknown constant").
        if !self.datatype_sorts.contains(&tname) {
            for (i, _) in td.invariants.iter().enumerate() {
                self.emit(&format!("; invariant {} skipped: non-mappable type (generic/opaque)", i));
                self.report.skip(X7006_INVARIANT, &tname,
                    format!("invariant {} on non-mappable type '{}'", i, tname));
            }
            self.emit("(pop)");
            self.emit("");
            return;
        }

        for (i, inv) in td.invariants.iter().enumerate() {
            let label = self.unique_label(&format!("inv_{}_{}", smt_escape(&tname), i));
            self.emit(&format!("; invariant {}: {}", i, expr_display(inv)));
            let saved = self.unsupported.take();
            let term = self.translate_expr_to_val(inv);
            let unsup = self.unsupported.take();
            self.unsupported = saved;
            if let Some(reason) = unsup {
                self.emit(&format!("; skipped: {}", reason));
                self.report.skip(X7006_INVARIANT, &tname,
                    format!("invariant {} unsupported: {}", i, reason));
                continue;
            }
            self.emit("(push)");
            self.emit(&format!("(assert (! (not {})", term));
            self.emit(&format!(" :named |{}|))", label));
            self.emit("(check-sat)");
            self.emit("(get-model)");
            self.emit("(pop)");
        }
        self.emit("(pop)");
        self.emit("");
    }

    fn sort_for_named(&self, name: &str) -> String {
        if self.datatype_sorts.contains(name) {
            name.to_string()
        } else {
            format!("|xiom_{}|", name)
        }
    }

    // =====================================================================
    // Expression translation
    // =====================================================================

    fn translate_expr_to_val(&mut self, expr: &Expr) -> String {
        let mut val = String::new();
        std::mem::swap(&mut self.buf, &mut val);
        self.translate_expr(expr);
        std::mem::swap(&mut self.buf, &mut val);
        val
    }

    /// Mark the current translation inexpressible (first reason wins).
    fn mark_unsupported(&mut self, reason: String) {
        if self.unsupported.is_none() {
            self.unsupported = Some(reason);
        }
    }

    /// Record a body-encoding gap (first wins). Gates all obligations.
    fn note_body_gap(&mut self, reason: &str) {
        if self.body_unsupported.is_none() {
            self.body_unsupported = Some(reason.to_string());
            self.report.skip(X7007_UNKNOWN, "<body>", format!("function body incomplete: {}", reason));
        }
    }

    /// Latest binding term for an identifier (SSA-aware). Used by callers
    /// that have already validated the identifier is known.
    #[allow(dead_code)]
    fn ident_term(&self, name: &str) -> String {
        self.latest.get(name).cloned()
            .unwrap_or_else(|| smt_escape(name))
    }

    fn translate_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(n, _) => {
                // Unbounded Int numeral (bit-pattern u64 reinterpreted as
                // signed i64 to match runtime semantics).
                self.buf.push_str(&format!("{}", *n as i64));
            }
            Expr::Float(f, _) => {
                // Real decimal literal -- SMT-LIB decimals have NO exponent
                // form; expand scientific notation.
                self.buf.push_str(&real_literal(*f));
            }
            Expr::Bool(b, _) => {
                self.buf.push_str(if *b { "true" } else { "false" });
            }
            Expr::Char(c, _) => {
                self.buf.push_str(&format!("{}", *c as i64));
            }
            Expr::Ident(id) => {
                // SOUNDNESS FIX (2026-10-04): never emit an undeclared bare
                // symbol. The old fallback produced `initialized`/`self`/`null`
                // names with no declaration and z3 rejected the whole script.
                // Known bindings: SSA `latest`, then declared
                // params/consts/`result`/`self` in `var_sort_map`; anything
                // else makes the obligation UNKNOWN instead.
                if let Some(term) = self.latest.get(&id.name).cloned() {
                    self.buf.push_str(&term);
                } else if self.var_sort_map.contains_key(&id.name) {
                    self.buf.push_str(&smt_escape(&id.name));
                } else {
                    self.mark_unsupported(format!("unknown identifier '{}'", id.name));
                    self.buf.push('0');
                }
            }
            Expr::Str(s, _) => {
                self.buf.push_str(&format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")));
            }
            Expr::BigInt(n, _) => {
                // Fits i64 -> numeral; wider -> honest UNKNOWN (unbounded SMT
                // Int accepts big numerals, but our runtime semantics are
                // i64-bits; do not claim more than we verify).
                if *n <= i64::MAX as u128 {
                    self.buf.push_str(&format!("{}", *n as i64));
                } else {
                    self.mark_unsupported("BigInt literal beyond i64 range".to_string());
                }
            }
            Expr::Imply(left, right, _) => {
                let ls = self.infer_sort(left);
                let rs = self.infer_sort(right);
                if ls.as_deref().map_or(false, |s| s != "Bool")
                    || rs.as_deref().map_or(false, |s| s != "Bool")
                {
                    self.mark_unsupported("implication with non-Bool operands".to_string());
                    return;
                }
                self.buf.push_str("(=> ");
                self.translate_expr(left);
                self.buf.push(' ');
                self.translate_expr(right);
                self.buf.push(')');
            }            Expr::Binary(left, op, right, span) => {
                // Sort-aware emission. Determine operand sorts first.
                let ls = self.infer_sort(left);
                let rs = self.infer_sort(right);
                let numeric = match (&ls, &rs) {
                    (Some(a), Some(b)) if a == b && (a == "Int" || a == "Real") => Some(a.clone()),
                    _ => None,
                };

                // SOUNDNESS FIX (2026-10-04): only form an operator when the
                // operand sorts are known to be well-sorted. The old emitter
                // happily built `(<= xiom_T x)` / `(+ (Array ...) ...)` and
                // z3 rejected the WHOLE script ("Sort mismatch") -- which the
                // harness then reported as a bogus toolchain error. These
                // shapes now skip the obligation as X7007 and never poison
                // the script.
                let arithmetic_or_compare = matches!(
                    op,
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
                        | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge
                );
                if arithmetic_or_compare && numeric.is_none() {
                    self.mark_unsupported(format!(
                        "operator {:?} on non-numeric operands (sorts {:?}/{:?})", op, ls, rs));
                    return;
                }
                if matches!(op, BinOp::And | BinOp::Or)
                    && (ls.as_deref().map_or(false, |s| s != "Bool")
                        || rs.as_deref().map_or(false, |s| s != "Bool"))
                {
                    self.mark_unsupported(format!("logical {:?} on non-Bool operands", op));
                    return;
                }
                if matches!(op, BinOp::Eq | BinOp::Neq) {
                    match (&ls, &rs) {
                        (Some(a), Some(b)) if a != b => {
                            self.mark_unsupported(format!("equality across sorts {a} vs {b}"));
                            return;
                        }
                        (None, _) | (_, None) => {
                            self.mark_unsupported(
                                "equality with unresolved operand sort".to_string());
                            return;
                        }
                        _ => {}
                    }
                }

                // Div/rem-by-zero side conditions (well-sorted under the
                // unified policy; Real uses the 0.0 literal).
                if matches!(op, BinOp::Div | BinOp::Rem) {
                    let zero = match &ls {
                        Some(s) if s == "Real" => "0.0".to_string(),
                        _ => "0".to_string(),
                    };
                    let mut div_buf = String::new();
                    std::mem::swap(&mut self.buf, &mut div_buf);
                    self.buf.push_str("(not (= ");
                    self.translate_expr(right);
                    self.buf.push_str(&format!(" {}))", zero));
                    let div_smt = std::mem::replace(&mut self.buf, div_buf);
                    let dup = self.side_conditions.iter()
                        .any(|sc| sc.smt == div_smt && sc.code == X7004_DIV_BY_ZERO);
                    if !dup {
                        self.side_conditions.push(SideCondition {
                            code: X7004_DIV_BY_ZERO,
                            message: format!("division by zero at line {}", span.line),
                            smt: div_smt,
                        });
                    }
                }

                // Logical operators.
                if matches!(op, BinOp::And | BinOp::Or) {
                    let op_str = if *op == BinOp::And { "and" } else { "or" };
                    self.buf.push('(');
                    self.buf.push_str(op_str);
                    self.buf.push(' ');
                    self.translate_expr(left);
                    self.buf.push(' ');
                    self.translate_expr(right);
                    self.buf.push(')');
                    return;
                }

                if op == &BinOp::Neq {
                    self.buf.push_str("(not (= ");
                    self.translate_expr(left);
                    self.buf.push(' ');
                    self.translate_expr(right);
                    self.buf.push_str("))");
                    return;
                }

                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div | BinOp::Rem => match numeric.as_deref() {
                        Some("Real") => "/",
                        Some(_) => "div",
                        None => {
                            self.mark_unsupported(format!(
                                "operator {:?} on unresolved/mixed sorts", op));
                            return;
                        }
                    },
                    BinOp::Eq => "=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::Le => "<=",
                    BinOp::Ge => ">=",
                    other => {
                        self.mark_unsupported(format!("binary operator {:?}", other));
                        return;
                    }
                };
                // Rem is Int-only (SMT has no real remainder).
                if *op == BinOp::Rem && numeric.as_deref() == Some("Real") {
                    self.mark_unsupported("remainder on Real operands".to_string());
                    return;
                }
                self.buf.push('(');
                self.buf.push_str(op_str);
                self.buf.push(' ');
                self.translate_expr(left);
                self.buf.push(' ');
                self.translate_expr(right);
                self.buf.push(')');
            }
            Expr::Unary(op, inner, _) => {
                match op {
                    UnaryOp::Not => {
                        self.buf.push_str("(not ");
                        self.translate_expr(inner);
                        self.buf.push(')');
                    }
                    UnaryOp::Neg => {
                        self.buf.push_str("(- ");
                        self.translate_expr(inner);
                        self.buf.push(')');
                    }
                    other => {
                        self.mark_unsupported(format!("unary operator {:?}", other));
                    }
                }
            }
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
                // Only calls to KNOWN signatures translate; unknown callees
                // would emit undeclared symbols (z3 hard error).
                let callee = call_callee_name(func);
                let known = callee.as_ref()
                    .map(|n| self.fn_sigs.contains_key(n))
                    .unwrap_or(false);
                if !known {
                    self.mark_unsupported(match callee {
                        Some(n) => format!("call to unknown function '{}'", n),
                        None => "complex call target".to_string(),
                    });
                    return;
                }
                self.buf.push_str(&format!("(|{}|", smt_escape(callee.as_deref().unwrap_or("?"))));
                for arg in args {
                    self.buf.push(' ');
                    self.translate_expr(arg);
                }
                self.buf.push(')');
            }
            Expr::Field(obj, field, _) => {
                // Datatype selector when the receiver sort is a declared
                // struct; otherwise honest UNKNOWN (never an undeclared
                // uninterpreted function -- audited z3-error machine).
                let obj_sort = self.infer_sort(obj);
                let sel = obj_sort.as_ref()
                    .and_then(|os| {
                        let dt = os.trim_matches('|').to_string();
                        if self.datatype_sorts.contains(&dt) {
                            Some(format!("{}-{}", dt, field.name))
                        } else {
                            None
                        }
                    });
                match sel {
                    Some(sel) => {
                        self.buf.push_str(&format!("({} ", sel));
                        self.translate_expr(obj);
                        self.buf.push(')');
                    }
                    None => {
                        self.mark_unsupported(format!(
                            "field access '.{}' on non-datatype receiver", field.name));
                    }
                }
            }
            // AUDIT #3 FIX: unsupported expressions NEVER fabricate a term
            // (the old `false` guaranteed spurious VIOLATED verdicts). The
            // caller checks self.unsupported and skips the whole obligation,
            // surfacing UNKNOWN with the reason.
            other => {
                self.mark_unsupported(display_kind(other));
            }
        }
    }

    fn emit(&mut self, line: &str) {
        self.buf.push_str(line);
        self.buf.push('\n');
    }
}

impl Default for SMTGenerator {
    fn default() -> Self { Self::new() }
}

fn call_callee_name(func: &Expr) -> Option<String> {
    match func {
        Expr::Ident(id) => Some(id.name.clone()),
        _ => None,
    }
}

/// Short human kind tag for unsupported expressions.
fn display_kind(e: &Expr) -> String {
    let k = match e {
        Expr::Index(..) => "array indexing",
        Expr::Match(..) => "match expression",
        Expr::BlockExpr(..) => "block expression",
        Expr::ConstBlock(..) => "const block",
        Expr::Closure(..) | Expr::PipeClosure(..) => "closure",
        Expr::Some(..) | Expr::None(_) | Expr::Ok(..) | Expr::Err(..) => "enum constructor",
        Expr::Struct(..) => "struct literal",
        Expr::Array(..) => "array literal",
        Expr::Tuple(..) => "tuple",
        Expr::As(..) => "cast expression",
        _ => "expression",
    };
    format!("unsupported {} in contract", k)
}

/// SMT-LIB Real decimal literal (no exponent form allowed).
fn real_literal(f: f64) -> String {
    if f.is_nan() || f.is_infinite() {
        // No NaN/inf in SMT Reals -- caller obligations containing these
        // become unsupported upstream via sort mismatch; emit a benign
        // distinct numeral to keep the buffer well-formed.
        return "0.0".to_string();
    }
    for prec in 1..=17 {
        let s = format!("{:.*}", prec, f);
        if !s.contains('e') && !s.contains('E') {
            if let Ok(back) = s.parse::<f64>() {
                if back == f {
                    return s;
                }
            }
        }
    }
    // Fallback: fixed 6 places (tiny drift acceptable only in pathological
    // literals; contracts rarely carry them).
    format!("{:.6}", f)
}

// =========================================================================
// SMT-LIB Identifier Escaping
// =========================================================================

fn smt_escape(name: &str) -> String {
    if name.is_empty() {
        return "||".to_string();
    }
    let needs_escape = name.contains(|c: char| {
        !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '!' && c != '?' && c != '.'
    }) || name.starts_with(|c: char| c.is_ascii_digit());

    if needs_escape {
        format!("|{}|", name)
    } else {
        name.to_string()
    }
}

// =========================================================================
// z3 Subprocess Runner -- TEMP-FILE protocol (audit #14 fix)
// =========================================================================

pub struct Z3Runner {
    pub z3_path: String,
    pub timeout_ms: u64,
}

impl Z3Runner {
    pub fn new() -> Self {
        Self {
            z3_path: Self::find_z3().unwrap_or_else(|| "z3".to_string()),
            timeout_ms: 5000,
        }
    }

    /// Auto-detect z3 binary: env override, bundled, common installs, PATH.
    ///
    /// CRB-3b: the bundled check is PLATFORM-AWARE (`z3.exe` on Windows,
    /// `z3` elsewhere) and also probes `<exe_dir>/../bin/`, so the release
    /// archives' `bin/z3` (Linux/macOS) and `bin/z3.exe` (Windows) are found
    /// next to the compiler. Z3_PATH and PATH remain as fallbacks.
    pub fn find_z3() -> Option<String> {
        if let Ok(path) = std::env::var("Z3_PATH") {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                for candidate in Self::bundled_candidates(dir) {
                    if candidate.exists() {
                        return Some(candidate.to_string_lossy().to_string());
                    }
                }
            }
        }
        let candidates: &[&str] = if cfg!(windows) {
            &[r"C:\Program Files\z3\bin\z3.exe", r"C:\z3\bin\z3.exe"]
        } else {
            &[
                "/usr/bin/z3",
                "/usr/local/bin/z3",
                "/opt/homebrew/bin/z3",
                "/opt/local/bin/z3",
            ]
        };
        for c in candidates {
            if std::path::Path::new(c).exists() {
                return Some((*c).to_string());
            }
        }
        if std::process::Command::new(Self::Z3_BINARY)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_or(false, |s| s.success())
        {
            return Some(Self::Z3_BINARY.to_string());
        }
        None
    }

    /// Platform-correct bundled binary name.
    pub const Z3_BINARY: &'static str = if cfg!(windows) { "z3.exe" } else { "z3" };

    /// Candidate bundled locations relative to the running executable:
    /// a sibling (`<archive>/bin/z3` next to `xiom`) and, for dev builds in
    /// `target/debug|release`, the workspace-side `../bin` layout.
    pub fn bundled_candidates(exe_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut out = vec![exe_dir.join(Self::Z3_BINARY)];
        if let Some(parent) = exe_dir.parent() {
            out.push(parent.join("bin").join(Self::Z3_BINARY));
        }
        out
    }

    pub fn with_z3_path(mut self, path: &str) -> Self {
        self.z3_path = path.to_string();
        self
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms.max(100);
        self
    }

    /// Run z3 over SMT-LIB2 text.
    ///
    /// AUDIT #14 FIX: the old implementation wrote the ENTIRE query to
    /// z3's stdin pipe and only THEN called wait_with_output() -- if the
    /// query exceeded the OS pipe buffer, both processes blocked forever.
    /// z3 now reads a UNIQUE TEMP FILE (randomized name; deleted afterwards),
    /// so there is no stdin pipe to deadlock on. The parent also enforces
    /// its own deadline with try_wait + kill (the -T flag takes SECONDS;
    /// the old code passed milliseconds into it).
    pub fn verify(&self, smt: &str) -> Vec<VerifyResult> {
        // Unique temp file: pid + monotonic nanos (no predictable shared name).
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir()
            .join(format!("xiom_verify_{}_{}.smt2", std::process::id(), nanos));

        if let Err(e) = std::fs::write(&path, smt.as_bytes()) {
            return vec![VerifyResult::Error {
                message: format!("failed to write z3 input file {}: {}", path.display(), e),
            }];
        }

        let secs = (self.timeout_ms / 1000).max(1);
        let spawned = Command::new(&self.z3_path)
            .arg(format!("-T:{}", secs))
            .arg(&path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let mut child = match spawned {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_file(&path);
                return vec![VerifyResult::Error {
                    message: format!("Failed to launch z3: {}. Install z3 and ensure it is on PATH.", e),
                }];
            }
        };

        // Parent-side deadline: budget + 50% grace, poll every 25ms, then kill.
        let hard_deadline = std::time::Instant::now()
            + std::time::Duration::from_millis(self.timeout_ms + self.timeout_ms / 2);
        let status = loop {
            match child.try_wait() {
                Ok(Some(st)) => break Some(st),
                Ok(None) => {
                    if std::time::Instant::now() >= hard_deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        break None;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
                Err(_) => break None,
            }
        };

        let output = child.wait_with_output().unwrap_or_else(|e| {
            std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: vec![],
                stderr: format!("z3 wait error: {}", e).into_bytes(),
            }
        });
        let _ = std::fs::remove_file(&path);

        if status.is_none() {
            return vec![VerifyResult::Inconclusive {
                reason: format!("z3 killed after {}ms wall-clock budget", self.timeout_ms),
            }];
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stderr.is_empty() {
            let benign = stderr.contains("model is not available")
                || stderr.trim().starts_with("WARNING");
            let is_error = stderr.trim().starts_with("(error") || stderr.trim().starts_with("error");
            if !benign && is_error {
                return vec![VerifyResult::Error {
                    message: format!("z3 error: {}", stderr.trim()),
                }];
            }
        }

        self.parse_z3_output(&stdout)
    }

    /// Parse z3 output: extract sat/unsat/unknown per check-sat and models.
    pub fn parse_z3_output(&self, output: &str) -> Vec<VerifyResult> {
        let mut results = Vec::new();
        let mut current_model = String::new();
        let mut in_model = false;

        for line in output.lines() {
            let trimmed = line.trim();

            if trimmed == "sat" {
                in_model = false;
                current_model.clear();
            } else if trimmed == "unsat" {
                in_model = false;
                results.push(VerifyResult::Proven);
            } else if trimmed == "unknown" {
                in_model = false;
                results.push(VerifyResult::Inconclusive {
                    reason: "z3 returned unknown (timeout or incomplete theory)".to_string(),
                });
            } else if trimmed.starts_with("(model") || in_model {
                in_model = true;
                current_model.push_str(trimmed);
                current_model.push('\n');
                if trimmed == ")" && in_model {
                    in_model = false;
                    let ce = self.parse_model(&current_model);
                    results.push(VerifyResult::Violated {
                        code: X7001_ENSURES_VIOLATION,
                        message: "ensures clause violated".to_string(),
                        span: None,
                        counterexample: Some(ce),
                    });
                    current_model.clear();
                }
            } else if trimmed == "(" && !in_model {
                // z3 4.13.4 (get-model) outputs raw "(\n  (define-fun ..." without "(model"
                in_model = true;
                current_model.push_str(trimmed);
                current_model.push('\n');
            } else if trimmed.starts_with("(error") {
                if !trimmed.contains("model is not available") {
                    results.push(VerifyResult::Error {
                        message: trimmed.to_string(),
                    });
                }
            }
        }

        if results.is_empty() {
            results.push(VerifyResult::Error {
                message: format!("Could not parse z3 output:\n{}", output),
            });
        }

        results
    }

    /// Parse (model ...) s-expr into Counterexample.
    fn parse_model(&self, model_text: &str) -> Counterexample {
        let mut values = HashMap::new();

        for line in model_text.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("(define-fun ") {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() >= 1 {
                    let name = parts[0].trim_matches('|').to_string();
                    let after_sort = rest
                        .split(')')
                        .nth(1)
                        .unwrap_or("")
                        .trim();
                    if !after_sort.is_empty() {
                        let value = after_sort
                            .split(')')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !value.is_empty() {
                            values.insert(name, value);
                        }
                    }
                }
            }
        }

        Counterexample {
            values,
            raw_model: model_text.to_string(),
        }
    }
}

impl Default for Z3Runner {
    fn default() -> Self { Self::new() }
}

/// AI-06: Parse Z3 model text and extract counterexample values.
pub fn parse_z3_model(model_text: &str) -> Vec<(String, String)> {
    let mut values = Vec::new();
    for line in model_text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("(define-fun ") {
            let name_end = rest.find(|c: char| c.is_whitespace() || c == '(').unwrap_or(rest.len());
            let name = rest[..name_end].trim_matches('|').to_string();
            let after_sig = &rest[name_end..];
            if let Some(close_paren) = after_sig.find(')') {
                let after_sort = after_sig[close_paren + 1..].trim();
                let parts: Vec<&str> = after_sort.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let value = parts[1].trim_end_matches(')').trim().to_string();
                    if !value.is_empty() && !name.is_empty() {
                        values.push((name, value));
                    }
                }
            }
        }
    }
    values
}

// =========================================================================
// Helpers
pub fn expr_display(expr: &Expr) -> String {
    match expr {
        Expr::Ident(id) => id.name.clone(),
        Expr::Int(n, _) => n.to_string(),
        Expr::Float(f, _) => f.to_string(),
        Expr::Bool(b, _) => if *b { "true".to_string() } else { "false".to_string() },
        Expr::Binary(left, op, right, _) => {
            let op_str = match op {
                BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*", BinOp::Div => "/",
                BinOp::Eq => "==", BinOp::Neq => "!=", BinOp::Lt => "<", BinOp::Gt => ">",
                BinOp::Le => "<=", BinOp::Ge => ">=", BinOp::And => "&&", BinOp::Or => "||",
                _ => "?",
            };
            format!("{} {} {}", expr_display(left), op_str, expr_display(right))
        }
        Expr::Unary(op, inner, _) => match op {
            UnaryOp::Not => format!("!{}", expr_display(inner)),
            UnaryOp::Neg => format!("-{}", expr_display(inner)),
            _ => "?".to_string(),
        },
        _ => "?".to_string(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// CRB-3b: the bundled-z3 lookup must be platform-aware -- a sibling
    /// `z3.exe` on Windows, `z3` elsewhere, plus the `../bin` layout.
    #[test]
    fn bundled_candidates_are_platform_aware() {
        let dir = if cfg!(windows) {
            std::path::Path::new(r"C:\xiom\bin")
        } else {
            std::path::Path::new("/opt/xiom/bin")
        };
        let expected = if cfg!(windows) { "z3.exe" } else { "z3" };
        assert_eq!(Z3Runner::Z3_BINARY, expected);
        let candidates = Z3Runner::bundled_candidates(dir);
        assert_eq!(candidates[0], dir.join(expected));
        assert_eq!(candidates[1].file_name().unwrap().to_string_lossy(), expected);
        assert!(candidates[1].to_string_lossy().contains("bin"));
    }

    fn parse_program(src: &str) -> Program {
        let mut parser = xiom_parser::Parser::new(xiom_lexer::Lexer::new(src).tokenize());
        parser
            .parse_program()
            .unwrap_or_else(|e| panic!("test fixture parse: {e:?}"))
    }

    fn generate(src: &str) -> (String, GenReport) {
        let program = parse_program(src);
        let mut generator = SMTGenerator::new();
        generator.generate_with_report(&program)
    }

    /// Parse the generated SMT with z3 when available. `None` = z3 missing
    /// (test skips); `Some(false)` = z3 rejected the script.
    fn z3_parses(smt: &str) -> Option<bool> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "xiom_verify_emit_test_{}_{}.smt2",
            std::process::id(),
            nanos
        ));
        std::fs::write(&path, smt).ok()?;
        let out = std::process::Command::new("z3")
            .arg("-smt2")
            .arg(&path)
            .output()
            .ok()?;
        let _ = std::fs::remove_file(&path);
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        Some(!combined.to_lowercase().contains("error"))
    }

    #[test]
    fn method_self_selectors_and_labels_are_valid_smt() {
        // Method `self`, datatype selectors, and repeated clauses must all
        // produce a script z3 accepts (2026-10-04 emission fixes).
        let (smt, report) = generate(
            r#"
module verify_emission

pub type Positive = {
  current: Int;
  invariant: current >= 0;
}

pub fn Positive.bump(self, by: Int) -> Positive
requires: by >= 0
ensures: result.current >= 0
ensures: result.current >= self.current
{
  return Positive { current: self.current + by };
}
"#,
        );
        assert!(smt.contains("(declare-const self"), "receiver declared:\n{smt}");
        assert!(smt.contains("(Positive-current"), "qualified selector:\n{smt}");
        // Every `:named` label must be unique.
        let mut seen = std::collections::HashSet::new();
        for token in smt.split(":named |").skip(1) {
            let label = token.split('|').next().unwrap_or_default().to_string();
            assert!(seen.insert(label.clone()), "duplicate :named label {label}");
        }
        // Bare invariant fields are not modeled yet -- they must surface as
        // X7006 UNKNOWN, never as emitted undeclared symbols.
        assert!(
            report.skipped.iter().all(|s| s.code == X7006_INVARIANT || s.code == X7007_UNKNOWN),
            "unexpected skips: {:?}",
            report.skipped
        );
        if let Some(ok) = z3_parses(&smt) {
            assert!(ok, "z3 rejected the generated SMT:\n{smt}");
        }
    }

    #[test]
    fn unknown_identifiers_and_generic_comparisons_skip_not_poison() {
        // A generic comparison (`T` has no modeled order) and a bare unknown
        // identifier must become X7007/X7006 skips -- never undeclared
        // symbols that make z3 reject the whole script.
        let (smt, report) = generate(
            r#"
module verify_emission_generic

pub type Pair[T] = {
  lo: T;
  hi: T;
  invariant: lo <= hi;
}

pub fn maybe(value: Int) -> Int
requires: value >= 0
ensures: result == phantom_value
{
  return value;
}
"#,
        );
        assert!(
            !smt.contains("(<= lo hi)"),
            "unmodeled generic field must not be emitted:\n{smt}"
        );
        let emitted_phantom = smt.lines().any(|l| {
            !l.trim_start().starts_with(';') && l.contains("phantom_value")
        });
        assert!(
            !emitted_phantom,
            "unknown identifier must not be emitted as a term:\n{smt}"
        );
        assert!(
            report.skipped.iter().any(|s| s.code == X7006_INVARIANT),
            "generic invariant must be reported UNKNOWN: {:?}",
            report.skipped
        );
        if let Some(ok) = z3_parses(&smt) {
            assert!(ok, "z3 rejected the generated SMT:\n{smt}");
        }
    }
}
