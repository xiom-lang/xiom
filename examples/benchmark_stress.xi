// XIOM — Mega Stress Benchmark (Combined Single-File)
// Pushes the selfhost compiler to its absolute limits.
// 28 inline modules: 24 benchmarks + 3 data modules + 1 data processor
// Total: ~10,000 lines of extreme compiler stress testing.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

// --- Shared Benchmark Result Type ---
pub type BenchResult = {
  name: Str;
  score: Int;
  max_score: Int;
  passed: Bool;
  elapsed_ms: Int;
} derive[Clone]

// ============================================================
// MODULE: data_primes
// ============================================================
module data_primes {

  // Return the first 500 primes as a Vec[Int]
  pub fn primes_table() -> Vec[Int] {
    var p = Vec[Int].new();
    // Primes 0-99
    p.push(2); p.push(3); p.push(5); p.push(7); p.push(11); p.push(13); p.push(17); p.push(19); p.push(23); p.push(29);
    p.push(31); p.push(37); p.push(41); p.push(43); p.push(47); p.push(53); p.push(59); p.push(61); p.push(67); p.push(71);
    p.push(73); p.push(79); p.push(83); p.push(89); p.push(97); p.push(101); p.push(103); p.push(107); p.push(109); p.push(113);
    p.push(127); p.push(131); p.push(137); p.push(139); p.push(149); p.push(151); p.push(157); p.push(163); p.push(167); p.push(173);
    p.push(179); p.push(181); p.push(191); p.push(193); p.push(197); p.push(199); p.push(211); p.push(223); p.push(227); p.push(229);
    p.push(233); p.push(239); p.push(241); p.push(251); p.push(257); p.push(263); p.push(269); p.push(271); p.push(277); p.push(281);
    p.push(283); p.push(293); p.push(307); p.push(311); p.push(313); p.push(317); p.push(331); p.push(337); p.push(347); p.push(349);
    p.push(353); p.push(359); p.push(367); p.push(373); p.push(379); p.push(383); p.push(389); p.push(397); p.push(401); p.push(409);
    p.push(419); p.push(421); p.push(431); p.push(433); p.push(439); p.push(443); p.push(449); p.push(457); p.push(461); p.push(463);
    p.push(467); p.push(479); p.push(487); p.push(491); p.push(499); p.push(503); p.push(509); p.push(521); p.push(523); p.push(541);
    // Primes 100-199
    p.push(547); p.push(557); p.push(563); p.push(569); p.push(571); p.push(577); p.push(587); p.push(593); p.push(599); p.push(601);
    p.push(607); p.push(613); p.push(617); p.push(619); p.push(631); p.push(641); p.push(643); p.push(647); p.push(653); p.push(659);
    p.push(661); p.push(673); p.push(677); p.push(683); p.push(691); p.push(701); p.push(709); p.push(719); p.push(727); p.push(733);
    p.push(739); p.push(743); p.push(751); p.push(757); p.push(761); p.push(769); p.push(773); p.push(787); p.push(797); p.push(809);
    p.push(811); p.push(821); p.push(823); p.push(827); p.push(829); p.push(839); p.push(853); p.push(857); p.push(859); p.push(863);
    p.push(877); p.push(881); p.push(883); p.push(887); p.push(907); p.push(911); p.push(919); p.push(929); p.push(937); p.push(941);
    p.push(947); p.push(953); p.push(967); p.push(971); p.push(977); p.push(983); p.push(991); p.push(997); p.push(1009); p.push(1013);
    p.push(1019); p.push(1021); p.push(1031); p.push(1033); p.push(1039); p.push(1049); p.push(1051); p.push(1061); p.push(1063); p.push(1069);
    p.push(1087); p.push(1091); p.push(1093); p.push(1097); p.push(1103); p.push(1109); p.push(1117); p.push(1123); p.push(1129); p.push(1151);
    p.push(1153); p.push(1163); p.push(1171); p.push(1181); p.push(1187); p.push(1193); p.push(1201); p.push(1213); p.push(1217); p.push(1223);
    // Primes 200-299
    p.push(1229); p.push(1231); p.push(1237); p.push(1249); p.push(1259); p.push(1277); p.push(1279); p.push(1283); p.push(1289); p.push(1291);
    p.push(1297); p.push(1301); p.push(1303); p.push(1307); p.push(1319); p.push(1321); p.push(1327); p.push(1361); p.push(1367); p.push(1373);
    p.push(1381); p.push(1399); p.push(1409); p.push(1423); p.push(1427); p.push(1429); p.push(1433); p.push(1439); p.push(1447); p.push(1451);
    p.push(1453); p.push(1459); p.push(1471); p.push(1481); p.push(1483); p.push(1487); p.push(1489); p.push(1493); p.push(1499); p.push(1511);
    p.push(1523); p.push(1531); p.push(1543); p.push(1549); p.push(1553); p.push(1559); p.push(1567); p.push(1571); p.push(1579); p.push(1583);
    p.push(1597); p.push(1601); p.push(1607); p.push(1609); p.push(1613); p.push(1619); p.push(1621); p.push(1627); p.push(1637); p.push(1657);
    p.push(1663); p.push(1667); p.push(1669); p.push(1693); p.push(1697); p.push(1699); p.push(1709); p.push(1721); p.push(1723); p.push(1733);
    p.push(1741); p.push(1747); p.push(1753); p.push(1759); p.push(1777); p.push(1783); p.push(1787); p.push(1789); p.push(1801); p.push(1811);
    p.push(1823); p.push(1831); p.push(1847); p.push(1861); p.push(1867); p.push(1871); p.push(1873); p.push(1877); p.push(1879); p.push(1889);
    p.push(1901); p.push(1907); p.push(1913); p.push(1931); p.push(1933); p.push(1949); p.push(1951); p.push(1973); p.push(1979); p.push(1987);
    // Primes 300-399
    p.push(1993); p.push(1997); p.push(1999); p.push(2003); p.push(2011); p.push(2017); p.push(2027); p.push(2029); p.push(2039); p.push(2053);
    p.push(2063); p.push(2069); p.push(2081); p.push(2083); p.push(2087); p.push(2089); p.push(2099); p.push(2111); p.push(2113); p.push(2129);
    p.push(2131); p.push(2137); p.push(2141); p.push(2143); p.push(2153); p.push(2161); p.push(2179); p.push(2203); p.push(2207); p.push(2213);
    p.push(2221); p.push(2237); p.push(2239); p.push(2243); p.push(2251); p.push(2267); p.push(2269); p.push(2273); p.push(2281); p.push(2287);
    p.push(2293); p.push(2297); p.push(2309); p.push(2311); p.push(2333); p.push(2339); p.push(2341); p.push(2347); p.push(2351); p.push(2357);
    p.push(2371); p.push(2377); p.push(2381); p.push(2383); p.push(2389); p.push(2393); p.push(2399); p.push(2411); p.push(2417); p.push(2423);
    p.push(2437); p.push(2441); p.push(2447); p.push(2459); p.push(2467); p.push(2473); p.push(2477); p.push(2503); p.push(2521); p.push(2531);
    p.push(2539); p.push(2543); p.push(2549); p.push(2551); p.push(2557); p.push(2579); p.push(2591); p.push(2593); p.push(2609); p.push(2617);
    p.push(2621); p.push(2633); p.push(2647); p.push(2657); p.push(2659); p.push(2663); p.push(2671); p.push(2677); p.push(2683); p.push(2687);
    p.push(2689); p.push(2693); p.push(2699); p.push(2707); p.push(2711); p.push(2713); p.push(2719); p.push(2729); p.push(2731); p.push(2741);
    // Primes 400-499
    p.push(2749); p.push(2753); p.push(2767); p.push(2777); p.push(2789); p.push(2791); p.push(2797); p.push(2801); p.push(2803); p.push(2819);
    p.push(2833); p.push(2837); p.push(2843); p.push(2851); p.push(2857); p.push(2861); p.push(2879); p.push(2887); p.push(2897); p.push(2903);
    p.push(2909); p.push(2917); p.push(2927); p.push(2939); p.push(2953); p.push(2957); p.push(2963); p.push(2969); p.push(2971); p.push(2999);
    p.push(3001); p.push(3011); p.push(3019); p.push(3023); p.push(3037); p.push(3041); p.push(3049); p.push(3061); p.push(3067); p.push(3079);
    p.push(3083); p.push(3089); p.push(3109); p.push(3119); p.push(3121); p.push(3137); p.push(3163); p.push(3167); p.push(3169); p.push(3181);
    p.push(3187); p.push(3191); p.push(3203); p.push(3209); p.push(3217); p.push(3221); p.push(3229); p.push(3251); p.push(3253); p.push(3257);
    p.push(3259); p.push(3271); p.push(3299); p.push(3301); p.push(3307); p.push(3313); p.push(3319); p.push(3323); p.push(3329); p.push(3331);
    p.push(3343); p.push(3347); p.push(3359); p.push(3361); p.push(3371); p.push(3373); p.push(3389); p.push(3391); p.push(3407); p.push(3413);
    p.push(3433); p.push(3449); p.push(3457); p.push(3461); p.push(3463); p.push(3467); p.push(3469); p.push(3491); p.push(3499); p.push(3511);
    p.push(3517); p.push(3527); p.push(3529); p.push(3533); p.push(3539); p.push(3541); p.push(3547); p.push(3557); p.push(3559); p.push(3571);
    return p;
  }

  // Return the nth prime (0-indexed)
  pub fn nth_prime(n: Int) -> Int {
    var table = primes_table();
    if n < 0 || n >= table.len() { return -1; }
    return table[n];
  }

  // Verify that all entries in the table are actually prime
  pub fn verify_primes() -> Bool {
    var table = primes_table();
    var i = 0;
    while i < table.len() {
      var p = table[i];
      if p < 2 { return false; }
      var d = 2;
      while d * d <= p {
        if p % d == 0 { return false; }
        d = d + 1;
      }
      i = i + 1;
    }
    return true;
  }

  // Verify the table is strictly increasing
  pub fn verify_monotonic() -> Bool {
    var table = primes_table();
    var i = 1;
    while i < table.len() {
      if table[i] <= table[i - 1] { return false; }
      i = i + 1;
    }
    return true;
  }

  // Find a prime in the table using binary search
  pub fn contains(target: Int) -> Bool {
    var table = primes_table();
    var lo = 0;
    var hi = table.len();
    if hi == 0 { return false; }
    hi = hi - 1;
    while lo <= hi {
      var mid = (lo + hi) / 2;
      if table[mid] == target { return true; }
      if table[mid] < target { lo = mid + 1; } else { hi = mid - 1; }
    }
    return false;
  }

}

// ============================================================
// MODULE: data_tables
// ============================================================
module data_tables {

  // ============================================================
  // TABLE 1: Factorials 0! through 20!
  // ============================================================
  pub fn factorials() -> Vec[Int] {
    var f = Vec[Int].new();
    f.push(1);     // 0!
    f.push(1);     // 1!
    f.push(2);     // 2!
    f.push(6);     // 3!
    f.push(24);    // 4!
    f.push(120);   // 5!
    f.push(720);   // 6!
    f.push(5040);  // 7!
    f.push(40320); // 8!
    f.push(362880); // 9!
    f.push(3628800); // 10!
    f.push(39916800); // 11!
    f.push(479001600); // 12!
    f.push(6227020800); // 13!
    f.push(87178291200); // 14!
    f.push(1307674368000); // 15!
    f.push(20922789888000); // 16!
    f.push(355687428096000); // 17!
    f.push(6402373705728000); // 18!
    f.push(121645100408832000); // 19!
    f.push(2432902008176640000); // 20!
    return f;
  }

  // ============================================================
  // TABLE 2: Fibonacci Numbers F(0) through F(50)
  // ============================================================
  pub fn fibonacci_table() -> Vec[Int] {
    var fib = Vec[Int].new();
    fib.push(0);      // F(0)
    fib.push(1);      // F(1)
    fib.push(1);      // F(2)
    fib.push(2);      // F(3)
    fib.push(3);      // F(4)
    fib.push(5);      // F(5)
    fib.push(8);      // F(6)
    fib.push(13);     // F(7)
    fib.push(21);     // F(8)
    fib.push(34);     // F(9)
    fib.push(55);     // F(10)
    fib.push(89);     // F(11)
    fib.push(144);    // F(12)
    fib.push(233);    // F(13)
    fib.push(377);    // F(14)
    fib.push(610);    // F(15)
    fib.push(987);    // F(16)
    fib.push(1597);   // F(17)
    fib.push(2584);   // F(18)
    fib.push(4181);   // F(19)
    fib.push(6765);   // F(20)
    fib.push(10946);  // F(21)
    fib.push(17711);  // F(22)
    fib.push(28657);  // F(23)
    fib.push(46368);  // F(24)
    fib.push(75025);  // F(25)
    fib.push(121393); // F(26)
    fib.push(196418); // F(27)
    fib.push(317811); // F(28)
    fib.push(514229); // F(29)
    fib.push(832040); // F(30)
    fib.push(1346269); // F(31)
    fib.push(2178309); // F(32)
    fib.push(3524578); // F(33)
    fib.push(5702887); // F(34)
    fib.push(9227465); // F(35)
    fib.push(14930352); // F(36)
    fib.push(24157817); // F(37)
    fib.push(39088169); // F(38)
    fib.push(63245986); // F(39)
    fib.push(102334155); // F(40)
    fib.push(165580141); // F(41)
    fib.push(267914296); // F(42)
    fib.push(433494437); // F(43)
    fib.push(701408733); // F(44)
    fib.push(1134903170); // F(45)
    fib.push(1836311903); // F(46)
    fib.push(2971215073); // F(47)
    fib.push(4807526976); // F(48)
    fib.push(7778742049); // F(49)
    fib.push(12586269025); // F(50)
    return fib;
  }

  // ============================================================
  // TABLE 3: Powers of 2 — 2^0 through 2^30
  // ============================================================
  pub fn powers_of_two() -> Vec[Int] {
    var p = Vec[Int].new();
    p.push(1);          // 2^0
    p.push(2);          // 2^1
    p.push(4);          // 2^2
    p.push(8);          // 2^3
    p.push(16);         // 2^4
    p.push(32);         // 2^5
    p.push(64);         // 2^6
    p.push(128);        // 2^7
    p.push(256);        // 2^8
    p.push(512);        // 2^9
    p.push(1024);       // 2^10
    p.push(2048);       // 2^11
    p.push(4096);       // 2^12
    p.push(8192);       // 2^13
    p.push(16384);      // 2^14
    p.push(32768);      // 2^15
    p.push(65536);      // 2^16
    p.push(131072);     // 2^17
    p.push(262144);     // 2^18
    p.push(524288);     // 2^19
    p.push(1048576);    // 2^20
    p.push(2097152);    // 2^21
    p.push(4194304);    // 2^22
    p.push(8388608);    // 2^23
    p.push(16777216);   // 2^24
    p.push(33554432);   // 2^25
    p.push(67108864);   // 2^26
    p.push(134217728);  // 2^27
    p.push(268435456);  // 2^28
    p.push(536870912);  // 2^29
    p.push(1073741824); // 2^30
    return p;
  }

  // ============================================================
  // TABLE 4: Powers of 3 — 3^0 through 3^15
  // ============================================================
  pub fn powers_of_three() -> Vec[Int] {
    var p = Vec[Int].new();
    p.push(1);          // 3^0
    p.push(3);          // 3^1
    p.push(9);          // 3^2
    p.push(27);         // 3^3
    p.push(81);         // 3^4
    p.push(243);        // 3^5
    p.push(729);        // 3^6
    p.push(2187);       // 3^7
    p.push(6561);       // 3^8
    p.push(19683);      // 3^9
    p.push(59049);      // 3^10
    p.push(177147);     // 3^11
    p.push(531441);     // 3^12
    p.push(1594323);    // 3^13
    p.push(4782969);    // 3^14
    p.push(14348907);   // 3^15
    return p;
  }

  // ============================================================
  // TABLE 5: Squares 0^2 through 100^2
  // ============================================================
  pub fn squares() -> Vec[Int] {
    var s = Vec[Int].new();
    var i = 0;
    while i <= 100 {
      s.push(i * i);
      i = i + 1;
    }
    return s;
  }

  // ============================================================
  // TABLE 6: Cubes 0^3 through 50^3
  // ============================================================
  pub fn cubes() -> Vec[Int] {
    var c = Vec[Int].new();
    var i = 0;
    while i <= 50 {
      c.push(i * i * i);
      i = i + 1;
    }
    return c;
  }

  // ============================================================
  // TABLE 7: Triangular Numbers T(0) through T(100)
  // ============================================================
  pub fn triangular_numbers() -> Vec[Int] {
    var t = Vec[Int].new();
    var i = 0;
    while i <= 100 {
      t.push(i * (i + 1) / 2);
      i = i + 1;
    }
    return t;
  }

  // ============================================================
  // TABLE 8: Catalan Numbers C(0) through C(15)
  // ============================================================
  pub fn catalan_numbers() -> Vec[Int] {
    var c = Vec[Int].new();
    c.push(1);    // C(0)
    c.push(1);    // C(1)
    c.push(2);    // C(2)
    c.push(5);    // C(3)
    c.push(14);   // C(4)
    c.push(42);   // C(5)
    c.push(132);  // C(6)
    c.push(429);  // C(7)
    c.push(1430); // C(8)
    c.push(4862); // C(9)
    c.push(16796); // C(10)
    c.push(58786); // C(11)
    c.push(208012); // C(12)
    c.push(742900); // C(13)
    c.push(2674440); // C(14)
    c.push(9694845); // C(15)
    return c;
  }

  // ============================================================
  // TABLE 9: Bell Numbers B(0) through B(10)
  // ============================================================
  pub fn bell_numbers() -> Vec[Int] {
    var b = Vec[Int].new();
    b.push(1);     // B(0)
    b.push(1);     // B(1)
    b.push(2);     // B(2)
    b.push(5);     // B(3)
    b.push(15);    // B(4)
    b.push(52);    // B(5)
    b.push(203);   // B(6)
    b.push(877);   // B(7)
    b.push(4140);  // B(8)
    b.push(21147); // B(9)
    b.push(115975); // B(10)
    return b;
  }

  // ============================================================
  // Verifier: Check that the factorial table is consistent
  // ============================================================
  pub fn verify_factorials() -> Bool {
    var f = factorials();
    if f.len() == 0 { return false; }
    if f[0] != 1 { return false; }
    var i = 1;
    while i < f.len() {
      if f[i] != f[i - 1] * i { return false; }
      i = i + 1;
    }
    return true;
  }

  // Verifier: Check Fibonacci consistency
  pub fn verify_fibonacci() -> Bool {
    var fib = fibonacci_table();
    if fib.len() < 3 { return false; }
    if fib[0] != 0 || fib[1] != 1 { return false; }
    var i = 2;
    while i < fib.len() {
      if fib[i] != fib[i - 1] + fib[i - 2] { return false; }
      i = i + 1;
    }
    return true;
  }

  // Verifier: Check powers of 2
  pub fn verify_powers_of_two() -> Bool {
    var p = powers_of_two();
    if p[0] != 1 { return false; }
    var i = 1;
    while i < p.len() {
      if p[i] != p[i - 1] * 2 { return false; }
      i = i + 1;
    }
    return true;
  }

}

// ============================================================
// MODULE: data_random
// ============================================================
module data_random {

  // ============================================================
  // LCG-based PRNG (deterministic)
  // ============================================================
  pub fn lcg(seed: Int, count: Int) -> Vec[Int] {
    var result = Vec[Int].new();
    var state = seed;
    var i = 0;
    while i < count {
      state = (state * 1103515245 + 12345) % 2147483648;
      result.push(state % 1000000);
      i = i + 1;
    }
    return result;
  }

  // ============================================================
  // Generate dataset 1: 500 random values (seed=42)
  // ============================================================
  pub fn dataset_500() -> Vec[Int] {
    return lcg(42, 500);
  }

  // ============================================================
  // Generate dataset 2: 500 more random values (seed=137)
  // ============================================================
  pub fn dataset_500b() -> Vec[Int] {
    return lcg(137, 500);
  }

  // ============================================================
  // Concat both datasets into a 1000-element array
  // ============================================================
  pub fn dataset_1000() -> Vec[Int] {
    var d1 = dataset_500();
    var d2 = dataset_500b();
    var result = Vec[Int].new();
    var i = 0;
    while i < d1.len() {
      result.push(d1[i]);
      i = i + 1;
    }
    i = 0;
    while i < d2.len() {
      result.push(d2[i]);
      i = i + 1;
    }
    return result;
  }

  // ============================================================
  // Basic statistics on the dataset
  // ============================================================
  pub fn dataset_min(data: &Vec[Int]) -> Int {
    if data.len() == 0 { return 0; }
    var m = data[0];
    var i = 1;
    while i < data.len() {
      if data[i] < m { m = data[i]; }
      i = i + 1;
    }
    return m;
  }

  pub fn dataset_max(data: &Vec[Int]) -> Int {
    if data.len() == 0 { return 0; }
    var m = data[0];
    var i = 1;
    while i < data.len() {
      if data[i] > m { m = data[i]; }
      i = i + 1;
    }
    return m;
  }

  pub fn dataset_sum(data: &Vec[Int]) -> Int {
    var sum = 0;
    var i = 0;
    while i < data.len() {
      sum = sum + data[i];
      i = i + 1;
    }
    return sum;
  }

  pub fn dataset_mean(data: &Vec[Int]) -> Int {
    if data.len() == 0 { return 0; }
    return dataset_sum(data) / data.len();
  }

  pub fn dataset_count_in_range(data: &Vec[Int], lo: Int, hi: Int) -> Int {
    var count = 0;
    var i = 0;
    while i < data.len() {
      if data[i] >= lo && data[i] <= hi { count = count + 1; }
      i = i + 1;
    }
    return count;
  }

  pub fn dataset_has_duplicates(data: &Vec[Int]) -> Bool {
    var i = 0;
    while i < data.len() {
      var j = i + 1;
      while j < data.len() {
        if data[i] == data[j] { return true; }
        j = j + 1;
      }
      i = i + 1;
    }
    return false;
  }

  // ============================================================
  // Shuffle-like permutation generator
  // ============================================================
  pub fn identity_permutation(n: Int) -> Vec[Int] {
    var p = Vec[Int].new();
    var i = 0;
    while i < n {
      p.push(i);
      i = i + 1;
    }
    return p;
  }

  pub fn reverse_permutation(n: Int) -> Vec[Int] {
    var p = Vec[Int].new();
    var i = n;
    while i > 0 {
      i = i - 1;
      p.push(i);
    }
    return p;
  }

  // ============================================================
  // Fixed sine wave data (integer approximation)
  // ============================================================
  pub fn sine_wave(amplitude: Int, samples: Int) -> Vec[Int] {
    var wave = Vec[Int].new();
    var i = 0;
    while i < samples {
      // Approximate sin(2*pi*i/samples) using integer arithmetic
      var phase = (i * 360) / samples;
      var val = 0;
      if phase < 90 {
        val = (phase * amplitude) / 90;
      } elif phase < 180 {
        val = ((180 - phase) * amplitude) / 90;
      } elif phase < 270 {
        val = -((phase - 180) * amplitude) / 90;
      } else {
        val = -((360 - phase) * amplitude) / 90;
      }
      wave.push(val);
      i = i + 1;
    }
    return wave;
  }

  // ============================================================
  // Cumulative sum array
  // ============================================================
  pub fn prefix_sum(data: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var running = 0;
    var i = 0;
    while i < data.len() {
      running = running + data[i];
      result.push(running);
      i = i + 1;
    }
    return result;
  }

  // ============================================================
  // Histogram bins helper
  // ============================================================
  pub fn histogram(data: &Vec[Int], bins: Int, bin_width: Int) -> Vec[Int] {
    var counts = Vec[Int].new();
    var i = 0;
    while i < bins {
      counts.push(0);
      i = i + 1;
    }
    i = 0;
    while i < data.len() {
      var bin = data[i] / bin_width;
      if bin >= 0 && bin < bins {
        counts[bin] = counts[bin] + 1;
      }
      i = i + 1;
    }
    return counts;
  }

}

//__BENCH_APPEND__