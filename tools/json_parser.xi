#!/usr/bin/env xiom
// XIOM JSON Parser — Gap Discovery Script
// Usage: echo '{"key": "value"}' | xiom run tools/json_parser.xi
// Tests: string ops, Result handling, recursion, Vec ops, match statements

type JsonValue = {
  tag: Int;
  str_val: Str;
  num_val: Float64;
  bool_val: Bool;
  arr_val: Vec[JsonValue];
  obj_keys: Vec[Str];
  obj_vals: Vec[JsonValue];
}

fn json_null() -> JsonValue {
  JsonValue{
    tag: 0; str_val: ""; num_val: 0.0; bool_val: false;
    arr_val: Vec[JsonValue]::new();
    obj_keys: Vec[Str]::new();
    obj_vals: Vec[JsonValue]::new();
  }
}

fn json_bool(b: Bool) -> JsonValue {
  var v = json_null();
  if b {
    v.tag = 2; v.bool_val = true;
  } else {
    v.tag = 2; v.bool_val = false;
  };
  return v;
}

fn json_number(n: Float64) -> JsonValue {
  var v = json_null();
  v.tag = 3; v.num_val = n;
  return v;
}

fn json_string(s: Str) -> JsonValue {
  var v = json_null();
  v.tag = 1; v.str_val = s;
  return v;
}

fn json_array(items: Vec[JsonValue]) -> JsonValue {
  var v = json_null();
  v.tag = 4; v.arr_val = items;
  return v;
}

fn json_object(keys: Vec[Str], vals: Vec[JsonValue]) -> JsonValue {
  var v = json_null();
  v.tag = 5; v.obj_keys = keys; v.obj_vals = vals;
  return v;
}

// === Lexer ===
type Token = {
  kind: Int;  // 0=str 1=num 2=bool 3=null 4=lbrace 5=rbrace 6=lbrack 7=rbrack 8=colon 9=comma 10=eof 11=error
  s: Str; n: Float64; b: Bool;
}

fn lex_number(input: Str, start: Int) -> (Token, Int) {
  var pos = start;
  var has_dot = false;
  while pos < input.len() {
    var c = input.byte_at(pos);
    if c >= 48 && c <= 57 {
      pos = pos + 1;
    } elif c == 46 && !has_dot {
      has_dot = true; pos = pos + 1;
    } elif c == 45 && pos == start {
      pos = pos + 1;
    } else {
      break;
    };
  };
  var num_str = input.substr(start, pos);
  var val: Float64 = 0.0;
  var i = start;
  var neg = false;
  if num_str.byte_at(0) == 45 {
    neg = true; i = i + 1;
  };
  var int_part: Float64 = 0.0;
  while i < pos {
    var c = num_str.byte_at(i);
    if c == 46 { i = i + 1; break; };
    int_part = int_part * 10.0 + ((c as Int - 48) as Float64);
    i = i + 1;
  };
  var frac: Float64 = 0.0;
  var div: Float64 = 10.0;
  while i < pos {
    var c = num_str.byte_at(i);
    frac = frac + ((c as Int - 48) as Float64) / div;
    div = div * 10.0; i = i + 1;
  };
  val = int_part + frac;
  if neg { val = -val; };
  return (Token{ kind: 1; s: ""; n: val; b: false; }, pos);
}

fn lex_string(input: Str, start: Int) -> (Token, Int) {
  var pos = start + 1;
  var result = "";
  while pos < input.len() {
    var c = input.byte_at(pos);
    if c == 92 {
      pos = pos + 2;
    } elif c == 34 {
      pos = pos + 1; break;
    } else {
      pos = pos + 1;
    };
  };
  result = input.substr(start + 1, pos - 1);
  return (Token{ kind: 0; s: result; n: 0.0; b: false; }, pos);
}

fn lex(input: Str) -> Vec[Token] {
  var tokens: Vec[Token] = Vec[Token]::new();
  var pos = 0;
  while pos < input.len() {
    var c = input.byte_at(pos);
    if c == 32 || c == 10 || c == 13 || c == 9 {
      pos = pos + 1;
    } elif c == 123 {
      tokens.push(Token{ kind: 4; s: ""; n: 0.0; b: false; });
      pos = pos + 1;
    } elif c == 125 {
      tokens.push(Token{ kind: 5; s: ""; n: 0.0; b: false; });
      pos = pos + 1;
    } elif c == 91 {
      tokens.push(Token{ kind: 6; s: ""; n: 0.0; b: false; });
      pos = pos + 1;
    } elif c == 93 {
      tokens.push(Token{ kind: 7; s: ""; n: 0.0; b: false; });
      pos = pos + 1;
    } elif c == 58 {
      tokens.push(Token{ kind: 8; s: ""; n: 0.0; b: false; });
      pos = pos + 1;
    } elif c == 44 {
      tokens.push(Token{ kind: 9; s: ""; n: 0.0; b: false; });
      pos = pos + 1;
    } elif c == 34 {
      var (tok, newpos) = lex_string(input, pos);
      tokens.push(tok); pos = newpos;
    } elif (c >= 48 && c <= 57) || c == 45 {
      var (tok, newpos) = lex_number(input, pos);
      tokens.push(tok); pos = newpos;
    } elif c == 116 {
      pos = pos + 4;
      tokens.push(Token{ kind: 2; s: ""; n: 0.0; b: true; });
    } elif c == 102 {
      pos = pos + 5;
      tokens.push(Token{ kind: 2; s: ""; n: 0.0; b: false; });
    } elif c == 110 {
      pos = pos + 4;
      tokens.push(Token{ kind: 3; s: ""; n: 0.0; b: false; });
    } else {
      pos = pos + 1;
    };
  };
  tokens.push(Token{ kind: 10; s: ""; n: 0.0; b: false; });
  return tokens;
}

// === Parser ===
var parse_pos: Int = 0;
var parse_tokens: Vec[Token] = Vec[Token]::new();

fn cur_tok() -> Token {
  if parse_pos < parse_tokens.len() {
    return parse_tokens[parse_pos];
  };
  return Token{ kind: 10; s: ""; n: 0.0; b: false; };
}

fn advance() {
  parse_pos = parse_pos + 1;
}

fn parse_value() -> JsonValue {
  var tok = cur_tok();
  if tok.kind == 0 { advance(); return json_string(tok.s); }
  elif tok.kind == 1 { advance(); return json_number(tok.n); }
  elif tok.kind == 2 { advance(); return json_bool(tok.b); }
  elif tok.kind == 3 { advance(); return json_null(); }
  elif tok.kind == 4 { return parse_object(); }
  elif tok.kind == 6 { return parse_array(); }
  else {
    io.println("parse error at token kind " + convert.int_to_string(tok.kind));
    return json_null();
  };
}

fn parse_object() -> JsonValue {
  advance();
  var keys: Vec[Str] = Vec[Str]::new();
  var vals: Vec[JsonValue] = Vec[JsonValue]::new();
  var tok = cur_tok();
  if tok.kind == 5 { advance(); return json_object(keys, vals); };
  while true {
    if cur_tok().kind != 0 {
      io.println("expected string key");
      break;
    };
    var key = cur_tok().s;
    advance();
    if cur_tok().kind != 8 {
      io.println("expected ':'");
      break;
    };
    advance();
    var val = parse_value();
    keys.push(key);
    vals.push(val);
    tok = cur_tok();
    if tok.kind == 5 { advance(); break; }
    elif tok.kind == 9 { advance(); }
    else { break; };
  };
  return json_object(keys, vals);
}

fn parse_array() -> JsonValue {
  advance();
  var items: Vec[JsonValue] = Vec[JsonValue]::new();
  var tok = cur_tok();
  if tok.kind == 7 { advance(); return json_array(items); };
  while true {
    var val = parse_value();
    items.push(val);
    tok = cur_tok();
    if tok.kind == 7 { advance(); break; }
    elif tok.kind == 9 { advance(); }
    else { break; };
  };
  return json_array(items);
}

// === Printer ===
fn print_value(v: JsonValue, indent: Int) {
  var sp = "";
  var i = 0;
  while i < indent { sp = sp + "  "; i = i + 1; };
  if v.tag == 0 {
    io.print("null");
  } elif v.tag == 1 {
    io.print("\"" + v.str_val + "\"");
  } elif v.tag == 2 {
    if v.bool_val { io.print("true"); } else { io.print("false"); };
  } elif v.tag == 3 {
    io.print(convert.float_to_string(v.num_val));
  } elif v.tag == 4 {
    io.println("[");
    var j = 0;
    while j < v.arr_val.len() {
      io.print(sp + "  ");
      print_value(v.arr_val[j], indent + 1);
      if j + 1 < v.arr_val.len() { io.print(","); };
      io.println("");
      j = j + 1;
    };
    io.print(sp + "]");
  } elif v.tag == 5 {
    io.println("{");
    var j = 0;
    while j < v.obj_keys.len() {
      io.print(sp + "  \"" + v.obj_keys[j] + "\": ");
      print_value(v.obj_vals[j], indent + 1);
      if j + 1 < v.obj_keys.len() { io.print(","); };
      io.println("");
      j = j + 1;
    };
    io.print(sp + "}");
  };
}

// === Main ===
fn main() {
  var result = io.read_file("tools/_test.json");
  var input: Str;
  match result {
    Ok(s) => { input = s; };
    Err(e) => { io.println("error: " + e.message); return; };
  };
  io.println("lexing " + convert.int_to_string(input.len()) + " bytes...");
  parse_tokens = lex(input);
  parse_pos = 0;
  io.println("tokens: " + convert.int_to_string(parse_tokens.len()));
  var json_result = parse_value();
  io.print("parsed: ");
  print_value(json_result, 0);
  io.println("");
}
