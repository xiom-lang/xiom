// XIOM -- Self-Hosted Lexer
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// Tokenizes source via character-at-position lookup.
// Since XIOM v0.2.5 does not support arbitrary string indexing,
// source characters are provided by source_at(pos) which maps
// positions to ASCII codes of a hardcoded test program.
//
// Test program: "fn main() -> Int { return 42; }"
//
// Token kinds:
//   0 = EOF
//   1 = fn      2 = return   3 = if       4 = else
//   5 = let     6 = var      7 = while     8 = Int
//   9 = Bool   10 = type
//  40 = =      41 = ==      43 = +        44 = -
//  45 = *      50 = (        51 = )        52 = {
//  53 = }      54 = ;        55 = :        56 = ->
//  57 = >      58 = <
// 100 = IDENT  200 = INTEGER_LIT
//
// Ownership: Int values are moved on function calls.
// Functions taking &Int create a read borrow (caller retains value).

module lexer {

// ============================================================================
// Token type
// ============================================================================
pub type Token = {
  kind: Int;
  line: Int;
  col: Int;
} derive[Eq, Clone]

// ============================================================================
// Character classification
// ============================================================================

pub fn is_alpha(c: Int) -> Bool {
  if c >= 65 && c <= 90 { return true; }
  if c >= 97 && c <= 122 { return true; }
  return false;
}

pub fn is_digit(c: Int) -> Bool {
  if c >= 48 && c <= 57 { return true; }
  return false;
}

pub fn is_alphanumeric(c: Int) -> Bool {
  if is_alpha(c) { return true; }
  if is_digit(c) { return true; }
  if c == 95 { return true; }
  return false;
}

// ============================================================================
// Source: provides characters by position for the test program.
// "fn main() -> Int { return 42; }"
// ============================================================================

pub fn source_at(pos: Int) -> Int {
  // f (102)
  if pos == 0 { return 102; }
  // n (110)
  if pos == 1 { return 110; }
  // ' ' (32)
  if pos == 2 { return 32; }
  // m (109)
  if pos == 3 { return 109; }
  // a (97)
  if pos == 4 { return 97; }
  // i (105)
  if pos == 5 { return 105; }
  // n (110)
  if pos == 6 { return 110; }
  // ( (40)
  if pos == 7 { return 40; }
  // ) (41)
  if pos == 8 { return 41; }
  // ' ' (32)
  if pos == 9 { return 32; }
  // - (45)
  if pos == 10 { return 45; }
  // > (62)
  if pos == 11 { return 62; }
  // ' ' (32)
  if pos == 12 { return 32; }
  // I (73)
  if pos == 13 { return 73; }
  // n (110)
  if pos == 14 { return 110; }
  // t (116)
  if pos == 15 { return 116; }
  // ' ' (32)
  if pos == 16 { return 32; }
  // { (123)
  if pos == 17 { return 123; }
  // ' ' (32)
  if pos == 18 { return 32; }
  // r (114)
  if pos == 19 { return 114; }
  // e (101)
  if pos == 20 { return 101; }
  // t (116)
  if pos == 21 { return 116; }
  // u (117)
  if pos == 22 { return 117; }
  // r (114)
  if pos == 23 { return 114; }
  // n (110)
  if pos == 24 { return 110; }
  // ' ' (32)
  if pos == 25 { return 32; }
  // 4 (52)
  if pos == 26 { return 52; }
  // 2 (50)
  if pos == 27 { return 50; }
  // ; (59)
  if pos == 28 { return 59; }
  // ' ' (32)
  if pos == 29 { return 32; }
  // } (125)
  if pos == 30 { return 125; }
  // EOF
  return 0;
}

pub fn source_len() -> Int {
  return 31;
}

// ============================================================================
// Keyword matching
// ============================================================================

pub fn match_keyword(first: Int, second: Int, third: Int, fourth: Int, fifth: Int, sixth: Int) -> Int {
  // "fn" -- f(102) + n(110)
  if first == 102 && second == 110 {
    return 1;
  }
  // "return" -- r(114)+e(101)+t(116)+u(117)+r(114)+n(110)
  if first == 114 && second == 101 && third == 116 && fourth == 117 && fifth == 114 && sixth == 110 {
    return 2;
  }
  // "if" -- i(105)+f(102)
  if first == 105 && second == 102 {
    return 3;
  }
  // "else" -- e(101)+l(108)+s(115)+e(101)
  if first == 101 && second == 108 && third == 115 && fourth == 101 {
    return 4;
  }
  // "let" -- l(108)+e(101)+t(116)
  if first == 108 && second == 101 && third == 116 {
    return 5;
  }
  // "var" -- v(118)+a(97)+r(114)
  if first == 118 && second == 97 && third == 114 {
    return 6;
  }
  // "while" -- w(119)+h(104)+i(105)+l(108)+e(101)
  if first == 119 && second == 104 && third == 105 && fourth == 108 && fifth == 101 {
    return 7;
  }
  // "Int" -- I(73)+n(110)+t(116)
  if first == 73 && second == 110 && third == 116 {
    return 8;
  }
  // "Bool" -- B(66)+o(111)+o(111)+l(108)
  if first == 66 && second == 111 && third == 111 && fourth == 108 {
    return 9;
  }
  // "type" -- t(116)+y(121)+p(112)+e(101)
  if first == 116 && second == 121 && third == 112 && fourth == 101 {
    return 10;
  }
  // Not a keyword -- it's an identifier
  return 100;
}

// ============================================================================
// Token factory
// ============================================================================
pub fn make_token(kind: Int, line: Int, col: Int) -> Token {
  return Token{ kind: kind, line: line, col: col };
}

// ============================================================================
// Scan an identifier or keyword: advance past alphanumeric chars,
// then match against keyword table.
// Takes first character code + position, returns (kind, new_pos).
// Since we can't return tuples, encode as kind * 1000000 + new_pos.
// ============================================================================
pub fn scan_word(pos: &Int, line: Int, col: Int, c1: Int) -> Int {
  // Look ahead up to 5 more characters for keyword matching
  var np = pos;
  let c2 = source_at(np + 1);
  let c3 = source_at(np + 2);
  let c4 = source_at(np + 3);
  let c5 = source_at(np + 4);
  let c6 = source_at(np + 5);
  np = np + 5;

  // Try keyword match
  let kind = match_keyword(c1, c2, c3, c4, c5, c6);

  // Advance position past the word (skip remaining alphanumeric chars)
  // For simplicity, advance by checking length:
  // "fn"=2, "if"=2, "Int"=3, "return"=6, "else"=4, "let"=3,
  // "var"=3, "while"=5, "Bool"=4, "type"=4
  // Identifier: advance until non-alphanumeric
  if kind == 1 || kind == 3 {
    // fn, if = 2 chars (already consumed c1)
    np = pos + 1;
  } elif kind == 2 {
    // return = 6 chars
    np = pos + 5;
  } elif kind == 4 {
    // else = 4 chars
    np = pos + 3;
  } elif kind == 5 || kind == 6 {
    // let, var = 3 chars
    np = pos + 2;
  } elif kind == 8 {
    // Int = 3 chars
    np = pos + 2;
  } elif kind == 7 {
    // while = 5 chars
    np = pos + 4;
  } elif kind == 9 {
    // Bool = 4 chars
    np = pos + 3;
  } elif kind == 10 {
    // type = 4 chars
    np = pos + 3;
  } else {
    // Identifier -- scan past alphanumeric chars
    np = pos + 1;
    var done = 1 == 0;
    while !(done) {
      if np >= source_len() {
        done = true;
      } elif !(is_alphanumeric(source_at(np + 0))) {
        done = true;
      } else {
        np = np + 1;
        if np - pos > 20 {
          done = true;
        }
      }
    }
    np = np - 1;
  }

  return kind * 1000000 + np + 1;
}

// ============================================================================
// Scan a number literal
// ============================================================================
pub fn scan_number(pos: &Int) -> Int {
  var np = pos + 1;
  var done = 1 == 0;
  while !(done) {
    if np >= source_len() {
      done = true;
    } elif !(is_digit(source_at(np + 0))) {
      done = true;
    } else {
      np = np + 1;
      if np - pos > 10 {
        done = true;
      }
    }
  }
  return 200 * 1000000 + np;
}

// ============================================================================
// Main tokenizer
// Returns token count produced from the hardcoded source.
// ============================================================================
pub fn tokenize() -> Int {
  var pos = 0;
  var count = 0;
  var line = 1;
  var col = 1;
  let max_pos = source_len();

  while pos < max_pos {
    let c = source_at(pos + 0);

    // Whitespace
    if c == 32 || c == 9 || c == 13 {
      pos = pos + 1;
      col = col + 1;
    } elif c == 10 {
      pos = pos + 1;
      line = line + 1;
      col = 1;
    }
    // Alphabetic -- identifier or keyword
    elif is_alpha(c + 0) || c == 95 {
      let res = scan_word(&pos, line + 0, col + 0, c + 0);
      let kind = res / 1000000;
      let new_pos = res - kind * 1000000;
      pos = new_pos;
      col = col + 1;
      count = count + 1;
    }
    // Digit -- number literal
    elif is_digit(c + 0) {
      let res = scan_number(&pos);
      let kind = res / 1000000;
      let new_pos = res - kind * 1000000;
      pos = new_pos;
      col = col + 1;
      count = count + 1;
    }
    // Operators and punctuation
    elif c == 40 {
      // (
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 41 {
      // )
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 123 {
      // {
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 125 {
      // }
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 59 {
      // ;
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 45 {
      // - (could be ->)
      let c2 = source_at(pos + 1);
      if c2 == 62 {
        pos = pos + 2; col = col + 2; count = count + 1;
      } else {
        pos = pos + 1; col = col + 1; count = count + 1;
      }
    } elif c == 62 {
      // >
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 61 {
      // = (could be ==)
      let c2 = source_at(pos + 1);
      if c2 == 61 {
        pos = pos + 2; col = col + 2; count = count + 1;
      } else {
        pos = pos + 1; col = col + 1; count = count + 1;
      }
    } elif c == 43 {
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 42 {
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 58 {
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 44 {
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 46 {
      pos = pos + 1; col = col + 1; count = count + 1;
    } elif c == 47 {
      // '/' -- skip comment to end of line
      var np = pos + 1;
      if np < max_pos {
        let nc = source_at(np + 0);
        if nc == 47 {
          // // comment -- skip to newline
          var found_nl = 1 == 0;
          while !(found_nl) && np < max_pos {
            if source_at(np + 0) == 10 {
              found_nl = true;
            } else {
              np = np + 1;
            }
          }
          pos = np;
          line = line + 1;
          col = 1;
        } else {
          pos = pos + 1; col = col + 1; count = count + 1;
        }
      } else {
        pos = pos + 1; col = col + 1;
      }
    } elif c == 38 {
      let c2 = source_at(pos + 1);
      if c2 == 38 {
        pos = pos + 2; col = col + 2; count = count + 1;
      } else {
        pos = pos + 1; col = col + 1;
      }
    } elif c == 124 {
      let c2 = source_at(pos + 1);
      if c2 == 124 {
        pos = pos + 2; col = col + 2; count = count + 1;
      } else {
        pos = pos + 1; col = col + 1;
      }
    } elif c == 33 {
      let c2 = source_at(pos + 1);
      if c2 == 61 {
        pos = pos + 2; col = col + 2; count = count + 1;
      } else {
        pos = pos + 1; col = col + 1; count = count + 1;
      }
    } elif c == 60 {
      let c2 = source_at(pos + 1);
      if c2 == 61 {
        pos = pos + 2; col = col + 2; count = count + 1;
      } else {
        pos = pos + 1; col = col + 1; count = count + 1;
      }
    } else {
      // Unknown character -- skip
      pos = pos + 1;
      col = col + 1;
    }
  }

  // Return count + 1000 * (line) to encode line info
  return count + line * 1000;
}

} // end module lexer

// ============================================================================
// Module exports
// ============================================================================
use lexer.Token;
use lexer.make_token;
use lexer.tokenize;
use lexer.is_alpha;
use lexer.is_digit;
use lexer.is_alphanumeric;
use lexer.source_at;
use lexer.source_len;
use lexer.match_keyword;

// ============================================================================
// Tests
// ============================================================================

fn test_char_class() -> Int {
  if !(is_alpha(65)) { return 1; }
  if !(is_digit(48)) { return 2; }
  if !(is_alphanumeric(95)) { return 3; }
  if is_alpha(48) { return 4; }
  return 0;
}

fn test_keywords() -> Int {
  // "fn" = f,n
  if match_keyword(102,110,0,0,0,0) != 1 { return 1; }
  // "return" = r,e,t,u,r,n
  if match_keyword(114,101,116,117,114,110) != 2 { return 2; }
  // "Int" = I,n,t
  if match_keyword(73,110,116,0,0,0) != 8 { return 3; }
  // "let" = l,e,t
  if match_keyword(108,101,116,0,0,0) != 5 { return 4; }
  // gibberish -> identifier
  if match_keyword(120,120,120,0,0,0) != 100 { return 5; }
  return 0;
}

fn test_source() -> Int {
  if source_at(0) != 102 { return 1; }
  if source_at(1) != 110 { return 2; }
  if source_len() != 31 { return 3; }
  if source_at(30) != 125 { return 4; }
  return 0;
}

fn test_tokenize() -> Int {
  let res = tokenize();
  // res = count + line * 1000
  let line = res / 1000;
  let count = res - line * 1000;
  // "fn main() -> Int { return 42; }" should produce ~12 tokens
  if count < 5 { return 1; }
  if count > 20 { return 2; }
  return 0;
}

// ============================================================================
// Entry point
// ============================================================================
fn main() -> Int {
  var exit = test_char_class();
  if exit != 0 { return 100 + exit; }

  exit = test_keywords();
  if exit != 0 { return 200 + exit; }

  exit = test_source();
  if exit != 0 { return 300 + exit; }

  exit = test_tokenize();
  if exit != 0 { return 400 + exit; }

  return 0;
}
