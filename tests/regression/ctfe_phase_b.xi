// CTFE Phase B -- Function Evaluation Tests
// Verifies compile-time evaluation of pure functions:
//   - Simple function calls with const args
//   - Recursive functions (factorial)
//   - Iterative functions (sum via while loop)
//   - if/else branching in functions
//   - Multiple function calls
// Returns 0 if all pass.

// ---- Pure function: factorial (recursive) ----
fn factorial(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}

// ---- Pure function: fibonacci (recursive) ----
fn fib(n: Int) -> Int {
  if n <= 1 { return n; }
  return fib(n - 1) + fib(n - 2);
}

// ---- Pure function: square (single-expression) ----
fn square(x: Int) -> Int {
  return x * x;
}

// ---- Pure function: is_even (boolean return) ----
fn is_even(n: Int) -> Bool {
  return n % 2 == 0;
}

// ---- Pure function: sum_to (while loop) ----
fn sum_to(n: Int) -> Int {
  var total: Int = 0;
  var i: Int = 1;
  while i <= n {
    total = total + i;
    i = i + 1;
  }
  return total;
}

// ---- Pure function: max_of_three (if/elif/else) ----
fn max_of_three(a: Int, b: Int, c: Int) -> Int {
  if a >= b && a >= c { return a; }
  elif b >= a && b >= c { return b; }
  return c;
}

// ---- Compile-time evaluated constants ----
const FACT5: Int = factorial(5);      // 120
const FACT10: Int = factorial(10);    // 3628800
const FIB10: Int = fib(10);           // 55
const SQ7: Int = square(7);           // 49
const IS_EVEN_42: Bool = is_even(42); // true
const IS_EVEN_43: Bool = is_even(43); // false
const SUM10: Int = sum_to(10);        // 55
const MAX_3_5_9: Int = max_of_three(3, 5, 9); // 9

fn main() -> Int {
  // Verify factorial
  if FACT5 != 120 { return 1; }
  if FACT10 != 3628800 { return 2; }

  // Verify fibonacci
  if FIB10 != 55 { return 3; }

  // Verify square
  if SQ7 != 49 { return 4; }

  // Verify is_even
  if not IS_EVEN_42 { return 5; }
  if IS_EVEN_43 { return 6; }

  // Verify sum_to
  if SUM10 != 55 { return 7; }

  // Verify max_of_three
  if MAX_3_5_9 != 9 { return 8; }

  return 0;
}
