// XIOM — Data Processing Benchmark

module benchmark.bench_data

use benchmark.main.BenchResult;
use benchmark.data_primes;
use benchmark.data_tables;
use benchmark.data_random;

fn test_primes_data() -> Int {
  var score = 0;
  var primes = data_primes.primes_table();
  if primes.len() == 500 { score = score + 1; }
  if primes[0] == 2 { score = score + 1; }
  if primes[1] == 3 { score = score + 1; }
  if primes[9] == 29 { score = score + 1; }
  if primes[99] == 541 { score = score + 1; }
  if primes[499] == 3571 { score = score + 1; }
  if data_primes.verify_primes() { score = score + 1; }
  if data_primes.verify_monotonic() { score = score + 1; }
  if data_primes.contains(17) { score = score + 1; }
  if data_primes.contains(3571) { score = score + 1; }
  if !(data_primes.contains(4)) { score = score + 1; }
  if data_primes.nth_prime(0) == 2 { score = score + 1; }
  if data_primes.nth_prime(499) == 3571 { score = score + 1; }
  if data_primes.nth_prime(-1) == -1 { score = score + 1; }
  if data_primes.nth_prime(500) == -1 { score = score + 1; }
  return score;
}

fn test_tables_data() -> Int {
  var score = 0;
  var fact = data_tables.factorials();
  if fact.len() == 21 { score = score + 1; }
  if fact[0] == 1 { score = score + 1; }
  if fact[5] == 120 { score = score + 1; }
  if fact[10] == 3628800 { score = score + 1; }
  if data_tables.verify_factorials() { score = score + 1; }
  var fib = data_tables.fibonacci_table();
  if fib.len() == 51 { score = score + 1; }
  if fib[0] == 0 && fib[1] == 1 { score = score + 1; }
  if fib[10] == 55 { score = score + 1; }
  if fib[50] == 12586269025 { score = score + 1; }
  if data_tables.verify_fibonacci() { score = score + 1; }
  var pow2 = data_tables.powers_of_two();
  if pow2.len() == 31 { score = score + 1; }
  if pow2[0] == 1 { score = score + 1; }
  if pow2[10] == 1024 { score = score + 1; }
  if pow2[30] == 1073741824 { score = score + 1; }
  if data_tables.verify_powers_of_two() { score = score + 1; }
  var pow3 = data_tables.powers_of_three();
  if pow3.len() == 16 { score = score + 1; }
  if pow3[5] == 243 { score = score + 1; }
  if pow3[10] == 59049 { score = score + 1; }
  var sq = data_tables.squares();
  if sq.len() == 101 { score = score + 1; }
  if sq[0] == 0 { score = score + 1; }
  if sq[10] == 100 { score = score + 1; }
  if sq[100] == 10000 { score = score + 1; }
  var cb = data_tables.cubes();
  if cb.len() == 51 { score = score + 1; }
  if cb[0] == 0 { score = score + 1; }
  if cb[10] == 1000 { score = score + 1; }
  var tri = data_tables.triangular_numbers();
  if tri.len() == 101 { score = score + 1; }
  if tri[10] == 55 { score = score + 1; }
  var cat = data_tables.catalan_numbers();
  if cat.len() == 16 { score = score + 1; }
  if cat[3] == 5 { score = score + 1; }
  var bell = data_tables.bell_numbers();
  if bell.len() == 11 { score = score + 1; }
  if bell[3] == 5 { score = score + 1; }
  return score;
}

fn test_random_data() -> Int {
  var score = 0;
  var d500 = data_random.dataset_500();
  if d500.len() == 500 { score = score + 1; }
  var d1000 = data_random.dataset_1000();
  if d1000.len() == 1000 { score = score + 1; }
  var min_v = data_random.dataset_min(&d1000);
  var max_v = data_random.dataset_max(&d1000);
  if min_v < max_v { score = score + 1; }
  if min_v >= 0 { score = score + 1; }
  if max_v > 0 { score = score + 1; }
  var mean_v = data_random.dataset_mean(&d1000);
  if mean_v >= 0 { score = score + 1; }
  var count_low = data_random.dataset_count_in_range(&d1000, 0, 100);
  if count_low >= 0 { score = score + 1; }
  var perm = data_random.identity_permutation(50);
  if perm.len() == 50 { score = score + 1; }
  if perm[0] == 0 && perm[49] == 49 { score = score + 1; }
  var rev = data_random.reverse_permutation(50);
  if rev[0] == 49 && rev[49] == 0 { score = score + 1; }
  var wave = data_random.sine_wave(100, 64);
  if wave.len() == 64 { score = score + 1; }
  var psum = data_random.prefix_sum(&d500);
  if psum.len() == 500 { score = score + 1; }
  var hist = data_random.histogram(&d1000, 10, 100000);
  if hist.len() == 10 { score = score + 1; }
  return score;
}

fn test_cross_verification() -> Int {
  var score = 0;
  var primes = data_primes.primes_table();
  var sample_ok = 1; var i = 0;
  while i < primes.len() && i < 20 {
    var p = primes[i]; var is_p = 1; var d = 2;
    while d * d <= p { if p % d == 0 { is_p = 0; } d = d + 1; }
    if is_p == 0 { sample_ok = 0; }
    i = i + 1;
  }
  if sample_ok == 1 { score = score + 1; }
  var fact = data_tables.factorials();
  var fact_ok = 1; i = 0;
  while i < fact.len() && i < 10 {
    var computed = 1; var j = 1;
    while j <= i { computed = computed * j; j = j + 1; }
    if computed != fact[i] { fact_ok = 0; }
    i = i + 1;
  }
  if fact_ok == 1 { score = score + 1; }
  var fib = data_tables.fibonacci_table();
  var fib_ok = 1;
  if fib[0] != 0 { fib_ok = 0; }
  if fib.len() >= 2 && fib[1] != 1 { fib_ok = 0; }
  i = 2;
  while i < fib.len() { if fib[i] != fib[i - 1] + fib[i - 2] { fib_ok = 0; } i = i + 1; }
  if fib_ok == 1 { score = score + 1; }
  var pow2 = data_tables.powers_of_two();
  var pow2_ok = pow2[0] == 1; i = 1;
  while i < pow2.len() { if pow2[i] != pow2[i - 1] * 2 { pow2_ok = 0; } i = i + 1; }
  if pow2_ok { score = score + 1; }
  return score;
}

pub fn run_all() -> BenchResult {
  var total = 0; var max_score = 0;
  var s1 = test_primes_data(); total = total + s1; max_score = max_score + 14;
  var s2 = test_tables_data(); total = total + s2; max_score = max_score + 28;
  var s3 = test_random_data(); total = total + s3; max_score = max_score + 13;
  var s4 = test_cross_verification(); total = total + s4; max_score = max_score + 4;
  return BenchResult{ name: "data", score: total, max_score: max_score, passed: total == max_score, elapsed_ms: 0 };
}