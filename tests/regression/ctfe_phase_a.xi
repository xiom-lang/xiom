// CTFE Phase A — Production Test Suite
// Verifies compile-time constant evaluation for:
//   - Arithmetic (+, -, *, /, %)
//   - Comparison (==, !=, <, >, <=, >=)
//   - Boolean (and, or, not)
//   - Unary (negation, bitwise not)
//   - Const variable references
//   - if/else folding
//   - sizeof, align_of, type_id, field_offset builtins
//   - const { ... } blocks
// Returns 0 if all pass, non-zero on failure.

// ---- Arithmetic ----
const ADD_INT: Int = 40 + 2;        // 42
const SUB_INT: Int = 100 - 58;      // 42
const MUL_INT: Int = 6 * 7;         // 42
const DIV_INT: Int = 84 / 2;        // 42
const REM_INT: Int = 85 % 43;       // 42

const ADD_FLOAT: Float64 = 20.0 + 22.0;
const SUB_FLOAT: Float64 = 50.0 - 8.0;
const MUL_FLOAT: Float64 = 6.0 * 7.0;
const DIV_FLOAT: Float64 = 84.0 / 2.0;

// ---- Comparison (Int) ----
const EQ_INT: Bool = 42 == 42;
const NEQ_INT: Bool = 42 != 0;
const LT_INT: Bool = 10 < 20;
const GT_INT: Bool = 20 > 10;
const LE_INT: Bool = 42 <= 42;
const GE_INT: Bool = 42 >= 42;

// ---- Comparison (Float) ----
const EQ_FLOAT: Bool = 42.0 == 42.0;
const NEQ_FLOAT: Bool = 42.0 != 0.0;
const LT_FLOAT: Bool = 1.0 < 2.0;
const GT_FLOAT: Bool = 2.0 > 1.0;

// ---- Boolean ops ----
const AND_TT: Bool = true and true;
const AND_TF: Bool = true and false;
const OR_FT: Bool = false or true;
const OR_FF: Bool = false or false;
const NOT_T: Bool = not true;
const NOT_F: Bool = not false;

// ---- Unary ----
const NEG_INT: Int = -(42);         // wraps to -42
const BIT_NOT: Int = ~0;            // all-ones
const NEG_FLOAT: Float64 = -(42.0);

// ---- Const variable references ----
const BASE: Int = 21;
const DOUBLE: Int = BASE * 2;       // 42
const TRIPLE: Int = BASE + DOUBLE;  // 63

// ---- sizeof/align_of/type_id ----
const SIZEOF_INT: Int = sizeof::<Int>();
const ALIGNOF_INT: Int = align_of::<Int>();
const TYPE_ID_INT: Int = type_id::<Int>();

// ---- if/else folding (compile-time condition) ----
const IF_TRUE: Int = if true { 42 } else { 0 };
const IF_FALSE: Int = if false { 0 } else { 42 };
const IF_ELIF: Int = if false { 0 } elif true { 42 } else { 0 };

// ---- const { ... } block ----
const BLOCK_ADD: Int = const { 40 + 2 };
const BLOCK_MUL: Int = const { 6 * 7 };

// ---- is_signed builtin ----
const SIGNED_INT: Bool = is_signed::<Int>();
const SIGNED_INT8: Bool = is_signed::<Int8>();
const SIGNED_INT16: Bool = is_signed::<Int16>();
const SIGNED_INT32: Bool = is_signed::<Int32>();
const SIGNED_INT64: Bool = is_signed::<Int64>();
const SIGNED_UINT8: Bool = is_signed::<UInt8>();
const SIGNED_UINT32: Bool = is_signed::<UInt32>();
const SIGNED_BOOL: Bool = is_signed::<Bool>();
const SIGNED_FLOAT64: Bool = is_signed::<Float64>();
const SIGNED_STR: Bool = is_signed::<Str>();

// ---- Match folding (compile-time pattern matching) ----
const MATCH_INT: Int = match 42 {
  42 => 100,
  _ => 0,
};
const MATCH_BOOL: Int = match true {
  true => 1,
  false => 0,
};
const MATCH_OPT: Int = match Some(42) {
  Some(v) => v,
  None => 0,
};
const MATCH_OK: Int = match Ok(42) {
  Ok(v) => v,
  Err(_) => 0,
};
const MATCH_ERR: Int = match Err(99) {
  Ok(_) => 0,
  Err(v) => v,
};

fn main() -> Int {
  // Verify all arithmetic consts
  if ADD_INT != 42 { return 1; }
  if SUB_INT != 42 { return 2; }
  if MUL_INT != 42 { return 3; }
  if DIV_INT != 42 { return 4; }
  if REM_INT != 42 { return 5; }

  // Verify float comparison (using Int casting for simplicity)
  if ADD_FLOAT != 42.0 { return 6; }
  if SUB_FLOAT != 42.0 { return 7; }
  if MUL_FLOAT != 42.0 { return 8; }
  if DIV_FLOAT != 42.0 { return 9; }

  // Verify comparison ops
  if not EQ_INT { return 10; }
  if not NEQ_INT { return 11; }
  if not LT_INT { return 12; }
  if not GT_INT { return 13; }
  if not LE_INT { return 14; }
  if not GE_INT { return 15; }

  // Verify float comparisons
  if not EQ_FLOAT { return 16; }
  if not NEQ_FLOAT { return 17; }
  if not LT_FLOAT { return 18; }
  if not GT_FLOAT { return 19; }

  // Verify boolean ops
  if not AND_TT { return 20; }
  if AND_TF { return 21; }
  if not OR_FT { return 22; }
  if OR_FF { return 23; }
  if NOT_T { return 24; }
  if not NOT_F { return 25; }

  // Verify unary
  if NEG_INT != -42 { return 26; }
  if BIT_NOT != -1 { return 27; }
  if NEG_FLOAT != -42.0 { return 28; }

  // Verify const var references
  if BASE != 21 { return 29; }
  if DOUBLE != 42 { return 30; }
  if TRIPLE != 63 { return 31; }

  // Verify sizeof (Int is 8 bytes)
  if SIZEOF_INT != 8 { return 32; }
  // Verify align_of (Int is 8-byte aligned)
  if ALIGNOF_INT != 8 { return 33; }
  // Verify type_id is non-zero for known type
  if TYPE_ID_INT == 0 { return 34; }

  // Verify if/else folding
  if IF_TRUE != 42 { return 35; }
  if IF_FALSE != 42 { return 36; }
  if IF_ELIF != 42 { return 37; }

  // Verify const block
  if BLOCK_ADD != 42 { return 38; }
  if BLOCK_MUL != 42 { return 39; }

  // Verify is_signed builtin (use const values, not runtime calls)
  if not SIGNED_INT { return 40; }
  if not SIGNED_INT8 { return 41; }
  if not SIGNED_INT16 { return 42; }
  if not SIGNED_INT32 { return 43; }
  if not SIGNED_INT64 { return 44; }
  if SIGNED_UINT8 { return 45; }
  if SIGNED_UINT32 { return 46; }
  if SIGNED_BOOL { return 47; }
  if SIGNED_FLOAT64 { return 48; }
  if SIGNED_STR { return 49; }

  // Verify match folding (use const values)
  if MATCH_INT != 100 { return 50; }
  if MATCH_BOOL != 1 { return 51; }
  if MATCH_OPT != 42 { return 52; }
  if MATCH_OK != 42 { return 53; }
  if MATCH_ERR != 99 { return 54; }

  return 0;
}
