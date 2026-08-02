// XIOM Safety Audit — Phase 5d.9 Sandbox Pass
// -----------------------------------------------------------------------
// Walks the AST enumerating every `unsafe` block, categorising operations,
// scoring severity (HIGH/MEDIUM/LOW), and producing a structured safety
// report. Designed for CI/CD gating (--sandbox=strict). Deterministic.
// Zero external dependencies. Zero AI calls. Manual JSON serialisation.

use xiom_ast::*;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct SafetyFinding {
    pub id: u32,
    pub severity: String,
    pub category: String,
    pub line: u32,
    pub column: u32,
    pub function: String,
    pub is_public: bool,
    pub unsafe_block_size: u32,
    pub has_contract: bool,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone)]
pub struct SafetySummary {
    pub total_unsafe_blocks: u32,
    pub high_severity: u32,
    pub medium_severity: u32,
    pub low_severity: u32,
    pub public_unsafe_functions: u32,
    pub safety_score: String,
    pub score_value: u32,
}

#[derive(Debug, Clone)]
pub struct SafetyReport {
    pub schema_version: u32,
    pub compiler_version: String,
    pub file: String,
    pub summary: SafetySummary,
    pub findings: Vec<SafetyFinding>,
    pub ai_enhanced: bool,
}

impl SafetyReport {
    pub fn new(file: &str) -> Self {
        SafetyReport {
            schema_version: 1,
            compiler_version: "0.47.8".to_string(),
            file: file.to_string(),
            summary: SafetySummary {
                total_unsafe_blocks: 0,
                high_severity: 0, medium_severity: 0, low_severity: 0,
                public_unsafe_functions: 0,
                safety_score: "SAFE".into(),
                score_value: 0,
            },
            findings: Vec::new(),
            ai_enhanced: false,
        }
    }

    pub fn to_json(&self) -> String {
        let mut findings_json = String::new();
        for f in &self.findings {
            if !findings_json.is_empty() { findings_json.push(','); }
            findings_json.push_str(&format!(
                r#"{{"id":{},"severity":"{}","category":"{}","line":{},"column":{},"function":"{}","is_public":{},"unsafe_block_size":{},"has_contract":{},"description":"{}","suggestion":"{}"}}"#,
                f.id, f.severity, f.category, f.line, f.column, f.function, f.is_public, f.unsafe_block_size, f.has_contract, f.description, f.suggestion
            ));
        }
        format!(
            r#"{{"schema_version":{},"compiler_version":"{}","file":"{}","summary":{{"total_unsafe_blocks":{},"high_severity":{},"medium_severity":{},"low_severity":{},"public_unsafe_functions":{},"safety_score":"{}","score_value":{}}},"findings":[{}],"ai_enhanced":{}}}"#,
            self.schema_version, self.compiler_version, self.file,
            self.summary.total_unsafe_blocks, self.summary.high_severity, self.summary.medium_severity, self.summary.low_severity,
            self.summary.public_unsafe_functions, self.summary.safety_score, self.summary.score_value,
            findings_json, self.ai_enhanced
        )
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("═══ XIOM Safety Audit: {} ═══\n\n", self.file));
        out.push_str(&format!("  {} unsafe blocks  |  {} HIGH  |  {} MEDIUM  |  {} LOW  |  Score: {}\n\n",
            self.summary.total_unsafe_blocks, self.summary.high_severity, self.summary.medium_severity, self.summary.low_severity, self.summary.safety_score));
        for severity in &["HIGH", "MEDIUM", "LOW"] {
            let findings: Vec<&SafetyFinding> = self.findings.iter().filter(|f| &f.severity == severity).collect();
            if findings.is_empty() { continue; }
            out.push_str(&format!("━━━ {} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n", severity));
            for f in &findings {
                let pub_marker = if f.is_public { " (pub)" } else { "" };
                out.push_str(&format!("[{}] line {}: {}\n  fn {}{}\n  {}\n  → {}\n\n",
                    f.severity, f.line, f.category, f.function, pub_marker, f.description, f.suggestion));
            }
        }
        out.push_str("═══ End of report ═══\n");
        out
    }
}

// ============================================================================
// Audit Pass
// ============================================================================

pub struct SafetyAuditor {
    next_id: u32,
    current_function: String,
    current_is_pub: bool,
    current_has_contract: bool,
}

impl SafetyAuditor {
    pub fn new() -> Self {
        SafetyAuditor { next_id: 1, current_function: String::new(), current_is_pub: false, current_has_contract: false }
    }

    pub fn audit(&mut self, program: &Program, file: &str) -> SafetyReport {
        let mut report = SafetyReport::new(file);
        for item in &program.items {
            self.audit_top_decl(item, &mut report);
        }
        report.summary.total_unsafe_blocks = report.findings.len() as u32;
        report.summary.high_severity = report.findings.iter().filter(|f| f.severity == "HIGH").count() as u32;
        report.summary.medium_severity = report.findings.iter().filter(|f| f.severity == "MEDIUM").count() as u32;
        report.summary.low_severity = report.findings.iter().filter(|f| f.severity == "LOW").count() as u32;
        report.summary.public_unsafe_functions = report.findings.iter().filter(|f| f.is_public).count() as u32;

        let mut score: u32 = 0;
        for f in &report.findings {
            let weight = match f.severity.as_str() { "HIGH" => 5, "MEDIUM" => 2, _ => 1 };
            score += weight * (1 + f.unsafe_block_size / 10);
        }
        score += 5 * report.summary.public_unsafe_functions;
        if !report.findings.is_empty() { score += 3; }
        report.summary.score_value = score;
        report.summary.safety_score = if score == 0 { "SAFE" }
            else if score <= 10 { "LOW" } else if score <= 30 { "MEDIUM" }
            else if score <= 60 { "HIGH" } else { "CRITICAL" }.to_string();
        report
    }

    fn audit_top_decl(&mut self, decl: &TopDecl, report: &mut SafetyReport) {
        match decl {
            TopDecl::Fn(fd) => {
                self.current_function = fd.name.name.clone();
                self.current_is_pub = fd.is_pub;
                self.current_has_contract = !fd.contracts.is_empty();
                if let Some(body) = &fd.body {
                    self.audit_block(body, report);
                }
            }
            TopDecl::Module(m) => {
                for item in &m.items { self.audit_top_decl(item, report); }
            }
            _ => {}
        }
    }

    fn audit_match_body(&mut self, body: &MatchBody, report: &mut SafetyReport) {
        match body {
            MatchBody::Block(b) => self.audit_block(b, report),
            MatchBody::Expr(e) => self.audit_expr(e, report),
        }
    }

    fn audit_block(&mut self, block: &Block, report: &mut SafetyReport) {
        for soe in &block.stmts {
            match soe {
                StmtOrExpr::Stmt(s) => self.audit_stmt(s, report),
                StmtOrExpr::Expr(e) => self.audit_expr(e, report),
            }
        }
    }

    fn audit_stmt(&mut self, stmt: &Stmt, report: &mut SafetyReport) {
        match stmt {
            Stmt::Expr(expr, _) | Stmt::Return(Some(expr), _) => self.audit_expr(expr, report),
            Stmt::Let(_, _, value, _) | Stmt::Var(_, _, value, _) => self.audit_expr(value, report),
            Stmt::Assign(_, value, _) => self.audit_expr(value, report),
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                self.audit_expr(cond, report);
                self.audit_block(then_b, report);
                for (c, b) in elifs { self.audit_expr(c, report); self.audit_block(b, report); }
                if let Some(b) = else_b { self.audit_block(b, report); }
            }
            Stmt::While(cond, body, _, _) => { self.audit_expr(cond, report); self.audit_block(body, report); }
            Stmt::For(_, iter, body, _) => { self.audit_expr(iter, report); self.audit_block(body, report); }
            Stmt::Match(scrut, arms, _) => {
                self.audit_expr(scrut, report);
                for arm in arms { self.audit_match_body(&arm.body, report); }
            }
            Stmt::Spawn(body, _) => self.audit_block(body, report),
            _ => {}
        }
    }

    fn audit_expr(&mut self, expr: &Expr, report: &mut SafetyReport) {
        match expr {
            Expr::Unsafe(block, span) => {
                let block_size = block.stmts.len() as u32;
                let categories = self.categorise_unsafe(block);
                for cat in &categories {
                    let severity = self.severity_for(cat);
                    let (desc, sugg) = self.describe(cat);
                    report.findings.push(SafetyFinding {
                        id: self.next_id, severity, category: cat.clone(),
                        line: span.line, column: span.col,
                        function: self.current_function.clone(),
                        is_public: self.current_is_pub,
                        unsafe_block_size: block_size,
                        has_contract: self.current_has_contract,
                        description: desc, suggestion: sugg,
                    });
                    self.next_id += 1;
                }
                self.audit_block(block, report);
            }
            Expr::Binary(left, _, right, _) => { self.audit_expr(left, report); self.audit_expr(right, report); }
            Expr::Unary(_, inner, _) => self.audit_expr(inner, report),
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => { self.audit_expr(func, report); for a in args { self.audit_expr(a, report); } }
            Expr::Field(obj, _, _) | Expr::Index(obj, _, _) => self.audit_expr(obj, report),
            Expr::As(inner, _, _) | Expr::Ref(inner, _) | Expr::MutRef(inner, _) => self.audit_expr(inner, report),
            Expr::Struct(_, fields, _, _) => { for (_, v) in fields { self.audit_expr(v, report); } }
            Expr::Array(elems, _) => { for e in elems { self.audit_expr(e, report); } }
            Expr::If(cond, then_b, elifs, else_b, _) => {
                self.audit_expr(cond, report); self.audit_block(then_b, report);
                for (c, b) in elifs { self.audit_expr(c, report); self.audit_block(b, report); }
                if let Some(b) = else_b { self.audit_block(b, report); }
            }
            Expr::Match(scrut, arms, _) => { self.audit_expr(scrut, report); for arm in arms { self.audit_match_body(&arm.body, report); } }
            Expr::Try(inner, _) | Expr::Paren(inner, _) => self.audit_expr(inner, report),
            _ => {}
        }
    }

    fn categorise_unsafe(&self, block: &Block) -> Vec<String> {
        let mut cats = Vec::new();
        let stmt_count = block.stmts.len() as u32;
        let mut has_extern_call = false;
        let mut has_ptr_arith = false;
        let mut has_deref = false;
        let mut has_type_cast = false;

        for soe in &block.stmts {
            match soe {
                StmtOrExpr::Stmt(s) => self.scan_stmt_for_unsafe(s, &mut has_extern_call, &mut has_ptr_arith, &mut has_deref, &mut has_type_cast),
                StmtOrExpr::Expr(e) => self.scan_expr_for_unsafe(e, &mut has_extern_call, &mut has_ptr_arith, &mut has_deref, &mut has_type_cast),
            }
        }

        if has_extern_call && !self.current_has_contract { cats.push("extern_c_call_without_contract".into()); }
        if has_extern_call { cats.push("extern_c_call_unchecked_return".into()); }
        if has_ptr_arith { cats.push("raw_pointer_arithmetic".into()); }
        if has_deref { cats.push("raw_pointer_deref".into()); }
        if has_type_cast { cats.push("type_punning".into()); }
        if stmt_count > 10 { cats.push("large_unsafe_block".into()); }
        if self.current_is_pub && !cats.is_empty() { cats.push("unsafe_in_public_api".into()); }
        if cats.is_empty() { cats.push("unsafe_block_without_comment".into()); }
        cats
    }

    fn scan_stmt_for_unsafe(&self, stmt: &Stmt, ec: &mut bool, pa: &mut bool, dr: &mut bool, tc: &mut bool) {
        match stmt {
            Stmt::Expr(expr, _) | Stmt::Return(Some(expr), _) => self.scan_expr_for_unsafe(expr, ec, pa, dr, tc),
            Stmt::Let(_, _, value, _) | Stmt::Var(_, _, value, _) => self.scan_expr_for_unsafe(value, ec, pa, dr, tc),
            Stmt::Assign(_, value, _) => self.scan_expr_for_unsafe(value, ec, pa, dr, tc),
            _ => {}
        }
    }

    fn scan_expr_for_unsafe(&self, expr: &Expr, ec: &mut bool, pa: &mut bool, dr: &mut bool, tc: &mut bool) {
        match expr {
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
                if let Expr::Ident(id) = func.as_ref() {
                    if id.name.starts_with("xvk_") || id.name.starts_with("glfw") || id.name.starts_with("vk") { *ec = true; }
                }
                if let Expr::Field(obj, method, _) = func.as_ref() {
                    if let Expr::Ident(id) = obj.as_ref() {
                        if id.name.starts_with("xvk_") || method.name.starts_with("xvk_") { *ec = true; }
                    }
                }
            }
            Expr::Binary(left, op, right, _) => {
                if matches!(op, BinOp::Add | BinOp::Sub) { *pa = true; }
                self.scan_expr_for_unsafe(left, ec, pa, dr, tc);
                self.scan_expr_for_unsafe(right, ec, pa, dr, tc);
            }
            Expr::Unary(UnaryOp::Deref, inner, _) => { *dr = true; self.scan_expr_for_unsafe(inner, ec, pa, dr, tc); }
            Expr::Unary(_, inner, _) => self.scan_expr_for_unsafe(inner, ec, pa, dr, tc),
            Expr::As(inner, _, _) => { *tc = true; self.scan_expr_for_unsafe(inner, ec, pa, dr, tc); }
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => self.scan_expr_for_unsafe(inner, ec, pa, dr, tc),
            Expr::Unsafe(block, _) => {
                for soe in &block.stmts {
                    match soe {
                        StmtOrExpr::Stmt(s) => self.scan_stmt_for_unsafe(s, ec, pa, dr, tc),
                        StmtOrExpr::Expr(e) => self.scan_expr_for_unsafe(e, ec, pa, dr, tc),
                    }
                }
            }
            _ => {}
        }
    }

    fn severity_for(&self, category: &str) -> String {
        match category {
            "extern_c_call_without_contract" | "raw_pointer_deref" | "raw_pointer_arithmetic"
            | "null_pointer_deref_risk" | "type_punning" => "HIGH".into(),
            "extern_c_call_unchecked_return" | "large_unsafe_block" | "unsafe_in_public_api"
            | "unchecked_array_index" => "MEDIUM".into(),
            _ => "LOW".into(),
        }
    }

    fn describe(&self, category: &str) -> (String, String) {
        match category {
            "extern_c_call_without_contract" => (
                format!("extern C call in fn {} has no requires/ensures contract. Returned pointer may be used without null check.", self.current_function),
                "Add `requires: size > 0; ensures: result != null;` to the containing function, or wrap the call in a null-check guard.".into(),
            ),
            "extern_c_call_unchecked_return" => (
                "extern C return value used directly without guard check.".into(),
                "Store the return value and validate it before use (e.g. null check for pointers, error code check for handles).".into(),
            ),
            "raw_pointer_deref" => (
                "Raw pointer dereference inside unsafe block without bounds or null guard.".into(),
                "Wrap the dereference in a null check or bounds check before accessing.".into(),
            ),
            "raw_pointer_arithmetic" => (
                "Unsafe pointer arithmetic without bounds check. May cause out-of-bounds access.".into(),
                "Add bounds verification before the arithmetic: check offset against allocation size.".into(),
            ),
            "type_punning" => (
                "Pointer type cast followed by dereference of different type. May violate strict aliasing.".into(),
                "Use explicit memory reinterpretation with documented layout assumptions, or use a union/safe wrapper.".into(),
            ),
            "large_unsafe_block" => (
                "Large unsafe block (>10 statements) — high risk surface area.".into(),
                "Split into smaller unsafe blocks, each scoped to the minimum required operation.".into(),
            ),
            "unsafe_in_public_api" => (
                format!("Unsafe block in public function `{}` — exposed to external callers.", self.current_function),
                "Document the safety invariants in a `// SAFETY:` comment. Consider making the function non-public or adding contracts.".into(),
            ),
            _ => (
                "Unsafe block without a preceding `// SAFETY:` comment explaining why it's safe.".into(),
                "Document why each unsafe operation is sound: invariants it relies on, why those hold, and who maintains them.".into(),
            ),
        }
    }
}

impl Default for SafetyAuditor {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use xiom_lexer::Lexer;
    use xiom_parser::Parser;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new(source).tokenize();
        Parser::new(tokens).parse_program().unwrap()
    }

    fn audit(source: &str) -> SafetyReport {
        let program = parse(source);
        let mut auditor = SafetyAuditor::new();
        auditor.audit(&program, "test.xi")
    }

    #[test]
    fn test_no_unsafe_no_findings() {
        let report = audit("fn main() -> Int { return 0; }");
        assert_eq!(report.findings.len(), 0);
        assert_eq!(report.summary.safety_score, "SAFE");
    }

    #[test]
    fn test_single_unsafe_block_found() {
        let src = "fn main() -> Int { unsafe { let x = 1; }; return 0; }";
        let report = audit(src);
        assert!(report.findings.len() >= 1, "Should find at least 1 finding");
    }

    #[test]
    fn test_extern_call_without_contract_is_high() {
        let src = "fn read_buffer(ptr: *UInt8, len: Int) -> Int { var result: Int = 0; unsafe { xvk_read(ptr, len); }; return result; }";
        let report = audit(src);
        let has_high = report.findings.iter().any(|f| f.severity == "HIGH");
        assert!(has_high, "extern call without contract should be HIGH");
    }

    #[test]
    fn test_public_unsafe_fn_is_flagged() {
        let src = "pub fn public_unsafe_fn() -> Int { var x = 0; unsafe { let y = 1; x = y; }; return x; }";
        let report = audit(src);
        let pub_finding = report.findings.iter().any(|f| f.is_public);
        assert!(pub_finding, "should mark public function");
    }

    #[test]
    fn test_multiple_unsafe_blocks_counted() {
        let src = "fn many_unsafe() -> Int { unsafe { let a = 1; }; unsafe { let b = 2; }; unsafe { let c = 3; }; return 0; }";
        let report = audit(src);
        assert_eq!(report.summary.total_unsafe_blocks, 3);
    }

    #[test]
    fn test_contract_present_not_flagged_extern() {
        let src = "fn safe_extern(ptr: *UInt8, len: Int) -> Int requires: ptr != null; ensures: result >= 0; { unsafe { return xvk_safe_call(ptr, len); } }";
        let report = audit(src);
        let no_contract_high = report.findings.iter().any(|f| f.category == "extern_c_call_without_contract");
        assert!(!no_contract_high, "should not flag when contract present");
    }

    #[test]
    fn test_safety_score_computation() {
        let src = "fn main() -> Int { unsafe { xvk_do_stuff(0); }; return 0; }";
        let report = audit(src);
        assert!(report.summary.score_value > 0, "should have positive score");
        assert_ne!(report.summary.safety_score, "SAFE");
    }

    #[test]
    fn test_json_output_valid() {
        let src = "fn main() -> Int { unsafe { let x = 1; }; return 0; }";
        let report = audit(src);
        let json = report.to_json();
        assert!(json.contains("schema_version"));
        assert!(json.contains("findings"));
        assert!(json.contains("summary"));
    }

    #[test]
    fn test_text_output_valid() {
        let report = audit("fn main() -> Int { unsafe { let x = 1; }; return 0; }");
        let text = report.to_text();
        assert!(text.contains("Safety Audit"));
        assert!(text.contains("End of report"));
    }

    #[test]
    fn test_severity_categories_match() {
        let auditor = SafetyAuditor::new();
        assert_eq!(auditor.severity_for("extern_c_call_without_contract"), "HIGH");
        assert_eq!(auditor.severity_for("extern_c_call_unchecked_return"), "MEDIUM");
        assert_eq!(auditor.severity_for("unsafe_block_without_comment"), "LOW");
    }
}
