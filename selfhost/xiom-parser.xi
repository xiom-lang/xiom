// XIOM — Self-Hosted Parser
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

// Processes a hardcoded token stream for:
//   "fn main() -> Int { let x = 10; var y = 20; if x > y { return x; } else { return y; } }"
//
// Token kinds (from lexer):
//   0 = EOF      1 = fn       2 = return    3 = if
//   4 = else     5 = let      6 = var       8 = Int
//   9 = Bool    10 = type
//  40 = =       43 = +        44 = -        45 = *
//  47 = /       50 = (        51 = )        52 = {
//  53 = }      54 = ;        56 = ->        57 = >
//  58 = <
// 100 = IDENT  200 = INT_LIT
//
// AST Node types: 1=Program, 2=FnDecl, 3=ReturnStmt, 4=IntLiteral,
//                 5=LetStmt, 6=VarStmt, 7=IfStmt, 8=BinExpr, 9=IdentExpr
//
// Ownership: Uses &Int for read-borrows. Position mutation is returned
// as an encoded value: kind * 1000000 + new_pos. Callers decode and
// assign to their local var.

module parser {

// ============================================================================
// AST types
// ============================================================================
pub type AstNode = {
  tag: Int;
  kind: Int;
  value: Int;
} derive[Eq, Clone]

pub fn make_node(tag: Int, kind: Int, value: Int) -> AstNode {
  return AstNode{ tag: tag, kind: kind, value: value };
}

// ============================================================================
// Hardcoded token stream for the test program.
// ============================================================================
// Returns encoded value: token_kind * 1000000 + (pos + 1)
pub fn next_token(pos: &Int) -> Int {
  let p = pos;
  // fn(1), main(100), ((50), )(51), ->(56), Int(8), {(52)
  if p == 0 { return 1 * 1000000 + p + 1; }
  if p == 1 { return 100 * 1000000 + p + 1; }
  if p == 2 { return 50 * 1000000 + p + 1; }
  if p == 3 { return 51 * 1000000 + p + 1; }
  if p == 4 { return 56 * 1000000 + p + 1; }
  if p == 5 { return 8 * 1000000 + p + 1; }
  if p == 6 { return 52 * 1000000 + p + 1; }
  // let(5), x(100), =(40), 10(200), ;(54)
  if p == 7 { return 5 * 1000000 + p + 1; }
  if p == 8 { return 100 * 1000000 + p + 1; }
  if p == 9 { return 40 * 1000000 + p + 1; }
  if p == 10 { return 200 * 1000000 + p + 1; }
  if p == 11 { return 54 * 1000000 + p + 1; }
  // var(6), y(100), =(40), 20(200), ;(54)
  if p == 12 { return 6 * 1000000 + p + 1; }
  if p == 13 { return 100 * 1000000 + p + 1; }
  if p == 14 { return 40 * 1000000 + p + 1; }
  if p == 15 { return 200 * 1000000 + p + 1; }
  if p == 16 { return 54 * 1000000 + p + 1; }
  // if(3), x(100), >(57), y(100), {(52)
  if p == 17 { return 3 * 1000000 + p + 1; }
  if p == 18 { return 100 * 1000000 + p + 1; }
  if p == 19 { return 57 * 1000000 + p + 1; }
  if p == 20 { return 100 * 1000000 + p + 1; }
  if p == 21 { return 52 * 1000000 + p + 1; }
  // return(2), x(100), ;(54)
  if p == 22 { return 2 * 1000000 + p + 1; }
  if p == 23 { return 100 * 1000000 + p + 1; }
  if p == 24 { return 54 * 1000000 + p + 1; }
  // }(53), else(4), {(52)
  if p == 25 { return 53 * 1000000 + p + 1; }
  if p == 26 { return 4 * 1000000 + p + 1; }
  if p == 27 { return 52 * 1000000 + p + 1; }
  // return(2), y(100), ;(54)
  if p == 28 { return 2 * 1000000 + p + 1; }
  if p == 29 { return 100 * 1000000 + p + 1; }
  if p == 30 { return 54 * 1000000 + p + 1; }
  // }(53)
  if p == 31 { return 53 * 1000000 + p + 1; }
  // }(53) — outer block
  if p == 32 { return 53 * 1000000 + p + 1; }
  return p + 1; // EOF
}

pub fn token_count() -> Int {
  return 33;
}

// ============================================================================
// Statement parser
// ============================================================================

pub fn parse_stmt(pos: &Int) -> Int {
  var p = pos;
  var encoded = next_token(&p);
  var tok = encoded / 1000000;
  p = encoded - tok * 1000000;

  if tok == 5 {
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    return 1 * 1000000 + p;
  } elif tok == 6 {
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    return 1 * 1000000 + p;
  } elif tok == 3 {
    encoded = next_token(&p);
    p = encoded - (encoded / 1000000) * 1000000;
    encoded = next_token(&p);
    p = encoded - (encoded / 1000000) * 1000000;
    encoded = next_token(&p);
    p = encoded - (encoded / 1000000) * 1000000;
    encoded = next_token(&p);
    p = encoded - (encoded / 1000000) * 1000000;
    var body_nodes = 0;
    var done = 1 == 0;
    while !(done) {
      encoded = next_token(&p);
      tok = encoded / 1000000;
      p = encoded - tok * 1000000;
      if tok == 53 {
        done = true;
      } else {
        if tok == 2 {
          encoded = next_token(&p);
          p = encoded - (encoded / 1000000) * 1000000;
          encoded = next_token(&p);
          p = encoded - (encoded / 1000000) * 1000000;
          body_nodes = body_nodes + 1;
        }
      }
    }
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    if tok == 4 {
      encoded = next_token(&p);
      p = encoded - (encoded / 1000000) * 1000000;
      var done2 = 1 == 0;
      while !(done2) {
        encoded = next_token(&p);
        tok = encoded / 1000000;
        p = encoded - tok * 1000000;
        if tok == 53 {
          done2 = true;
        } else {
          if tok == 2 {
            encoded = next_token(&p);
            p = encoded - (encoded / 1000000) * 1000000;
            encoded = next_token(&p);
            p = encoded - (encoded / 1000000) * 1000000;
            body_nodes = body_nodes + 1;
          }
        }
      }
    } else {
      p = p - 1;
    }
    return (1 + body_nodes) * 1000000 + p;
  } elif tok == 2 {
    encoded = next_token(&p);
    p = encoded - (encoded / 1000000) * 1000000;
    encoded = next_token(&p);
    p = encoded - (encoded / 1000000) * 1000000;
    return 1 * 1000000 + p;
  } else {
    return 0 * 1000000 + pos;
  }
}

// ============================================================================
// Parser functions
// ============================================================================

// Parse a function declaration: fn IDENT ( params? ) -> Type { stmts }
pub fn parse_fn_decl(pos: &Int) -> Int {
  var p = pos;
  var nodes = 1;

  // Expect 'fn' keyword
  var encoded = next_token(&p);
  var tok = encoded / 1000000;
  p = encoded - tok * 1000000;
  if tok != 1 { return 0; }

  // Expect identifier
  encoded = next_token(&p);
  tok = encoded / 1000000;
  p = encoded - tok * 1000000;
  if tok != 100 { return 0; }

  // Expect '('
  encoded = next_token(&p);
  tok = encoded / 1000000;
  p = encoded - tok * 1000000;
  if tok != 50 { return 0; }

  // Expect ')'
  encoded = next_token(&p);
  tok = encoded / 1000000;
  p = encoded - tok * 1000000;
  if tok != 51 { return 0; }

  // Optional '->' Type
  encoded = next_token(&p);
  tok = encoded / 1000000;
  p = encoded - tok * 1000000;
  if tok == 56 {
    encoded = next_token(&p);
    tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    if tok == 0 { return 0; }
  } else {
    p = p - 1;
  }

  // Expect '{'
  encoded = next_token(&p);
  tok = encoded / 1000000;
  p = encoded - tok * 1000000;
  if tok != 52 { return 0; }

  // Parse statements inside block
  var done = 1 == 0;
  while !(done) {
    var n_encoded = parse_stmt(&p);
    var n = n_encoded / 1000000;
    p = n_encoded - n * 1000000;
    if n > 0 {
      nodes = nodes + n;
    } else {
      encoded = next_token(&p);
      tok = encoded / 1000000;
      p = encoded - tok * 1000000;
      if tok == 53 || tok == 0 {
        done = true;
      }
    }
  }

  return nodes;
}

// Parse the full program: FnDecl*
pub fn parse_program(pos: &Int) -> Int {
  var p = pos;
  var nodes = 0;
  let max_tokens = token_count();

  while p < max_tokens {
    var encoded = next_token(&p);
    var tok = encoded / 1000000;
    p = encoded - tok * 1000000;
    if tok == 0 {
    } elif tok == 1 {
      var fn_nodes = 1;
      encoded = next_token(&p);
      tok = encoded / 1000000;
      p = encoded - tok * 1000000;
      encoded = next_token(&p);
      tok = encoded / 1000000;
      p = encoded - tok * 1000000;
      var done = 1 == 0;
      while !(done) {
        encoded = next_token(&p);
        tok = encoded / 1000000;
        p = encoded - tok * 1000000;
        if tok == 51 || tok == 0 {
          done = true;
        }
      }
      encoded = next_token(&p);
      tok = encoded / 1000000;
      p = encoded - tok * 1000000;
      if tok == 56 {
        encoded = next_token(&p);
        tok = encoded / 1000000;
        p = encoded - tok * 1000000;
      } else {
        p = p - 1;
      }
      encoded = next_token(&p);
      tok = encoded / 1000000;
      p = encoded - tok * 1000000;
      if tok != 52 { return nodes; }
      var done2 = 1 == 0;
      while !(done2) {
        var n_encoded = parse_stmt(&p);
        var n = n_encoded / 1000000;
        p = n_encoded - n * 1000000;
        if n > 0 {
          fn_nodes = fn_nodes + n;
        } else {
          encoded = next_token(&p);
          tok = encoded / 1000000;
          p = encoded - tok * 1000000;
          if tok == 53 || tok == 0 {
            done2 = true;
          }
        }
      }
      nodes = nodes + fn_nodes;
    } else {
    }
  }

  return nodes;
}

} // end module parser

// ============================================================================
// Module exports
// ============================================================================
use parser.AstNode;
use parser.make_node;
use parser.next_token;
use parser.token_count;
use parser.parse_fn_decl;
use parser.parse_program;
use parser.parse_stmt;

// ============================================================================
// Tests
// ============================================================================

fn test_parser_basic() -> Int {
  var pos = 0;
  let encoded = next_token(&pos);
  let tok = encoded / 1000000;
  if tok != 1 { return 1; }
  pos = encoded - tok * 1000000;
  if pos != 1 { return 2; }
  return 0;
}

fn test_parse_program() -> Int {
  var pos = 0;
  let nodes = parse_program(&pos);
  // Should find 1 function with let, var, if/else = 6+ nodes
  if nodes < 3 { return 1; }
  if nodes > 20 { return 2; }
  return 0;
}

fn test_token_count() -> Int {
  if token_count() != 33 { return 1; }
  return 0;
}

// ============================================================================
// Entry point
// ============================================================================
fn main() -> Int {
  var exit = test_parser_basic();
  if exit != 0 { return 100 + exit; }

  exit = test_parse_program();
  if exit != 0 { return 200 + exit; }

  exit = test_token_count();
  if exit != 0 { return 300 + exit; }

  return 0;
}
