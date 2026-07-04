module benchmark {
  
  use benchmark.main.BenchResult;
  
  // ============================================================
  // SECTION 1: Classic Numeric Algorithms
  // ============================================================
  
  pub fn sieve_eratosthenes(limit: Int) -> Int {
    if limit < 2 { return 0; }
    var count = 0;
    var n = 2;
    while n <= limit {
      var is_prime_n = 1;
      var d = 2;
      while d * d <= n {
        if n % d == 0 { is_prime_n = 0; d = n; }
        d = d + 1;
      }
      count = count + is_prime_n;
      n = n + 1;
    }
    return count;
  }
  
  pub fn is_prime(n: Int) -> Bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    var i = 3;
    while i * i <= n {
      if n % i == 0 { return false; }
      i = i + 2;
    }
    return true;
  }
  
  pub fn gcd(a: Int, b: Int) -> Int {
    if b == 0 { return a; }
    return gcd(b, a % b);
  }
  
  pub fn lcm(a: Int, b: Int) -> Int {
    return a * b / gcd(a, b);
  }
  
  fn test_numeric_algorithms() -> Int {
    var score = 0;
    if sieve_eratosthenes(10) == 4 { score = score + 1; }
    if sieve_eratosthenes(30) == 10 { score = score + 1; }
    if sieve_eratosthenes(1) == 0 { score = score + 1; }
  
    if is_prime(17) { score = score + 1; }
    if is_prime(97) { score = score + 1; }
    if !(is_prime(91)) { score = score + 1; }
    if !(is_prime(1)) { score = score + 1; }
  
    if gcd(48, 18) == 6 { score = score + 1; }
    if gcd(7, 13) == 1 { score = score + 1; }
  
    if lcm(4, 6) == 12 { score = score + 1; }
    if lcm(7, 11) == 77 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 2: Array Algorithms
  // ============================================================
  
  pub fn reverse(v: Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = v.len();
    while i > 0 {
      i = i - 1;
      result.push(v[i]);
    }
    return result;
  }
  
  pub fn rotate_left(v: Vec[Int], k: Int) -> Vec[Int] {
    if v.len() == 0 { return v; }
    var shift = k % v.len();
    var result = Vec[Int].new();
    var i = shift;
    while i < v.len() {
      result.push(v[i]);
      i = i + 1;
    }
    i = 0;
    while i < shift {
      result.push(v[i]);
      i = i + 1;
    }
    return result;
  }
  
  pub fn min_max(v: &Vec[Int]) -> (Int, Int) {
    if v.len() == 0 { return (0, 0); }
    var min_val = v[0];
    var max_val = v[0];
    var i = 1;
    while i < v.len() {
      if v[i] < min_val { min_val = v[i]; }
      if v[i] > max_val { max_val = v[i]; }
      i = i + 1;
    }
    return (min_val, max_val);
  }
  
  pub fn prefix_sum(v: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var running = 0;
    var i = 0;
    while i < v.len() {
      running = running + v[i];
      result.push(running);
      i = i + 1;
    }
    return result;
  }
  
  fn test_array_algorithms() -> Int {
    var score = 0;
    var nums = [1, 2, 3, 4, 5];
    var rev = reverse(nums);
    if rev[0] == 5 { score = score + 1; }
    if rev[4] == 1 { score = score + 1; }
  
    var rotated = rotate_left(nums, 2);
    if rotated[0] == 3 { score = score + 1; }
    if rotated[3] == 1 { score = score + 1; }
  
    var (min_val, max_val) = min_max(&nums);
    if min_val == 1 { score = score + 1; }
    if max_val == 5 { score = score + 1; }
  
    var pref_sum = prefix_sum(&nums);
    if pref_sum.len() == 5 { score = score + 1; }
    if pref_sum[0] == 1 { score = score + 1; }
    if pref_sum[4] == 15 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 3: Dynamic Programming
  // ============================================================
  
  pub fn fibonacci_dp(n: Int) -> Int {
    if n <= 1 { return n; }
    var prev2 = 0;
    var prev1 = 1;
    var i = 2;
    while i <= n {
      var curr = prev1 + prev2;
      prev2 = prev1;
      prev1 = curr;
      i = i + 1;
    }
    return prev1;
  }
  
  pub fn coin_change(amount: Int, coins: &Vec[Int]) -> Int {
    if amount == 0 { return 0; }
    if coins.len() == 0 { return -1; }
    var min_count = amount + 1;
    var i = 0;
    while i < coins.len() {
      if coins[i] <= amount {
        var sub_result = coin_change(amount - coins[i], coins);
        if sub_result >= 0 {
          var total = sub_result + 1;
          if total < min_count { min_count = total; }
        }
      }
      i = i + 1;
    }
    if min_count > amount { return -1; }
    return min_count;
  }
  
  pub fn max_subarray_sum(v: &Vec[Int]) -> Int {
    if v.len() == 0 { return 0; }
    var max_so_far = v[0];
    var max_ending = v[0];
    var i = 1;
    while i < v.len() {
      if max_ending + v[i] > v[i] {
        max_ending = max_ending + v[i];
      } else {
        max_ending = v[i];
      }
      if max_ending > max_so_far {
        max_so_far = max_ending;
      }
      i = i + 1;
    }
    return max_so_far;
  }
  
  fn test_dynamic_programming() -> Int {
    var score = 0;
    if fibonacci_dp(0) == 0 { score = score + 1; }
    if fibonacci_dp(1) == 1 { score = score + 1; }
    if fibonacci_dp(10) == 55 { score = score + 1; }
    if fibonacci_dp(20) == 6765 { score = score + 1; }
  
    var coins = [1, 5, 10, 25];
    if coin_change(30, &coins) == 2 { score = score + 1; }
    if coin_change(17, &coins) == 4 { score = score + 1; }
    if coin_change(0, &coins) == 0 { score = score + 1; }
  
    var arr = [-2, 1, -3, 4, -1, 2, 1, -5, 4];
    if max_subarray_sum(&arr) == 6 { score = score + 1; }
  
    var all_neg = [-5, -2, -3, -1];
    if max_subarray_sum(&all_neg) == -1 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 4: Distance & Geometry Algorithms
  // ============================================================
  
  pub fn manhattan(x1: Int, y1: Int, x2: Int, y2: Int) -> Int {
    var dx = x1 - x2;
    if dx < 0 { dx = -dx; }
    var dy = y1 - y2;
    if dy < 0 { dy = -dy; }
    return dx + dy;
  }
  
  pub fn euclidean_sq(x1: Int, y1: Int, x2: Int, y2: Int) -> Int {
    var dx = x1 - x2;
    var dy = y1 - y2;
    return dx * dx + dy * dy;
  }
  
  pub fn point_in_triangle(px: Int, py: Int, x1: Int, y1: Int, x2: Int, y2: Int, x3: Int, y3: Int) -> Bool {
    var d1 = sign_point(px, py, x1, y1, x2, y2);
    var d2 = sign_point(px, py, x2, y2, x3, y3);
    var d3 = sign_point(px, py, x3, y3, x1, y1);
    var has_neg = d1 < 0 || d2 < 0 || d3 < 0;
    var has_pos = d1 > 0 || d2 > 0 || d3 > 0;
    return !(has_neg && has_pos);
  }
  
  pub fn sign_point(px: Int, py: Int, x1: Int, y1: Int, x2: Int, y2: Int) -> Int {
    return (px - x1) * (y2 - y1) - (py - y1) * (x2 - x1);
  }
  
  fn test_geometry() -> Int {
    var score = 0;
    if manhattan(0, 0, 3, 4) == 7 { score = score + 1; }
    if manhattan(1, 1, 4, 5) == 7 { score = score + 1; }
  
    if euclidean_sq(0, 0, 3, 4) == 25 { score = score + 1; }
  
    if point_in_triangle(2, 2, 0, 0, 4, 0, 2, 4) { score = score + 1; }
    if !(point_in_triangle(10, 10, 0, 0, 4, 0, 2, 4)) { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 5: String Algorithms (Int-based simulation)
  // ============================================================
  
  pub fn levenshtein(a: &Vec[Int], b: &Vec[Int]) -> Int {
    var m = a.len();
    var n = b.len();
    if m == 0 { return n; }
    if n == 0 { return m; }
    var cost = 0;
    if a[m - 1] != b[n - 1] { cost = 1; }
    var a_prefix = slice_vec(a, 0, m - 1);
    var b_prefix = slice_vec(b, 0, n - 1);
    var d1 = levenshtein(&a_prefix, b) + 1;
    var d2 = levenshtein(a, &b_prefix) + 1;
    var d3 = levenshtein(&a_prefix, &b_prefix) + cost;
    var min_val = d1;
    if d2 < min_val { min_val = d2; }
    if d3 < min_val { min_val = d3; }
    return min_val;
  }
  
  pub fn slice_vec(v: &Vec[Int], start: Int, end: Int) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = start;
    while i < end {
      result.push(v[i]);
      i = i + 1;
    }
    return result;
  }
  
  pub fn longest_common_prefix(a: &Vec[Int], b: &Vec[Int]) -> Int {
    var count = 0;
    var min_len = a.len();
    if b.len() < min_len { min_len = b.len(); }
    var i = 0;
    while i < min_len {
      if a[i] == b[i] { count = count + 1; }
      else { return count; }
      i = i + 1;
    }
    return count;
  }
  
  fn test_string_algorithms() -> Int {
    var score = 0;
    // LCP
    var s1 = [1, 2, 3, 4, 5];
    var s2 = [1, 2, 3, 9, 0];
    if longest_common_prefix(&s1, &s2) == 3 { score = score + 1; }
  
    var s3 = [1, 2, 3, 4, 5];
    var s4 = [1, 2, 3, 4, 5];
    if longest_common_prefix(&s3, &s4) == 5 { score = score + 1; }
  
    var s5 = [9, 8, 7];
    if longest_common_prefix(&s1, &s5) == 0 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================
  
  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;
  
    var s1 = test_numeric_algorithms();
    total = total + s1;
    max_score = max_score + 11;
  
    var s2 = test_array_algorithms();
    total = total + s2;
    max_score = max_score + 8;
  
    var s3 = test_dynamic_programming();
    total = total + s3;
    max_score = max_score + 9;
  
    var s4 = test_geometry();
    total = total + s4;
    max_score = max_score + 5;
  
    var s5 = test_string_algorithms();
    total = total + s5;
    max_score = max_score + 3;
  
    return BenchResult{
      name: "algorithms",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }
  
  use benchmark.main.BenchResult;
  
  // ============================================================
  // SECTION 1: Basic Closure Definitions
  // ============================================================
  
  fn test_basic_closures() -> Int {
    var score = 0;
  
    // Named function-style closures
    var doubler = fn(x: Int) -> Int { return x * 2; };
    if doubler(5) == 10 { score = score + 1; }
    if doubler(0) == 0 { score = score + 1; }
    if doubler(-3) == -6 { score = score + 1; }
  
    var tripler = fn(x: Int) -> Int { return x * 3; };
    if tripler(7) == 21 { score = score + 1; }
  
    var square = fn(x: Int) -> Int { return x * x; };
    if square(5) == 25 { score = score + 1; }
    if square(10) == 100 { score = score + 1; }
  
    var is_even = fn(n: Int) -> Bool { return n % 2 == 0; };
    if is_even(2) { score = score + 1; }
    if !(is_even(3)) { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 2: Higher-Order Functions
  // ============================================================
  
  fn apply_int(f: fn(Int) -> Int, x: Int) -> Int {
    return f(x);
  }
  
  fn apply_bool(f: fn(Int) -> Bool, x: Int) -> Bool {
    return f(x);
  }
  
  fn compose_ints(f: fn(Int) -> Int, g: fn(Int) -> Int, x: Int) -> Int {
    return f(g(x));
  }
  
  fn apply_twice(f: fn(Int) -> Int, x: Int) -> Int {
    return f(f(x));
  }
  
  fn test_higher_order() -> Int {
    var score = 0;
    var add5 = fn(x: Int) -> Int { return x + 5; };
    var mul3 = fn(x: Int) -> Int { return x * 3; };
    var is_pos = fn(x: Int) -> Bool { return x > 0; };
  
    if apply_int(add5, 10) == 15 { score = score + 1; }
    if apply_int(mul3, 7) == 21 { score = score + 1; }
  
    if apply_bool(is_pos, 5) { score = score + 1; }
    if !(apply_bool(is_pos, -3)) { score = score + 1; }
  
    if compose_ints(add5, mul3, 4) == 17 { score = score + 1; }
    if compose_ints(mul3, add5, 4) == 27 { score = score + 1; }
  
    if apply_twice(add5, 0) == 10 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 3: Closure as Parameters
  // ============================================================
  
  fn map_vec(v: &Vec[Int], f: fn(Int) -> Int) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      result.push(f(v[i]));
      i = i + 1;
    }
    return result;
  }
  
  fn filter_vec(v: &Vec[Int], pred: fn(Int) -> Bool) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      if pred(v[i]) { result.push(v[i]); }
      i = i + 1;
    }
    return result;
  }
  
  fn fold_vec(v: &Vec[Int], initial: Int, f: fn(Int, Int) -> Int) -> Int {
    var accum = initial;
    var i = 0;
    while i < v.len() {
      accum = f(accum, v[i]);
      i = i + 1;
    }
    return accum;
  }
  
  fn test_closure_params() -> Int {
    var score = 0;
    var nums = [1, 2, 3, 4, 5];
  
    var doubled = map_vec(&nums, fn(x: Int) -> Int { return x * 2; });
    if doubled[0] == 2 { score = score + 1; }
    if doubled[4] == 10 { score = score + 1; }
  
    var evens = filter_vec(&nums, fn(n: Int) -> Bool { return n % 2 == 0; });
    if evens.len() == 2 { score = score + 1; }
    if evens[0] == 2 { score = score + 1; }
  
    var sum = fold_vec(&nums, 0, fn(a: Int, b: Int) -> Int { return a + b; });
    if sum == 15 { score = score + 1; }
  
    var product = fold_vec(&nums, 1, fn(a: Int, b: Int) -> Int { return a * b; });
    if product == 120 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 4: Closure Factories
  // ============================================================
  
  fn make_multiplier(factor: Int) -> fn(Int) -> Int {
    return fn(x: Int) -> Int { return x * factor; };
  }
  
  fn make_adder(amount: Int) -> fn(Int) -> Int {
    return fn(x: Int) -> Int { return x + amount; };
  }
  
  fn make_discriminant(pivot: Int) -> fn(Int) -> Int {
    return fn(x: Int) -> Int {
      if x > pivot { return 1; }
      if x < pivot { return -1; }
      return 0;
    };
  }
  
  fn test_closure_factories() -> Int {
    var score = 0;
    var times10 = make_multiplier(10);
    if times10(5) == 50 { score = score + 1; }
    if times10(0) == 0 { score = score + 1; }
  
    var times3 = make_multiplier(3);
    if times3(7) == 21 { score = score + 1; }
  
    var add100 = make_adder(100);
    if add100(50) == 150 { score = score + 1; }
  
    var disc = make_discriminant(10);
    if disc(20) == 1 { score = score + 1; }
    if disc(5) == -1 { score = score + 1; }
    if disc(10) == 0 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 5: Closure Chaining
  // ============================================================
  
  fn chain_two(f: fn(Int) -> Int, g: fn(Int) -> Int) -> fn(Int) -> Int {
    return fn(x: Int) -> Int { return f(g(x)); };
  }
  
  fn chain_three(f: fn(Int) -> Int, g: fn(Int) -> Int, h: fn(Int) -> Int) -> fn(Int) -> Int {
    return fn(x: Int) -> Int { return f(g(h(x))); };
  }
  
  fn test_closure_chains() -> Int {
    var score = 0;
    var add1 = fn(x: Int) -> Int { return x + 1; };
    var double = fn(x: Int) -> Int { return x * 2; };
    var square = fn(x: Int) -> Int { return x * x; };
  
    var d_plus_1 = chain_two(add1, double);
    if d_plus_1(5) == 11 { score = score + 1; }
  
    var chain3 = chain_three(add1, double, square);
    if chain3(3) == 19 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================
  
  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;
  
    var s1 = test_basic_closures();
    total = total + s1;
    max_score = max_score + 8;
  
    var s2 = test_higher_order();
    total = total + s2;
    max_score = max_score + 8;
  
    var s3 = test_closure_params();
    total = total + s3;
    max_score = max_score + 6;
  
    var s4 = test_closure_factories();
    total = total + s4;
    max_score = max_score + 8;
  
    var s5 = test_closure_chains();
    total = total + s5;
    max_score = max_score + 2;
  
    return BenchResult{
      name: "closures",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }
  
  use benchmark.main.BenchResult;
  
  // ============================================================
  // SECTION 1: Vec Operations at Scale
  // ============================================================
  
  pub fn vec_range(start: Int, end: Int) -> Vec[Int] {
    var v = Vec[Int].new();
    var i = start;
    while i < end {
      v.push(i);
      i = i + 1;
    }
    return v;
  }
  
  pub fn vec_sum(v: &Vec[Int]) -> Int {
    var sum = 0;
    var i = 0;
    while i < v.len() {
      sum = sum + v[i];
      i = i + 1;
    }
    return sum;
  }
  
  pub fn vec_product(v: &Vec[Int]) -> Int {
    if v.len() == 0 { return 0; }
    var prod = 1;
    var i = 0;
    while i < v.len() {
      prod = prod * v[i];
      i = i + 1;
    }
    return prod;
  }
  
  pub fn vec_copy(v: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      result.push(v[i]);
      i = i + 1;
    }
    return result;
  }
  
  pub fn vec_append(a: &Vec[Int], b: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < a.len() {
      result.push(a[i]);
      i = i + 1;
    }
    i = 0;
    while i < b.len() {
      result.push(b[i]);
      i = i + 1;
    }
    return result;
  }
  
  pub fn vec_zip(a: &Vec[Int], b: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var min_len = a.len();
    if b.len() < min_len { min_len = b.len(); }
    var i = 0;
    while i < min_len {
      result.push(a[i] + b[i]);
      i = i + 1;
    }
    return result;
  }
  
  fn test_vec_ops() -> Int {
    var score = 0;
    var v = vec_range(1, 11);
    if v.len() == 10 { score = score + 1; }
    if v[0] == 1 { score = score + 1; }
    if v[9] == 10 { score = score + 1; }
    if vec_sum(&v) == 55 { score = score + 1; }
  
    var v2 = vec_range(1, 6);
    if vec_product(&v2) == 120 { score = score + 1; }
  
    var v_copy = vec_copy(&v);
    if v_copy.len() == 10 { score = score + 1; }
    if v_copy[0] == 1 { score = score + 1; }
  
    var a = [1, 2, 3];
    var b = [4, 5, 6];
    var appended = vec_append(&a, &b);
    if appended.len() == 6 { score = score + 1; }
    if appended[0] == 1 { score = score + 1; }
    if appended[5] == 6 { score = score + 1; }
  
    var zipped = vec_zip(&a, &b);
    if zipped.len() == 3 { score = score + 1; }
    if zipped[0] == 5 { score = score + 1; }
    if zipped[2] == 9 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 2: Sorting Algorithms
  // ============================================================
  
  pub fn bubble_sort(v: Vec[Int]) -> Vec[Int] {
    var arr = vec_copy(&v);
    var n = arr.len();
    var i = 0;
    while i < n {
      var j = 0;
      while j < n - i - 1 {
        if arr[j] > arr[j + 1] {
          var temp = arr[j];
          arr[j] = arr[j + 1];
          arr[j + 1] = temp;
        }
        j = j + 1;
      }
      i = i + 1;
    }
    return arr;
  }
  
  pub fn insertion_sort(v: Vec[Int]) -> Vec[Int] {
    var arr = vec_copy(&v);
    var i = 1;
    while i < arr.len() {
      var key = arr[i];
      var j = i;
      while j > 0 && arr[j - 1] > key {
        arr[j] = arr[j - 1];
        j = j - 1;
      }
      arr[j] = key;
      i = i + 1;
    }
    return arr;
  }
  
  pub fn is_sorted(v: &Vec[Int]) -> Bool {
    var i = 1;
    while i < v.len() {
      if v[i - 1] > v[i] { return false; }
      i = i + 1;
    }
    return true;
  }
  
  fn test_sorting() -> Int {
    var score = 0;
    var unsorted = [5, 2, 8, 1, 9, 3, 7, 4, 6];
  
    var sorted_bubble = bubble_sort(unsorted);
    if is_sorted(&sorted_bubble) { score = score + 1; }
    if sorted_bubble[0] == 1 { score = score + 1; }
    if sorted_bubble[sorted_bubble.len() - 1] == 9 { score = score + 1; }
    if sorted_bubble.len() == 9 { score = score + 1; }
  
    var unsorted2 = [9, 8, 7, 6, 5, 4, 3, 2, 1];
    var sorted_insert = insertion_sort(unsorted2);
    if is_sorted(&sorted_insert) { score = score + 1; }
    if sorted_insert[0] == 1 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 3: Searching
  // ============================================================
  
  pub fn linear_search(v: &Vec[Int], target: Int) -> Int {
    var i = 0;
    while i < v.len() {
      if v[i] == target { return i; }
      i = i + 1;
    }
    return -1;
  }
  
  pub fn binary_search(sorted: &Vec[Int], target: Int) -> Int {
    var lo = 0;
    var hi = sorted.len();
    if hi == 0 { return -1; }
    hi = hi - 1;
    while lo <= hi {
      var mid = (lo + hi) / 2;
      if sorted[mid] == target { return mid; }
      if sorted[mid] < target {
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }
    return -1;
  }
  
  pub fn count_occurrences(v: &Vec[Int], target: Int) -> Int {
    var count = 0;
    var i = 0;
    while i < v.len() {
      if v[i] == target { count = count + 1; }
      i = i + 1;
    }
    return count;
  }
  
  fn test_searching() -> Int {
    var score = 0;
    var nums = [3, 7, 2, 9, 1, 5, 8, 4, 6];
    if linear_search(&nums, 7) == 1 { score = score + 1; }
    if linear_search(&nums, 99) == -1 { score = score + 1; }
  
    var empty: Vec[Int] = [];
    if linear_search(&empty, 5) == -1 { score = score + 1; }
  
    var sorted = [1, 3, 5, 7, 9, 11, 13, 15];
    if binary_search(&sorted, 7) == 3 { score = score + 1; }
    if binary_search(&sorted, 1) == 0 { score = score + 1; }
    if binary_search(&sorted, 15) == 7 { score = score + 1; }
    if binary_search(&sorted, 8) == -1 { score = score + 1; }
    if binary_search(&empty, 5) == -1 { score = score + 1; }
  
    var repeats = [1, 2, 2, 3, 2, 4, 2];
    if count_occurrences(&repeats, 2) == 4 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 4: 2D Vector Matrix Operations
  // ============================================================
  
  pub fn matrix_new(rows: Int, cols: Int) -> Vec[Vec[Int]] {
    var m = Vec[Vec[Int]].new();
    var i = 0;
    while i < rows {
      var row = Vec[Int].new();
      var j = 0;
      while j < cols {
        row.push(0);
        j = j + 1;
      }
      m.push(row);
      i = i + 1;
    }
    return m;
  }
  
  pub fn matrix_fill(mut m: Vec[Vec[Int]], val: Int) -> Vec[Vec[Int]] {
    var i = 0;
    while i < m.len() {
      var j = 0;
      while j < m[i].len() {
        m[i][j] = val;
        j = j + 1;
      }
      i = i + 1;
    }
    return m;
  }
  
  pub fn matrix_sum(m: &Vec[Vec[Int]]) -> Int {
    var total = 0;
    var i = 0;
    while i < m.len() {
      var j = 0;
      while j < m[i].len() {
        total = total + m[i][j];
        j = j + 1;
      }
      i = i + 1;
    }
    return total;
  }
  
  fn test_matrix() -> Int {
    var score = 0;
    var m = matrix_new(3, 4);
    if m.len() == 3 { score = score + 1; }
    if m[0].len() == 4 { score = score + 1; }
  
    var m2 = matrix_fill(m, 5);
    if m2[0][0] == 5 { score = score + 1; }
    if m2[2][3] == 5 { score = score + 1; }
    if matrix_sum(&m2) == 60 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 5: Partition and Group
  // ============================================================
  
  pub fn partition(v: &Vec[Int], pred: fn(Int) -> Bool) -> (Vec[Int], Vec[Int]) {
    var yes = Vec[Int].new();
    var no = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      if pred(v[i]) { yes.push(v[i]); } else { no.push(v[i]); }
      i = i + 1;
    }
    return (yes, no);
  }
  
  pub fn dedup(v: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      var j = 0;
      var found = 0;
      while j < result.len() && found == 0 {
        if result[j] == v[i] { found = 1; }
        j = j + 1;
      }
      if found == 0 { result.push(v[i]); }
      i = i + 1;
    }
    return result;
  }
  
  pub fn unique_count(v: &Vec[Int]) -> Int {
    return dedup(v).len();
  }
  
  fn is_even(n: Int) -> Bool { return n % 2 == 0; }
  
  fn test_partition() -> Int {
    var score = 0;
    var nums = [1, 2, 3, 4, 5, 6];
    var (evens, odds) = partition(&nums, is_even);
    if evens.len() == 3 { score = score + 1; }
    if odds.len() == 3 { score = score + 1; }
  
    var with_dupes = [1, 2, 2, 3, 3, 3, 4];
    if unique_count(&with_dupes) == 4 { score = score + 1; }
  
    var deduped = dedup(&with_dupes);
    if deduped.len() == 4 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================
  
  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;
  
    var s1 = test_vec_ops();
    total = total + s1;
    max_score = max_score + 13;
  
    var s2 = test_sorting();
    total = total + s2;
    max_score = max_score + 6;
  
    var s3 = test_searching();
    total = total + s3;
    max_score = max_score + 9;
  
    var s4 = test_matrix();
    total = total + s4;
    max_score = max_score + 5;
  
    var s5 = test_partition();
    total = total + s5;
    max_score = max_score + 4;
  
    return BenchResult{
      name: "collections",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }
  
  use benchmark.main.BenchResult;
  
  // ============================================================
  // SECTION 1: Constant Expressions
  // ============================================================
  
  // The compiler should fold these at compile time
  pub fn constant_math() -> Int {
    var a = 42;
    var b = 100;
    var c = a * 2;
    var d = b / 2;
    var e = c + d;
    var f = e * 3;
    var g = f / 7;
    return g;
  }
  
  pub fn constant_bool() -> Bool {
    var a = true;
    var b = false;
    var c = a && true;
    var d = b || false;
    var e = c || d;
    var f = !e;
    return !f;
  }
  
  fn test_constants() -> Int {
    var score = 0;
    if constant_math() == 60 { score = score + 1; }
    if constant_bool() { score = score + 1; }
    return score;
  }
  
  // ============================================================
  // SECTION 2: Function-Level Constant Propagation
  // ============================================================
  
  pub fn fold_add(a: Int, b: Int) -> Int {
    return a + b;
  }
  
  pub fn fold_mul(a: Int, b: Int) -> Int {
    return a * b;
  }
  
  pub fn fold_nested(a: Int, b: Int, c: Int) -> Int {
    var sum = fold_add(a, b);
    return fold_mul(sum, c);
  }
  
  pub fn fold_triple(a: Int) -> Int {
    var x = fold_add(a, 1);
    var y = fold_mul(x, 2);
    return fold_add(y, 3);
  }
  
  fn test_folding() -> Int {
    var score = 0;
    if fold_add(100, 200) == 300 { score = score + 1; }
    if fold_mul(6, 7) == 42 { score = score + 1; }
    if fold_nested(2, 3, 4) == 20 { score = score + 1; }
    if fold_triple(5) == 15 { score = score + 1; }
    return score;
  }
  
  // ============================================================
  // SECTION 3: Loop with Known Iterations
  // ============================================================
  
  pub fn loop_known_iters() -> Int {
    var sum = 0;
    var i = 0;
    while i < 10 {
      sum = sum + i;
      i = i + 1;
    }
    return sum;
  }
  
  pub fn nested_known_loop() -> Int {
    var total = 0;
    var i = 0;
    while i < 5 {
      var j = 0;
      while j < 3 {
        total = total + i * j;
        j = j + 1;
      }
      i = i + 1;
    }
    return total;
  }
  
  fn test_loop_folding() -> Int {
    var score = 0;
    if loop_known_iters() == 45 { score = score + 1; }
    if nested_known_loop() == 30 { score = score + 1; }
    return score;
  }
  
  // ============================================================
  // SECTION 4: Inline-Capable Functions
  // ============================================================
  
  pub fn is_even(n: Int) -> Bool { return n % 2 == 0; }
  pub fn is_odd(n: Int) -> Bool { return n % 2 != 0; }
  
  pub fn parity_check(n: Int) -> Bool {
    if is_even(n) { return true; }
    return false;
  }
  
  pub fn parity_negate(n: Int) -> Bool {
    if is_odd(n) { return false; }
    return true;
  }
  
  fn test_inline() -> Int {
    var score = 0;
    if parity_check(2) { score = score + 1; }
    if !(parity_check(3)) { score = score + 1; }
    if parity_negate(2) { score = score + 1; }
    if !(parity_negate(3)) { score = score + 1; }
    return score;
  }
  
  // ============================================================
  // SECTION 5: Aggregate Runner
  // ============================================================
  
  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;
  
    var s1 = test_constants();
    total = total + s1;
    max_score = max_score + 2;
  
    var s2 = test_folding();
    total = total + s2;
    max_score = max_score + 4;
  
    var s3 = test_loop_folding();
    total = total + s3;
    max_score = max_score + 2;
  
    var s4 = test_inline();
    total = total + s4;
    max_score = max_score + 4;
  
    return BenchResult{
      name: "comptime",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }
  
  use benchmark.main.BenchResult;
  
  // ============================================================
  // SECTION 1: Simulated Workers (sequential for now, structural for compile check)
  // ============================================================
  
  pub type WorkerState = {
    id: Int;
    task_count: Int;
    completed: Int;
  } derive[Clone]
  
  pub fn WorkerState.new(id: Int) -> WorkerState {
    return WorkerState{ id: id, task_count: 0, completed: 0 };
  }
  
  pub fn WorkerState.assign_tasks(n: Int) -> WorkerState {
    return WorkerState{ id: id, task_count: task_count + n, completed: completed };
  }
  
  pub fn WorkerState.complete_one() -> WorkerState {
    if completed < task_count {
      return WorkerState{ id: id, task_count: task_count, completed: completed + 1 };
    }
    return WorkerState{ id: id, task_count: task_count, completed: completed };
  }
  
  pub fn WorkerState.is_done() -> Bool {
    return completed >= task_count;
  }
  
  pub fn WorkerState.progress() -> Int {
    if task_count == 0 { return 0; }
    return completed * 100 / task_count;
  }
  
  fn test_worker() -> Int {
    var score = 0;
    var w = WorkerState.new(1);
    if w.id == 1 { score = score + 1; }
    if w.task_count == 0 { score = score + 1; }
    if w.progress() == 0 { score = score + 1; }
  
    var w1 = w.assign_tasks(10);
    if w1.task_count == 10 { score = score + 1; }
  
    var w2 = w1.complete_one().complete_one().complete_one();
    if w2.progress() == 30 { score = score + 1; }
  
    // Complete rest
    var wf = w2;
    var i = 0;
    while i < 7 {
      wf = wf.complete_one();
      i = i + 1;
    }
    if wf.is_done() { score = score + 1; }
    if wf.progress() == 100 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 2: Channel Simulation (immutable-style prod/cons)
  // ============================================================
  
  pub type Channel[T] = {
    items: Vec[T];
  } derive[Clone]
  
  pub fn Channel.new[T]() -> Channel[T] {
    return Channel[T]{ items: Vec[T].new() };
  }
  
  pub fn Channel.push[T](item: T) -> Channel[T] {
    var new_items = items.clone();
    new_items.push(item);
    return Channel[T]{ items: new_items };
  }
  
  pub fn Channel.len[T]() -> Int {
    return items.len();
  }
  
  pub fn Channel.sum[T]() -> Int {
    var total = 0;
    var i = 0;
    while i < items.len() {
      total = total + items[i];
      i = i + 1;
    }
    return total;
  }
  
  fn test_channel() -> Int {
    var score = 0;
    var ch: Channel[Int] = Channel.new[Int]();
    if ch.len() == 0 { score = score + 1; }
  
    var ch1 = ch.push(10);
    var ch2 = ch1.push(20);
    var ch3 = ch2.push(30);
  
    if ch3.len() == 3 { score = score + 1; }
    if ch3.sum() == 60 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 3: Lock/Unlock Simulation (immutable state transitions)
  // ============================================================
  
  pub type Guard = {
    value: Int;
    is_locked: Int;
  } derive[Clone]
  
  pub fn Guard.new(val: Int) -> Guard {
    return Guard{ value: val, is_locked: 0 };
  }
  
  pub fn Guard.lock() -> Guard {
    return Guard{ value: value, is_locked: 1 };
  }
  
  pub fn Guard.unlock() -> Guard {
    return Guard{ value: value, is_locked: 0 };
  }
  
  pub fn Guard.is_locked() -> Bool {
    return is_locked == 1;
  }
  
  pub fn Guard.get() -> Int {
    return value;
  }
  
  pub fn Guard.add(amount: Int) -> Guard {
    return Guard{ value: value + amount, is_locked: is_locked };
  }
  
  fn test_guard() -> Int {
    var score = 0;
    var g = Guard.new(42);
  
    if !(g.is_locked()) { score = score + 1; }
    if g.get() == 42 { score = score + 1; }
  
    var g2 = g.lock();
    if g2.is_locked() { score = score + 1; }
  
    var g3 = g2.add(8);
    if g3.get() == 50 { score = score + 1; }
  
    var g4 = g3.unlock();
    if !(g4.is_locked()) { score = score + 1; }
    if g4.get() == 50 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 4: Aggregate Runner
  // ============================================================
  
  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;
  
    var s1 = test_worker();
    total = total + s1;
    max_score = max_score + 8;
  
    var s2 = test_channel();
    total = total + s2;
    max_score = max_score + 3;
  
    var s3 = test_guard();
    total = total + s3;
    max_score = max_score + 5;
  
    return BenchResult{
      name: "concurrency",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }
  
  use benchmark.main.BenchResult;
  
  // ============================================================
  // SECTION 1: Simple Requires/Ensures
  // ============================================================
  
  fn safe_divide(a: Float64, b: Float64) -> Float64
    requires: b != 0.0
  {
    return a / b;
  }
  
  fn safe_sqrt(x: Float64) -> Float64
    requires: x >= 0.0
  {
    // Newton's method
    if x == 0.0 { return 0.0; }
    var guess = x / 2.0;
    var i = 0;
    while i < 50 {
      var next_guess = (guess + x / guess) / 2.0;
      guess = next_guess;
      i = i + 1;
    }
    return guess;
  }
  
  fn abs_float(x: Float64) -> Float64 {
    if x < 0.0 { return -x; }
    return x;
  }
  
  fn test_requires_ensures() -> Int {
    var score = 0;
    var d1 = safe_divide(10.0, 2.0);
    var d1_ok = d1 > 4.9 && d1 < 5.1;
    if d1_ok { score = score + 1; }
  
    var d2 = safe_divide(1.0, 4.0);
    var d2_ok = d2 > 0.24 && d2 < 0.26;
    if d2_ok { score = score + 1; }
  
    var s1 = safe_sqrt(4.0);
    var s1_ok = s1 > 1.9 && s1 < 2.1;
    if s1_ok { score = score + 1; }
  
    var s2 = safe_sqrt(0.0);
    if s2 == 0.0 { score = score + 1; }
  
    var s3 = safe_sqrt(100.0);
    var s3_ok = s3 > 9.9 && s3 < 10.1;
    if s3_ok { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 2: Type Invariants
  // ============================================================
  
  pub type PositiveInt = {
    value: Int;
    invariant: value > 0;
  }
  
  pub type BoundedInt = {
    value: Int;
    lo: Int;
    hi: Int;
    invariant: value >= lo;
    invariant: value <= hi;
  }
  
  pub fn PositiveInt.new(val: Int) -> PositiveInt
    requires: val > 0
  {
    return PositiveInt{ value: val };
  }
  
  pub fn PositiveInt.add(other: &PositiveInt) -> PositiveInt
    requires: value + other.value > 0
  {
    return PositiveInt{ value: value + other.value };
  }
  
  pub fn BoundedInt.new(val: Int, lo: Int, hi: Int) -> BoundedInt
    requires: val >= lo
    requires: val <= hi
    requires: lo <= hi
  {
    return BoundedInt{ value: val, lo: lo, hi: hi };
  }
  
  pub fn BoundedInt.inc() -> BoundedInt
    requires: value < hi
    ensures: result.value == value@pre + 1
  {
    return BoundedInt{ value: value + 1, lo: lo, hi: hi };
  }
  
  pub fn BoundedInt.dec() -> BoundedInt
    requires: value > lo
    ensures: result.value == value@pre - 1
  {
    return BoundedInt{ value: value - 1, lo: lo, hi: hi };
  }
  
  fn test_invariants() -> Int {
    var score = 0;
    var pi = PositiveInt.new(5);
    if pi.value == 5 { score = score + 1; }
  
    var pi2 = PositiveInt.new(10);
    var pi3 = pi.add(&pi2);
    if pi3.value == 15 { score = score + 1; }
  
    var bi = BoundedInt.new(5, 0, 10);
    if bi.value == 5 { score = score + 1; }
    if bi.lo == 0 { score = score + 1; }
    if bi.hi == 10 { score = score + 1; }
  
    var bi2 = bi.inc();
    if bi2.value == 6 { score = score + 1; }
  
    var bi3 = bi2.inc().inc().inc();
    if bi3.value == 9 { score = score + 1; }
  
    var bi4 = bi3.dec();
    if bi4.value == 8 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 3: Stack with Invariants
  // ============================================================
  
  pub type SafeStack[T] = {
    items: Vec[T];
    capacity: Int;
    invariant: items.len() <= capacity;
    invariant: capacity > 0;
  }
  
  pub fn SafeStack.new[T](cap: Int) -> SafeStack[T]
    requires: cap > 0
  {
    return SafeStack[T]{ items: Vec[T].new(), capacity: cap };
  }
  
  pub fn SafeStack.push[T](val: T) -> Bool {
    if items.len() >= capacity { return false; }
    items.push(val);
    return true;
  }
  
  pub fn SafeStack.len[T]() -> Int {
    return items.len();
  }
  
  pub fn SafeStack.is_full[T]() -> Bool {
    return items.len() >= capacity;
  }
  
  pub fn SafeStack.is_empty[T]() -> Bool {
    return items.len() == 0;
  }
  
  fn test_safe_stack() -> Int {
    var score = 0;
    var ss: SafeStack[Int] = SafeStack.new[Int](3);
    if ss.len() == 0 { score = score + 1; }
    if ss.is_empty() { score = score + 1; }
    if !(ss.is_full()) { score = score + 1; }
  
    if ss.push(1) { score = score + 1; }
    if ss.push(2) { score = score + 1; }
    if ss.push(3) { score = score + 1; }
    if ss.is_full() { score = score + 1; }
    if !(ss.push(4)) { score = score + 1; }
    if ss.len() == 3 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 4: Range with Invariants
  // ============================================================
  
  pub type Range[T: Eq + Ord] = {
    lo: T;
    hi: T;
    invariant: lo <= hi;
  }
  
  pub fn Range.new[T: Eq + Ord](lo: T, hi: T) -> Range[T]
    requires: lo <= hi
  {
    return Range[T]{ lo: lo, hi: hi };
  }
  
  pub fn Range.contains[T: Eq + Ord](val: T) -> Bool {
    return val >= lo && val <= hi;
  }
  
  pub fn Range.width[T: Eq + Ord](val: T) -> Int {
    return 0;
  }
  
  fn test_range_invariant() -> Int {
    var score = 0;
    var r: Range[Int] = Range.new[Int](0, 100);
    if r.lo == 0 { score = score + 1; }
    if r.hi == 100 { score = score + 1; }
    if r.contains(50) { score = score + 1; }
    if r.contains(0) { score = score + 1; }
    if r.contains(100) { score = score + 1; }
    if !(r.contains(101)) { score = score + 1; }
    if !(r.contains(-1)) { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 5: Account with Complex Contracts
  // ============================================================
  
  pub type Account = {
    balance: Int;
    overdraft: Int;
    invariant: balance >= -overdraft;
    invariant: overdraft >= 0;
  }
  
  pub fn Account.new(initial: Int, overdraft_limit: Int) -> Account
    requires: initial >= -overdraft_limit
    requires: overdraft_limit >= 0
  {
    return Account{ balance: initial, overdraft: overdraft_limit };
  }
  
  pub fn Account.deposit(amount: Int) -> Account
    requires: amount > 0
    ensures: result.balance == balance@pre + amount
  {
    return Account{ balance: balance + amount, overdraft: overdraft };
  }
  
  pub fn Account.withdraw(amount: Int) -> Account
    requires: amount > 0
    requires: balance - amount >= -overdraft
    ensures: result.balance == balance@pre - amount
  {
    return Account{ balance: balance - amount, overdraft: overdraft };
  }
  
  pub fn Account.available() -> Int {
    return balance + overdraft;
  }
  
  fn test_account() -> Int {
    var score = 0;
    var acc = Account.new(100, 50);
    if acc.balance == 100 { score = score + 1; }
    if acc.overdraft == 50 { score = score + 1; }
    if acc.available() == 150 { score = score + 1; }
  
    var acc2 = acc.deposit(50);
    if acc2.balance == 150 { score = score + 1; }
  
    var acc3 = acc2.withdraw(30);
    if acc3.balance == 120 { score = score + 1; }
  
    // Withdraw to max overdraft
    var acc4 = acc3.withdraw(170);
    if acc4.balance == -50 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 6: Multiple Invariants
  // ============================================================
  
  pub type Health = {
    current: Int;
    maximum: Int;
    invariant: current >= 0;
    invariant: current <= maximum;
    invariant: maximum > 0;
  }
  
  pub fn Health.new(max_val: Int) -> Health
    requires: max_val > 0
  {
    return Health{ current: max_val, maximum: max_val };
  }
  
  pub fn Health.damage(amount: Int) -> Health
    requires: amount >= 0
    ensures: result.current >= 0
  {
    var new_val = current - amount;
    if new_val < 0 { new_val = 0; }
    return Health{ current: new_val, maximum: maximum };
  }
  
  pub fn Health.heal(amount: Int) -> Health
    requires: amount >= 0
    ensures: result.current <= maximum
  {
    var new_val = current + amount;
    if new_val > maximum { new_val = maximum; }
    return Health{ current: new_val, maximum: maximum };
  }
  
  pub fn Health.is_alive() -> Bool {
    return current > 0;
  }
  
  pub fn Health.health_ratio() -> Int {
    return current * 100 / maximum;
  }
  
  fn test_health() -> Int {
    var score = 0;
    var hp = Health.new(100);
    if hp.current == 100 { score = score + 1; }
    if hp.maximum == 100 { score = score + 1; }
    if hp.is_alive() { score = score + 1; }
    if hp.health_ratio() == 100 { score = score + 1; }
  
    var hp2 = hp.damage(30);
    if hp2.current == 70 { score = score + 1; }
    if hp2.health_ratio() == 70 { score = score + 1; }
  
    var hp3 = hp2.heal(10);
    if hp3.current == 80 { score = score + 1; }
  
    // Over-heal shouldn't exceed max
    var hp4 = hp3.heal(50);
    if hp4.current == 100 { score = score + 1; }
  
    // Over-damage shouldn't go negative
    var hp5 = hp4.damage(200);
    if hp5.current == 0 { score = score + 1; }
    if !(hp5.is_alive()) { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 7: Contract Utility Functions
  // ============================================================
  
  pub fn is_sorted(items: &Vec[Int]) -> Bool {
    if items.len() <= 1 { return true; }
    var i = 1;
    while i < items.len() {
      if items[i - 1] > items[i] { return false; }
      i = i + 1;
    }
    return true;
  }
  
  pub fn all_positive(items: &Vec[Int]) -> Bool {
    var i = 0;
    while i < items.len() {
      if items[i] <= 0 { return false; }
      i = i + 1;
    }
    return true;
  }
  
  pub fn any_even(items: &Vec[Int]) -> Bool {
    var i = 0;
    while i < items.len() {
      if items[i] % 2 == 0 { return true; }
      i = i + 1;
    }
    return false;
  }
  
  fn test_contract_utils() -> Int {
    var score = 0;
    var empty: Vec[Int] = [];
    if is_sorted(&empty) { score = score + 1; }
  
    var sorted = [1, 2, 3, 4, 5];
    if is_sorted(&sorted) { score = score + 1; }
  
    var unsorted = [3, 1, 4, 2];
    if !(is_sorted(&unsorted)) { score = score + 1; }
  
    var all_pos = [5, 10, 15];
    if all_positive(&all_pos) { score = score + 1; }
  
    var has_neg = [5, -1, 10];
    if !(all_positive(&has_neg)) { score = score + 1; }
  
    var has_even = [1, 2, 3];
    if any_even(&has_even) { score = score + 1; }
  
    var all_odd = [1, 3, 5, 7];
    if !(any_even(&all_odd)) { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 8: Aggregate Runner
  // ============================================================
  
  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;
  
    var s1 = test_requires_ensures();
    total = total + s1;
    max_score = max_score + 5;
  
    var s2 = test_invariants();
    total = total + s2;
    max_score = max_score + 8;
  
    var s3 = test_safe_stack();
    total = total + s3;
    max_score = max_score + 8;
  
    var s4 = test_range_invariant();
    total = total + s4;
    max_score = max_score + 7;
  
    var s5 = test_account();
    total = total + s5;
    max_score = max_score + 7;
  
    var s6 = test_health();
    total = total + s6;
    max_score = max_score + 11;
  
    var s7 = test_contract_utils();
    total = total + s7;
    max_score = max_score + 7;
  
    return BenchResult{
      name: "contracts",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }
  
  use benchmark.main.BenchResult;
  
  // ============================================================
  // SECTION 1: Deep If/Elif/Else Chains
  // ============================================================
  
  pub fn classify_number(n: Int) -> Int {
    if n == 0 { return 0; }
    elif n == 1 { return 1; }
    elif n == 2 { return 2; }
    elif n == 3 { return 3; }
    elif n == 4 { return 4; }
    elif n == 5 { return 5; }
    elif n == 6 { return 6; }
    elif n == 7 { return 7; }
    elif n == 8 { return 8; }
    elif n == 9 { return 9; }
    elif n == 10 { return 10; }
    elif n == 11 { return 11; }
    elif n == 12 { return 12; }
    elif n == 13 { return 13; }
    elif n == 14 { return 14; }
    elif n == 15 { return 15; }
    elif n == 16 { return 16; }
    elif n == 17 { return 17; }
    elif n == 18 { return 18; }
    elif n == 19 { return 19; }
    elif n == 20 { return 20; }
    elif n == 21 { return 21; }
    elif n == 22 { return 22; }
    elif n == 23 { return 23; }
    elif n == 24 { return 24; }
    elif n == 25 { return 25; }
    elif n == 26 { return 26; }
    elif n == 27 { return 27; }
    elif n == 28 { return 28; }
    elif n == 29 { return 29; }
    elif n == 30 { return 30; }
    else { return -1; }
  }
  
  pub fn classify_triple(a: Int, b: Int, c: Int) -> Int {
    if a == b && b == c { return 3; }
    elif a == b || b == c || a == c { return 2; }
    elif a != b && b != c && a != c { return 0; }
    else { return 1; }
  }
  
  fn test_deep_if() -> Int {
    var score = 0;
    if classify_number(0) == 0 { score = score + 1; }
    if classify_number(15) == 15 { score = score + 1; }
    if classify_number(30) == 30 { score = score + 1; }
    if classify_number(31) == -1 { score = score + 1; }
    if classify_number(-1) == -1 { score = score + 1; }
  
    if classify_triple(1, 1, 1) == 3 { score = score + 1; }
    if classify_triple(1, 1, 2) == 2 { score = score + 1; }
    if classify_triple(1, 2, 3) == 0 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 2: FizzBuzz Variants
  // ============================================================
  
  pub fn fizzbuzz(n: Int) -> Int {
    if n % 15 == 0 { return 0; }
    elif n % 3 == 0 { return 1; }
    elif n % 5 == 0 { return 2; }
    else { return 3; }
  }
  
  pub fn fizzbuzz_multi(n: Int, a: Int, b: Int) -> Int {
    var mod_a = n % a;
    var mod_b = n % b;
    if mod_a == 0 && mod_b == 0 { return 0; }
    elif mod_a == 0 { return 1; }
    elif mod_b == 0 { return 2; }
    else { return 3; }
  }
  
  pub fn fizzbuzz_range(lo: Int, hi: Int) -> Int {
    var count_fizzbuzz = 0;
    var n = lo;
    while n <= hi {
      if fizzbuzz(n) == 0 { count_fizzbuzz = count_fizzbuzz + 1; }
      n = n + 1;
    }
    return count_fizzbuzz;
  }
  
  fn test_fizzbuzz() -> Int {
    var score = 0;
    if fizzbuzz(15) == 0 { score = score + 1; }
    if fizzbuzz(3) == 1 { score = score + 1; }
    if fizzbuzz(5) == 2 { score = score + 1; }
    if fizzbuzz(7) == 3 { score = score + 1; }
    if fizzbuzz(30) == 0 { score = score + 1; }
  
    if fizzbuzz_multi(10, 2, 5) == 0 { score = score + 1; }
    if fizzbuzz_multi(4, 2, 5) == 1 { score = score + 1; }
    if fizzbuzz_multi(5, 3, 5) == 2 { score = score + 1; }
    if fizzbuzz_multi(7, 3, 5) == 3 { score = score + 1; }
  
    if fizzbuzz_range(1, 15) == 1 { score = score + 1; }
    if fizzbuzz_range(1, 30) == 2 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 3: Match Expressions
  // ============================================================
  
  pub fn match_number(n: Int) -> Int {
    match n {
      0 => 10,
      1 => 20,
      2 => 30,
      3 => 40,
      4 => 50,
      5 => 60,
      6 => 70,
      7 => 80,
      8 => 90,
      9 => 100,
      _ => 0,
    }
  }
  
  pub fn match_sign(n: Int) -> Int {
    match n {
      x if x > 0 => 1,
      x if x < 0 => -1,
      _ => 0,
    }
  }
  
  fn test_match() -> Int {
    var score = 0;
    if match_number(0) == 10 { score = score + 1; }
    if match_number(5) == 60 { score = score + 1; }
    if match_number(9) == 100 { score = score + 1; }
    if match_number(10) == 0 { score = score + 1; }
    if match_number(-1) == 0 { score = score + 1; }
  
    if match_sign(5) == 1 { score = score + 1; }
    if match_sign(-3) == -1 { score = score + 1; }
    if match_sign(0) == 0 { score = score + 1; }
  
    return score;
  }
  
  // ============================================================
  // SECTION 4: While Loop Patterns
  // ============================================================
  
  pub fn sum_while(limit: Int) -> Int {
    var total = 0;
    var i = 0;
    while i <= limit {
      total = total + i;
      i = i + 1;
    }
    return total;
  }
  
  pub fn countdown(n: Int) -> Int {
    var x = n;
    var steps = 0;
