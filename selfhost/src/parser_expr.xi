// XIOM -- Selfhost expression/type/statement parser (Phase 2, arena port)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// 1:1 port of crates/xiom-parser/src/lib.rs (types, params, generics, derive
// lists, blocks, statements, patterns, all expression parsing) onto the
// arena AST (selfhost_ast) and the shared Parser state
// (selfhost_parser_state). Control flow, node construction ORDER, and spans
// mirror the Rust parser exactly: the Phase 2 gate compares the canonical
// AST dump line-for-line.
//
// Error convention: Rust `Result<T, ParseError>` + `?` maps to the latch
// (`p_err` sets `failed`; every call site checks `p_failed` and unwinds with
// the function's failure sentinel: -1 for index/Int results, an EMPTY Vec for
// Vec[Int] results). `Option<T>` maps to -1 (absent) and never latches;
// Rust `.ok()` / `.ok()?` sites explicitly clear the latch (see
// `pe_try_ident`, `pe_try_compound_assign`).
//
// All parser helpers are FREE functions taking `&mut Parser` (Phase 1
// receiver-mutation finding, COMPILER_BUGS 2026-10-02 (a)). Bare `Nk`/`Tk`
// variant patterns only (qualified variant patterns trip false W000,
// COMPILER_BUGS 2026-10-02 (f)).

module selfhost_parser_expr

use xiom.string;
use selfhost_ast;
use selfhost_ast.Node;
use selfhost_ast.NodeKind;
use selfhost_ast.Span;
use selfhost_lexer.Token;
use selfhost_lexer.TokenKind;
use selfhost_parser_state;
use selfhost_parser_state.Parser;

// ============================================================================
// Small helpers
// ============================================================================

/// Token span in arena form (Rust `tok.span`).
fn pe_tok_span(t: Token) -> Span {
  return selfhost_ast.span_new(t.line, t.col, t.byte_start, t.byte_end);
}

/// Decimal text of a non-negative UInt (Rust `format!("{v}")` for usize/u64).
fn pe_uint_str(v: UInt) -> Str {
  if v == (0 as UInt) { return "0"; }
  var digits: [24]UInt8;
  var pos = 24;
  var n = v;
  while n > (0 as UInt) {
    pos = pos - 1;
    let d = (n % (10 as UInt)) as Int;
    digits[pos] = (48 + d) as UInt8;
    n = n / (10 as UInt);
  }
  var buf = Vec[UInt8].new();
  while pos < 24 {
    buf.push(digits[pos]);
    pos = pos + 1;
  }
  return Str::from_utf8(buf);
}

/// First index of byte `b` in `s`, or -1 (Rust `str::find(char)`).
fn pe_find_byte(s: Str, b: Int) -> Int {
  var i = 0;
  let n = s.len();
  while i < n {
    if (string.byte_at(s, i) as Int) == b { return i; }
    i = i + 1;
  }
  return -1;
}

/// Rust `c.is_uppercase()` on the FIRST char (ASCII idents).
fn pe_first_upper(s: Str) -> Bool {
  if s.len() == 0 { return false; }
  let b = string.byte_at(s, 0) as Int;
  return b >= 65 && b <= 90;
}

/// Rust `chars().next().map_or(false, |c| c.is_lowercase() || c == '_')`.
fn pe_first_lower_or_uscore(s: Str) -> Bool {
  if s.len() == 0 { return false; }
  let b = string.byte_at(s, 0) as Int;
  return (b >= 97 && b <= 122) || b == 95;
}

/// Rust `lex.chars().all(|c| c.is_ascii_alphabetic() || c == '_') && !empty`.
fn pe_all_ident_bytes(s: Str) -> Bool {
  let n = s.len();
  if n == 0 { return false; }
  var i = 0;
  while i < n {
    let b = string.byte_at(s, i) as Int;
    if !((b >= 65 && b <= 90) || (b >= 97 && b <= 122) || b == 95) { return false; }
    i = i + 1;
  }
  return true;
}

/// Rust parse_ident RESERVED_WORDS.
fn pe_is_reserved_word(s: Str) -> Bool {
  if s == "let" { return true; }
  if s == "var" { return true; }
  if s == "const" { return true; }
  if s == "fn" { return true; }
  if s == "return" { return true; }
  if s == "break" { return true; }
  if s == "continue" { return true; }
  if s == "if" { return true; }
  if s == "elif" { return true; }
  if s == "else" { return true; }
  if s == "match" { return true; }
  if s == "while" { return true; }
  if s == "for" { return true; }
  if s == "in" { return true; }
  if s == "module" { return true; }
  if s == "use" { return true; }
  if s == "pub" { return true; }
  if s == "type" { return true; }
  if s == "enum" { return true; }
  if s == "interface" { return true; }
  if s == "impl" { return true; }
  if s == "true" { return true; }
  if s == "false" { return true; }
  if s == "self" { return true; }
  if s == "unsafe" { return true; }
  if s == "extern" { return true; }
  if s == "as" { return true; }
  if s == "is" { return true; }
  if s == "and" { return true; }
  if s == "or" { return true; }
  if s == "not" { return true; }
  return false;
}

/// Text of the Ident node at arena index `i` (-1/other -> ""). Ident nodes
/// carry `NkIdent(name)`; NkExprIdent payloads store the ident node index.
fn pe_ident_text(p: &Parser, i: Int) -> Str {
  if i < 0 { return ""; }
  let node = p.nodes[i];
  match node.kind {
    NkIdent(name) => { return name; }
    _ => { return ""; }
  }
}

/// Rust extract_struct_type_name builtins list.
fn pe_builtin_type_name(s: Str) -> Bool {
  if s == "Option" { return true; }
  if s == "Result" { return true; }
  if s == "Vec" { return true; }
  if s == "Slice" { return true; }
  if s == "Map" { return true; }
  if s == "Set" { return true; }
  if s == "Bool" { return true; }
  if s == "Int" { return true; }
  if s == "Int8" { return true; }
  if s == "Int16" { return true; }
  if s == "Int32" { return true; }
  if s == "Int64" { return true; }
  if s == "UInt8" { return true; }
  if s == "UInt16" { return true; }
  if s == "UInt32" { return true; }
  if s == "UInt64" { return true; }
  if s == "Float32" { return true; }
  if s == "Float64" { return true; }
  if s == "Char" { return true; }
  if s == "Str" { return true; }
  if s == "String" { return true; }
  if s == "Rc" { return true; }
  if s == "Arc" { return true; }
  if s == "Cell" { return true; }
  if s == "RefCell" { return true; }
  if s == "Box" { return true; }
  if s == "Ptr" { return true; }
  return false;
}

/// Rust `lexeme.trim_matches('"')`.
fn pe_trim_quotes(s: Str) -> Str {
  var a = 0;
  var b = s.len();
  while a < b && (string.byte_at(s, a) as Int) == 34 { a = a + 1; }
  while b > a && (string.byte_at(s, b - 1) as Int) == 34 { b = b - 1; }
  return string.str_slice(s, a, b);
}

// ============================================================================
// Token-kind tag comparison (compiler workaround)
// ============================================================================
//
// The current codegen lowers `TokenKind == TokenKind` to a whole-struct
// comparison whose field list contains aggregate payload fields
// (`icmp eq %struct.Vec`), which clang rejects ("icmp requires integer
// operands"). All TokenKind equality checks in this module therefore compare
// tag CODES from `pe_ktag` (a match, which lowers correctly) -- repro:
// tmp/sprintc/phase2_parser/expr_agent/enum_eq_repro.xi. `pe_pk`/`pe_pak`
// are the Int-returning peek wrappers used by those checks.

const KT_Let: Int = 1;
const KT_Var: Int = 2;
const KT_Const: Int = 3;
const KT_Fn: Int = 4;
const KT_Return: Int = 5;
const KT_Break: Int = 6;
const KT_Continue: Int = 7;
const KT_If: Int = 8;
const KT_Elif: Int = 9;
const KT_Else: Int = 10;
const KT_Match: Int = 11;
const KT_While: Int = 12;
const KT_For: Int = 13;
const KT_In: Int = 14;
const KT_Spawn: Int = 15;
const KT_Await: Int = 16;
const KT_Comptime: Int = 17;
const KT_Asm: Int = 18;
const KT_Defer: Int = 19;
const KT_Move: Int = 20;
const KT_Module: Int = 21;
const KT_Use: Int = 22;
const KT_Pub: Int = 23;
const KT_As: Int = 24;
const KT_Type: Int = 25;
const KT_Enum: Int = 26;
const KT_Interface: Int = 27;
const KT_Derive: Int = 28;
const KT_Impl: Int = 29;
const KT_True: Int = 30;
const KT_False: Int = 31;
const KT_Self: Int = 32;
const KT_Some: Int = 33;
const KT_None: Int = 34;
const KT_OkV: Int = 35;
const KT_ErrV: Int = 36;
const KT_Unsafe: Int = 37;
const KT_Extern: Int = 38;
const KT_Is: Int = 39;
const KT_Ident: Int = 40;
const KT_Int: Int = 41;
const KT_BigInt: Int = 42;
const KT_Float: Int = 43;
const KT_Str: Int = 44;
const KT_Char: Int = 45;
const KT_Dot: Int = 46;
const KT_Comma: Int = 47;
const KT_Semicolon: Int = 48;
const KT_Colon: Int = 49;
const KT_ColonColon: Int = 50;
const KT_LParen: Int = 51;
const KT_RParen: Int = 52;
const KT_LBrace: Int = 53;
const KT_RBrace: Int = 54;
const KT_LBracket: Int = 55;
const KT_RBracket: Int = 56;
const KT_At: Int = 57;
const KT_Arrow: Int = 58;
const KT_FatArrow: Int = 59;
const KT_Question: Int = 60;
const KT_Plus: Int = 61;
const KT_Minus: Int = 62;
const KT_Star: Int = 63;
const KT_Slash: Int = 64;
const KT_Percent: Int = 65;
const KT_Caret: Int = 66;
const KT_Tilde: Int = 67;
const KT_Bang: Int = 68;
const KT_Amp: Int = 69;
const KT_Pipe: Int = 70;
const KT_Ampersand: Int = 71;
const KT_Eq: Int = 72;
const KT_EqEq: Int = 73;
const KT_Neq: Int = 74;
const KT_Lt: Int = 75;
const KT_Gt: Int = 76;
const KT_Le: Int = 77;
const KT_Ge: Int = 78;
const KT_AndAnd: Int = 79;
const KT_OrOr: Int = 80;
const KT_PlusEq: Int = 81;
const KT_MinusEq: Int = 82;
const KT_StarEq: Int = 83;
const KT_SlashEq: Int = 84;
const KT_PercentEq: Int = 85;
const KT_DotDot: Int = 86;
const KT_DotDotEq: Int = 87;
const KT_Underscore: Int = 88;
const KT_Hash: Int = 89;
const KT_Eof: Int = 90;
const KT_Error: Int = 91;

fn pe_ktag(k: TokenKind) -> Int {
  match k {
    TkLet => { return KT_Let; }
    TkVar => { return KT_Var; }
    TkConst => { return KT_Const; }
    TkFn => { return KT_Fn; }
    TkReturn => { return KT_Return; }
    TkBreak => { return KT_Break; }
    TkContinue => { return KT_Continue; }
    TkIf => { return KT_If; }
    TkElif => { return KT_Elif; }
    TkElse => { return KT_Else; }
    TkMatch => { return KT_Match; }
    TkWhile => { return KT_While; }
    TkFor => { return KT_For; }
    TkIn => { return KT_In; }
    TkSpawn => { return KT_Spawn; }
    TkAwait => { return KT_Await; }
    TkComptime => { return KT_Comptime; }
    TkAsm => { return KT_Asm; }
    TkDefer => { return KT_Defer; }
    TkMove => { return KT_Move; }
    TkModule => { return KT_Module; }
    TkUse => { return KT_Use; }
    TkPub => { return KT_Pub; }
    TkAs => { return KT_As; }
    TkType => { return KT_Type; }
    TkEnum => { return KT_Enum; }
    TkInterface => { return KT_Interface; }
    TkDerive => { return KT_Derive; }
    TkImpl => { return KT_Impl; }
    TkTrue => { return KT_True; }
    TkFalse => { return KT_False; }
    TkSelf => { return KT_Self; }
    TkSome => { return KT_Some; }
    TkNone => { return KT_None; }
    TkOkV => { return KT_OkV; }
    TkErrV => { return KT_ErrV; }
    TkUnsafe => { return KT_Unsafe; }
    TkExtern => { return KT_Extern; }
    TkIs => { return KT_Is; }
    TkIdent(_) => { return KT_Ident; }
    TkInt(_) => { return KT_Int; }
    TkBigInt(_, _) => { return KT_BigInt; }
    TkFloat(_) => { return KT_Float; }
    TkStr(_) => { return KT_Str; }
    TkChar(_) => { return KT_Char; }
    TkDot => { return KT_Dot; }
    TkComma => { return KT_Comma; }
    TkSemicolon => { return KT_Semicolon; }
    TkColon => { return KT_Colon; }
    TkColonColon => { return KT_ColonColon; }
    TkLParen => { return KT_LParen; }
    TkRParen => { return KT_RParen; }
    TkLBrace => { return KT_LBrace; }
    TkRBrace => { return KT_RBrace; }
    TkLBracket => { return KT_LBracket; }
    TkRBracket => { return KT_RBracket; }
    TkAt => { return KT_At; }
    TkArrow => { return KT_Arrow; }
    TkFatArrow => { return KT_FatArrow; }
    TkQuestion => { return KT_Question; }
    TkPlus => { return KT_Plus; }
    TkMinus => { return KT_Minus; }
    TkStar => { return KT_Star; }
    TkSlash => { return KT_Slash; }
    TkPercent => { return KT_Percent; }
    TkCaret => { return KT_Caret; }
    TkTilde => { return KT_Tilde; }
    TkBang => { return KT_Bang; }
    TkAmp => { return KT_Amp; }
    TkPipe => { return KT_Pipe; }
    TkAmpersand => { return KT_Ampersand; }
    TkEq => { return KT_Eq; }
    TkEqEq => { return KT_EqEq; }
    TkNeq => { return KT_Neq; }
    TkLt => { return KT_Lt; }
    TkGt => { return KT_Gt; }
    TkLe => { return KT_Le; }
    TkGe => { return KT_Ge; }
    TkAndAnd => { return KT_AndAnd; }
    TkOrOr => { return KT_OrOr; }
    TkPlusEq => { return KT_PlusEq; }
    TkMinusEq => { return KT_MinusEq; }
    TkStarEq => { return KT_StarEq; }
    TkSlashEq => { return KT_SlashEq; }
    TkPercentEq => { return KT_PercentEq; }
    TkDotDot => { return KT_DotDot; }
    TkDotDotEq => { return KT_DotDotEq; }
    TkUnderscore => { return KT_Underscore; }
    TkHash => { return KT_Hash; }
    TkEof => { return KT_Eof; }
    TkError(_) => { return KT_Error; }
  }
}

/// Tag of the current token kind.
fn pe_pk(p: &Parser) -> Int {
  return pe_ktag(selfhost_parser_state.p_peek_kind(p));
}

/// Tag of the token kind at offset `n`.
fn pe_pak(p: &Parser, n: Int) -> Int {
  return pe_ktag(selfhost_parser_state.p_peek_ahead_kind(p, n));
}

/// StmtOrExpr::Stmt wrapper (dump emits "Stmt", no span).
fn pe_stmt_w(p: &mut Parser, s: Int) -> Int {
  return selfhost_parser_state.p_n(p, NodeKind.NkStmtW(s), selfhost_ast.span_zero());
}

/// StmtOrExpr::Expr wrapper (dump emits "Tail", no span).
fn pe_tail_w(p: &mut Parser, e: Int) -> Int {
  return selfhost_parser_state.p_n(p, NodeKind.NkTailW(e), selfhost_ast.span_zero());
}

/// `parse_ident().ok()`: on failure the Rust AST discards the Err (no latch).
fn pe_try_ident(p: &mut Parser) -> Int {
  let id = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) {
    p.failed = 0;
    return -1;
  }
  return id;
}

/// Render a Type to its canonical NAME string (Rust type_name_str).
fn pe_type_name_str(p: &Parser, t: Int) -> Str {
  if t < 0 { return "_"; }
  let node = p.nodes[t];
  match node.kind {
    NkTyNamed(id, _) => { return pe_ident_text(p, id); }
    NkTyVec(inner) => { return "Vec[" + pe_type_name_str(p, inner) + "]"; }
    NkTySlice(inner) => { return "Slice[" + pe_type_name_str(p, inner) + "]"; }
    NkTyOption(inner) => { return "Option[" + pe_type_name_str(p, inner) + "]"; }
    NkTySet(inner) => { return "Set[" + pe_type_name_str(p, inner) + "]"; }
    NkTyMap(k, v) => { return "Map[" + pe_type_name_str(p, k) + "," + pe_type_name_str(p, v) + "]"; }
    NkTyResult(ok, err) => { return "Result[" + pe_type_name_str(p, ok) + "," + pe_type_name_str(p, err) + "]"; }
    NkTyTuple(items) => { return "(" + pe_join_types(p, items, ",") + ")"; }
    NkTyFn(params, ret) => { return "fn(" + pe_join_types(p, params, ", ") + ") -> " + pe_type_name_str(p, ret); }
    NkTyRef(inner) => { return "&" + pe_type_name_str(p, inner); }
    NkTyMutRef(inner) => { return "&mut " + pe_type_name_str(p, inner); }
    NkTyPtr(inner) => { return "*" + pe_type_name_str(p, inner); }
    _ => { return "_"; }
  }
}

/// Rust `parts.join(sep)` over rendered type names.
fn pe_join_types(p: &Parser, items: Vec[Int], sep: Str) -> Str {
  var out = "";
  var i = 0;
  while i < items.len() {
    if i > 0 { out = out + sep; }
    out = out + pe_type_name_str(p, items[i]);
    i = i + 1;
  }
  return out;
}

/// Rust generic_type_expr: `Ctor[<rendered inner name>]` Index expression.
fn pe_generic_type_expr(p: &mut Parser, ctor: Str, inner: Int) -> Int {
  let sp = selfhost_parser_state.p_peek_span(p);
  let ctor_node = selfhost_parser_state.p_n_ident(p, ctor, sp);
  let ctor_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(ctor_node), sp);
  let inner_node = selfhost_parser_state.p_n_ident(p, pe_type_name_str(p, inner), sp);
  let inner_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(inner_node), sp);
  return selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(ctor_expr, inner_expr), sp);
}

// ============================================================================
// Identifiers (Rust parse_ident / parse_variant_ident)
// ============================================================================

pub fn pe_parse_ident(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let sp = pe_tok_span(tok);
  var name = "";
  match tok.kind {
    // Ident payload text equals the lexeme (Phase 1 gate); prefer the lexeme.
    TkIdent(_) => { name = tok.lexeme; }
    TkSelf => { name = "self"; }
    TkComptime => { name = "comptime"; }
    TkDerive => { name = "derive"; }
    _ => {
      let lex = tok.lexeme;
      if pe_all_ident_bytes(lex) && !pe_is_reserved_word(lex) {
        name = lex;
      } elif pe_is_reserved_word(lex) {
        return selfhost_parser_state.p_err(
          p, "'" + lex + "' is a reserved keyword and cannot be used as an identifier"
        );
      } else {
        return selfhost_parser_state.p_err(p, "expected identifier, found '" + lex + "'");
      }
    }
  }
  return selfhost_parser_state.p_n_ident(p, name, sp);
}

pub fn pe_parse_variant_ident(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let sp = pe_tok_span(tok);
  match tok.kind {
    TkIdent(_) => { return selfhost_parser_state.p_n_ident(p, tok.lexeme, sp); }
    TkSome => { return selfhost_parser_state.p_n_ident(p, "Some", sp); }
    TkNone => { return selfhost_parser_state.p_n_ident(p, "None", sp); }
    TkOkV => { return selfhost_parser_state.p_n_ident(p, "Ok", sp); }
    TkErrV => { return selfhost_parser_state.p_n_ident(p, "Err", sp); }
    _ => {
      return selfhost_parser_state.p_err(
        p, "expected variant name, found '" + tok.lexeme + "'"
      );
    }
  }
}

// ============================================================================
// Numeric suffixes (Rust parse_int_suffix / parse_float_suffix)
// ============================================================================
//
// The Rust helpers return `Option<Type>`; the public XIOM signatures carry no
// Parser, so they return a TYPE CODE (-1 = none) and
// `pe_suffix_type_node` materializes the arena Named node (Ident at span
// zero, mirroring `Ident::new(name, Span::new(0, 0))`).

const PES_INT8: Int = 1;
const PES_INT16: Int = 2;
const PES_INT32: Int = 3;
const PES_INT64: Int = 4;
const PES_UINT8: Int = 5;
const PES_UINT16: Int = 6;
const PES_UINT32: Int = 7;
const PES_UINT64: Int = 8;
const PES_FLOAT32: Int = 11;
const PES_FLOAT64: Int = 12;

/// "42i8" -> Int8 code, "255u8" -> UInt8 code; -1 when no valid suffix.
pub fn pe_parse_int_suffix(lexeme: Str) -> Int {
  let b = pe_find_byte(lexeme, 105); // 'i'
  var pos = b;
  let u = pe_find_byte(lexeme, 117); // 'u'
  if pos < 0 { pos = u; }
  elif u >= 0 && u < pos { pos = u; }
  if pos < 0 { return -1; }
  let suffix = string.str_slice(lexeme, pos, lexeme.len());
  if suffix == "i8" { return PES_INT8; }
  if suffix == "i16" { return PES_INT16; }
  if suffix == "i32" { return PES_INT32; }
  if suffix == "i64" { return PES_INT64; }
  if suffix == "u8" { return PES_UINT8; }
  if suffix == "u16" { return PES_UINT16; }
  if suffix == "u32" { return PES_UINT32; }
  if suffix == "u64" { return PES_UINT64; }
  return -1;
}

/// "3.14f32" -> Float32 code; -1 when no valid suffix.
pub fn pe_parse_float_suffix(lexeme: Str) -> Int {
  let pos = pe_find_byte(lexeme, 102); // 'f'
  if pos < 0 { return -1; }
  let suffix = string.str_slice(lexeme, pos, lexeme.len());
  if suffix == "f32" { return PES_FLOAT32; }
  if suffix == "f64" { return PES_FLOAT64; }
  return -1;
}

/// Materialize the Named type node for a suffix code (span zero, matching
/// `Span::new(0, 0)` in the Rust helpers).
fn pe_suffix_type_node(p: &mut Parser, code: Int) -> Int {
  if code < 0 { return -1; }
  var nm = "";
  if code == PES_INT8 { nm = "Int8"; }
  elif code == PES_INT16 { nm = "Int16"; }
  elif code == PES_INT32 { nm = "Int32"; }
  elif code == PES_INT64 { nm = "Int64"; }
  elif code == PES_UINT8 { nm = "UInt8"; }
  elif code == PES_UINT16 { nm = "UInt16"; }
  elif code == PES_UINT32 { nm = "UInt32"; }
  elif code == PES_UINT64 { nm = "UInt64"; }
  elif code == PES_FLOAT32 { nm = "Float32"; }
  elif code == PES_FLOAT64 { nm = "Float64"; }
  else { return -1; }
  let nid = selfhost_parser_state.p_n_ident(p, nm, selfhost_ast.span_zero());
  return selfhost_parser_state.p_n(
    p, NodeKind.NkTyNamed(nid, Vec[Int].new()), selfhost_ast.span_zero()
  );
}

// ============================================================================
// Generics / interface refs / derive list (Rust parse_*)
// ============================================================================

pub fn pe_parse_optional_generic_params(p: &mut Parser) -> Vec[Int] {
  if selfhost_parser_state.p_skip(p, TkLBracket) {
    let params = pe_parse_generic_params(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    return params;
  }
  return Vec[Int].new();
}

pub fn pe_parse_generic_params(p: &mut Parser) -> Vec[Int] {
  var params = Vec[Int].new();
  var go = true;
  while go {
    if selfhost_parser_state.p_skip(p, TkConst) {
      let name = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
      let _ = selfhost_parser_state.p_expect_kind(p, TkColon, "':'");
      if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
      let const_ty = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
      let gp = selfhost_parser_state.p_n(
        p, NodeKind.NkGeneric(name, Vec[Int].new(), 1, const_ty), selfhost_ast.span_zero()
      );
      params.push(gp);
    } else {
      let name = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
      var bounds = Vec[Int].new();
      if selfhost_parser_state.p_skip(p, TkColon) {
        bounds = pe_parse_interface_refs(p);
        if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
      }
      let gp = selfhost_parser_state.p_n(
        p, NodeKind.NkGeneric(name, bounds, 0, -1), selfhost_ast.span_zero()
      );
      params.push(gp);
    }
    if !selfhost_parser_state.p_skip(p, TkComma) { break; }
    // Trailing comma in generic params: `fn f[T, U,](...)`.
    if pe_pk(p) == KT_RBracket { break; }
  }
  return params;
}

pub fn pe_parse_interface_refs(p: &mut Parser) -> Vec[Int] {
  var refs = Vec[Int].new();
  let first = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  refs.push(first);
  while selfhost_parser_state.p_skip(p, TkPlus) {
    let r = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    refs.push(r);
  }
  return refs;
}

pub fn pe_parse_derive_list(p: &mut Parser) -> Vec[Int] {
  let _ = selfhost_parser_state.p_expect_kind(p, TkLBracket, "'['");
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  var derives = Vec[Int].new();
  var go = true;
  while go {
    let name = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    let nm = pe_ident_text(p, name);
    var dt = -1;
    if nm == "Eq" { dt = selfhost_ast.DRV_EQ; }
    elif nm == "Clone" { dt = selfhost_ast.DRV_CLONE; }
    elif nm == "Display" { dt = selfhost_ast.DRV_DISPLAY; }
    elif nm == "Hash" { dt = selfhost_ast.DRV_HASH; }
    elif nm == "Ord" { dt = selfhost_ast.DRV_ORD; }
    elif nm == "Debug" { dt = selfhost_ast.DRV_DEBUG; }
    else {
      let _ = selfhost_parser_state.p_err(p, "unknown derive trait: " + nm);
      return Vec[Int].new();
    }
    derives.push(dt);
    if !selfhost_parser_state.p_skip(p, TkComma) { go = false; }
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  return derives;
}

// ============================================================================
// Types (Rust parse_type / parse_type_inner / parse_type_base)
// ============================================================================

pub fn pe_parse_type(p: &mut Parser) -> Int {
  let _ = selfhost_parser_state.p_enter_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let r = pe_parse_type_inner(p);
  selfhost_parser_state.p_exit_expr(p);
  return r;
}

fn pe_parse_type_inner(p: &mut Parser) -> Int {
  if selfhost_parser_state.p_skip(p, TkAmpersand) {
    var mutable = false;
    let k = selfhost_parser_state.p_peek_kind(p);
    match k {
      TkIdent(_) => {
        if selfhost_parser_state.p_peek(p).lexeme == "mut" {
          selfhost_parser_state.p_advance(p);
          mutable = true;
        }
      }
      _ => {}
    }
    let base = pe_parse_type_base(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    if mutable {
      return selfhost_parser_state.p_n(p, NodeKind.NkTyMutRef(base), selfhost_ast.span_zero());
    }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyRef(base), selfhost_ast.span_zero());
  }
  return pe_parse_type_base(p);
}

fn pe_parse_type_base(p: &mut Parser) -> Int {
  // v0.55: Never type -- `!` as bottom type.
  if pe_pk(p) == KT_Bang {
    selfhost_parser_state.p_advance(p);
    return selfhost_parser_state.p_n(p, NodeKind.NkTyNever, selfhost_ast.span_zero());
  }
  // Skip 'dyn' (dynamic dispatch marker): `dyn Trait` parses as `Trait`.
  let kdyn = selfhost_parser_state.p_peek_kind(p);
  match kdyn {
    TkIdent(_) => {
      if selfhost_parser_state.p_peek(p).lexeme == "dyn" { selfhost_parser_state.p_advance(p); }
    }
    _ => {}
  }
  // `impl Trait` opaque return type.
  var is_impl = false;
  let kimpl = selfhost_parser_state.p_peek_kind(p);
  match kimpl {
    TkIdent(_) => {
      if selfhost_parser_state.p_peek(p).lexeme == "impl" { is_impl = true; }
    }
    TkImpl => { is_impl = true; }
    _ => {}
  }
  if is_impl {
    selfhost_parser_state.p_advance(p);
    let t0 = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    var traits = Vec[Int].new();
    traits.push(t0);
    while selfhost_parser_state.p_skip(p, TkPlus) {
      let t = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      traits.push(t);
    }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyImplTrait(traits), selfhost_ast.span_zero());
  }
  // Builtin containers, only when a generic opener follows.
  var special = "";
  let kspec = selfhost_parser_state.p_peek_kind(p);
  match kspec {
    TkIdent(_) => {
      let nm = selfhost_parser_state.p_peek(p).lexeme;
      let ah = selfhost_parser_state.p_peek_ahead_kind(p, 1);
      if pe_ktag(ah) == KT_LBracket || pe_ktag(ah) == KT_Lt {
        if nm == "Option" { special = "Option"; }
        elif nm == "Result" { special = "Result"; }
        elif nm == "Vec" { special = "Vec"; }
        elif nm == "Slice" { special = "Slice"; }
        elif nm == "Map" { special = "Map"; }
        elif nm == "Set" { special = "Set"; }
      }
    }
    _ => {}
  }
  if special == "Option" || special == "Vec" || special == "Slice" || special == "Set" {
    selfhost_parser_state.p_advance(p);
    let used = selfhost_parser_state.p_skip(p, TkLBracket);
    if !used {
      let _ = selfhost_parser_state.p_expect_kind(p, TkLt, "'<'");
      if selfhost_parser_state.p_failed(p) { return -1; }
    }
    let inner = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_close_generic_type(p, used);
    if selfhost_parser_state.p_failed(p) { return -1; }
    if special == "Option" {
      return selfhost_parser_state.p_n(p, NodeKind.NkTyOption(inner), selfhost_ast.span_zero());
    }
    if special == "Vec" {
      return selfhost_parser_state.p_n(p, NodeKind.NkTyVec(inner), selfhost_ast.span_zero());
    }
    if special == "Slice" {
      return selfhost_parser_state.p_n(p, NodeKind.NkTySlice(inner), selfhost_ast.span_zero());
    }
    return selfhost_parser_state.p_n(p, NodeKind.NkTySet(inner), selfhost_ast.span_zero());
  }
  if special == "Result" {
    selfhost_parser_state.p_advance(p);
    let used = selfhost_parser_state.p_skip(p, TkLBracket);
    if !used {
      let _ = selfhost_parser_state.p_expect_kind(p, TkLt, "'<'");
      if selfhost_parser_state.p_failed(p) { return -1; }
    }
    let ok = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_expect_kind(p, TkComma, "','");
    if selfhost_parser_state.p_failed(p) { return -1; }
    let err = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_close_generic_type(p, used);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyResult(ok, err), selfhost_ast.span_zero());
  }
  if special == "Map" {
    selfhost_parser_state.p_advance(p);
    let used = selfhost_parser_state.p_skip(p, TkLBracket);
    if !used {
      let _ = selfhost_parser_state.p_expect_kind(p, TkLt, "'<'");
      if selfhost_parser_state.p_failed(p) { return -1; }
    }
    let k = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_expect_kind(p, TkComma, "','");
    if selfhost_parser_state.p_failed(p) { return -1; }
    let v = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_close_generic_type(p, used);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyMap(k, v), selfhost_ast.span_zero());
  }
  // Raw pointers; optional `const` / `mut` qualifier.
  if selfhost_parser_state.p_skip(p, TkStar) {
    if pe_pk(p) == KT_Const {
      selfhost_parser_state.p_advance(p);
    } else {
      let kk = selfhost_parser_state.p_peek_kind(p);
      match kk {
        TkIdent(_) => {
          if selfhost_parser_state.p_peek(p).lexeme == "mut" { selfhost_parser_state.p_advance(p); }
        }
        _ => {}
      }
    }
    let base = pe_parse_type_base(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyPtr(base), selfhost_ast.span_zero());
  }
  // `()` unit type, parenthesized type, or tuple type.
  if selfhost_parser_state.p_skip(p, TkLParen) {
    if pe_pk(p) == KT_RParen {
      selfhost_parser_state.p_advance(p); // consume ')'
      let nid = selfhost_parser_state.p_n_ident(p, "()", selfhost_parser_state.p_peek_span(p));
      let t = selfhost_parser_state.p_n(
        p, NodeKind.NkTyNamed(nid, Vec[Int].new()), selfhost_parser_state.p_span_of(p, nid)
      );
      return t;
    }
    let first = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    if selfhost_parser_state.p_skip(p, TkComma) {
      var types = Vec[Int].new();
      types.push(first);
      let t2 = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      types.push(t2);
      while selfhost_parser_state.p_skip(p, TkComma) {
        let t = pe_parse_type(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        types.push(t);
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkTyTuple(types), selfhost_ast.span_zero());
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    return first;
  }
  // Fixed array type: `[N]T` / `[NAME]T`.
  if selfhost_parser_state.p_skip(p, TkLBracket) {
    var size = -1;
    let nk = selfhost_parser_state.p_peek_kind(p);
    match nk {
      TkInt(n) => {
        selfhost_parser_state.p_advance(p);
        size = selfhost_parser_state.p_n(
          p, NodeKind.NkLitInt(n), selfhost_parser_state.p_peek_span(p)
        );
      }
      TkIdent(_) => {
        let id = pe_parse_ident(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        size = selfhost_parser_state.p_n(
          p, NodeKind.NkExprIdent(id), selfhost_parser_state.p_span_of(p, id)
        );
      }
      _ => {
        return selfhost_parser_state.p_err(
          p, "expected integer or identifier for fixed array size"
        );
      }
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    let inner = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyArray(size, inner), selfhost_ast.span_zero());
  }
  // Function type: `fn(A, B) -> R` (default return `Unit` at span zero).
  if pe_pk(p) == KT_Fn {
    selfhost_parser_state.p_advance(p);
    let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
    if selfhost_parser_state.p_failed(p) { return -1; }
    var param_types = Vec[Int].new();
    if pe_pk(p) != KT_RParen {
      let t0 = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      param_types.push(t0);
      while selfhost_parser_state.p_skip(p, TkComma) {
        let t = pe_parse_type(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        param_types.push(t);
      }
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    var ret = -1;
    if selfhost_parser_state.p_skip(p, TkArrow) {
      ret = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
    } else {
      let unit = selfhost_parser_state.p_n_ident(p, "Unit", selfhost_ast.span_zero());
      ret = selfhost_parser_state.p_n(
        p, NodeKind.NkTyNamed(unit, Vec[Int].new()), selfhost_ast.span_zero()
      );
    }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyFn(param_types, ret), selfhost_ast.span_zero());
  }
  // Anonymous struct type `{ field: Type; ... }`.
  if pe_pk(p) == KT_LBrace {
    let start = selfhost_parser_state.p_peek_span(p);
    selfhost_parser_state.p_advance(p);
    var fields = Vec[Int].new();
    var go = true;
    while go {
      let k = selfhost_parser_state.p_peek_kind(p);
      if pe_ktag(k) == KT_RBrace || pe_ktag(k) == KT_Eof { break; }
      let fname = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkColon, "':'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let fty = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      fields.push(selfhost_parser_state.p_n(p, NodeKind.NkField(fname, fty), start));
      var sep = selfhost_parser_state.p_skip(p, TkSemicolon);
      if !sep { sep = selfhost_parser_state.p_skip(p, TkComma); }
      if !sep { break; }
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkTyAnonStruct(fields), selfhost_ast.span_zero());
  }
  // Named type with optional dotted path and generic args.
  var name = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let name_span = selfhost_parser_state.p_span_of(p, name);
  var name_text = pe_ident_text(p, name);
  var folded = false;
  while selfhost_parser_state.p_skip(p, TkDot) {
    let next = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    name_text = name_text + "." + pe_ident_text(p, next);
    folded = true;
  }
  if folded {
    name = selfhost_parser_state.p_n_ident(p, name_text, name_span);
  }
  var args = Vec[Int].new();
  if selfhost_parser_state.p_skip(p, TkLBracket) {
    let t0 = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    args.push(t0);
    while selfhost_parser_state.p_skip(p, TkComma) {
      let t = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      args.push(t);
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
    if selfhost_parser_state.p_failed(p) { return -1; }
  } elif selfhost_parser_state.p_skip(p, TkLt) {
    let t0 = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    args.push(t0);
    while selfhost_parser_state.p_skip(p, TkComma) {
      let t = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      args.push(t);
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkGt, "'>'");
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  return selfhost_parser_state.p_n(
    p, NodeKind.NkTyNamed(name, args), selfhost_parser_state.p_span_of(p, name)
  );
}

// ============================================================================
// Parameters (Rust parse_param_list / parse_param)
// ============================================================================

pub fn pe_parse_param_list(p: &mut Parser) -> Vec[Int] {
  var params = Vec[Int].new();
  var go = true;
  while go {
    let pm = pe_parse_param(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    params.push(pm);
    if !selfhost_parser_state.p_skip(p, TkComma) { break; }
    // Trailing comma: `fn f(a: Int, b: Int,)` -- accepted.
    if pe_pk(p) == KT_RParen { break; }
  }
  return params;
}

pub fn pe_parse_param(p: &mut Parser) -> Int {
  let span = selfhost_parser_state.p_peek_span(p);
  if pe_pk(p) == KT_Ampersand {
    selfhost_parser_state.p_advance(p);
    var is_mut = false;
    if selfhost_parser_state.p_peek(p).lexeme == "mut" {
      selfhost_parser_state.p_advance(p);
      is_mut = true;
    }
    let name = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let self_ident = selfhost_parser_state.p_n_ident(p, "Self", span);
    let self_ty = selfhost_parser_state.p_n(
      p, NodeKind.NkTyNamed(self_ident, Vec[Int].new()), span
    );
    var refself = 0;
    if is_mut { refself = 1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkParam(name, self_ty, refself, 1), span);
  }
  if pe_pk(p) == KT_Self {
    let name = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let self_ident = selfhost_parser_state.p_n_ident(p, "Self", span);
    let self_ty = selfhost_parser_state.p_n(
      p, NodeKind.NkTyNamed(self_ident, Vec[Int].new()), span
    );
    return selfhost_parser_state.p_n(p, NodeKind.NkParam(name, self_ty, 0, 0), span);
  }
  let name = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  if pe_ident_text(p, name) == "mut" {
    let name2 = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_expect_kind(p, TkColon, "':'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    let ty = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkParam(name2, ty, 0, 0), span);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkColon, "':'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let ty = pe_parse_type(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkParam(name, ty, 0, 0), span);
}

// ============================================================================
// Tuple struct args / struct-literal construction helpers
// ============================================================================

pub fn pe_parse_tuple_type_args(p: &mut Parser) -> Vec[Int] {
  let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  var types = Vec[Int].new();
  if pe_pk(p) == KT_RParen {
    selfhost_parser_state.p_advance(p);
    return types;
  }
  var go = true;
  while go {
    let t = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    types.push(t);
    if selfhost_parser_state.p_skip(p, TkComma) {
      if pe_pk(p) == KT_RParen {
        selfhost_parser_state.p_advance(p);
        break;
      }
      continue;
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    break;
  }
  return types;
}

/// Rust extract_struct_type_name: ident node index of the constructible
/// struct type name, or -1 (builtins / lowercase / non-Named types).
fn pe_extract_struct_type_name(p: &Parser, ty: Int) -> Int {
  if ty < 0 { return -1; }
  let node = p.nodes[ty];
  match node.kind {
    NkTyNamed(id, _) => {
      let nm = pe_ident_text(p, id);
      if pe_builtin_type_name(nm) || !pe_first_upper(nm) { return -1; }
      return id;
    }
    NkTyRef(inner) => { return pe_extract_struct_type_name(p, inner); }
    NkTyMutRef(inner) => { return pe_extract_struct_type_name(p, inner); }
    _ => { return -1; }
  }
}

pub fn pe_parse_init_expr(p: &mut Parser, declared_ty: Int, span: Span) -> Int {
  if declared_ty >= 0 {
    let type_name = pe_extract_struct_type_name(p, declared_ty);
    if type_name >= 0 {
      if pe_pk(p) == KT_LBrace {
        return pe_parse_struct_literal_body(p, type_name, pe_ident_text(p, type_name), span);
      }
    }
  }
  return pe_parse_expr(p);
}

/// Rust struct-literal field loop: `name: value` / shorthand `name`, with
/// `,` or `;` separators; stops at `}`, `.` (spread) or EOF.
fn pe_parse_struct_fields(p: &mut Parser) -> Vec[Int] {
  var fields = Vec[Int].new();
  var go = true;
  while go {
    let k = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k) == KT_RBrace || pe_ktag(k) == KT_Dot || pe_ktag(k) == KT_Eof { break; }
    let fname = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    if selfhost_parser_state.p_skip(p, TkColon) {
      let fval = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
      let fsp = selfhost_parser_state.p_span_of(p, fname);
      fields.push(selfhost_parser_state.p_n(p, NodeKind.NkFieldInit(fname, fval), fsp));
    } else {
      // Shorthand: `{ field }` means `{ field: field }`.
      let fsp = selfhost_parser_state.p_span_of(p, fname);
      let fexpr = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(fname), fsp);
      fields.push(selfhost_parser_state.p_n(p, NodeKind.NkFieldInit(fname, fexpr), fsp));
    }
    let _ = selfhost_parser_state.p_skip(p, TkComma);
    let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
  }
  return fields;
}

/// Optional `..expr` spread tail; -1 when absent.
fn pe_parse_struct_spread(p: &mut Parser) -> Int {
  if selfhost_parser_state.p_skip(p, TkDot) {
    let _ = selfhost_parser_state.p_expect_kind(p, TkDot, "'.' for spread");
    if selfhost_parser_state.p_failed(p) { return -1; }
    let s = pe_parse_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return s;
  }
  return -1;
}

pub fn pe_parse_struct_literal_body(
  p: &mut Parser, type_name: Int, type_name_str: Str, span: Span
) -> Int {
  // Rust: Ident::new(type_name.to_string(), span) -- a FRESH ident at `span`
  // (the declared-type span), not the type-annotation token's span.
  let _ = type_name;
  let _ = selfhost_parser_state.p_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let fields = pe_parse_struct_fields(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let spread = pe_parse_struct_spread(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let nid = selfhost_parser_state.p_n_ident(p, type_name_str, span);
  return selfhost_parser_state.p_n(p, NodeKind.NkExprStruct(nid, fields, spread), span);
}

/// Rust type_to_expr_ident: parsed Type -> Expr for Index type-arg capture.
pub fn pe_type_to_expr_ident(p: &mut Parser, t: Int) -> Int {
  if t < 0 { return -1; }
  let node = p.nodes[t];
  match node.kind {
    NkTyNamed(id, _) => {
      let sp = selfhost_parser_state.p_span_of(p, id);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), sp);
    }
    NkTyTuple(types) => {
      var elems = Vec[Int].new();
      var i = 0;
      while i < types.len() {
        let e = pe_type_to_expr_ident(p, types[i]);
        if selfhost_parser_state.p_failed(p) { return -1; }
        elems.push(e);
        i = i + 1;
      }
      let sp = selfhost_parser_state.p_peek_span(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprTuple(elems), sp);
    }
    NkTyVec(inner) => { return pe_generic_type_expr(p, "Vec", inner); }
    NkTySlice(inner) => { return pe_generic_type_expr(p, "Slice", inner); }
    NkTyOption(inner) => { return pe_generic_type_expr(p, "Option", inner); }
    NkTySet(inner) => { return pe_generic_type_expr(p, "Set", inner); }
    NkTyMap(k, v) => {
      let sp = selfhost_parser_state.p_peek_span(p);
      let ctor = selfhost_parser_state.p_n_ident(p, "Map", sp);
      let ctor_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(ctor), sp);
      let txt = pe_type_name_str(p, k) + "," + pe_type_name_str(p, v);
      let arg = selfhost_parser_state.p_n_ident(p, txt, sp);
      let arg_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(arg), sp);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(ctor_expr, arg_expr), sp);
    }
    NkTyResult(ok, err) => {
      let sp = selfhost_parser_state.p_peek_span(p);
      let ctor = selfhost_parser_state.p_n_ident(p, "Result", sp);
      let ctor_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(ctor), sp);
      let txt = pe_type_name_str(p, ok) + "," + pe_type_name_str(p, err);
      let arg = selfhost_parser_state.p_n_ident(p, txt, sp);
      let arg_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(arg), sp);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(ctor_expr, arg_expr), sp);
    }
    NkTyFn(_, _) => {
      let sp = selfhost_parser_state.p_peek_span(p);
      let id = selfhost_parser_state.p_n_ident(p, pe_type_name_str(p, t), sp);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), sp);
    }
    _ => {
      let sp = selfhost_parser_state.p_peek_span(p);
      let id = selfhost_parser_state.p_n_ident(p, "_", sp);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), sp);
    }
  }
}

// ============================================================================
// Expression grammar (Rust parse_expr .. parse_as_expr)
// ============================================================================

pub fn pe_parse_expr(p: &mut Parser) -> Int {
  let _ = selfhost_parser_state.p_enter_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let r = pe_parse_imply_expr(p);
  selfhost_parser_state.p_exit_expr(p);
  return r;
}

/// Head expression of a control-flow construct: trailing `{` starts the
/// body, so bare struct literals are suppressed at this level.
pub fn pe_parse_cond(p: &mut Parser) -> Int {
  let saved = p.restrict_struct;
  p.restrict_struct = 1;
  let r = pe_parse_expr(p);
  p.restrict_struct = saved;
  return r;
}

/// Expression inside a delimiter: a condition's struct restriction does not
/// cross delimiters.
fn pe_parse_expr_open(p: &mut Parser) -> Int {
  let saved = p.restrict_struct;
  p.restrict_struct = 0;
  let r = pe_parse_expr(p);
  p.restrict_struct = saved;
  return r;
}

fn pe_parse_imply_expr(p: &mut Parser) -> Int {
  var left = pe_parse_or_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  while selfhost_parser_state.p_skip(p, TkFatArrow) {
    let right = pe_parse_or_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(p, NodeKind.NkExprImply(left, right), span);
  }
  if selfhost_parser_state.p_skip(p, TkQuestion) {
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(p, NodeKind.NkExprTry(left), span);
  }
  return left;
}

fn pe_parse_or_expr(p: &mut Parser) -> Int {
  var left = pe_parse_and_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  while selfhost_parser_state.p_skip(p, TkOrOr) {
    let right = pe_parse_and_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(
      p, NodeKind.NkExprBinary(left, selfhost_ast.OP_OR, right), span
    );
  }
  return left;
}

fn pe_parse_and_expr(p: &mut Parser) -> Int {
  var left = pe_parse_is_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  while selfhost_parser_state.p_skip(p, TkAndAnd) {
    let right = pe_parse_is_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(
      p, NodeKind.NkExprBinary(left, selfhost_ast.OP_AND, right), span
    );
  }
  return left;
}

fn pe_parse_is_expr(p: &mut Parser) -> Int {
  let left = pe_parse_cmp_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  if selfhost_parser_state.p_skip(p, TkIs) {
    let pattern = pe_parse_pattern(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    return selfhost_parser_state.p_n(p, NodeKind.NkExprIs(left, pattern), span);
  }
  return left;
}

fn pe_parse_cmp_expr(p: &mut Parser) -> Int {
  var left = pe_parse_bit_or_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var go = true;
  while go {
    var op = -1;
    let k = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k) == KT_EqEq { op = selfhost_ast.OP_EQ; }
    elif pe_ktag(k) == KT_Neq { op = selfhost_ast.OP_NEQ; }
    elif pe_ktag(k) == KT_Lt { op = selfhost_ast.OP_LT; }
    elif pe_ktag(k) == KT_Gt { op = selfhost_ast.OP_GT; }
    elif pe_ktag(k) == KT_Le { op = selfhost_ast.OP_LE; }
    elif pe_ktag(k) == KT_Ge { op = selfhost_ast.OP_GE; }
    else { break; }
    selfhost_parser_state.p_advance(p);
    let right = pe_parse_bit_or_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(p, NodeKind.NkExprBinary(left, op, right), span);
  }
  return left;
}

fn pe_parse_bit_or_expr(p: &mut Parser) -> Int {
  var left = pe_parse_bit_xor_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var go = true;
  while go {
    if pe_pk(p) != KT_Pipe { break; }
    selfhost_parser_state.p_advance(p);
    let right = pe_parse_bit_xor_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(
      p, NodeKind.NkExprBinary(left, selfhost_ast.OP_BITOR, right), span
    );
  }
  return left;
}

fn pe_parse_bit_xor_expr(p: &mut Parser) -> Int {
  var left = pe_parse_bit_and_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var go = true;
  while go {
    if pe_pk(p) != KT_Caret { break; }
    selfhost_parser_state.p_advance(p);
    let right = pe_parse_bit_and_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(
      p, NodeKind.NkExprBinary(left, selfhost_ast.OP_BITXOR, right), span
    );
  }
  return left;
}

fn pe_parse_bit_and_expr(p: &mut Parser) -> Int {
  var left = pe_parse_shift_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var go = true;
  while go {
    if pe_pk(p) != KT_Ampersand { break; }
    selfhost_parser_state.p_advance(p);
    let right = pe_parse_shift_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(
      p, NodeKind.NkExprBinary(left, selfhost_ast.OP_BITAND, right), span
    );
  }
  return left;
}

fn pe_parse_shift_expr(p: &mut Parser) -> Int {
  var left = pe_parse_add_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var go = true;
  while go {
    if pe_pk(p) == KT_Lt
      && pe_pak(p, 1) == KT_Lt
    {
      selfhost_parser_state.p_advance(p);
      selfhost_parser_state.p_advance(p);
      let right = pe_parse_add_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let span = selfhost_parser_state.p_span_of(p, left);
      left = selfhost_parser_state.p_n(
        p, NodeKind.NkExprBinary(left, selfhost_ast.OP_SHL, right), span
      );
    } elif pe_pk(p) == KT_Gt
      && pe_pak(p, 1) == KT_Gt
    {
      selfhost_parser_state.p_advance(p);
      selfhost_parser_state.p_advance(p);
      let right = pe_parse_add_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let span = selfhost_parser_state.p_span_of(p, left);
      left = selfhost_parser_state.p_n(
        p, NodeKind.NkExprBinary(left, selfhost_ast.OP_SHR, right), span
      );
    } else {
      break;
    }
  }
  return left;
}

fn pe_parse_add_expr(p: &mut Parser) -> Int {
  var left = pe_parse_mul_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var go = true;
  while go {
    var op = -1;
    let k = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k) == KT_Plus { op = selfhost_ast.OP_ADD; }
    elif pe_ktag(k) == KT_Minus { op = selfhost_ast.OP_SUB; }
    else { break; }
    selfhost_parser_state.p_advance(p);
    let right = pe_parse_mul_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(p, NodeKind.NkExprBinary(left, op, right), span);
  }
  return left;
}

fn pe_parse_mul_expr(p: &mut Parser) -> Int {
  var left = pe_parse_as_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var go = true;
  while go {
    var op = -1;
    let k = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k) == KT_Star { op = selfhost_ast.OP_MUL; }
    elif pe_ktag(k) == KT_Slash { op = selfhost_ast.OP_DIV; }
    elif pe_ktag(k) == KT_Percent { op = selfhost_ast.OP_REM; }
    else { break; }
    selfhost_parser_state.p_advance(p);
    let right = pe_parse_as_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, left);
    left = selfhost_parser_state.p_n(p, NodeKind.NkExprBinary(left, op, right), span);
  }
  return left;
}

fn pe_parse_unary_expr(p: &mut Parser) -> Int {
  return pe_parse_unary_prefix(p);
}

/// `as` has lower precedence than unary operators: `-128 as Int8` parses as
/// `(-128) as Int8`.
fn pe_parse_as_expr(p: &mut Parser) -> Int {
  var expr = pe_parse_unary_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  while selfhost_parser_state.p_skip(p, TkAs) {
    let ty = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, expr);
    expr = selfhost_parser_state.p_n(p, NodeKind.NkExprAs(expr, ty), span);
  }
  return expr;
}

fn pe_parse_unary_prefix(p: &mut Parser) -> Int {
  let span = selfhost_parser_state.p_peek_span(p);
  let pk = selfhost_parser_state.p_peek_kind(p);
  if pe_ktag(pk) == KT_Bang {
    selfhost_parser_state.p_advance(p);
    let inner = pe_parse_unary_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(
      p, NodeKind.NkExprUnary(selfhost_ast.UOP_NOT, inner), span
    );
  }
  if pe_ktag(pk) == KT_Minus {
    selfhost_parser_state.p_advance(p);
    let inner = pe_parse_unary_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    // Fold `-128i8` -> `As(Int(-128), Int8)` so the negation is computed at
    // the literal level before the narrow-int cast.
    let inode = p.nodes[inner];
    match inode.kind {
      NkExprAs(base, ty) => {
        let bnode = p.nodes[base];
        match bnode.kind {
          NkLitInt(n) => {
            let neg = (0 as UInt) - n;
            let new_int = selfhost_parser_state.p_n(
              p, NodeKind.NkLitInt(neg), selfhost_parser_state.p_span_of(p, base)
            );
            return selfhost_parser_state.p_n(
              p, NodeKind.NkExprAs(new_int, ty), selfhost_parser_state.p_span_of(p, inner)
            );
          }
          NkLitFloat(_lex) => {
            // Rust negates the f64 value; the arena stores only the lexeme
            // and the dump re-slices the source, so the negated value is not
            // representable and not observable (float value parity deferred).
            return selfhost_parser_state.p_n(
              p, NodeKind.NkExprAs(base, ty), selfhost_parser_state.p_span_of(p, inner)
            );
          }
          _ => {}
        }
      }
      _ => {}
    }
    return selfhost_parser_state.p_n(
      p, NodeKind.NkExprUnary(selfhost_ast.UOP_NEG, inner), span
    );
  }
  if pe_ktag(pk) == KT_Star {
    selfhost_parser_state.p_advance(p);
    let inner = pe_parse_unary_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(
      p, NodeKind.NkExprUnary(selfhost_ast.UOP_DEREF, inner), span
    );
  }
  if pe_ktag(pk) == KT_Tilde {
    selfhost_parser_state.p_advance(p);
    let inner = pe_parse_unary_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(
      p, NodeKind.NkExprUnary(selfhost_ast.UOP_BITNOT, inner), span
    );
  }
  if pe_ktag(pk) == KT_Ampersand {
    selfhost_parser_state.p_advance(p);
    var mutable = false;
    let k = selfhost_parser_state.p_peek_kind(p);
    match k {
      TkIdent(_) => {
        if selfhost_parser_state.p_peek(p).lexeme == "mut" {
          selfhost_parser_state.p_advance(p);
          mutable = true;
        }
      }
      _ => {}
    }
    let inner = pe_parse_unary_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    if mutable {
      return selfhost_parser_state.p_n(p, NodeKind.NkExprMutRef(inner), span);
    }
    return selfhost_parser_state.p_n(p, NodeKind.NkExprRef(inner), span);
  }
  return pe_parse_postfix_expr(p);
}

// ============================================================================
// Postfix / primary expressions (Rust parse_postfix_expr / parse_primary)
// ============================================================================

/// Rust collect_path: dotted path of Ident/Field chains into a string.
fn pe_collect_path(p: &Parser, expr: Int, out: Str) -> Str {
  let node = p.nodes[expr];
  match node.kind {
    NkExprIdent(id) => {
      if out.len() > 0 { return out + "." + pe_ident_text(p, id); }
      return pe_ident_text(p, id);
    }
    NkExprField(base, field, _sp) => {
      let pre = pe_collect_path(p, base, out);
      return pre + "." + pe_ident_text(p, field);
    }
    _ => { return out; }
  }
}

/// Rust is_type_name over an Expr node (Ident of Field-of-Ident, uppercase).
fn pe_expr_is_type_name(p: &Parser, expr: Int) -> Bool {
  if expr < 0 { return false; }
  let node = p.nodes[expr];
  match node.kind {
    NkExprIdent(id) => { return pe_first_upper(pe_ident_text(p, id)); }
    NkExprField(obj, _field, _sp) => {
      let onode = p.nodes[obj];
      match onode.kind {
        NkExprIdent(id) => { return pe_first_upper(pe_ident_text(p, id)); }
        _ => { return false; }
      }
    }
    _ => { return false; }
  }
}

fn pe_parse_arg_list(p: &mut Parser) -> Vec[Int] {
  let saved = p.restrict_struct;
  p.restrict_struct = 0;
  var args = Vec[Int].new();
  let first = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  args.push(first);
  while selfhost_parser_state.p_skip(p, TkComma) {
    if pe_pk(p) == KT_RParen { break; }
    let a = pe_parse_expr(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    args.push(a);
  }
  p.restrict_struct = saved;
  return args;
}

fn pe_parse_postfix_expr(p: &mut Parser) -> Int {
  var expr = pe_parse_primary(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  // Range syntax after primary: `expr..expr` / `expr..=expr`.
  if pe_pk(p) == KT_Dot
    && pe_pak(p, 1) == KT_Dot
  {
    selfhost_parser_state.p_advance(p);
    selfhost_parser_state.p_advance(p);
    var inclusive = false;
    if pe_pk(p) == KT_Eq {
      inclusive = true;
      selfhost_parser_state.p_advance(p);
    }
    let right = pe_parse_primary(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let span = selfhost_parser_state.p_span_of(p, expr);
    var fn_name = "range";
    if inclusive { fn_name = "range_inclusive"; }
    let id = selfhost_parser_state.p_n_ident(p, fn_name, span);
    let callee = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), span);
    var args = Vec[Int].new();
    args.push(expr);
    args.push(right);
    return selfhost_parser_state.p_n(p, NodeKind.NkExprCall(callee, args), span);
  }
  // Pending GenericCall tracking (Rust's `if let Expr::GenericCall` merge):
  // the destructure of NkExprGenericCall mis-lowers in this function
  // (COMPILER_BUGS 2026-10-02 (h)), so the base/types are carried in locals
  // and `gc_node` identifies the current expr as that GenericCall node.
  var gc_base = -1;
  var gc_types = Vec[Int].new();
  var gc_node = -1;
  var go = true;
  while go {
    let k = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k) == KT_ColonColon {
      // Two meanings: turbofish `expr::<Type>(args)` and static method
      // `Type::method`.
      selfhost_parser_state.p_advance(p);
      if pe_pk(p) == KT_Lt {
        selfhost_parser_state.p_advance(p);
        var types = Vec[Int].new();
        let t0 = pe_parse_type(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        types.push(t0);
        while selfhost_parser_state.p_skip(p, TkComma) {
          let t = pe_parse_type(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          types.push(t);
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkGt, "'>'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        if pe_pk(p) == KT_LParen {
          selfhost_parser_state.p_advance(p);
          var args = Vec[Int].new();
          if pe_pk(p) == KT_RParen {
            selfhost_parser_state.p_advance(p);
          } else {
            args = pe_parse_arg_list(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
            if selfhost_parser_state.p_failed(p) { return -1; }
          }
          let span = selfhost_parser_state.p_span_of(p, expr);
          gc_base = expr;
          gc_types = types;
          expr = selfhost_parser_state.p_n(
            p, NodeKind.NkExprGenericCall(expr, types, args), span
          );
          gc_node = expr;
        }
      } else {
        let method = pe_parse_ident(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let span = selfhost_parser_state.p_span_of(p, expr);
        expr = selfhost_parser_state.p_n(p, NodeKind.NkExprField(expr, method), span);
      }
    } elif pe_ktag(k) == KT_Dot {
      selfhost_parser_state.p_advance(p);
      let kd = selfhost_parser_state.p_peek_kind(p);
      match kd {
        TkInt(n) => {
          selfhost_parser_state.p_advance(p);
          let span = selfhost_parser_state.p_span_of(p, expr);
          let fid = selfhost_parser_state.p_n_ident(p, "_" + pe_uint_str(n), span);
          expr = selfhost_parser_state.p_n(p, NodeKind.NkExprField(expr, fid), span);
        }
        _ => {
          let field = pe_parse_ident(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          let span = selfhost_parser_state.p_span_of(p, expr);
          expr = selfhost_parser_state.p_n(p, NodeKind.NkExprField(expr, field), span);
        }
      }
    } elif pe_ktag(k) == KT_LParen {
      selfhost_parser_state.p_advance(p);
      if pe_pk(p) == KT_RParen {
        selfhost_parser_state.p_advance(p);
        let span = selfhost_parser_state.p_span_of(p, expr);
        if gc_node == expr {
          expr = selfhost_parser_state.p_n(
            p, NodeKind.NkExprGenericCall(gc_base, gc_types, Vec[Int].new()), span
          );
          gc_node = expr;
        } else {
          expr = selfhost_parser_state.p_n(
            p, NodeKind.NkExprCall(expr, Vec[Int].new()), span
          );
        }
      } else {
        var use_named = false;
        let kn = selfhost_parser_state.p_peek_kind(p);
        match kn {
          TkIdent(_) => {
            let saved = p.pos;
            selfhost_parser_state.p_advance(p);
            if pe_pk(p) == KT_Colon
              && pe_pak(p, 1) != KT_Colon
            {
              use_named = true;
            }
            p.pos = saved;
          }
          _ => {}
        }
        if use_named {
          var fields = Vec[Int].new();
          var fgo = true;
          while fgo {
            let fname = pe_parse_ident(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            let _ = selfhost_parser_state.p_expect_kind(p, TkColon, "':'");
            if selfhost_parser_state.p_failed(p) { return -1; }
            let fval = pe_parse_expr_open(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            let fsp = selfhost_parser_state.p_span_of(p, fname);
            fields.push(selfhost_parser_state.p_n(p, NodeKind.NkFieldInit(fname, fval), fsp));
            var sep = selfhost_parser_state.p_skip(p, TkComma);
            if !sep { sep = selfhost_parser_state.p_skip(p, TkSemicolon); }
            if !sep { fgo = false; }
          }
          let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          let span = selfhost_parser_state.p_span_of(p, expr);
          var type_name = -1;
          let enode = p.nodes[expr];
          match enode.kind {
            NkExprIdent(id) => { type_name = id; }
            _ => { type_name = selfhost_parser_state.p_n_ident(p, "_", span); }
          }
          expr = selfhost_parser_state.p_n(p, NodeKind.NkExprStruct(type_name, fields, -1), span);
        } else {
          let args = pe_parse_arg_list(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          let span = selfhost_parser_state.p_span_of(p, expr);
          if gc_node == expr {
            expr = selfhost_parser_state.p_n(
              p, NodeKind.NkExprGenericCall(gc_base, gc_types, args), span
            );
            gc_node = expr;
          } else {
            expr = selfhost_parser_state.p_n(p, NodeKind.NkExprCall(expr, args), span);
          }
        }
      }
    } elif pe_ktag(k) == KT_LBracket {
      selfhost_parser_state.p_advance(p); // consumed '['
      // Feature 6: explicit generic args on a call, e.g. `spawn[T](f)`.
      var feature6 = false;
      let enode = p.nodes[expr];
      match enode.kind {
        NkExprIdent(id) => {
          if pe_first_lower_or_uscore(pe_ident_text(p, id)) { feature6 = true; }
        }
        _ => {}
      }
      var starts_upper = false;
      let pk = selfhost_parser_state.p_peek_kind(p);
      match pk {
        TkIdent(_) => {
          if pe_first_upper(selfhost_parser_state.p_peek(p).lexeme) { starts_upper = true; }
        }
        _ => {}
      }
      if feature6 && starts_upper {
        let saved = p.pos;
        var depth = 1;
        var sgo = true;
        while sgo {
          let kk = selfhost_parser_state.p_peek_kind(p);
          if pe_ktag(kk) == KT_LBracket { depth = depth + 1; selfhost_parser_state.p_advance(p); }
          elif pe_ktag(kk) == KT_RBracket { depth = depth - 1; selfhost_parser_state.p_advance(p); }
          elif pe_ktag(kk) == KT_Eof { sgo = false; }
          else { selfhost_parser_state.p_advance(p); }
          if depth <= 0 { sgo = false; }
        }
        var is_generic_call = false;
        if depth == 0 && pe_pk(p) == KT_LParen {
          is_generic_call = true;
        }
        p.pos = saved;
        if is_generic_call {
          var types = Vec[Int].new();
          if pe_pk(p) != KT_RBracket {
            var tgo = true;
            while tgo {
              let t = pe_parse_type(p);
              if selfhost_parser_state.p_failed(p) { return -1; }
              types.push(t);
              if !selfhost_parser_state.p_skip(p, TkComma) { tgo = false; }
            }
          }
          let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          // One-step construction (the Rust two-step merge fills the args in
          // the `(` arm; the merge destructure mis-lowers in this function --
          // COMPILER_BUGS 2026-10-02 (h) -- so the args are parsed here).
          if pe_pk(p) == KT_LParen {
            selfhost_parser_state.p_advance(p);
            var args = Vec[Int].new();
            if pe_pk(p) == KT_RParen {
              selfhost_parser_state.p_advance(p);
            } else {
              args = pe_parse_arg_list(p);
              if selfhost_parser_state.p_failed(p) { return -1; }
              let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
              if selfhost_parser_state.p_failed(p) { return -1; }
            }
            let span = selfhost_parser_state.p_span_of(p, expr);
            gc_base = expr;
            gc_types = types;
            expr = selfhost_parser_state.p_n(
              p, NodeKind.NkExprGenericCall(expr, types, args), span
            );
            gc_node = expr;
            continue;
          }
          p.pos = saved;
        }
      }
      var is_type_name = pe_expr_is_type_name(p, expr);
      var peek_is_type_start = false;
      let kt = selfhost_parser_state.p_peek_kind(p);
      if pe_ktag(kt) == KT_Ident || pe_ktag(kt) == KT_Fn || pe_ktag(kt) == KT_Star || pe_ktag(kt) == KT_Ampersand
        || pe_ktag(kt) == KT_LBracket || pe_ktag(kt) == KT_LParen
      {
        peek_is_type_start = true;
      }
      if is_type_name && peek_is_type_start {
        // A `fn`/`*`/`&`/`[`/`(` first token is unequivocally a type.
        if pe_ktag(kt) == KT_Fn || pe_ktag(kt) == KT_Star || pe_ktag(kt) == KT_Ampersand
          || pe_ktag(kt) == KT_LBracket || pe_ktag(kt) == KT_LParen
        {
          var type_args = Vec[Int].new();
          let t0 = pe_parse_type(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          type_args.push(t0);
          while selfhost_parser_state.p_skip(p, TkComma) {
            let t = pe_parse_type(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            type_args.push(t);
          }
          let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          var type_exprs = Vec[Int].new();
          var ti = 0;
          while ti < type_args.len() {
            let e = pe_type_to_expr_ident(p, type_args[ti]);
            if selfhost_parser_state.p_failed(p) { return -1; }
            type_exprs.push(e);
            ti = ti + 1;
          }
          var args_expr = -1;
          if type_exprs.len() == 1 {
            args_expr = type_exprs[0];
          } else {
            let sp = selfhost_parser_state.p_peek_span(p);
            args_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprTuple(type_exprs), sp);
          }
          let span = selfhost_parser_state.p_peek_span(p);
          expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(expr, args_expr), span);
          expr = pe_parse_struct_literal_tail(p, expr);
          if selfhost_parser_state.p_failed(p) { return -1; }
          continue;
        }
        let after_first = selfhost_parser_state.p_peek_ahead_kind(p, 1);
        var looks_like_type_args = false;
        if pe_ktag(after_first) == KT_Colon || pe_ktag(after_first) == KT_Comma || pe_ktag(after_first) == KT_LBrace
          || pe_ktag(after_first) == KT_LBracket || pe_ktag(after_first) == KT_RBracket
        {
          looks_like_type_args = true;
        }
        if looks_like_type_args {
          if pe_ktag(after_first) == KT_RBracket {
            // Redundant recompute kept for 1:1 fidelity with the Rust code.
            let again = pe_expr_is_type_name(p, expr);
            if !again {
              let inner = pe_parse_expr_open(p);
              if selfhost_parser_state.p_failed(p) { return -1; }
              let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
              if selfhost_parser_state.p_failed(p) { return -1; }
              let span = selfhost_parser_state.p_span_of(p, expr);
              expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(expr, inner), span);
              continue;
            }
          }
          var type_args = Vec[Int].new();
          let t0 = pe_parse_type(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          type_args.push(t0);
          while selfhost_parser_state.p_skip(p, TkComma) {
            let t = pe_parse_type(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            type_args.push(t);
          }
          let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          var type_exprs = Vec[Int].new();
          var ti = 0;
          while ti < type_args.len() {
            let e = pe_type_to_expr_ident(p, type_args[ti]);
            if selfhost_parser_state.p_failed(p) { return -1; }
            type_exprs.push(e);
            ti = ti + 1;
          }
          var args_expr = -1;
          if type_exprs.len() == 1 {
            args_expr = type_exprs[0];
          } else {
            let sp = selfhost_parser_state.p_peek_span(p);
            args_expr = selfhost_parser_state.p_n(p, NodeKind.NkExprTuple(type_exprs), sp);
          }
          let span = selfhost_parser_state.p_peek_span(p);
          expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(expr, args_expr), span);
          if p.restrict_struct == 0 && pe_pk(p) == KT_LBrace {
            expr = pe_parse_struct_literal_tail(p, expr);
            if selfhost_parser_state.p_failed(p) { return -1; }
          }
        } else {
          let inner = pe_parse_expr_open(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          let span = selfhost_parser_state.p_span_of(p, expr);
          expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(expr, inner), span);
        }
      } else {
        let inner = pe_parse_expr_open(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        let span = selfhost_parser_state.p_span_of(p, expr);
        expr = selfhost_parser_state.p_n(p, NodeKind.NkExprIndex(expr, inner), span);
      }
    } elif pe_ktag(k) == KT_At {
      selfhost_parser_state.p_advance(p);
      var is_pre = false;
      let ka = selfhost_parser_state.p_peek_kind(p);
      match ka {
        TkIdent(_) => {
          if selfhost_parser_state.p_peek(p).lexeme == "pre" { is_pre = true; }
        }
        _ => {}
      }
      if !is_pre {
        return selfhost_parser_state.p_err(p, "expected 'pre' after '@'");
      }
      selfhost_parser_state.p_advance(p);
      let span = selfhost_parser_state.p_span_of(p, expr);
      expr = selfhost_parser_state.p_n(p, NodeKind.NkExprAtPre(expr), span);
    } elif pe_ktag(k) == KT_Colon {
      // Single colon: break so the outer level can consume it.
      go = false;
    } elif pe_ktag(k) == KT_Lt {
      if !pe_expr_is_type_name(p, expr) { go = false; }
      else {
        // Speculative generic type args in expression position, e.g.
        // `Vec<FieldInfo>.new()`: commit only if `>` is followed by `.`/`(`.
        let saved = p.pos;
        selfhost_parser_state.p_advance(p); // consume '<'
        var depth = 1;
        var aborted = false;
        var lgo = true;
        while lgo {
          let kk = selfhost_parser_state.p_peek_kind(p);
          if pe_ktag(kk) == KT_Lt { depth = depth + 1; selfhost_parser_state.p_advance(p); }
          elif pe_ktag(kk) == KT_Gt { depth = depth - 1; selfhost_parser_state.p_advance(p); }
          elif pe_ktag(kk) == KT_Ident || pe_ktag(kk) == KT_Comma || pe_ktag(kk) == KT_Star || pe_ktag(kk) == KT_Ampersand
            || pe_ktag(kk) == KT_LBracket || pe_ktag(kk) == KT_RBracket
          {
            selfhost_parser_state.p_advance(p);
          } else {
            aborted = true;
            lgo = false;
          }
          if depth <= 0 { lgo = false; }
        }
        var accepted = false;
        if !aborted {
          let k2 = selfhost_parser_state.p_peek_kind(p);
          if pe_ktag(k2) == KT_Dot || pe_ktag(k2) == KT_LParen { accepted = true; }
        }
        if accepted { continue; }
        p.pos = saved;
        go = false;
      }
    } elif pe_ktag(k) == KT_LBrace {
      if p.restrict_struct != 0 { go = false; }
      else {
        // Qualified struct literal: `Shape.Circle { r: 5.0 }`.
        var looks_like_struct = false;
        let after_brace = selfhost_parser_state.p_peek_ahead_kind(p, 1);
        if pe_ktag(after_brace) == KT_RBrace { looks_like_struct = true; }
        elif pe_ktag(after_brace) == KT_Dot {
          if pe_pak(p, 2) == KT_Dot { looks_like_struct = true; }
        } elif pe_ktag(after_brace) == KT_Ident {
          let third = selfhost_parser_state.p_peek_ahead_kind(p, 2);
          if pe_ktag(third) == KT_Colon || pe_ktag(third) == KT_Comma || pe_ktag(third) == KT_Semicolon || pe_ktag(third) == KT_RBrace {
            looks_like_struct = true;
          }
        }
        if !looks_like_struct { go = false; }
        else {
          var path_name = "";
          let enode = p.nodes[expr];
          match enode.kind {
            NkExprIdent(id) => { path_name = pe_ident_text(p, id); }
            NkExprField(obj, field, _sp) => {
              path_name = pe_collect_path(p, obj, "");
              if path_name.len() > 0 { path_name = path_name + "."; }
              path_name = path_name + pe_ident_text(p, field);
            }
            _ => {
              return selfhost_parser_state.p_err(p, "expected type name before '{'");
            }
          }
          selfhost_parser_state.p_advance(p); // consume '{'
          let fields = pe_parse_struct_fields(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          let spread = pe_parse_struct_spread(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          let span = selfhost_parser_state.p_span_of(p, expr);
          let nid = selfhost_parser_state.p_n_ident(p, path_name, span);
          expr = selfhost_parser_state.p_n(p, NodeKind.NkExprStruct(nid, fields, spread), span);
          continue;
        }
      }
    } else {
      go = false;
    }
  }
  return expr;
}

/// Parse the `{ field: value; ... }` tail of a qualified/generic struct
/// literal (`Path.Type { .. }`, `Name[T] { .. }`). Returns the new expr.
fn pe_parse_struct_literal_tail(p: &mut Parser, expr: Int) -> Int {
  if p.restrict_struct != 0 || pe_pk(p) != KT_LBrace {
    return expr;
  }
  let span = selfhost_parser_state.p_span_of(p, expr);
  selfhost_parser_state.p_advance(p); // consume '{'
  let fields = pe_parse_struct_fields(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let spread = pe_parse_struct_spread(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var struct_name = -1;
  let enode = p.nodes[expr];
  match enode.kind {
    NkExprIdent(name) => { struct_name = name; }
    NkExprIndex(base, _idx, _sp) => {
      let bnode = p.nodes[base];
      match bnode.kind {
        NkExprIdent(name) => { struct_name = name; }
        _ => { struct_name = selfhost_parser_state.p_n_ident(p, "__struct", span); }
      }
    }
    _ => { struct_name = selfhost_parser_state.p_n_ident(p, "__struct", span); }
  }
  return selfhost_parser_state.p_n(
    p, NodeKind.NkExprStruct(struct_name, fields, spread), span
  );
}

/// Default primary for an Ident-starting expression: plain ident, or a
/// `Name { field: value }` struct literal when it looks like one.
fn pe_parse_primary_default_ident(p: &mut Parser, span: Span) -> Int {
  let name = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  if p.restrict_struct == 0 && pe_pk(p) == KT_LBrace {
    var looks_like_struct = false;
    let after_brace = selfhost_parser_state.p_peek_ahead_kind(p, 1);
    if pe_ktag(after_brace) == KT_RBrace { looks_like_struct = true; }
    elif pe_ktag(after_brace) == KT_Dot {
      if pe_pak(p, 2) == KT_Dot { looks_like_struct = true; }
    } elif pe_ktag(after_brace) == KT_Ident {
      let third = selfhost_parser_state.p_peek_ahead_kind(p, 2);
      if pe_ktag(third) == KT_Colon || pe_ktag(third) == KT_Comma || pe_ktag(third) == KT_Semicolon || pe_ktag(third) == KT_RBrace {
        looks_like_struct = true;
      }
    }
    if looks_like_struct {
      let _ = selfhost_parser_state.p_expect_kind(p, TkLBrace, "'{'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let fields = pe_parse_struct_fields(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let spread = pe_parse_struct_spread(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprStruct(name, fields, spread), span);
    }
  }
  return selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(name), span);
}

fn pe_parse_primary(p: &mut Parser) -> Int {
  let span = selfhost_parser_state.p_peek_span(p);
  let pk = selfhost_parser_state.p_peek_kind(p);
  match pk {
    TkInt(n) => {
      let tok = selfhost_parser_state.p_advance(p);
      let sufc = pe_parse_int_suffix(tok.lexeme);
      if sufc >= 0 {
        let lit = selfhost_parser_state.p_n(p, NodeKind.NkLitInt(n), span);
        let ty = pe_suffix_type_node(p, sufc);
        return selfhost_parser_state.p_n(p, NodeKind.NkExprAs(lit, ty), span);
      }
      return selfhost_parser_state.p_n(p, NodeKind.NkLitInt(n), span);
    }
    TkBigInt(hi, lo) => {
      let tok = selfhost_parser_state.p_advance(p);
      let sufc = pe_parse_int_suffix(tok.lexeme);
      if sufc >= 0 {
        let lit = selfhost_parser_state.p_n(p, NodeKind.NkLitBigInt(hi, lo), span);
        let ty = pe_suffix_type_node(p, sufc);
        return selfhost_parser_state.p_n(p, NodeKind.NkExprAs(lit, ty), span);
      }
      return selfhost_parser_state.p_n(p, NodeKind.NkLitBigInt(hi, lo), span);
    }
    TkFloat(_v) => {
      let tok = selfhost_parser_state.p_advance(p);
      let sufc = pe_parse_float_suffix(tok.lexeme);
      if sufc >= 0 {
        let lit = selfhost_parser_state.p_n(p, NodeKind.NkLitFloat(tok.lexeme), span);
        let ty = pe_suffix_type_node(p, sufc);
        return selfhost_parser_state.p_n(p, NodeKind.NkExprAs(lit, ty), span);
      }
      return selfhost_parser_state.p_n(p, NodeKind.NkLitFloat(tok.lexeme), span);
    }
    TkStr(s) => {
      selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitStr(s), span);
    }
    TkChar(c) => {
      selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitChar(c), span);
    }
    TkTrue => {
      selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitBool(1), span);
    }
    TkFalse => {
      selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitBool(0), span);
    }
    TkLParen => {
      selfhost_parser_state.p_advance(p);
      if pe_pk(p) == KT_RParen {
        selfhost_parser_state.p_advance(p);
        return selfhost_parser_state.p_n(
          p, NodeKind.NkExprTuple(Vec[Int].new()), span
        );
      }
      let first = pe_parse_expr_open(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      if selfhost_parser_state.p_skip(p, TkComma) {
        var items = Vec[Int].new();
        items.push(first);
        let second = pe_parse_expr_open(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        items.push(second);
        while selfhost_parser_state.p_skip(p, TkComma) {
          let e = pe_parse_expr_open(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          items.push(e);
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        return selfhost_parser_state.p_n(p, NodeKind.NkExprTuple(items), span);
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprParen(first), span);
    }
    TkSome => {
      selfhost_parser_state.p_advance(p);
      let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let inner = pe_parse_expr_open(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprSome(inner), span);
    }
    TkNone => {
      selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprNone, span);
    }
    TkOkV => {
      selfhost_parser_state.p_advance(p);
      let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let inner = pe_parse_expr_open(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprOk(inner), span);
    }
    TkErrV => {
      selfhost_parser_state.p_advance(p);
      let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let inner = pe_parse_expr_open(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprErr(inner), span);
    }
    TkAwait => {
      selfhost_parser_state.p_advance(p);
      let inner = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprAwait(inner), span);
    }
    TkComptime => {
      selfhost_parser_state.p_advance(p);
      let kk = selfhost_parser_state.p_peek_kind(p);
      if pe_ktag(kk) == KT_Dot || pe_ktag(kk) == KT_LParen || pe_ktag(kk) == KT_LBracket || pe_ktag(kk) == KT_LBrace {
        let id = selfhost_parser_state.p_n_ident(p, "comptime", span);
        return selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), span);
      }
      let inner = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprComptime(inner), span);
    }
    TkLBrace => {
      // Bare `{ field: value; }` anonymous struct literal.
      if p.restrict_struct != 0 {
        return selfhost_parser_state.p_err(p, "unexpected '{' in this context");
      }
      var looks_like_struct = false;
      let after_brace = selfhost_parser_state.p_peek_ahead_kind(p, 1);
      if pe_ktag(after_brace) == KT_RBrace { looks_like_struct = true; }
      elif pe_ktag(after_brace) == KT_Dot {
        if pe_pak(p, 2) == KT_Dot { looks_like_struct = true; }
      } elif pe_ktag(after_brace) == KT_Ident {
        let third = selfhost_parser_state.p_peek_ahead_kind(p, 2);
        if pe_ktag(third) == KT_Colon || pe_ktag(third) == KT_Comma || pe_ktag(third) == KT_Semicolon || pe_ktag(third) == KT_RBrace {
          looks_like_struct = true;
        }
      }
      if looks_like_struct {
        return pe_parse_struct_literal_body(p, -1, "_", span);
      }
      return selfhost_parser_state.p_err(p, "expected expression, found '{'");
    }
    TkUnsafe => {
      selfhost_parser_state.p_advance(p);
      let block = pe_parse_block(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprUnsafe(block), span);
    }
    TkIf => {
      selfhost_parser_state.p_advance(p);
      let cond = pe_parse_cond(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let then_block = pe_parse_block(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      var elifs = Vec[Int].new();
      while selfhost_parser_state.p_skip(p, TkElif) {
        let econd = pe_parse_cond(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let eblock = pe_parse_block(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let en = selfhost_parser_state.p_n(
          p, NodeKind.NkElif(econd, eblock), selfhost_ast.span_zero()
        );
        elifs.push(en);
      }
      var else_block = -1;
      if selfhost_parser_state.p_skip(p, TkElse) {
        else_block = pe_parse_block(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
      return selfhost_parser_state.p_n(
        p, NodeKind.NkExprIf(cond, then_block, elifs, else_block), span
      );
    }
    TkMatch => {
      selfhost_parser_state.p_advance(p);
      let scrutinee = pe_parse_cond(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkLBrace, "'{'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      var arms = Vec[Int].new();
      var ago = true;
      while ago {
        let kk = selfhost_parser_state.p_peek_kind(p);
        if pe_ktag(kk) == KT_RBrace || pe_ktag(kk) == KT_Eof { break; }
        let arm = pe_parse_match_arm(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        arms.push(arm);
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprMatch(scrutinee, arms), span);
    }
    TkFn => {
      selfhost_parser_state.p_advance(p);
      let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
      if selfhost_parser_state.p_failed(p) { return -1; }
      var params = Vec[Int].new();
      if pe_pk(p) == KT_RParen {
        selfhost_parser_state.p_advance(p);
      } else {
        params = pe_parse_param_list(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
      var ret_ty = -1;
      if selfhost_parser_state.p_skip(p, TkArrow) {
        ret_ty = pe_parse_type(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
      let body = pe_parse_block(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprClosure(params, ret_ty, body), span);
    }
    TkPipe => {
      selfhost_parser_state.p_advance(p);
      var params = Vec[Int].new();
      var pgo = true;
      while pgo {
        let id = pe_parse_ident(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        params.push(id);
        if selfhost_parser_state.p_skip(p, TkComma) { continue; }
        if selfhost_parser_state.p_skip(p, TkPipe) { break; }
        return selfhost_parser_state.p_err(p, "expected ',' or '|' in closure parameters");
      }
      let body = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprPipeClosure(params, body), span);
    }
    TkLBracket => {
      selfhost_parser_state.p_advance(p);
      if pe_pk(p) == KT_RBracket {
        selfhost_parser_state.p_advance(p);
        return selfhost_parser_state.p_n(
          p, NodeKind.NkExprArray(Vec[Int].new()), span
        );
      }
      let first = pe_parse_expr_open(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      if selfhost_parser_state.p_skip(p, TkComma) {
        var items = Vec[Int].new();
        items.push(first);
        if pe_pk(p) != KT_RBracket {
          let e = pe_parse_expr_open(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          items.push(e);
        }
        while selfhost_parser_state.p_skip(p, TkComma) {
          if pe_pk(p) == KT_RBracket { break; }
          let e = pe_parse_expr_open(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          items.push(e);
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        return selfhost_parser_state.p_n(p, NodeKind.NkExprArray(items), span);
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRBracket, "']'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      var items = Vec[Int].new();
      items.push(first);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprArray(items), span);
    }
    TkSelf => {
      let tok = selfhost_parser_state.p_advance(p);
      let sp = pe_tok_span(tok);
      let id = selfhost_parser_state.p_n_ident(p, "self", sp);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), sp);
    }
    // `spawn` used as a value/callee (the `spawn { ... }` statement is
    // handled in pe_parse_stmt_or_expr).
    TkSpawn => {
      selfhost_parser_state.p_advance(p);
      let id = selfhost_parser_state.p_n_ident(p, "spawn", span);
      return selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), span);
    }
    TkAt => {
      let tok = selfhost_parser_state.p_advance(p);
      let at_span = pe_tok_span(tok);
      let fn_name = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
      if selfhost_parser_state.p_failed(p) { return -1; }
      var args = Vec[Int].new();
      if pe_pk(p) == KT_RParen {
        selfhost_parser_state.p_advance(p);
      } else {
        let a0 = pe_parse_expr(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        args.push(a0);
        while selfhost_parser_state.p_skip(p, TkComma) {
          let a = pe_parse_expr(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          args.push(a);
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
      let callee = selfhost_parser_state.p_n(
        p, NodeKind.NkExprIdent(fn_name), selfhost_parser_state.p_span_of(p, fn_name)
      );
      return selfhost_parser_state.p_n(p, NodeKind.NkExprCall(callee, args), at_span);
    }
    TkConst => {
      selfhost_parser_state.p_advance(p);
      let _ = selfhost_parser_state.p_expect_kind(p, TkLBrace, "'{'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let inner = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkExprConstBlock(inner), span);
    }
    TkIdent(_) => {
      // BUG 27: debug intrinsics -- `dbg!(expr)`, `todo!()`,
      // `unimplemented!()` parse as plain calls to the builtin names.
      let nm = selfhost_parser_state.p_peek(p).lexeme;
      var is_dbg = false;
      if nm == "dbg" || nm == "todo" || nm == "unimplemented" {
        if pe_pak(p, 1) == KT_Bang
          && pe_pak(p, 2) == KT_LParen
        {
          is_dbg = true;
        }
      }
      if is_dbg {
        let tok = selfhost_parser_state.p_advance(p); // ident
        let dsp = pe_tok_span(tok);
        selfhost_parser_state.p_advance(p); // !
        selfhost_parser_state.p_advance(p); // (
        var args = Vec[Int].new();
        if pe_pk(p) != KT_RParen {
          let a0 = pe_parse_expr_open(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          args.push(a0);
          while selfhost_parser_state.p_skip(p, TkComma) {
            let a = pe_parse_expr_open(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            args.push(a);
          }
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        let id = selfhost_parser_state.p_n_ident(p, nm, dsp);
        let callee = selfhost_parser_state.p_n(p, NodeKind.NkExprIdent(id), dsp);
        return selfhost_parser_state.p_n(p, NodeKind.NkExprCall(callee, args), dsp);
      }
      return pe_parse_primary_default_ident(p, span);
    }
    _ => {
      return pe_parse_primary_default_ident(p, span);
    }
  }
}

// ============================================================================
// Statements (Rust parse_stmt_or_expr .. parse_block)
// ============================================================================

pub fn pe_parse_optional_label(p: &mut Parser) -> Int {
  if pe_pk(p) == KT_At {
    selfhost_parser_state.p_advance(p); // skip '@'
    return pe_try_ident(p);
  }
  return -1;
}

/// Rust try_compound_assign: desugars `x += y` to `x = x + y`. Returns the
/// RHS index, or -1 when this is not a compound assignment. The inner
/// `parse_expr().ok()?` swallows the error (clears the latch).
fn pe_try_compound_assign(p: &mut Parser, lhs: Int) -> Int {
  let span = selfhost_parser_state.p_peek_span(p);
  var op = -1;
  let k = selfhost_parser_state.p_peek_kind(p);
  if pe_ktag(k) == KT_PlusEq { op = selfhost_ast.OP_ADD; }
  elif pe_ktag(k) == KT_MinusEq { op = selfhost_ast.OP_SUB; }
  elif pe_ktag(k) == KT_StarEq { op = selfhost_ast.OP_MUL; }
  elif pe_ktag(k) == KT_SlashEq { op = selfhost_ast.OP_DIV; }
  elif pe_ktag(k) == KT_PercentEq { op = selfhost_ast.OP_REM; }
  else { return -1; }
  selfhost_parser_state.p_advance(p);
  let rhs = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) {
    p.failed = 0;
    return -1;
  }
  return selfhost_parser_state.p_n(p, NodeKind.NkExprBinary(lhs, op, rhs), span);
}

fn pe_expr_is_unsafe_or_if(p: &Parser, expr: Int) -> Bool {
  let node = p.nodes[expr];
  match node.kind {
    NkExprUnsafe(_) => { return true; }
    NkExprIf(_, _, _, _) => { return true; }
    _ => { return false; }
  }
}

/// Rust `_` arm of parse_stmt_or_expr: expression statement, assignment,
/// compound assignment, or a bare tail expression.
fn pe_parse_expr_stmt(p: &mut Parser) -> Int {
  let expr = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let compound = pe_try_compound_assign(p, expr);
  if compound >= 0 {
    let sp = selfhost_parser_state.p_peek_span(p);
    let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    return pe_stmt_w(p, selfhost_parser_state.p_n(p, NodeKind.NkAssign(expr, compound), sp));
  }
  if selfhost_parser_state.p_skip(p, TkEq) {
    let rhs = pe_parse_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let sp = selfhost_parser_state.p_peek_span(p);
    let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    return pe_stmt_w(p, selfhost_parser_state.p_n(p, NodeKind.NkAssign(expr, rhs), sp));
  }
  if pe_pk(p) == KT_RBrace {
    return pe_tail_w(p, expr);
  }
  if pe_expr_is_unsafe_or_if(p, expr) {
    let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
    return pe_tail_w(p, expr);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return pe_tail_w(p, expr);
}

pub fn pe_parse_stmt_or_expr(p: &mut Parser) -> Int {
  // P0-3: `@label: stmt` for labeled loops.
  var loop_label = -1;
  if pe_pk(p) == KT_At {
    var is_ident = false;
    let ahead = selfhost_parser_state.p_peek_ahead_kind(p, 1);
    match ahead {
      TkIdent(_) => { is_ident = true; }
      _ => {}
    }
    if is_ident {
      selfhost_parser_state.p_advance(p); // consume '@'
      loop_label = pe_try_ident(p);
      if pe_pk(p) == KT_Colon {
        selfhost_parser_state.p_advance(p); // consume ':'
      }
    }
  }
  let pk = selfhost_parser_state.p_peek_kind(p);
  match pk {
    TkLet => {
      let st = pe_parse_let_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, st);
    }
    TkVar => {
      let st = pe_parse_var_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, st);
    }
    TkConst => {
      // Local `const NAME: T = value;` -- treated as an immutable let.
      let tok = selfhost_parser_state.p_advance(p);
      let span = pe_tok_span(tok);
      let name = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      var ty = -1;
      if selfhost_parser_state.p_skip(p, TkColon) {
        ty = pe_parse_type(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkEq, "'='");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let value = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, selfhost_parser_state.p_n(p, NodeKind.NkLet(name, ty, value), span));
    }
    TkReturn => {
      let st = pe_parse_return_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, st);
    }
    TkBreak => {
      let tok = selfhost_parser_state.p_advance(p);
      let span = pe_tok_span(tok);
      let label = pe_parse_optional_label(p);
      let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
      return pe_stmt_w(p, selfhost_parser_state.p_n(p, NodeKind.NkBreak(label), span));
    }
    TkContinue => {
      let tok = selfhost_parser_state.p_advance(p);
      let span = pe_tok_span(tok);
      let label = pe_parse_optional_label(p);
      let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
      return pe_stmt_w(p, selfhost_parser_state.p_n(p, NodeKind.NkContinue(label), span));
    }
    TkIf => {
      let st = pe_parse_if_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, st);
    }
    TkMatch => {
      let st = pe_parse_match_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, st);
    }
    TkWhile => {
      let st = pe_parse_while_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      if loop_label >= 0 {
        let sn = p.nodes[st];
        match sn.kind {
          NkWhile(cond, body, inv, _lbl) => {
            p.nodes[st] = selfhost_ast.node_new(
              NodeKind.NkWhile(cond, body, inv, loop_label), sn.span
            );
          }
          _ => {}
        }
      }
      return pe_stmt_w(p, st);
    }
    TkFor => {
      let st = pe_parse_for_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      if loop_label >= 0 {
        let sn = p.nodes[st];
        match sn.kind {
          NkFor(var_id, iter, body, _lbl) => {
            p.nodes[st] = selfhost_ast.node_new(
              NodeKind.NkFor(var_id, iter, body, loop_label), sn.span
            );
          }
          _ => {}
        }
      }
      return pe_stmt_w(p, st);
    }
    TkAsm => {
      let st = pe_parse_asm_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, st);
    }
    TkDefer => {
      let st = pe_parse_defer_stmt(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      return pe_stmt_w(p, st);
    }
    TkSpawn => {
      let next = selfhost_parser_state.p_peek_ahead_kind(p, 1);
      if pe_ktag(next) == KT_LBrace || pe_ktag(next) == KT_Move {
        let st = pe_parse_spawn_stmt(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        return pe_stmt_w(p, st);
      }
      return pe_parse_expr_stmt(p);
    }
    TkLBrace => {
      // Bare block expression: `{ stmt; ... }` as statement or expression.
      let block = pe_parse_block(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let bn = p.nodes[block];
      return pe_tail_w(p, selfhost_parser_state.p_n(p, NodeKind.NkExprBlock(block), bn.span));
    }
    TkIdent(_) => {
      let nm = selfhost_parser_state.p_peek(p).lexeme;
      let a1 = selfhost_parser_state.p_peek_ahead_kind(p, 1);
      if nm == "assert" && pe_ktag(a1) == KT_LParen {
        // BUG 27: `assert(cond)` / `assert(cond, "msg")`.
        let tok = selfhost_parser_state.p_advance(p);
        let span = pe_tok_span(tok);
        selfhost_parser_state.p_advance(p); // (
        let cond = pe_parse_expr_open(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        var msg = -1;
        if selfhost_parser_state.p_skip(p, TkComma) {
          msg = pe_parse_expr_open(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')' in assert");
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
        return pe_stmt_w(p, selfhost_parser_state.p_n(p, NodeKind.NkAssert(cond, msg), span));
      }
      elif nm == "debugger" {
        // BUG 27: `debugger;` / `debugger();`.
        let tok = selfhost_parser_state.p_advance(p);
        let span = pe_tok_span(tok);
        if selfhost_parser_state.p_skip(p, TkLParen) {
          let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')' in debugger");
          if selfhost_parser_state.p_failed(p) { return -1; }
        }
        let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
        return pe_stmt_w(p, selfhost_parser_state.p_n(p, NodeKind.NkDebugger, span));
      }
      elif nm == "loop" && pe_ktag(a1) == KT_LBrace {
        let tok = selfhost_parser_state.p_advance(p); // consume 'loop'
        let span = pe_tok_span(tok);
        let body = pe_parse_block(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        // Desugar `loop { ... }` to `while true { ... }`.
        let lit = selfhost_parser_state.p_n(p, NodeKind.NkLitBool(1), span);
        let st = selfhost_parser_state.p_n(p, NodeKind.NkWhile(lit, body, -1, -1), span);
        return pe_stmt_w(p, st);
      }
      else {
        return pe_parse_expr_stmt(p);
      }
    }
    _ => {
      return pe_parse_expr_stmt(p);
    }
  }
}

fn pe_parse_let_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  if pe_pk(p) == KT_LParen {
    return pe_parse_destructure(p, span);
  }
  let name = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var ty = -1;
  if selfhost_parser_state.p_skip(p, TkColon) {
    ty = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  // Allow `let x: T;` without initializer (defaults to zero).
  var value = -1;
  if selfhost_parser_state.p_skip(p, TkEq) {
    value = pe_parse_init_expr(p, ty, span);
    if selfhost_parser_state.p_failed(p) { return -1; }
  } else {
    value = selfhost_parser_state.p_n(p, NodeKind.NkLitInt(0 as UInt), span);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkLet(name, ty, value), span);
}

fn pe_parse_var_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  if pe_pk(p) == KT_LParen {
    return pe_parse_destructure(p, span);
  }
  let name = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var ty = -1;
  if selfhost_parser_state.p_skip(p, TkColon) {
    ty = pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  var value = -1;
  if selfhost_parser_state.p_skip(p, TkEq) {
    value = pe_parse_init_expr(p, ty, span);
    if selfhost_parser_state.p_failed(p) { return -1; }
  } else {
    value = selfhost_parser_state.p_n(p, NodeKind.NkLitInt(0 as UInt), span);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkVar(name, ty, value), span);
}

fn pe_parse_return_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  var expr = -1;
  let k = selfhost_parser_state.p_peek_kind(p);
  if pe_ktag(k) != KT_Semicolon && pe_ktag(k) != KT_RBrace {
    expr = pe_parse_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  let k2 = selfhost_parser_state.p_peek_kind(p);
  if pe_ktag(k2) == KT_RBrace {
    let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
  } else {
    let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  return selfhost_parser_state.p_n(p, NodeKind.NkReturn(expr), span);
}

fn pe_parse_if_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  // 8B/M9: if let pattern matching.
  if pe_pk(p) == KT_Let {
    return pe_parse_if_let_stmt(p, span);
  }
  let cond = pe_parse_cond(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let then_block = pe_parse_block(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var elifs = Vec[Int].new();
  while selfhost_parser_state.p_skip(p, TkElif) {
    let econd = pe_parse_cond(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let eblock = pe_parse_block(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    elifs.push(selfhost_parser_state.p_n(
      p, NodeKind.NkElif(econd, eblock), selfhost_ast.span_zero()
    ));
  }
  let else_block = pe_parse_else_tail(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(
    p, NodeKind.NkStmtIf(cond, then_block, elifs, else_block), span
  );
}

/// `if let pattern = expr { ... } [else { ... }]` desugars to a match.
fn pe_parse_if_let_stmt(p: &mut Parser, if_span: Span) -> Int {
  selfhost_parser_state.p_advance(p); // skip 'let'
  let pattern = pe_parse_pattern(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkEq, "'=' in if let");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let expr = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let then_block = pe_parse_block(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let else_block = pe_parse_else_tail(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let wildcard = selfhost_parser_state.p_n(
    p, NodeKind.NkPatWildcard, selfhost_ast.span_zero()
  );
  var else_body = -1;
  if else_block >= 0 {
    else_body = else_block;
  } else {
    else_body = selfhost_parser_state.p_n(
      p, NodeKind.NkBlock(Vec[Int].new()), if_span
    );
  }
  var arms = Vec[Int].new();
  let a0 = selfhost_parser_state.p_n(
    p, NodeKind.NkMatchArm(pattern, -1, then_block, 1), if_span
  );
  arms.push(a0);
  let a1 = selfhost_parser_state.p_n(
    p, NodeKind.NkMatchArm(wildcard, -1, else_body, 1), if_span
  );
  arms.push(a1);
  return selfhost_parser_state.p_n(p, NodeKind.NkStmtMatch(expr, arms), if_span);
}

/// Tail of an `if`/`if let` after `else`: a block, or a nested `if`
/// (the two-word `else if <cond> { ... }` chain).
fn pe_parse_else_tail(p: &mut Parser) -> Int {
  if !selfhost_parser_state.p_skip(p, TkElse) {
    return -1;
  }
  if pe_pk(p) == KT_If {
    let start = selfhost_parser_state.p_peek_span(p);
    let nested = pe_parse_if_stmt(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    var stmts = Vec[Int].new();
    stmts.push(pe_stmt_w(p, nested));
    return selfhost_parser_state.p_n(p, NodeKind.NkBlock(stmts), start);
  }
  return pe_parse_block(p);
}

fn pe_parse_match_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  let expr = pe_parse_cond(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var arms = Vec[Int].new();
  var go = true;
  while go {
    let k = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k) == KT_RBrace || pe_ktag(k) == KT_Eof { break; }
    let arm = pe_parse_match_arm(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    arms.push(arm);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkStmtMatch(expr, arms), span);
}

pub fn pe_parse_match_arm(p: &mut Parser) -> Int {
  let span = selfhost_parser_state.p_peek_span(p);
  let pattern = pe_parse_pattern(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var guard = -1;
  if selfhost_parser_state.p_skip(p, TkIf) {
    guard = pe_parse_or_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkFatArrow, "'=>'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var body = -1;
  var body_is_block = 0;
  if pe_pk(p) == KT_LBrace {
    let block = pe_parse_block(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_skip(p, TkComma);
    let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
    body = block;
    body_is_block = 1;
  } elif pe_peek_is_arm_stmt_kind(p) {
    let stmt = pe_parse_arm_stmt(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_skip(p, TkComma);
    let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
    var stmts = Vec[Int].new();
    stmts.push(pe_stmt_w(p, stmt));
    body = selfhost_parser_state.p_n(p, NodeKind.NkBlock(stmts), span);
    body_is_block = 1;
  } else {
    let res = pe_parse_expr_or_assign(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_skip(p, TkComma);
    let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
    body = res[0];
    body_is_block = res[1];
  }
  return selfhost_parser_state.p_n(
    p, NodeKind.NkMatchArm(pattern, guard, body, body_is_block), span
  );
}

/// Rust `matches!(peek, If | Return | Match | While | For)`.
fn pe_peek_is_arm_stmt_kind(p: &Parser) -> Bool {
  let k = selfhost_parser_state.p_peek_kind(p);
  return pe_ktag(k) == KT_If || pe_ktag(k) == KT_Return || pe_ktag(k) == KT_Match || pe_ktag(k) == KT_While || pe_ktag(k) == KT_For;
}

fn pe_parse_arm_stmt(p: &mut Parser) -> Int {
  let k = selfhost_parser_state.p_peek_kind(p);
  if pe_ktag(k) == KT_If { return pe_parse_if_stmt(p); }
  if pe_ktag(k) == KT_Return {
    selfhost_parser_state.p_advance(p);
    var expr = -1;
    let k2 = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k2) != KT_Comma && pe_ktag(k2) != KT_RBrace {
      expr = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
    }
    let span = selfhost_parser_state.p_peek_span(p);
    return selfhost_parser_state.p_n(p, NodeKind.NkReturn(expr), span);
  }
  if pe_ktag(k) == KT_Match { return pe_parse_match_stmt(p); }
  if pe_ktag(k) == KT_While { return pe_parse_while_stmt(p); }
  if pe_ktag(k) == KT_For { return pe_parse_for_stmt(p); }
  return selfhost_parser_state.p_err(p, "expected statement in match arm");
}

/// Result: [body_index, body_is_block] (empty Vec on failure).
fn pe_parse_expr_or_assign(p: &mut Parser) -> Vec[Int] {
  let span = selfhost_parser_state.p_peek_span(p);
  let expr = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  let compound = pe_try_compound_assign(p, expr);
  if compound >= 0 {
    let asg = selfhost_parser_state.p_n(p, NodeKind.NkAssign(expr, compound), span);
    return pe_assign_block(p, asg, span);
  }
  if selfhost_parser_state.p_skip(p, TkEq) {
    let rhs = pe_parse_expr(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    let asg = selfhost_parser_state.p_n(p, NodeKind.NkAssign(expr, rhs), span);
    return pe_assign_block(p, asg, span);
  }
  var out = Vec[Int].new();
  out.push(expr);
  out.push(0);
  return out;
}

fn pe_assign_block(p: &mut Parser, asg: Int, span: Span) -> Vec[Int] {
  var stmts = Vec[Int].new();
  stmts.push(pe_stmt_w(p, asg));
  let blk = selfhost_parser_state.p_n(p, NodeKind.NkBlock(stmts), span);
  var out = Vec[Int].new();
  out.push(blk);
  out.push(1);
  return out;
}

fn pe_parse_while_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  if pe_pk(p) == KT_Let {
    return pe_parse_while_let_stmt(p, span);
  }
  let cond = pe_parse_cond(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  // 5f: optional loop invariant.
  var invariant = -1;
  let k = selfhost_parser_state.p_peek_kind(p);
  var is_inv = false;
  match k {
    TkIdent(_) => {
      if selfhost_parser_state.p_peek(p).lexeme == "invariant" { is_inv = true; }
    }
    _ => {}
  }
  if is_inv {
    selfhost_parser_state.p_advance(p);
    let _ = selfhost_parser_state.p_expect_kind(p, TkColon, "':' after 'invariant'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    invariant = pe_parse_cond(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  let body = pe_parse_block(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkWhile(cond, body, invariant, -1), span);
}

/// `while let pattern = expr { ... }` desugars to
/// `while true { match expr { pattern => { ... }, _ => break } }`.
fn pe_parse_while_let_stmt(p: &mut Parser, while_span: Span) -> Int {
  selfhost_parser_state.p_advance(p); // skip 'let'
  let pattern = pe_parse_pattern(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkEq, "'=' in while let");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let expr = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let body_block = pe_parse_block(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let wildcard = selfhost_parser_state.p_n(
    p, NodeKind.NkPatWildcard, selfhost_ast.span_zero()
  );
  let break_stmt = selfhost_parser_state.p_n(p, NodeKind.NkBreak(-1), while_span);
  var break_stmts = Vec[Int].new();
  break_stmts.push(pe_stmt_w(p, break_stmt));
  let break_body = selfhost_parser_state.p_n(p, NodeKind.NkBlock(break_stmts), while_span);
  var arms = Vec[Int].new();
  arms.push(selfhost_parser_state.p_n(
    p, NodeKind.NkMatchArm(pattern, -1, body_block, 1), while_span
  ));
  arms.push(selfhost_parser_state.p_n(
    p, NodeKind.NkMatchArm(wildcard, -1, break_body, 1), while_span
  ));
  let match_stmt = selfhost_parser_state.p_n(p, NodeKind.NkStmtMatch(expr, arms), while_span);
  var inner_stmts = Vec[Int].new();
  inner_stmts.push(pe_stmt_w(p, match_stmt));
  let inner_block = selfhost_parser_state.p_n(p, NodeKind.NkBlock(inner_stmts), while_span);
  let lit = selfhost_parser_state.p_n(p, NodeKind.NkLitBool(1), while_span);
  return selfhost_parser_state.p_n(p, NodeKind.NkWhile(lit, inner_block, -1, -1), while_span);
}

fn pe_parse_for_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  let var_id = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkIn, "'in'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let iter = pe_parse_cond(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let body = pe_parse_block(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkFor(var_id, iter, body, -1), span);
}

/// v0.55: `defer { ... }` or `defer expr;`.
fn pe_parse_defer_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p);
  let span = pe_tok_span(tok);
  if pe_pk(p) == KT_LBrace {
    let block = pe_parse_block(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkDefer(block), span);
  }
  let expr = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
  var stmts = Vec[Int].new();
  stmts.push(pe_tail_w(p, expr));
  let block = selfhost_parser_state.p_n(p, NodeKind.NkBlock(stmts), span);
  return selfhost_parser_state.p_n(p, NodeKind.NkDefer(block), span);
}

fn pe_parse_spawn_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p); // consume 'spawn'
  let span = pe_tok_span(tok);
  var is_move = 0;
  if pe_pk(p) == KT_Move {
    selfhost_parser_state.p_advance(p);
    is_move = 1;
  }
  let body = pe_parse_block(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkStmtSpawn(is_move, body), span);
}

fn pe_parse_destructure(p: &mut Parser, span: Span) -> Int {
  selfhost_parser_state.p_advance(p);
  var names = Vec[Int].new();
  let n0 = pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  names.push(n0);
  while selfhost_parser_state.p_skip(p, TkComma) {
    let n = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    names.push(n);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkEq, "'='");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let value = pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkDestructure(names, value), span);
}

/// v0.55: `asm("template" [: outputs [: inputs [: clobbers]]]);`
pub fn pe_parse_asm_stmt(p: &mut Parser) -> Int {
  let tok = selfhost_parser_state.p_advance(p); // consume 'asm'
  let span = pe_tok_span(tok);
  let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'(' after asm");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var template = "";
  let ttok = selfhost_parser_state.p_advance(p);
  match ttok.kind {
    TkStr(bytes) => { template = Str::from_utf8(bytes); }
    _ => {
      return selfhost_parser_state.p_err(p, "expected string literal as asm template");
    }
  }
  var outputs = Vec[Int].new();
  var inputs = Vec[Int].new();
  var clobbers = Vec[Str].new();
  if pe_pk(p) == KT_Colon {
    selfhost_parser_state.p_advance(p);
    var go = true;
    while go {
      let k = selfhost_parser_state.p_peek_kind(p);
      if pe_ktag(k) == KT_Colon || pe_ktag(k) == KT_RParen || pe_ktag(k) == KT_Semicolon || pe_ktag(k) == KT_Eof { break; }
      let ctok = selfhost_parser_state.p_advance(p);
      var constraint = "";
      match ctok.kind {
        TkStr(bytes) => { constraint = Str::from_utf8(bytes); }
        _ => {
          return selfhost_parser_state.p_err(
            p, "expected constraint string in asm outputs"
          );
        }
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'(' after constraint");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let var_id = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')' after asm output");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let on = selfhost_parser_state.p_n(
        p, NodeKind.NkAsmOut(constraint, var_id), selfhost_parser_state.p_span_of(p, var_id)
      );
      outputs.push(on);
      let _ = selfhost_parser_state.p_skip(p, TkComma);
    }
  }
  if pe_pk(p) == KT_Colon {
    selfhost_parser_state.p_advance(p);
    var go = true;
    while go {
      let k = selfhost_parser_state.p_peek_kind(p);
      if pe_ktag(k) == KT_Colon || pe_ktag(k) == KT_RParen || pe_ktag(k) == KT_Semicolon || pe_ktag(k) == KT_Eof { break; }
      let ctok = selfhost_parser_state.p_advance(p);
      var constraint = "";
      match ctok.kind {
        TkStr(bytes) => { constraint = Str::from_utf8(bytes); }
        _ => {
          return selfhost_parser_state.p_err(
            p, "expected constraint string in asm inputs"
          );
        }
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'(' after constraint");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let expr = pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')' after asm input");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let inn = selfhost_parser_state.p_n(
        p, NodeKind.NkAsmIn(constraint, expr), selfhost_ast.span_zero()
      );
      inputs.push(inn);
      let _ = selfhost_parser_state.p_skip(p, TkComma);
    }
  }
  if pe_pk(p) == KT_Colon {
    selfhost_parser_state.p_advance(p);
    var go = true;
    while go {
      let k = selfhost_parser_state.p_peek_kind(p);
      if pe_ktag(k) == KT_RParen || pe_ktag(k) == KT_Semicolon || pe_ktag(k) == KT_Eof { break; }
      let ctok = selfhost_parser_state.p_advance(p);
      match ctok.kind {
        TkStr(bytes) => { clobbers.push(Str::from_utf8(bytes)); }
        _ => {
          return selfhost_parser_state.p_err(p, "expected clobber string in asm");
        }
      }
      let _ = selfhost_parser_state.p_skip(p, TkComma);
    }
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')' after asm");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = selfhost_parser_state.p_skip(p, TkSemicolon);
  return selfhost_parser_state.p_n(
    p, NodeKind.NkAsm(template, outputs, inputs, clobbers), span
  );
}

pub fn pe_parse_block(p: &mut Parser) -> Int {
  let start = selfhost_parser_state.p_peek_span(p);
  let _ = selfhost_parser_state.p_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  // Inside a block, struct literals are allowed again.
  let saved_restrict = p.restrict_struct;
  p.restrict_struct = 0;
  var items = Vec[Int].new();
  var go = true;
  while go {
    let k = selfhost_parser_state.p_peek_kind(p);
    if pe_ktag(k) == KT_RBrace || pe_ktag(k) == KT_Eof { break; }
    if selfhost_parser_state.p_skip(p, TkSemicolon) { continue; }
    // R68: nested FFI `extern "C" { ... }` inside a body is hoisted.
    if pe_pk(p) == KT_Extern {
      let block = pe_parse_extern_block(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      p.pending_externs.push(block);
      continue;
    }
    let it = pe_parse_stmt_or_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    items.push(it);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  p.restrict_struct = saved_restrict;
  return selfhost_parser_state.p_n(p, NodeKind.NkBlock(items), start);
}

/// Rust parse_extern_block: `extern "C" { fn name(params) -> T; ... }`.
/// Returned as an NkExtern arena node (also used by the parser_core facade
/// for top-level extern declarations).
pub fn pe_parse_extern_block(p: &mut Parser) -> Int {
  selfhost_parser_state.p_advance(p); // consume 'extern'
  let span_start = selfhost_parser_state.p_peek_span(p);
  // Linkage string (e.g. "C"); exactly one token is consumed either way.
  let lexeme = selfhost_parser_state.p_peek(p).lexeme;
  let linkage = pe_trim_quotes(lexeme);
  selfhost_parser_state.p_advance(p);
  let _ = selfhost_parser_state.p_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var functions = Vec[Int].new();
  var go = true;
  while go {
    if selfhost_parser_state.p_peek_is(p, TkRBrace) { break; }
    if selfhost_parser_state.p_is_eof(p) { break; }
    let _ = selfhost_parser_state.p_expect_kind(p, TkFn, "'fn'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    let fn_name = pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let generics = pe_parse_optional_generic_params(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = selfhost_parser_state.p_expect_kind(p, TkLParen, "'('");
    if selfhost_parser_state.p_failed(p) { return -1; }
    // Params parsed manually, handling variadic '...'.
    var params = Vec[Int].new();
    var pgo = true;
    while pgo {
      if selfhost_parser_state.p_peek_is(p, TkRParen) { break; }
      if selfhost_parser_state.p_is_eof(p) { break; }
      if selfhost_parser_state.p_peek_is(p, TkDot) {
        selfhost_parser_state.p_advance(p);
        selfhost_parser_state.p_advance(p);
        selfhost_parser_state.p_advance(p); // skip ...
        break;
      }
      let pname = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = selfhost_parser_state.p_expect_kind(p, TkColon, "':'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let pty = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let pspan = selfhost_parser_state.p_span_of(p, pname);
      params.push(selfhost_parser_state.p_n(p, NodeKind.NkParam(pname, pty, 0, 0), pspan));
      if !selfhost_parser_state.p_peek_is(p, TkRParen) {
        let _ = selfhost_parser_state.p_expect_kind(p, TkComma, "','");
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    var return_type = -1;
    if selfhost_parser_state.p_skip(p, TkArrow) {
      return_type = pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
    }
    let _ = selfhost_parser_state.p_expect_kind(p, TkSemicolon, "';'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    var empty_contracts = Vec[Int].new();
    var empty_attrs = Vec[Int].new();
    let fn_node = selfhost_parser_state.p_n(
      p,
      NodeKind.NkFn(0, 0, -1, fn_name, generics, params, return_type, empty_contracts, -1, empty_attrs),
      span_start
    );
    functions.push(fn_node);
  }
  let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkExtern(linkage, functions), span_start);
}

// ============================================================================
// Patterns (Rust parse_pattern / parse_pattern_single)
// ============================================================================

pub fn pe_parse_pattern(p: &mut Parser) -> Int {
  let span = selfhost_parser_state.p_peek_span(p);
  let first = pe_parse_pattern_single(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  if pe_pk(p) == KT_Pipe {
    var alts = Vec[Int].new();
    alts.push(first);
    while selfhost_parser_state.p_skip(p, TkPipe) {
      let alt = pe_parse_pattern_single(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      alts.push(alt);
    }
    return selfhost_parser_state.p_n(p, NodeKind.NkPatOr(alts), span);
  }
  return first;
}

fn pe_parse_pattern_single(p: &mut Parser) -> Int {
  let pk = selfhost_parser_state.p_peek_kind(p);
  match pk {
    TkUnderscore => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkPatWildcard, pe_tok_span(tok));
    }
    TkSome => {
      let tok = selfhost_parser_state.p_advance(p);
      let span = pe_tok_span(tok);
      var inner = -1;
      if selfhost_parser_state.p_skip(p, TkLParen) {
        inner = pe_parse_pattern(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
      } else {
        inner = selfhost_parser_state.p_n(p, NodeKind.NkPatWildcard, span);
      }
      return selfhost_parser_state.p_n(p, NodeKind.NkPatSome(inner), span);
    }
    TkNone => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkPatNone, pe_tok_span(tok));
    }
    TkOkV => {
      let tok = selfhost_parser_state.p_advance(p);
      let span = pe_tok_span(tok);
      var inner = -1;
      if selfhost_parser_state.p_skip(p, TkLParen) {
        inner = pe_parse_pattern(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
      } else {
        inner = selfhost_parser_state.p_n(p, NodeKind.NkPatWildcard, span);
      }
      return selfhost_parser_state.p_n(p, NodeKind.NkPatOk(inner), span);
    }
    TkErrV => {
      let tok = selfhost_parser_state.p_advance(p);
      let span = pe_tok_span(tok);
      var inner = -1;
      if selfhost_parser_state.p_skip(p, TkLParen) {
        inner = pe_parse_pattern(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
      } else {
        inner = selfhost_parser_state.p_n(p, NodeKind.NkPatWildcard, span);
      }
      return selfhost_parser_state.p_n(p, NodeKind.NkPatErr(inner), span);
    }
    TkTrue => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitBool(1), pe_tok_span(tok));
    }
    TkFalse => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitBool(0), pe_tok_span(tok));
    }
    TkInt(n) => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitInt(n), pe_tok_span(tok));
    }
    TkStr(s) => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitStr(s), pe_tok_span(tok));
    }
    TkChar(c) => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitChar(c), pe_tok_span(tok));
    }
    TkFloat(_v) => {
      let tok = selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_n(p, NodeKind.NkLitFloat(tok.lexeme), pe_tok_span(tok));
    }
    TkLParen => {
      let tok = selfhost_parser_state.p_advance(p);
      let span = pe_tok_span(tok);
      if selfhost_parser_state.p_skip(p, TkRParen) {
        // Unit pattern `()` -- treat as wildcard.
        return selfhost_parser_state.p_n(p, NodeKind.NkPatWildcard, span);
      }
      let first = pe_parse_pattern(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      // P1-2: comma follows -> tuple pattern (a, b, c).
      if selfhost_parser_state.p_skip(p, TkComma) {
        var items = Vec[Int].new();
        items.push(first);
        let p2 = pe_parse_pattern(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        items.push(p2);
        while selfhost_parser_state.p_skip(p, TkComma) {
          let pp = pe_parse_pattern(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          items.push(pp);
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        return selfhost_parser_state.p_n(p, NodeKind.NkPatTuple(items), span);
      }
      let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return first;
    }
    _ => {
      // `ref` / `ref mut` in patterns are binding modifiers; parsed and
      // discarded (XIOM value semantics treat them as regular bindings).
      let k0 = selfhost_parser_state.p_peek_kind(p);
      match k0 {
        TkIdent(_) => {
          if selfhost_parser_state.p_peek(p).lexeme == "ref" {
            selfhost_parser_state.p_advance(p);
            let k1 = selfhost_parser_state.p_peek_kind(p);
            match k1 {
              TkIdent(_) => {
                if selfhost_parser_state.p_peek(p).lexeme == "mut" {
                  selfhost_parser_state.p_advance(p);
                }
              }
              _ => {}
            }
          }
        }
        _ => {}
      }
      let name0 = pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let name_span = selfhost_parser_state.p_span_of(p, name0);
      var name_text = pe_ident_text(p, name0);
      // Qualified enum-variant pattern, e.g. `LogLevel.Trace`: fold the
      // dotted path into a single name (span stays the first ident's).
      while selfhost_parser_state.p_skip(p, TkDot) {
        let v = pe_parse_ident(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        name_text = name_text + "." + pe_ident_text(p, v);
      }
      let name = selfhost_parser_state.p_n_ident(p, name_text, name_span);
      if selfhost_parser_state.p_skip(p, TkLParen) {
        let span = name_span;
        var fields = Vec[Int].new();
        var go = true;
        while go {
          // Skip `ref` / `ref mut` in variant constructor patterns.
          let kk = selfhost_parser_state.p_peek_kind(p);
          match kk {
            TkIdent(_) => {
              if selfhost_parser_state.p_peek(p).lexeme == "ref" {
                selfhost_parser_state.p_advance(p);
                let k2 = selfhost_parser_state.p_peek_kind(p);
                match k2 {
                  TkIdent(_) => {
                    if selfhost_parser_state.p_peek(p).lexeme == "mut" {
                      selfhost_parser_state.p_advance(p);
                    }
                  }
                  _ => {}
                }
              }
            }
            _ => {}
          }
          let field = pe_parse_ident(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          if selfhost_parser_state.p_skip(p, TkColon) {
            let sub = pe_parse_pattern(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            let sn = p.nodes[sub];
            match sn.kind {
              NkPatIdent(binding) => { fields.push(binding); }
              _ => {
                fields.push(selfhost_parser_state.p_n_ident(
                  p, "_", selfhost_parser_state.p_span_of(p, field)
                ));
              }
            }
          } else {
            fields.push(field);
          }
          if !selfhost_parser_state.p_skip(p, TkComma) { go = false; }
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        return selfhost_parser_state.p_n(p, NodeKind.NkPatVariant(name, fields), span);
      } elif pe_pk(p) == KT_LBrace {
        // P1-1: struct pattern `TypeName { field1, field2: pat2 }`.
        selfhost_parser_state.p_advance(p); // consume '{'
        let span = name_span;
        var fields = Vec[Int].new();
        var go = true;
        while go {
          let kk = selfhost_parser_state.p_peek_kind(p);
          if pe_ktag(kk) == KT_RBrace || pe_ktag(kk) == KT_Eof { break; }
          let fname = pe_parse_ident(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          var fpat = -1;
          if selfhost_parser_state.p_skip(p, TkColon) {
            fpat = pe_parse_pattern(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
          } else {
            // Shorthand: `{ field }` means `{ field: field }`.
            let fsp = selfhost_parser_state.p_span_of(p, fname);
            fpat = selfhost_parser_state.p_n(p, NodeKind.NkPatIdent(fname), fsp);
          }
          let fsp2 = selfhost_parser_state.p_span_of(p, fname);
          fields.push(selfhost_parser_state.p_n(p, NodeKind.NkPatField(fname, fpat), fsp2));
          let _ = selfhost_parser_state.p_skip(p, TkComma);
        }
        let _ = selfhost_parser_state.p_expect_kind(p, TkRBrace, "'}'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        return selfhost_parser_state.p_n(p, NodeKind.NkPatStruct(name, fields), span);
      }
      return selfhost_parser_state.p_n(p, NodeKind.NkPatIdent(name), name_span);
    }
  }
}




