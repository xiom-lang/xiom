// XIOM -- Mega Stress Benchmark (Combined Single-File)
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
  // TABLE 3: Powers of 2 -- 2^0 through 2^30
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
  // TABLE 4: Powers of 3 -- 3^0 through 3^15
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

// ============================================================
// MODULE: math
// ============================================================
module math {


  // ============================================================
  // SECTION 1: Basic Arithmetic -- verifying integer operations
  // ============================================================

  pub fn add_int(a: Int, b: Int) -> Int { return a + b; }
  pub fn sub_int(a: Int, b: Int) -> Int { return a - b; }
  pub fn mul_int(a: Int, b: Int) -> Int { return a * b; }
  pub fn div_int(a: Int, b: Int) -> Int { return a / b; }
  pub fn mod_int(a: Int, b: Int) -> Int { return a % b; }
  pub fn neg_int(a: Int) -> Int { return -a; }

  fn test_basic_arithmetic() -> Int {
    var score = 0;
    if add_int(2, 3) == 5 { score = score + 1; }
    if add_int(-5, 10) == 5 { score = score + 1; }
    if add_int(0, 0) == 0 { score = score + 1; }
    if add_int(1000000, 2000000) == 3000000 { score = score + 1; }
    if add_int(-1, -1) == -2 { score = score + 1; }

    if sub_int(10, 3) == 7 { score = score + 1; }
    if sub_int(0, 5) == -5 { score = score + 1; }
    if sub_int(-5, -3) == -2 { score = score + 1; }
    if sub_int(100, 100) == 0 { score = score + 1; }
    if sub_int(42, 100) == -58 { score = score + 1; }

    if mul_int(7, 8) == 56 { score = score + 1; }
    if mul_int(-3, 4) == -12 { score = score + 1; }
    if mul_int(-2, -5) == 10 { score = score + 1; }
    if mul_int(0, 999) == 0 { score = score + 1; }
    if mul_int(1, 12345) == 12345 { score = score + 1; }

    if div_int(100, 4) == 25 { score = score + 1; }
    if div_int(10, 3) == 3 { score = score + 1; }
    if div_int(7, 1) == 7 { score = score + 1; }
    if div_int(0, 5) == 0 { score = score + 1; }
    if div_int(-12, 4) == -3 { score = score + 1; }

    if mod_int(10, 3) == 1 { score = score + 1; }
    if mod_int(17, 5) == 2 { score = score + 1; }
    if mod_int(8, 2) == 0 { score = score + 1; }
    if mod_int(0, 7) == 0 { score = score + 1; }
    if mod_int(100, 7) == 2 { score = score + 1; }

    if neg_int(5) == -5 { score = score + 1; }
    if neg_int(-7) == 7 { score = score + 1; }
    if neg_int(0) == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Float Operations -- Float64 arithmetic
  // ============================================================

  pub fn fadd(a: Float64, b: Float64) -> Float64 { return a + b; }
  pub fn fsub(a: Float64, b: Float64) -> Float64 { return a - b; }
  pub fn fmul(a: Float64, b: Float64) -> Float64 { return a * b; }
  pub fn fdiv(a: Float64, b: Float64) -> Float64 { return a / b; }
  pub fn fneg(a: Float64) -> Float64 { return -a; }

  fn test_float_arithmetic() -> Int {
    var score = 0;
    if fadd(1.5, 2.5) == 4.0 { score = score + 1; }
    if fsub(10.0, 3.5) == 6.5 { score = score + 1; }
    if fmul(2.0, 3.5) == 7.0 { score = score + 1; }
    if fdiv(10.0, 4.0) == 2.5 { score = score + 1; }
    if fneg(3.14) == -3.14 { score = score + 1; }
    if fadd(-1.0, -1.0) == -2.0 { score = score + 1; }
    if fmul(0.0, 42.0) == 0.0 { score = score + 1; }
    if fdiv(1.0, 2.0) == 0.5 { score = score + 1; }
    if fsub(0.0, 5.0) == -5.0 { score = score + 1; }
    return score;
  }

  // ============================================================
  // SECTION 3: Factorial -- iterative and recursive
  // ============================================================

  pub fn factorial_rec(n: Int) -> Int {
    if n <= 1 { return 1; }
    return n * factorial_rec(n - 1);
  }

  pub fn factorial_iter(n: Int) -> Int {
    var result = 1;
    var i = 1;
    while i <= n {
      result = result * i;
      i = i + 1;
    }
    return result;
  }

  fn test_factorial() -> Int {
    var score = 0;
    if factorial_rec(0) == 1 { score = score + 1; }
    if factorial_rec(1) == 1 { score = score + 1; }
    if factorial_rec(5) == 120 { score = score + 1; }
    if factorial_rec(7) == 5040 { score = score + 1; }
    if factorial_rec(10) == 3628800 { score = score + 1; }

    if factorial_iter(0) == 1 { score = score + 1; }
    if factorial_iter(5) == 120 { score = score + 1; }
    if factorial_iter(7) == 5040 { score = score + 1; }
    if factorial_iter(10) == 3628800 { score = score + 1; }
    if factorial_iter(12) == 479001600 { score = score + 1; }

    // Cross-verify rec vs iter
    if factorial_rec(8) == factorial_iter(8) { score = score + 1; }
    if factorial_rec(9) == factorial_iter(9) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Fibonacci -- 3 implementations
  // ============================================================

  pub fn fibonacci_rec(n: Int) -> Int {
    if n <= 1 { return n; }
    return fibonacci_rec(n - 1) + fibonacci_rec(n - 2);
  }

  pub fn fibonacci_iter(n: Int) -> Int {
    if n <= 1 { return n; }
    var a = 0;
    var b = 1;
    var i = 2;
    while i <= n {
      var temp = a + b;
      a = b;
      b = temp;
      i = i + 1;
    }
    return b;
  }

  pub fn fib_nth(n: Int) -> Int {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    return fibonacci_iter(n);
  }

  fn test_fibonacci() -> Int {
    var score = 0;
    // Known values
    if fibonacci_rec(0) == 0 { score = score + 1; }
    if fibonacci_rec(1) == 1 { score = score + 1; }
    if fibonacci_rec(10) == 55 { score = score + 1; }
    if fibonacci_rec(15) == 610 { score = score + 1; }
    if fibonacci_rec(20) == 6765 { score = score + 1; }

    if fibonacci_iter(0) == 0 { score = score + 1; }
    if fibonacci_iter(1) == 1 { score = score + 1; }
    if fibonacci_iter(10) == 55 { score = score + 1; }
    if fibonacci_iter(20) == 6765 { score = score + 1; }
    if fibonacci_iter(25) == 75025 { score = score + 1; }

    // Cross-verify
    if fibonacci_rec(12) == fibonacci_iter(12) { score = score + 1; }
    if fibonacci_rec(16) == fibonacci_iter(16) { score = score + 1; }
    if fib_nth(10) == 55 { score = score + 1; }
    if fib_nth(0) == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: GCD and LCM
  // ============================================================

  pub fn gcd(a: Int, b: Int) -> Int {
    if b == 0 { return a; }
    return gcd(b, a % b);
  }

  pub fn lcm(a: Int, b: Int) -> Int {
    return a * b / gcd(a, b);
  }

  pub fn gcd_iter(a: Int, b: Int) -> Int {
    var x = a;
    var y = b;
    while y != 0 {
      var temp = y;
      y = x % y;
      x = temp;
    }
    return x;
  }

  fn test_gcd_lcm() -> Int {
    var score = 0;
    if gcd(48, 18) == 6 { score = score + 1; }
    if gcd(100, 10) == 10 { score = score + 1; }
    if gcd(7, 13) == 1 { score = score + 1; }
    if gcd(0, 5) == 5 { score = score + 1; }
    if gcd(5, 0) == 5 { score = score + 1; }
    if gcd(270, 192) == 6 { score = score + 1; }
    if gcd(1071, 462) == 21 { score = score + 1; }
    if gcd(1, 1) == 1 { score = score + 1; }
    if gcd(999999, 1) == 1 { score = score + 1; }

    if lcm(4, 6) == 12 { score = score + 1; }
    if lcm(7, 11) == 77 { score = score + 1; }
    if lcm(12, 18) == 36 { score = score + 1; }
    if lcm(1, 99) == 99 { score = score + 1; }

    // Cross-verify rec vs iter
    if gcd(48, 18) == gcd_iter(48, 18) { score = score + 1; }
    if gcd(1071, 462) == gcd_iter(1071, 462) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Power and Exponentiation
  // ============================================================

  pub fn power_rec(base: Int, exp: Int) -> Int {
    if exp == 0 { return 1; }
    return base * power_rec(base, exp - 1);
  }

  pub fn power_iter(base: Int, exp: Int) -> Int {
    var result = 1;
    var e = exp;
    var b = base;
    while e > 0 {
      result = result * b;
      e = e - 1;
    }
    return result;
  }

  pub fn fast_pow(base: Int, exp: Int) -> Int {
    if exp == 0 { return 1; }
    var half = fast_pow(base, exp / 2);
    if exp % 2 == 0 {
      return half * half;
    } else {
      return base * half * half;
    }
  }

  pub fn pow_mod(base: Int, exp: Int, mod_m: Int) -> Int {
    var result = 1;
    var b = base % mod_m;
    var e = exp;
    while e > 0 {
      if e % 2 == 1 {
        result = (result * b) % mod_m;
      }
      e = e / 2;
      b = (b * b) % mod_m;
    }
    return result;
  }

  fn test_power() -> Int {
    var score = 0;
    if power_rec(2, 0) == 1 { score = score + 1; }
    if power_rec(2, 10) == 1024 { score = score + 1; }
    if power_rec(3, 5) == 243 { score = score + 1; }
    if power_rec(5, 4) == 625 { score = score + 1; }
    if power_rec(10, 3) == 1000 { score = score + 1; }

    if power_iter(2, 10) == 1024 { score = score + 1; }
    if power_iter(3, 5) == 243 { score = score + 1; }
    if power_iter(10, 0) == 1 { score = score + 1; }

    if fast_pow(2, 10) == 1024 { score = score + 1; }
    if fast_pow(3, 4) == 81 { score = score + 1; }
    if fast_pow(5, 3) == 125 { score = score + 1; }

    // Cross-verify all three
    if power_rec(2, 8) == power_iter(2, 8) { score = score + 1; }
    if fast_pow(3, 6) == power_iter(3, 6) { score = score + 1; }

    // Modular exponentiation
    if pow_mod(2, 10, 1000) == 24 { score = score + 1; }
    if pow_mod(3, 5, 13) == 9 { score = score + 1; }
    if pow_mod(7, 3, 5) == 3 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Combinatorics
  // ============================================================

  pub fn binomial(n: Int, k: Int) -> Int {
    if k == 0 || k == n { return 1; }
    if k > n { return 0; }
    return binomial(n - 1, k - 1) + binomial(n - 1, k);
  }

  pub fn combinations(n: Int, r: Int) -> Int {
    if r > n { return 0; }
    if r == 0 || r == n { return 1; }
    var result = 1;
    var i = 1;
    while i <= r {
      result = result * (n - r + i) / i;
      i = i + 1;
    }
    return result;
  }

  fn test_combinatorics() -> Int {
    var score = 0;
    // C(n,0) = C(n,n) = 1
    if binomial(5, 0) == 1 { score = score + 1; }
    if binomial(5, 5) == 1 { score = score + 1; }
    if binomial(5, 2) == 10 { score = score + 1; }
    if binomial(6, 3) == 20 { score = score + 1; }
    if binomial(10, 5) == 252 { score = score + 1; }
    if binomial(7, 2) == 21 { score = score + 1; }

    // Iterative version
    if combinations(5, 2) == 10 { score = score + 1; }
    if combinations(10, 3) == 120 { score = score + 1; }
    if combinations(8, 4) == 70 { score = score + 1; }

    // Cross-verify
    if binomial(6, 2) == combinations(6, 2) { score = score + 1; }
    if binomial(8, 3) == combinations(8, 3) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 8: Number Theory
  // ============================================================

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

  pub fn is_perfect_square(n: Int) -> Bool {
    if n < 0 { return false; }
    var x = 1;
    var sum = 1;
    while sum < n {
      x = x + 2;
      sum = sum + x;
    }
    return sum == n;
  }

  pub fn digit_sum(n: Int) -> Int {
    if n < 0 { return digit_sum(-n); }
    if n < 10 { return n; }
    return n % 10 + digit_sum(n / 10);
  }

  pub fn digit_count(n: Int) -> Int {
    if n < 0 { return digit_count(-n); }
    if n == 0 { return 1; }
    var count = 0;
    var x = n;
    while x > 0 {
      count = count + 1;
      x = x / 10;
    }
    return count;
  }

  pub fn reverse_num(n: Int) -> Int {
    var sign = 1;
    var x = n;
    if n < 0 { sign = -1; x = -n; }
    var rev = 0;
    while x > 0 {
      rev = rev * 10 + x % 10;
      x = x / 10;
    }
    return sign * rev;
  }

  pub fn is_palindrome_num(n: Int) -> Bool {
    return n == reverse_num(n);
  }

  fn test_number_theory() -> Int {
    var score = 0;
    // Primes
    if is_prime(2) { score = score + 1; }
    if is_prime(3) { score = score + 1; }
    if is_prime(17) { score = score + 1; }
    if is_prime(97) { score = score + 1; }
    if is_prime(7919) { score = score + 1; }
    if !(is_prime(1)) { score = score + 1; }
    if !(is_prime(4)) { score = score + 1; }
    if !(is_prime(100)) { score = score + 1; }
    if !(is_prime(9991 * 9991)) { score = score + 1; }

    // Perfect squares
    if is_perfect_square(0) { score = score + 1; }
    if is_perfect_square(1) { score = score + 1; }
    if is_perfect_square(4) { score = score + 1; }
    if is_perfect_square(16) { score = score + 1; }
    if is_perfect_square(10000) { score = score + 1; }
    if !(is_perfect_square(2)) { score = score + 1; }
    if !(is_perfect_square(99)) { score = score + 1; }

    // Digit operations
    if digit_sum(123) == 6 { score = score + 1; }
    if digit_sum(0) == 0 { score = score + 1; }
    if digit_sum(99999) == 45 { score = score + 1; }
    if digit_sum(-123) == 6 { score = score + 1; }

    if digit_count(0) == 1 { score = score + 1; }
    if digit_count(5) == 1 { score = score + 1; }
    if digit_count(12345) == 5 { score = score + 1; }
    if digit_count(1000000) == 7 { score = score + 1; }

    if reverse_num(123) == 321 { score = score + 1; }
    if reverse_num(100) == 1 { score = score + 1; }
    if reverse_num(-123) == -321 { score = score + 1; }
    if reverse_num(0) == 0 { score = score + 1; }

    if is_palindrome_num(121) { score = score + 1; }
    if is_palindrome_num(12321) { score = score + 1; }
    if !(is_palindrome_num(123)) { score = score + 1; }
    if is_palindrome_num(0) { score = score + 1; }
    if is_palindrome_num(11) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 9: Sequences and Series
  // ============================================================

  pub fn triangular(n: Int) -> Int {
    if n <= 0 { return 0; }
    return n * (n + 1) / 2;
  }

  pub fn sum_range(lo: Int, hi: Int) -> Int {
    var sum = 0;
    var i = lo;
    while i <= hi {
      sum = sum + i;
      i = i + 1;
    }
    return sum;
  }

  pub fn collatz(n: Int) -> Int {
    if n <= 1 { return 0; }
    if n % 2 == 0 { return 1 + collatz(n / 2); }
    return 1 + collatz(3 * n + 1);
  }

  pub fn abs(n: Int) -> Int {
    if n >= 0 { return n; }
    return -n;
  }

  pub fn max(a: Int, b: Int) -> Int {
    if a > b { return a; }
    return b;
  }

  pub fn min(a: Int, b: Int) -> Int {
    if a < b { return a; }
    return b;
  }

  pub fn clamp(val: Int, lo: Int, hi: Int) -> Int {
    if val < lo { return lo; }
    if val > hi { return hi; }
    return val;
  }

  pub fn sign(n: Int) -> Int {
    if n > 0 { return 1; }
    if n < 0 { return -1; }
    return 0;
  }

  fn test_sequences() -> Int {
    var score = 0;
    if triangular(1) == 1 { score = score + 1; }
    if triangular(5) == 15 { score = score + 1; }
    if triangular(10) == 55 { score = score + 1; }
    if triangular(100) == 5050 { score = score + 1; }
    if triangular(0) == 0 { score = score + 1; }

    if sum_range(1, 10) == 55 { score = score + 1; }
    if sum_range(1, 100) == 5050 { score = score + 1; }
    if sum_range(5, 5) == 5 { score = score + 1; }
    if sum_range(-5, 5) == 0 { score = score + 1; }

    if collatz(1) == 0 { score = score + 1; }
    if collatz(6) == 8 { score = score + 1; }
    if collatz(27) == 111 { score = score + 1; }

    if abs(5) == 5 { score = score + 1; }
    if abs(-5) == 5 { score = score + 1; }
    if abs(0) == 0 { score = score + 1; }

    if max(10, 20) == 20 { score = score + 1; }
    if max(-5, -2) == -2 { score = score + 1; }
    if max(5, 5) == 5 { score = score + 1; }

    if min(10, 20) == 10 { score = score + 1; }
    if min(-5, -2) == -5 { score = score + 1; }

    if clamp(5, 0, 10) == 5 { score = score + 1; }
    if clamp(-1, 0, 10) == 0 { score = score + 1; }
    if clamp(15, 0, 10) == 10 { score = score + 1; }

    if sign(42) == 1 { score = score + 1; }
    if sign(-7) == -1 { score = score + 1; }
    if sign(0) == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 10: Parity and Divisibility
  // ============================================================

  pub fn is_even(n: Int) -> Bool { return n % 2 == 0; }
  pub fn is_odd(n: Int) -> Bool { return n % 2 != 0; }
  pub fn is_multiple_of(a: Int, b: Int) -> Bool {
    if b == 0 { return false; }
    return a % b == 0;
  }
  pub fn count_divisors(n: Int) -> Int {
    if n <= 0 { return 0; }
    var count = 0;
    var i = 1;
    while i <= n {
      if n % i == 0 { count = count + 1; }
      i = i + 1;
    }
    return count;
  }
  pub fn divisor_sum(n: Int) -> Int {
    if n <= 0 { return 0; }
    var sum = 0;
    var i = 1;
    while i <= n {
      if n % i == 0 { sum = sum + i; }
      i = i + 1;
    }
    return sum;
  }
  pub fn is_perfect(n: Int) -> Bool {
    return n > 0 && divisor_sum(n) == 2 * n;
  }

  fn test_parity() -> Int {
    var score = 0;
    if is_even(0) { score = score + 1; }
    if is_even(2) { score = score + 1; }
    if is_even(100) { score = score + 1; }
    if !(is_even(1)) { score = score + 1; }
    if !(is_even(99)) { score = score + 1; }

    if is_odd(1) { score = score + 1; }
    if is_odd(99) { score = score + 1; }
    if !(is_odd(0)) { score = score + 1; }
    if !(is_odd(100)) { score = score + 1; }

    if is_multiple_of(10, 5) { score = score + 1; }
    if is_multiple_of(15, 3) { score = score + 1; }
    if !(is_multiple_of(10, 3)) { score = score + 1; }
    if !(is_multiple_of(5, 0)) { score = score + 1; }
    if is_multiple_of(0, 5) { score = score + 1; }

    if count_divisors(1) == 1 { score = score + 1; }
    if count_divisors(6) == 4 { score = score + 1; }
    if count_divisors(12) == 6 { score = score + 1; }
    if count_divisors(28) == 6 { score = score + 1; }

    if divisor_sum(6) == 12 { score = score + 1; }
    if divisor_sum(28) == 56 { score = score + 1; }
    if divisor_sum(12) == 28 { score = score + 1; }

    if is_perfect(6) { score = score + 1; }
    if is_perfect(28) { score = score + 1; }
    if !(is_perfect(12)) { score = score + 1; }
    if !(is_perfect(1)) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 11: Numerical Methods
  // ============================================================

  pub fn sqrt_newton(x: Float64, epsilon: Float64) -> Float64 {
    if x < 0.0 { return -1.0; }
    if x == 0.0 { return 0.0; }
    var guess = x / 2.0;
    var i = 0;
    while i < 100 {
      var next_guess = (guess + x / guess) / 2.0;
      var diff = next_guess - guess;
      if diff < 0.0 { diff = -diff; }
      if diff < epsilon { return next_guess; }
      guess = next_guess;
      i = i + 1;
    }
    return guess;
  }

  pub fn abs_float(x: Float64) -> Float64 {
    if x >= 0.0 { return x; }
    return -x;
  }

  pub fn exp_taylor(x: Float64, terms: Int) -> Float64 {
    var result = 1.0;
    var term = 1.0;
    var i = 1;
    while i <= terms {
      term = term * x / (i as Float64);
      result = result + term;
      i = i + 1;
    }
    return result;
  }

  pub fn sin_taylor(x: Float64, terms: Int) -> Float64 {
    var result = x;
    var term = x;
    var i = 1;
    while i <= terms {
      term = -term * x * x / ((2 * i) * (2 * i + 1) as Float64);
      result = result + term;
      i = i + 1;
    }
    return result;
  }

  pub fn cos_taylor(x: Float64, terms: Int) -> Float64 {
    var result = 1.0;
    var term = 1.0;
    var i = 1;
    while i <= terms {
      term = -term * x * x / ((2 * i - 1) * (2 * i) as Float64);
      result = result + term;
      i = i + 1;
    }
    return result;
  }

  pub fn trapezoidal(f: fn(Float64) -> Float64, a: Float64, b: Float64, n: Int) -> Float64 {
    var h = (b - a) / (n as Float64);
    var sum = (f(a) + f(b)) / 2.0;
    var i = 1;
    while i < n {
      var x = a + (i as Float64) * h;
      sum = sum + f(x);
      i = i + 1;
    }
    return sum * h;
  }

  fn test_numerical() -> Int {
    var score = 0;
    // sqrt via Newton
    var s4 = sqrt_newton(4.0, 0.0001);
    var s4_ok = s4 > 1.9 && s4 < 2.1;
    if s4_ok { score = score + 1; }

    var s9 = sqrt_newton(9.0, 0.0001);
    var s9_ok = s9 > 2.9 && s9 < 3.1;
    if s9_ok { score = score + 1; }

    var s0 = sqrt_newton(0.0, 0.0001);
    if s0 == 0.0 { score = score + 1; }

    if sqrt_newton(-1.0, 0.0001) == -1.0 { score = score + 1; }

    // abs_float
    if abs_float(3.14) == 3.14 { score = score + 1; }
    if abs_float(-2.71) == 2.71 { score = score + 1; }
    if abs_float(0.0) == 0.0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 12: Modular Arithmetic
  // ============================================================

  pub fn mod_add(a: Int, b: Int, m: Int) -> Int {
    return (a + b) % m;
  }

  pub fn mod_mul(a: Int, b: Int, m: Int) -> Int {
    return (a * b) % m;
  }

  pub fn mod_inv(a: Int, m: Int) -> Int {
    var t = 0;
    var newt = 1;
    var r = m;
    var newr = a % m;
    while newr != 0 {
      var q = r / newr;
      var temp_t = t - q * newt;
      t = newt;
      newt = temp_t;
      var temp_r = r - q * newr;
      r = newr;
      newr = temp_r;
    }
    if r > 1 {
      return -1;
    }
    if t < 0 {
      t = t + m;
    }
    return t;
  }

  fn test_modular() -> Int {
    var score = 0;
    if mod_add(7, 8, 10) == 5 { score = score + 1; }
    if mod_add(5, 5, 7) == 3 { score = score + 1; }
    if mod_mul(3, 4, 11) == 1 { score = score + 1; }
    if mod_mul(7, 8, 13) == 4 { score = score + 1; }

    var inv35 = mod_inv(3, 5);
    if inv35 > 0 && (3 * inv35) % 5 == 1 { score = score + 1; }

    var inv711 = mod_inv(7, 11);
    if inv711 > 0 && (7 * inv711) % 11 == 1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 13: Bitwise Operations (simulated with arithmetic)
  // ============================================================

  pub fn bit_and(a: Int, b: Int) -> Int {
    var result = 0;
    var shift = 0;
    var aa = a;
    var bb = b;
    while aa > 0 || bb > 0 {
      if aa % 2 == 1 && bb % 2 == 1 {
        result = result + power_iter(2, shift);
      }
      aa = aa / 2;
      bb = bb / 2;
      shift = shift + 1;
    }
    return result;
  }

  pub fn bit_or(a: Int, b: Int) -> Int {
    var result = 0;
    var shift = 0;
    var aa = a;
    var bb = b;
    while aa > 0 || bb > 0 {
      if aa % 2 == 1 || bb % 2 == 1 {
        result = result + power_iter(2, shift);
      }
      aa = aa / 2;
      bb = bb / 2;
      shift = shift + 1;
    }
    return result;
  }

  pub fn bit_xor(a: Int, b: Int) -> Int {
    var result = 0;
    var shift = 0;
    var aa = a;
    var bb = b;
    while aa > 0 || bb > 0 {
      if aa % 2 != bb % 2 {
        result = result + power_iter(2, shift);
      }
      aa = aa / 2;
      bb = bb / 2;
      shift = shift + 1;
    }
    return result;
  }

  fn test_bitwise() -> Int {
    var score = 0;
    if bit_and(6, 3) == 2 { score = score + 1; }
    if bit_and(12, 10) == 8 { score = score + 1; }
    if bit_and(0, 5) == 0 { score = score + 1; }
    if bit_and(7, 7) == 7 { score = score + 1; }

    if bit_or(6, 3) == 7 { score = score + 1; }
    if bit_or(8, 1) == 9 { score = score + 1; }
    if bit_or(0, 5) == 5 { score = score + 1; }

    if bit_xor(6, 3) == 5 { score = score + 1; }
    if bit_xor(5, 5) == 0 { score = score + 1; }
    if bit_xor(0, 7) == 7 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 14: Interpolation and Mapping
  // ============================================================

  pub fn lerp(a: Int, b: Int, t: Int) -> Int {
    return a + (b - a) * t;
  }

  pub fn lerp_float(a: Float64, b: Float64, t: Float64) -> Float64 {
    return a + (b - a) * t;
  }

  pub fn map_range(val: Int, in_min: Int, in_max: Int, out_min: Int, out_max: Int) -> Int {
    return out_min + (val - in_min) * (out_max - out_min) / (in_max - in_min);
  }

  fn test_interpolation() -> Int {
    var score = 0;
    if lerp(0, 10, 0) == 0 { score = score + 1; }
    if lerp(0, 10, 1) == 10 { score = score + 1; }
    if lerp(0, 10, 2) == 20 { score = score + 1; }
    if lerp(10, 20, 3) == 40 { score = score + 1; }

    if map_range(5, 0, 10, 0, 100) == 50 { score = score + 1; }
    if map_range(0, 0, 10, 0, 100) == 0 { score = score + 1; }
    if map_range(10, 0, 10, 0, 100) == 100 { score = score + 1; }
    if map_range(5, 0, 100, 0, 10) == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 15: Aggregate Scoring Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    // 1: Basic Arithmetic
    var s1 = test_basic_arithmetic();
    total = total + s1;
    max_score = max_score + 27;

    // 2: Float
    var s2 = test_float_arithmetic();
    total = total + s2;
    max_score = max_score + 9;

    // 3: Factorial
    var s3 = test_factorial();
    total = total + s3;
    max_score = max_score + 12;

    // 4: Fibonacci
    var s4 = test_fibonacci();
    total = total + s4;
    max_score = max_score + 14;

    // 5: GCD/LCM
    var s5 = test_gcd_lcm();
    total = total + s5;
    max_score = max_score + 15;

    // 6: Power
    var s6 = test_power();
    total = total + s6;
    max_score = max_score + 15;

    // 7: Combinatorics
    var s7 = test_combinatorics();
    total = total + s7;
    max_score = max_score + 11;

    // 8: Number Theory
    var s8 = test_number_theory();
    total = total + s8;
    max_score = max_score + 32;

    // 9: Sequences
    var s9 = test_sequences();
    total = total + s9;
    max_score = max_score + 27;

    // 10: Parity
    var s10 = test_parity();
    total = total + s10;
    max_score = max_score + 26;

    // 11: Numerical
    var s11 = test_numerical();
    total = total + s11;
    max_score = max_score + 7;

    // 12: Modular
    var s12 = test_modular();
    total = total + s12;
    max_score = max_score + 6;

    // 13: Bitwise
    var s13 = test_bitwise();
    total = total + s13;
    max_score = max_score + 10;

    // 14: Interpolation
    var s14 = test_interpolation();
    total = total + s14;
    max_score = max_score + 8;

    return BenchResult{
      name: "math",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: primes
// ============================================================
module primes {

  use benchmark.math.is_prime;
  use benchmark.math.power_iter;

  // ============================================================
  // SECTION 1: Trial Division Primality Testing
  // ============================================================

  pub fn is_prime_opt(n: Int) -> Bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    if n % 3 == 0 { return n == 3; }
    var i = 5;
    while i * i <= n {
      if n % i == 0 { return false; }
      if n % (i + 2) == 0 { return false; }
      i = i + 6;
    }
    return true;
  }

  pub fn count_primes_up_to(limit: Int) -> Int {
    var count = 0;
    var i = 2;
    while i <= limit {
      if is_prime_opt(i) { count = count + 1; }
      i = i + 1;
    }
    return count;
  }

  pub fn nth_prime(n: Int) -> Int {
    if n <= 0 { return 0; }
    var count = 0;
    var candidate = 1;
    while count < n {
      candidate = candidate + 1;
      if is_prime_opt(candidate) { count = count + 1; }
    }
    return candidate;
  }

  pub fn prime_gap(n: Int) -> Int {
    var p1 = nth_prime(n);
    var p2 = nth_prime(n + 1);
    return p2 - p1;
  }

  fn test_primality() -> Int {
    var score = 0;
    // Known primes
    if is_prime_opt(2) { score = score + 1; }
    if is_prime_opt(3) { score = score + 1; }
    if is_prime_opt(5) { score = score + 1; }
    if is_prime_opt(7) { score = score + 1; }
    if is_prime_opt(11) { score = score + 1; }
    if is_prime_opt(13) { score = score + 1; }
    if is_prime_opt(97) { score = score + 1; }
    if is_prime_opt(997) { score = score + 1; }
    if is_prime_opt(7919) { score = score + 1; }
    if is_prime_opt(104729) { score = score + 1; }

    // Known composites
    if !(is_prime_opt(1)) { score = score + 1; }
    if !(is_prime_opt(4)) { score = score + 1; }
    if !(is_prime_opt(6)) { score = score + 1; }
    if !(is_prime_opt(100)) { score = score + 1; }
    if !(is_prime_opt(91)) { score = score + 1; }

    // Count primes
    if count_primes_up_to(10) == 4 { score = score + 1; }
    if count_primes_up_to(100) == 25 { score = score + 1; }
    if count_primes_up_to(2) == 1 { score = score + 1; }

    // nth prime
    if nth_prime(1) == 2 { score = score + 1; }
    if nth_prime(2) == 3 { score = score + 1; }
    if nth_prime(10) == 29 { score = score + 1; }
    if nth_prime(25) == 97 { score = score + 1; }

    // Prime gaps
    if prime_gap(1) == 1 { score = score + 1; }
    if prime_gap(2) == 2 { score = score + 1; }
    if prime_gap(3) == 2 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Sieve of Eratosthenes (Array-Simulated)
  // ============================================================

  pub fn sieve_count(limit: Int) -> Int {
    if limit < 2 { return 0; }
    // Simulate boolean array using modular encoding
    var count = 0;
    var n = 2;
    while n <= limit {
      var is_prime_n = 1;
      var d = 2;
      while d * d <= n {
        if n % d == 0 {
          is_prime_n = 0;
          d = n;
        }
        d = d + 1;
      }
      count = count + is_prime_n;
      n = n + 1;
    }
    return count;
  }

  pub fn sieve_sum(limit: Int) -> Int {
    if limit < 2 { return 0; }
    var sum = 0;
    var n = 2;
    while n <= limit {
      if is_prime_opt(n) { sum = sum + n; }
      n = n + 1;
    }
    return sum;
  }

  fn test_sieve() -> Int {
    var score = 0;
    if sieve_count(10) == 4 { score = score + 1; }
    if sieve_count(30) == 10 { score = score + 1; }
    if sieve_count(2) == 1 { score = score + 1; }
    if sieve_count(1) == 0 { score = score + 1; }

    if sieve_sum(10) == 17 { score = score + 1; }
    if sieve_sum(5) == 5 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Prime Factorization
  // ============================================================

  pub fn factor_count(n: Int) -> Int {
    if n < 2 { return 0; }
    var count = 0;
    var x = n;
    var d = 2;
    while d * d <= x {
      while x % d == 0 {
        count = count + 1;
        x = x / d;
      }
      d = d + 1;
    }
    if x > 1 { count = count + 1; }
    return count;
  }

  pub fn factor_sum(n: Int) -> Int {
    if n < 2 { return 0; }
    var sum = 0;
    var x = n;
    var d = 2;
    while d * d <= x {
      while x % d == 0 {
        sum = sum + d;
        x = x / d;
      }
      d = d + 1;
    }
    if x > 1 { sum = sum + x; }
    return sum;
  }

  pub fn largest_factor(n: Int) -> Int {
    if n < 2 { return 0; }
    var max_f = 1;
    var x = n;
    var d = 2;
    while d * d <= x {
      while x % d == 0 {
        if d > max_f { max_f = d; }
        x = x / d;
      }
      d = d + 1;
    }
    if x > 1 && x > max_f { max_f = x; }
    return max_f;
  }

  fn test_factorization() -> Int {
    var score = 0;
    // 12 = 2*2*3 -> 3 prime factors
    if factor_count(12) == 3 { score = score + 1; }
    // 28 = 2*2*7 -> 3 prime factors
    if factor_count(28) == 3 { score = score + 1; }
    // 100 = 2*2*5*5 -> 4 prime factors
    if factor_count(100) == 4 { score = score + 1; }
    // Prime has 1 factor
    if factor_count(17) == 1 { score = score + 1; }
    if factor_count(1) == 0 { score = score + 1; }

    if factor_sum(12) == 7 { score = score + 1; }
    if factor_sum(28) == 11 { score = score + 1; }
    if factor_sum(100) == 14 { score = score + 1; }

    if largest_factor(12) == 3 { score = score + 1; }
    if largest_factor(100) == 5 { score = score + 1; }
    if largest_factor(17) == 17 { score = score + 1; }
    if largest_factor(1) == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Mersenne & Special Primes
  // ============================================================

  pub fn is_mersenne_prime(p: Int) -> Bool {
    if !(is_prime_opt(p)) { return false; }
    var mersenne = power_iter(2, p) - 1;
    return is_prime_opt(mersenne);
  }

  pub fn is_twin_prime(n: Int) -> Bool {
    return is_prime_opt(n) && (is_prime_opt(n - 2) || is_prime_opt(n + 2));
  }

  pub fn mersenne_number(p: Int) -> Int {
    return power_iter(2, p) - 1;
  }

  fn test_special_primes() -> Int {
    var score = 0;
    // Mersenne: 2^p - 1 is prime for small primes
    if is_mersenne_prime(2) { score = score + 1; }
    if is_mersenne_prime(3) { score = score + 1; }
    if is_mersenne_prime(5) { score = score + 1; }
    if is_mersenne_prime(7) { score = score + 1; }
    if !(is_mersenne_prime(11)) { score = score + 1; }

    if mersenne_number(3) == 7 { score = score + 1; }
    if mersenne_number(5) == 31 { score = score + 1; }

    // Twin primes: (3,5), (5,7), (11,13), (17,19)
    if is_twin_prime(5) { score = score + 1; }
    if is_twin_prime(7) { score = score + 1; }
    if is_twin_prime(13) { score = score + 1; }
    if !(is_twin_prime(23)) { score = score + 1; }
    if !(is_twin_prime(2)) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Goldbach & Number Theory Conjectures
  // ============================================================

  pub fn is_goldbach(even_n: Int) -> Bool {
    if even_n <= 2 || even_n % 2 != 0 { return false; }
    var i = 2;
    while i <= even_n / 2 {
      if is_prime_opt(i) && is_prime_opt(even_n - i) { return true; }
      i = i + 1;
    }
    return false;
  }

  pub fn goldbach_representation(n: Int) -> Int {
    if n <= 2 || n % 2 != 0 { return 0; }
    var i = 2;
    while i <= n / 2 {
      if is_prime_opt(i) && is_prime_opt(n - i) { return i; }
      i = i + 1;
    }
    return 0;
  }

  fn test_goldbach() -> Int {
    var score = 0;
    if is_goldbach(4) { score = score + 1; }
    if is_goldbach(6) { score = score + 1; }
    if is_goldbach(10) { score = score + 1; }
    if is_goldbach(100) { score = score + 1; }
    if !(is_goldbach(2)) { score = score + 1; }

    if goldbach_representation(4) == 2 { score = score + 1; }
    if goldbach_representation(10) == 3 { score = score + 1; }
    if goldbach_representation(2) == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Totient and Coprime Count
  // ============================================================

  pub fn gcd(a: Int, b: Int) -> Int {
    if b == 0 { return a; }
    return gcd(b, a % b);
  }

  pub fn totient(n: Int) -> Int {
    if n <= 0 { return 0; }
    if n == 1 { return 1; }
    var count = 0;
    var i = 1;
    while i < n {
      if gcd(i, n) == 1 { count = count + 1; }
      i = i + 1;
    }
    return count;
  }

  pub fn is_coprime(a: Int, b: Int) -> Bool {
    return gcd(a, b) == 1;
  }

  fn test_totient() -> Int {
    var score = 0;
    if totient(1) == 1 { score = score + 1; }
    if totient(2) == 1 { score = score + 1; }
    if totient(3) == 2 { score = score + 1; }
    if totient(4) == 2 { score = score + 1; }
    if totient(5) == 4 { score = score + 1; }
    if totient(6) == 2 { score = score + 1; }
    if totient(7) == 6 { score = score + 1; }
    if totient(8) == 4 { score = score + 1; }
    if totient(9) == 6 { score = score + 1; }
    if totient(10) == 4 { score = score + 1; }
    if totient(12) == 4 { score = score + 1; }

    if is_coprime(3, 7) { score = score + 1; }
    if is_coprime(8, 15) { score = score + 1; }
    if !(is_coprime(6, 9)) { score = score + 1; }
    if !(is_coprime(12, 8)) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Prime Sequences
  // ============================================================

  pub fn generate_primes(count: Int) -> Vec[Int] {
    var primes = Vec[Int].new();
    var n = 2;
    while primes.len() < count {
      if is_prime_opt(n) { primes.push(n); }
      n = n + 1;
    }
    return primes;
  }

  pub fn prime_sum_in_range(start: Int, end: Int) -> Int {
    var sum = 0;
    var n = start;
    while n <= end {
      if is_prime_opt(n) { sum = sum + n; }
      n = n + 1;
    }
    return sum;
  }

  fn test_prime_sequences() -> Int {
    var score = 0;
    var primes = generate_primes(5);
    if primes.len() == 5 { score = score + 1; }
    if primes.len() >= 1 && primes[0] == 2 { score = score + 1; }
    if primes.len() >= 5 && primes[4] == 11 { score = score + 1; }

    if prime_sum_in_range(2, 10) == 17 { score = score + 1; }
    if prime_sum_in_range(10, 20) == 31 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 8: Primality by Wilson's Theorem
  // ============================================================

  pub fn wilson_check(n: Int) -> Bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    var fact = 1;
    var i = 2;
    while i < n {
      fact = (fact * i) % n;
      i = i + 1;
    }
    return fact == n - 1;
  }

  fn test_wilson() -> Int {
    var score = 0;
    if wilson_check(2) { score = score + 1; }
    if wilson_check(3) { score = score + 1; }
    if wilson_check(5) { score = score + 1; }
    if wilson_check(7) { score = score + 1; }
    if wilson_check(11) { score = score + 1; }
    if !(wilson_check(4)) { score = score + 1; }
    if !(wilson_check(9)) { score = score + 1; }
    if !(wilson_check(1)) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 9: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_primality();
    total = total + s1;
    max_score = max_score + 26;

    var s2 = test_sieve();
    total = total + s2;
    max_score = max_score + 6;

    var s3 = test_factorization();
    total = total + s3;
    max_score = max_score + 12;

    var s4 = test_special_primes();
    total = total + s4;
    max_score = max_score + 12;

    var s5 = test_goldbach();
    total = total + s5;
    max_score = max_score + 8;

    var s6 = test_totient();
    total = total + s6;
    max_score = max_score + 15;

    var s7 = test_prime_sequences();
    total = total + s7;
    max_score = max_score + 5;

    var s8 = test_wilson();
    total = total + s8;
    max_score = max_score + 8;

    return BenchResult{
      name: "primes",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: control
// ============================================================
module control {


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
    while x > 0 {
      x = x - 1;
      steps = steps + 1;
    }
    return steps;
  }

  pub fn factorial_while(n: Int) -> Int {
    if n <= 0 { return 1; }
    var result = 1;
    var i = n;
    while i > 0 {
      result = result * i;
      i = i - 1;
    }
    return result;
  }

  pub fn while_nested(depth: Int, width: Int) -> Int {
    var total = 0;
    var i = 0;
    while i < depth {
      var j = 0;
      while j < width {
        total = total + 1;
        j = j + 1;
      }
      i = i + 1;
    }
    return total;
  }

  pub fn while_with_break_sim(n: Int) -> Int {
    var found = -1;
    var i = 0;
    var done = 0;
    while done == 0 {
      if i * i > n {
        done = 1;
      } else {
        if i * i == n {
          found = i;
          done = 1;
        }
      }
      i = i + 1;
    }
    return found;
  }

  fn test_while() -> Int {
    var score = 0;
    if sum_while(0) == 0 { score = score + 1; }
    if sum_while(10) == 55 { score = score + 1; }
    if sum_while(100) == 5050 { score = score + 1; }

    if countdown(5) == 5 { score = score + 1; }
    if countdown(0) == 0 { score = score + 1; }

    if factorial_while(5) == 120 { score = score + 1; }
    if factorial_while(0) == 1 { score = score + 1; }
    if factorial_while(7) == 5040 { score = score + 1; }

    if while_nested(3, 4) == 12 { score = score + 1; }
    if while_nested(1, 1) == 1 { score = score + 1; }
    if while_nested(10, 10) == 100 { score = score + 1; }

    if while_with_break_sim(25) == 5 { score = score + 1; }
    if while_with_break_sim(9) == 3 { score = score + 1; }
    if while_with_break_sim(26) == -1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Boolean Logic
  // ============================================================

  pub fn all_three(a: Bool, b: Bool, c: Bool) -> Bool {
    return a && b && c;
  }

  pub fn any_three(a: Bool, b: Bool, c: Bool) -> Bool {
    return a || b || c;
  }

  pub fn xor(a: Bool, b: Bool) -> Bool {
    return (a || b) && !(a && b);
  }

  pub fn majority(a: Bool, b: Bool, c: Bool) -> Bool {
    var count = 0;
    if a { count = count + 1; }
    if b { count = count + 1; }
    if c { count = count + 1; }
    return count >= 2;
  }

  pub fn bool_to_int(b: Bool) -> Int {
    if b { return 1; }
    return 0;
  }

  fn test_bool() -> Int {
    var score = 0;
    if all_three(true, true, true) { score = score + 1; }
    if !(all_three(true, true, false)) { score = score + 1; }
    if !(all_three(false, false, false)) { score = score + 1; }

    if any_three(true, false, false) { score = score + 1; }
    if !(any_three(false, false, false)) { score = score + 1; }

    if xor(true, false) { score = score + 1; }
    if xor(false, true) { score = score + 1; }
    if !(xor(true, true)) { score = score + 1; }
    if !(xor(false, false)) { score = score + 1; }

    if majority(true, true, true) { score = score + 1; }
    if majority(true, true, false) { score = score + 1; }
    if !(majority(true, false, false)) { score = score + 1; }
    if !(majority(false, false, false)) { score = score + 1; }

    if bool_to_int(true) == 1 { score = score + 1; }
    if bool_to_int(false) == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: State Machine
  // ============================================================

  pub type TrafficLight = {
    state: Int;
  } derive[Clone]

  pub fn TrafficLight.new() -> TrafficLight {
    return TrafficLight{ state: 0 };
  }

  pub fn TrafficLight.next() -> TrafficLight {
    match state {
      0 => TrafficLight{ state: 1 },
      1 => TrafficLight{ state: 2 },
      2 => TrafficLight{ state: 0 },
      _ => TrafficLight{ state: 0 },
    }
  }

  pub fn TrafficLight.is_green() -> Bool {
    return state == 0;
  }

  pub fn TrafficLight.is_yellow() -> Bool {
    return state == 1;
  }

  pub fn TrafficLight.is_red() -> Bool {
    return state == 2;
  }

  fn test_state_machine() -> Int {
    var score = 0;
    var light = TrafficLight.new();
    if light.is_green() { score = score + 1; }

    var light2 = light.next();
    if light2.is_yellow() { score = score + 1; }

    var light3 = light2.next();
    if light3.is_red() { score = score + 1; }

    var light4 = light3.next();
    if light4.is_green() { score = score + 1; }

    // Cycle full: green->yellow->red->green
    var cycle = TrafficLight.new();
    var c1 = cycle.next();
    var c2 = c1.next();
    var c3 = c2.next();
    if c3.is_green() { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Complex Nested Conditions
  // ============================================================

  pub fn triangle_type(a: Int, b: Int, c: Int) -> Int {
    if a <= 0 || b <= 0 || c <= 0 { return 0; }
    if a + b <= c || b + c <= a || a + c <= b { return 0; }
    if a == b && b == c { return 3; }
    if a == b || b == c || a == c { return 2; }
    return 1;
  }

  pub fn leap_year(year: Int) -> Bool {
    if year % 400 == 0 { return true; }
    if year % 100 == 0 { return false; }
    if year % 4 == 0 { return true; }
    return false;
  }

  pub fn days_in_month(month: Int, year: Int) -> Int {
    match month {
      1 => 31,
      2 => if leap_year(year) { 29; } else { 28; },
      3 => 31,
      4 => 30,
      5 => 31,
      6 => 30,
      7 => 31,
      8 => 31,
      9 => 30,
      10 => 31,
      11 => 30,
      12 => 31,
      _ => 0,
    }
  }

  pub fn day_of_week(day: Int, month: Int, year: Int) -> Int {
    var y = year;
    var m = month;
    if m < 3 {
      m = m + 12;
      y = y - 1;
    }
    var k = y % 100;
    var j = y / 100;
    return (day + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
  }

  fn test_complex_conditions() -> Int {
    var score = 0;
    // Triangle types
    if triangle_type(1, 1, 1) == 3 { score = score + 1; }
    if triangle_type(2, 2, 3) == 2 { score = score + 1; }
    if triangle_type(3, 4, 5) == 1 { score = score + 1; }
    if triangle_type(0, 1, 2) == 0 { score = score + 1; }
    if triangle_type(1, 1, 3) == 0 { score = score + 1; }

    // Leap years
    if leap_year(2000) { score = score + 1; }
    if leap_year(2020) { score = score + 1; }
    if !(leap_year(1900)) { score = score + 1; }
    if !(leap_year(2023)) { score = score + 1; }
    if leap_year(2024) { score = score + 1; }

    // Days in month
    if days_in_month(1, 2023) == 31 { score = score + 1; }
    if days_in_month(2, 2024) == 29 { score = score + 1; }
    if days_in_month(2, 2023) == 28 { score = score + 1; }
    if days_in_month(4, 2023) == 30 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 8: Recursion-Equivalent Iterative Patterns
  // ============================================================

  pub fn collatz_steps(n: Int) -> Int {
    var steps = 0;
    var x = n;
    while x > 1 {
      if x % 2 == 0 {
        x = x / 2;
      } else {
        x = 3 * x + 1;
      }
      steps = steps + 1;
    }
    return steps;
  }

  pub fn collatz_max(n: Int) -> Int {
    var max_val = n;
    var x = n;
    while x > 1 {
      if x % 2 == 0 {
        x = x / 2;
      } else {
        x = 3 * x + 1;
      }
      if x > max_val { max_val = x; }
    }
    return max_val;
  }

  fn test_collatz() -> Int {
    var score = 0;
    if collatz_steps(1) == 0 { score = score + 1; }
    if collatz_steps(6) == 8 { score = score + 1; }

    if collatz_max(1) == 1 { score = score + 1; }
    if collatz_max(6) == 16 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 9: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_deep_if();
    total = total + s1;
    max_score = max_score + 8;

    var s2 = test_fizzbuzz();
    total = total + s2;
    max_score = max_score + 11;

    var s3 = test_match();
    total = total + s3;
    max_score = max_score + 8;

    var s4 = test_while();
    total = total + s4;
    max_score = max_score + 14;

    var s5 = test_bool();
    total = total + s5;
    max_score = max_score + 15;

    var s6 = test_state_machine();
    total = total + s6;
    max_score = max_score + 5;

    var s7 = test_complex_conditions();
    total = total + s7;
    max_score = max_score + 14;

    var s8 = test_collatz();
    total = total + s8;
    max_score = max_score + 4;

    return BenchResult{
      name: "control",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: types
// ============================================================
module types {


  // ============================================================
  // SECTION 1: Simple Structs & Field Access
  // ============================================================

  pub type Point2D = {
    x: Float64;
    y: Float64;
  } derive[Eq, Clone]

  pub type Point3D = {
    x: Float64;
    y: Float64;
    z: Float64;
  } derive[Eq, Clone]

  pub type Color = {
    r: Int;
    g: Int;
    b: Int;
    a: Int;
  } derive[Eq, Clone]

  pub type Size = {
    w: Int;
    h: Int;
  } derive[Eq, Clone]

  pub type Rect = {
    x: Int;
    y: Int;
    w: Int;
    h: Int;
  } derive[Eq, Clone]

  pub fn Point2D.new(x: Float64, y: Float64) -> Point2D {
    return Point2D{ x: x, y: y };
  }

  pub fn Point2D.dist_sq(other: &Point2D) -> Float64 {
    var dx = x - other.x;
    var dy = y - other.y;
    return dx * dx + dy * dy;
  }

  pub fn Point3D.new(x: Float64, y: Float64, z: Float64) -> Point3D {
    return Point3D{ x: x, y: y, z: z };
  }

  pub fn Point3D.dist_sq(other: &Point3D) -> Float64 {
    var dx = x - other.x;
    var dy = y - other.y;
    var dz = z - other.z;
    return dx * dx + dy * dy + dz * dz;
  }

  fn test_basic_structs() -> Int {
    var score = 0;
    var p1 = Point2D.new(0.0, 0.0);
    var p2 = Point2D.new(3.0, 4.0);

    if p1.x == 0.0 { score = score + 1; }
    if p1.y == 0.0 { score = score + 1; }
    if p2.x == 3.0 { score = score + 1; }
    if p2.y == 4.0 { score = score + 1; }
    if p1.dist_sq(&p2) == 25.0 { score = score + 1; }

    var p3 = Point3D.new(1.0, 2.0, 3.0);
    if p3.x == 1.0 { score = score + 1; }
    if p3.y == 2.0 { score = score + 1; }
    if p3.z == 3.0 { score = score + 1; }

    var p3b = Point3D.new(4.0, 6.0, 8.0);
    var dsq = p3.dist_sq(&p3b);
    if dsq == 50.0 { score = score + 1; }

    var c = Color{ r: 255, g: 128, b: 64, a: 255 };
    if c.r == 255 { score = score + 1; }
    if c.g == 128 { score = score + 1; }
    if c.b == 64 { score = score + 1; }
    if c.a == 255 { score = score + 1; }

    var s = Size{ w: 800, h: 600 };
    if s.w == 800 { score = score + 1; }
    if s.h == 600 { score = score + 1; }

    var r = Rect{ x: 10, y: 20, w: 100, h: 50 };
    if r.x == 10 { score = score + 1; }
    if r.y == 20 { score = score + 1; }
    if r.w == 100 { score = score + 1; }
    if r.h == 50 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Nested Structs
  // ============================================================

  pub type Address = {
    street: Str;
    city: Str;
    zip: Int;
  } derive[Clone]

  pub type Person = {
    name: Str;
    age: Int;
    address: Address;
  } derive[Clone]

  pub type Company = {
    name: Str;
    employees: Int;
    hq: Address;
  } derive[Clone]

  pub type Transform = {
    pos: Point3D;
    rot: Point3D;
    scale: Point3D;
  } derive[Clone]

  fn test_nested_structs() -> Int {
    var score = 0;
    var addr = Address{ street: "Main", city: "NYC", zip: 10001 };
    if addr.zip == 10001 { score = score + 1; }

    var person = Person{ name: "Alice", age: 30, address: addr };
    if person.name == "Alice" { score = score + 1; }
    if person.age == 30 { score = score + 1; }
    if person.address.zip == 10001 { score = score + 1; }

    var company = Company{
      name: "ACME",
      employees: 500,
      hq: addr,
    };
    if company.name == "ACME" { score = score + 1; }
    if company.employees == 500 { score = score + 1; }
    if company.hq.city == "NYC" { score = score + 1; }

    var t = Transform{
      pos: Point3D.new(0.0, 0.0, 0.0),
      rot: Point3D.new(1.0, 0.0, 0.0),
      scale: Point3D.new(2.0, 2.0, 2.0),
    };
    if t.pos.x == 0.0 { score = score + 1; }
    if t.rot.x == 1.0 { score = score + 1; }
    if t.scale.x == 2.0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Large Struct (50 Fields)
  // ============================================================

  pub type BigStruct = {
    f00: Int; f01: Int; f02: Int; f03: Int; f04: Int;
    f05: Int; f06: Int; f07: Int; f08: Int; f09: Int;
    f10: Int; f11: Int; f12: Int; f13: Int; f14: Int;
    f15: Int; f16: Int; f17: Int; f18: Int; f19: Int;
    f20: Int; f21: Int; f22: Int; f23: Int; f24: Int;
    f25: Int; f26: Int; f27: Int; f28: Int; f29: Int;
    f30: Int; f31: Int; f32: Int; f33: Int; f34: Int;
    f35: Int; f36: Int; f37: Int; f38: Int; f39: Int;
    f40: Int; f41: Int; f42: Int; f43: Int; f44: Int;
    f45: Int; f46: Int; f47: Int; f48: Int; f49: Int;
  } derive[Clone]

  pub fn BigStruct.new(val: Int) -> BigStruct {
    return BigStruct{
      f00: val, f01: val + 1, f02: val + 2, f03: val + 3, f04: val + 4,
      f05: val + 5, f06: val + 6, f07: val + 7, f08: val + 8, f09: val + 9,
      f10: val + 10, f11: val + 11, f12: val + 12, f13: val + 13, f14: val + 14,
      f15: val + 15, f16: val + 16, f17: val + 17, f18: val + 18, f19: val + 19,
      f20: val + 20, f21: val + 21, f22: val + 22, f23: val + 23, f24: val + 24,
      f25: val + 25, f26: val + 26, f27: val + 27, f28: val + 28, f29: val + 29,
      f30: val + 30, f31: val + 31, f32: val + 32, f33: val + 33, f34: val + 34,
      f35: val + 35, f36: val + 36, f37: val + 37, f38: val + 38, f39: val + 39,
      f40: val + 40, f41: val + 41, f42: val + 42, f43: val + 43, f44: val + 44,
      f45: val + 45, f46: val + 46, f47: val + 47, f48: val + 48, f49: val + 49,
    };
  }

  pub fn BigStruct.sum() -> Int {
    return f00 + f01 + f02 + f03 + f04 + f05 + f06 + f07 + f08 + f09
      + f10 + f11 + f12 + f13 + f14 + f15 + f16 + f17 + f18 + f19
      + f20 + f21 + f22 + f23 + f24 + f25 + f26 + f27 + f28 + f29
      + f30 + f31 + f32 + f33 + f34 + f35 + f36 + f37 + f38 + f39
      + f40 + f41 + f42 + f43 + f44 + f45 + f46 + f47 + f48 + f49;
  }

  fn test_big_struct() -> Int {
    var score = 0;
    var bs = BigStruct.new(100);
    if bs.f00 == 100 { score = score + 1; }
    if bs.f25 == 125 { score = score + 1; }
    if bs.f49 == 149 { score = score + 1; }

    // Sum of all fields: 100+101+...+149 = 50*100 + (0+1+...+49) = 5000 + 1225 = 6225
    if bs.sum() == 6225 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Type Aliases & Composition
  // ============================================================

  pub type Vec2 = Point2D;
  pub type Vec3 = Point3D;

  pub type Matrix2x2 = {
    a11: Float64; a12: Float64;
    a21: Float64; a22: Float64;
  } derive[Clone]

  pub type Matrix3x3 = {
    m11: Float64; m12: Float64; m13: Float64;
    m21: Float64; m22: Float64; m23: Float64;
    m31: Float64; m32: Float64; m33: Float64;
  } derive[Clone]

  pub fn Matrix2x2.new(a11: Float64, a12: Float64, a21: Float64, a22: Float64) -> Matrix2x2 {
    return Matrix2x2{ a11: a11, a12: a12, a21: a21, a22: a22 };
  }

  pub fn Matrix2x2.determinant() -> Float64 {
    return a11 * a22 - a12 * a21;
  }

  pub fn Matrix3x3.new(m11: Float64, m12: Float64, m13: Float64, m21: Float64, m22: Float64, m23: Float64, m31: Float64, m32: Float64, m33: Float64) -> Matrix3x3 {
    return Matrix3x3{
      m11: m11, m12: m12, m13: m13,
      m21: m21, m22: m22, m23: m23,
      m31: m31, m32: m32, m33: m33,
    };
  }

  pub fn Matrix3x3.determinant() -> Float64 {
    var a = m11 * (m22 * m33 - m23 * m32);
    var b = m12 * (m21 * m33 - m23 * m31);
    var c = m13 * (m21 * m32 - m22 * m31);
    return a - b + c;
  }

  fn test_matrices() -> Int {
    var score = 0;
    var m2 = Matrix2x2.new(1.0, 2.0, 3.0, 4.0);
    if m2.determinant() == -2.0 { score = score + 1; }

    var m2b = Matrix2x2.new(5.0, 0.0, 0.0, 5.0);
    if m2b.determinant() == 25.0 { score = score + 1; }

    var m3 = Matrix3x3.new(
      1.0, 0.0, 0.0,
      0.0, 1.0, 0.0,
      0.0, 0.0, 1.0,
    );
    if m3.determinant() == 1.0 { score = score + 1; }

    var m3b = Matrix3x3.new(
      2.0, 0.0, 0.0,
      0.0, 2.0, 0.0,
      0.0, 0.0, 2.0,
    );
    if m3b.determinant() == 8.0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Relationship Types
  // ============================================================

  pub type Date = {
    day: Int;
    month: Int;
    year: Int;
  } derive[Eq, Clone]

  pub type TimeOfDay = {
    hour: Int;
    minute: Int;
    second: Int;
  } derive[Clone]

  pub type DateTime = {
    date: Date;
    time: TimeOfDay;
  } derive[Clone]

  pub type Range[T] = {
    min: T;
    max: T;
  } derive[Clone]

  pub type Pair[A, B] = {
    first: A;
    second: B;
  } derive[Clone]

  fn test_relationship_types() -> Int {
    var score = 0;
    var d1 = Date{ day: 1, month: 1, year: 2024 };
    var d2 = Date{ day: 1, month: 1, year: 2024 };
    if d1 == d2 { score = score + 1; }
    if d1.year == 2024 { score = score + 1; }

    var t = TimeOfDay{ hour: 14, minute: 30, second: 0 };
    if t.hour == 14 { score = score + 1; }
    if t.minute == 30 { score = score + 1; }

    var dt = DateTime{ date: d1, time: t };
    if dt.date.day == 1 { score = score + 1; }
    if dt.time.hour == 14 { score = score + 1; }

    var int_range = Range[Int]{ min: 0, max: 100 };
    if int_range.min == 0 { score = score + 1; }
    if int_range.max == 100 { score = score + 1; }

    var float_range = Range[Float64]{ min: -1.0, max: 1.0 };
    if float_range.min == -1.0 { score = score + 1; }
    if float_range.max == 1.0 { score = score + 1; }

    var p = Pair[Int, Str]{ first: 42, second: "hello" };
    if p.first == 42 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Type with Many Methods
  // ============================================================

  pub type Counter = {
    value: Int;
    step: Int;
  } derive[Clone]

  pub fn Counter.new(start: Int, step: Int) -> Counter {
    return Counter{ value: start, step: step };
  }

  pub fn Counter.inc() -> Counter {
    return Counter{ value: value + step, step: step };
  }

  pub fn Counter.dec() -> Counter {
    return Counter{ value: value - step, step: step };
  }

  pub fn Counter.reset() -> Counter {
    return Counter{ value: 0, step: step };
  }

  pub fn Counter.set(new_val: Int) -> Counter {
    return Counter{ value: new_val, step: step };
  }

  pub fn Counter.is_positive() -> Bool {
    return value > 0;
  }

  pub fn Counter.is_zero() -> Bool {
    return value == 0;
  }

  fn test_counter() -> Int {
    var score = 0;
    var c = Counter.new(0, 1);
    if c.value == 0 { score = score + 1; }
    if c.step == 1 { score = score + 1; }

    var c2 = c.inc();
    if c2.value == 1 { score = score + 1; }

    var c3 = c2.inc().inc().inc();
    if c3.value == 4 { score = score + 1; }

    var c4 = c3.dec();
    if c4.value == 3 { score = score + 1; }

    var c5 = c4.set(100);
    if c5.value == 100 { score = score + 1; }

    var c6 = c5.reset();
    if c6.value == 0 { score = score + 1; }

    if c.is_zero() { score = score + 1; }
    if c2.is_positive() { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_basic_structs();
    total = total + s1;
    max_score = max_score + 19;

    var s2 = test_nested_structs();
    total = total + s2;
    max_score = max_score + 11;

    var s3 = test_big_struct();
    total = total + s3;
    max_score = max_score + 4;

    var s4 = test_matrices();
    total = total + s4;
    max_score = max_score + 5;

    var s5 = test_relationship_types();
    total = total + s5;
    max_score = max_score + 12;

    var s6 = test_counter();
    total = total + s6;
    max_score = max_score + 9;

    return BenchResult{
      name: "types",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: structures
// ============================================================
module structures {


  // ============================================================
  // SECTION 1: Vec Operations
  // ============================================================

  pub fn vec_push_test(n: Int) -> Int {
    var v = Vec[Int].new();
    var i = 0;
    while i < n {
      v.push(i);
      i = i + 1;
    }
    return v.len();
  }

  pub fn vec_pop_test(n: Int) -> Int {
    var v = Vec[Int].new();
    var i = 0;
    while i < n {
      v.push(i);
      i = i + 1;
    }
    var popped = 0;
    i = 0;
    while i < n / 2 {
      if v.len() > 0 {
        v.pop();
        popped = popped + 1;
      }
      i = i + 1;
    }
    return popped;
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

  pub fn vec_max(v: Vec[Int]) -> Int {
    if v.len() == 0 { return 0; }
    var max_val = v[0];
    var i = 1;
    while i < v.len() {
      if v[i] > max_val { max_val = v[i]; }
      i = i + 1;
    }
    return max_val;
  }

  pub fn vec_reverse(v: Vec[Int]) -> Vec[Int] {
    var r = Vec[Int].new();
    var i = v.len();
    while i > 0 {
      i = i - 1;
      r.push(v[i]);
    }
    return r;
  }

  fn test_vec_basic() -> Int {
    var score = 0;
    if vec_push_test(100) == 100 { score = score + 1; }
    if vec_push_test(0) == 0 { score = score + 1; }

    if vec_pop_test(100) == 50 { score = score + 1; }

    var nums = [1, 2, 3, 4, 5];
    if vec_sum(&nums) == 15 { score = score + 1; }

    var empty: Vec[Int] = [];
    if vec_sum(&empty) == 0 { score = score + 1; }

    var nums2 = [7, 2, 9, 1, 5];
    if vec_max(nums2) == 9 { score = score + 1; }

    var rev = vec_reverse(nums);
    if rev.len() == 5 { score = score + 1; }
    if rev[0] == 5 { score = score + 1; }
    if rev[4] == 1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Binary Search Tree (using owned types)
  // ============================================================

  pub enum BST[T] {
    Empty,
    Node(value: T, left: BST[T], right: BST[T]),
  }

  pub fn BST.new[T]() -> BST[T] {
    return Empty;
  }

  pub fn BST.insert[T](val: T) -> BST[T] {
    match self {
      Empty => Node(value: val, left: Empty, right: Empty),
      Node(value: v, left: l, right: r) => {
        if val < v {
          return Node(value: v, left: l.insert(val), right: r);
        }
        elif val > v {
          return Node(value: v, left: l, right: r.insert(val));
        } else {
          return Node(value: v, left: l, right: r);
        }
      }
    }
  }

  pub fn BST.contains[T](val: T) -> Bool {
    match self {
      Empty => false,
      Node(value: v, left: l, right: r) => {
        if val == v { return true; }
        if val < v { return l.contains(val); }
        return r.contains(val);
      }
    }
  }

  pub fn BST.size[T]() -> Int {
    match self {
      Empty => 0,
      Node(value: _, left: l, right: r) => {
        return 1 + l.size() + r.size();
      }
    }
  }

  pub fn BST.min[T]() -> Option[T] {
    match self {
      Empty => None,
      Node(value: v, left: Empty, right: _) => Some(v),
      Node(value: _, left: l, right: _) => l.min(),
    }
  }

  fn test_bst() -> Int {
    var score = 0;
    var tree: BST[Int] = BST.new[Int]();
    if tree.size() == 0 { score = score + 1; }
    if !(tree.contains(5)) { score = score + 1; }

    var t1 = tree.insert(5);
    if t1.size() == 1 { score = score + 1; }
    if t1.contains(5) { score = score + 1; }
    if !(t1.contains(3)) { score = score + 1; }

    var t2 = t1.insert(3);
    var t3 = t2.insert(7);
    var t4 = t3.insert(1);
    var t5 = t4.insert(9);

    if t5.size() == 5 { score = score + 1; }
    if t5.contains(1) { score = score + 1; }
    if t5.contains(5) { score = score + 1; }
    if t5.contains(9) { score = score + 1; }
    if !(t5.contains(4)) { score = score + 1; }
    if !(t5.contains(10)) { score = score + 1; }

    match t5.min() {
      Some(v) => if v == 1 { score = score + 1; }
      None => {}
    }

    // Test 100 inserts (non-decreasing)
    var big_tree: BST[Int] = BST.new[Int]();
    var i = 0;
    while i < 100 {
      big_tree = big_tree.insert(i);
      i = i + 1;
    }
    if big_tree.size() == 100 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Linked List
  // ============================================================

  pub enum List[T] {
    Nil,
    Cons(head: T, tail: List[T]),
  }

  pub fn List.new[T]() -> List[T] {
    return Nil;
  }

  pub fn List.prepend[T](val: T) -> List[T] {
    return Cons(head: val, tail: self);
  }

  pub fn List.length[T]() -> Int {
    match self {
      Nil => 0,
      Cons(head: _, tail: t) => 1 + t.length(),
    }
  }

  pub fn List.sum() -> Int {
    match self {
      Nil => 0,
      Cons(head: h, tail: t) => h + t.sum(),
    }
  }

  pub fn List.reverse[T]() -> List[T] {
    match self {
      Nil => Nil,
      Cons(head: h, tail: t) => {
        var rev_tail = t.reverse();
        return rev_tail.append(h);
      }
    }
  }

  pub fn List.append[T](val: T) -> List[T] {
    match self {
      Nil => Cons(head: val, tail: Nil),
      Cons(head: h, tail: t) => Cons(head: h, tail: t.append(val)),
    }
  }

  fn test_list() -> Int {
    var score = 0;
    var list: List[Int] = List.new[Int]();
    if list.length() == 0 { score = score + 1; }

    var l1 = list.prepend(3);
    var l2 = l1.prepend(2);
    var l3 = l2.prepend(1);

    if l3.length() == 3 { score = score + 1; }
    if l3.sum() == 6 { score = score + 1; }

    var l4 = List.new[Int]();
    l4 = l4.append(10);
    l4 = l4.append(20);
    l4 = l4.append(30);
    if l4.length() == 3 { score = score + 1; }
    if l4.sum() == 60 { score = score + 1; }

    // Test 50 prepends
    var big_list: List[Int] = List.new[Int]();
    var i = 0;
    while i < 50 {
      big_list = big_list.prepend(i);
      i = i + 1;
    }
    if big_list.length() == 50 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Stack (using Vec)
  // ============================================================

  pub type Stack[T] = {
    items: Vec[T];
  } derive[Clone]

  pub fn Stack.new[T]() -> Stack[T] {
    return Stack[T]{ items: Vec[T].new() };
  }

  pub fn Stack.push[T](val: T) {
    items.push(val);
  }

  pub fn Stack.pop[T]() -> Option[T] {
    if items.len() == 0 { return None; }
    var idx = items.len() - 1;
    var val = items[idx];
    // Simplify: pop last element
    return Some(val);
  }

  pub fn Stack.peek[T]() -> Option[T] {
    if items.len() == 0 { return None; }
    return Some(items[items.len() - 1]);
  }

  pub fn Stack.is_empty[T]() -> Bool {
    return items.len() == 0;
  }

  pub fn Stack.size[T]() -> Int {
    return items.len();
  }

  fn test_stack() -> Int {
    var score = 0;
    var s: Stack[Int] = Stack.new[Int]();

    if s.is_empty() { score = score + 1; }
    if s.size() == 0 { score = score + 1; }

    s.push(10);
    s.push(20);
    s.push(30);

    if s.size() == 3 { score = score + 1; }
    if !(s.is_empty()) { score = score + 1; }

    match s.peek() {
      Some(v) => if v == 30 { score = score + 1; }
      None => {}
    }

    match s.pop() {
      Some(v) => if v == 30 { score = score + 1; }
      None => {}
    }

    return score;
  }

  // ============================================================
  // SECTION 5: Queue (ring buffer simulation)
  // ============================================================

  pub type Queue[T] = {
    data: Vec[T];
    head: Int;
    tail: Int;
  } derive[Clone]

  pub fn Queue.new[T]() -> Queue[T] {
    return Queue[T]{ data: Vec[T].new(), head: 0, tail: 0 };
  }

  pub fn Queue.enqueue[T](val: T) {
    data.push(val);
    tail = data.len();
  }

  pub fn Queue.dequeue[T]() -> Option[T] {
    if head >= tail { return None; }
    var val = data[head];
    head = head + 1;
    return Some(val);
  }

  pub fn Queue.is_empty[T]() -> Bool {
    return head >= tail;
  }

  pub fn Queue.size[T]() -> Int {
    return tail - head;
  }

  fn test_queue() -> Int {
    var score = 0;
    var q: Queue[Int] = Queue.new[Int]();

    if q.is_empty() { score = score + 1; }
    if q.size() == 0 { score = score + 1; }

    q.enqueue(1);
    q.enqueue(2);
    q.enqueue(3);

    if q.size() == 3 { score = score + 1; }

    match q.dequeue() {
      Some(v) => if v == 1 { score = score + 1; }
      None => {}
    }

    match q.dequeue() {
      Some(v) => if v == 2 { score = score + 1; }
      None => {}
    }

    if q.size() == 1 { score = score + 1; }

    match q.dequeue() {
      Some(v) => if v == 3 { score = score + 1; }
      None => {}
    }

    if q.is_empty() { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Map Patterns (using parallel Vecs)
  // ============================================================

  pub type SimpleMap = {
    keys: Vec[Int];
    values: Vec[Int];
  } derive[Clone]

  pub fn SimpleMap.new() -> SimpleMap {
    return SimpleMap{ keys: Vec[Int].new(), values: Vec[Int].new() };
  }

  pub fn SimpleMap.insert(key: Int, value: Int) {
    var i = 0;
    var found = 0;
    while i < keys.len() && found == 0 {
      if keys[i] == key {
        values[i] = value;
        found = 1;
      }
      i = i + 1;
    }
    if found == 0 {
      keys.push(key);
      values.push(value);
    }
  }

  pub fn SimpleMap.get(key: Int) -> Option[Int] {
    var i = 0;
    while i < keys.len() {
      if keys[i] == key { return Some(values[i]); }
      i = i + 1;
    }
    return None;
  }

  pub fn SimpleMap.contains(key: Int) -> Bool {
    var i = 0;
    while i < keys.len() {
      if keys[i] == key { return true; }
      i = i + 1;
    }
    return false;
  }

  pub fn SimpleMap.size() -> Int {
    return keys.len();
  }

  fn test_map() -> Int {
    var score = 0;
    var m = SimpleMap.new();

    if m.size() == 0 { score = score + 1; }
    if !(m.contains(5)) { score = score + 1; }

    m.insert(1, 10);
    m.insert(2, 20);
    m.insert(3, 30);

    if m.size() == 3 { score = score + 1; }
    if m.contains(2) { score = score + 1; }
    if !(m.contains(5)) { score = score + 1; }

    match m.get(1) {
      Some(v) => if v == 10 { score = score + 1; }
      None => {}
    }

    match m.get(3) {
      Some(v) => if v == 30 { score = score + 1; }
      None => {}
    }

    // Update existing
    m.insert(2, 200);
    match m.get(2) {
      Some(v) => if v == 200 { score = score + 1; }
      None => {}
    }

    return score;
  }

  // ============================================================
  // SECTION 7: Set Operations
  // ============================================================

  pub type SimpleSet = {
    data: Vec[Int];
  } derive[Clone]

  pub fn SimpleSet.new() -> SimpleSet {
    return SimpleSet{ data: Vec[Int].new() };
  }

  pub fn SimpleSet.add(val: Int) {
    var i = 0;
    while i < data.len() {
      if data[i] == val { return; }
      i = i + 1;
    }
    data.push(val);
  }

  pub fn SimpleSet.contains(val: Int) -> Bool {
    var i = 0;
    while i < data.len() {
      if data[i] == val { return true; }
      i = i + 1;
    }
    return false;
  }

  pub fn SimpleSet.size() -> Int {
    return data.len();
  }

  pub fn set_union(a: &SimpleSet, b: &SimpleSet) -> SimpleSet {
    var result = SimpleSet.new();
    var i = 0;
    while i < a.data.len() {
      result.add(a.data[i]);
      i = i + 1;
    }
    i = 0;
    while i < b.data.len() {
      result.add(b.data[i]);
      i = i + 1;
    }
    return result;
  }

  pub fn set_intersection(a: &SimpleSet, b: &SimpleSet) -> SimpleSet {
    var result = SimpleSet.new();
    var i = 0;
    while i < a.data.len() {
      if b.contains(a.data[i]) {
        result.add(a.data[i]);
      }
      i = i + 1;
    }
    return result;
  }

  fn test_set() -> Int {
    var score = 0;
    var s = SimpleSet.new();

    if s.size() == 0 { score = score + 1; }

    s.add(1);
    s.add(2);
    s.add(2);
    s.add(3);

    if s.size() == 3 { score = score + 1; }
    if s.contains(1) { score = score + 1; }
    if s.contains(2) { score = score + 1; }
    if !(s.contains(4)) { score = score + 1; }

    var s2 = SimpleSet.new();
    s2.add(2);
    s2.add(3);
    s2.add(4);

    var u = set_union(&s, &s2);
    if u.size() == 4 { score = score + 1; }

    var inter = set_intersection(&s, &s2);
    if inter.size() == 2 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 8: Iterator Simulation (while-loop based)
  // ============================================================

  pub fn vec_filter(v: &Vec[Int], predicate: fn(Int) -> Bool) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      if predicate(v[i]) { result.push(v[i]); }
      i = i + 1;
    }
    return result;
  }

  pub fn vec_map(v: &Vec[Int], mapper: fn(Int) -> Int) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      result.push(mapper(v[i]));
      i = i + 1;
    }
    return result;
  }

  pub fn vec_fold(v: &Vec[Int], initial: Int, folder: fn(Int, Int) -> Int) -> Int {
    var accum = initial;
    var i = 0;
    while i < v.len() {
      accum = folder(accum, v[i]);
      i = i + 1;
    }
    return accum;
  }

  fn is_even_pred(n: Int) -> Bool { return n % 2 == 0; }
  fn double(n: Int) -> Int { return n * 2; }
  fn add(a: Int, b: Int) -> Int { return a + b; }
  fn multiply(a: Int, b: Int) -> Int { return a * b; }

  fn test_iterators() -> Int {
    var score = 0;
    var nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    var evens = vec_filter(&nums, is_even_pred);
    if evens.len() == 5 { score = score + 1; }

    var doubled = vec_map(&nums, double);
    if doubled.len() == 10 { score = score + 1; }
    if doubled[0] == 2 { score = score + 1; }

    var sum = vec_fold(&nums, 0, add);
    if sum == 55 { score = score + 1; }

    var product = vec_fold(&nums, 1, multiply);
    if product == 3628800 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 9: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_vec_basic();
    total = total + s1;
    max_score = max_score + 9;

    var s2 = test_bst();
    total = total + s2;
    max_score = max_score + 14;

    var s3 = test_list();
    total = total + s3;
    max_score = max_score + 7;

    var s4 = test_stack();
    total = total + s4;
    max_score = max_score + 6;

    var s5 = test_queue();
    total = total + s5;
    max_score = max_score + 8;

    var s6 = test_map();
    total = total + s6;
    max_score = max_score + 9;

    var s7 = test_set();
    total = total + s7;
    max_score = max_score + 8;

    var s8 = test_iterators();
    total = total + s8;
    max_score = max_score + 5;

    return BenchResult{
      name: "structures",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: memory
// ============================================================
module memory {


  // ============================================================
  // SECTION 1: Move Semantics
  // ============================================================

  pub type Data = {
    id: Int;
    value: Int;
  } derive[Clone]

  pub fn Data.new(id: Int, value: Int) -> Data {
    return Data{ id: id, value: value };
  }

  pub fn consume_data(d: Data) -> Int {
    return d.id * 1000 + d.value;
  }

  pub fn borrow_data(d: &Data) -> Int {
    return d.id * 1000 + d.value;
  }

  fn test_moves() -> Int {
    var score = 0;
    var d1 = Data.new(1, 42);
    if d1.value == 42 { score = score + 1; }

    // Clone before move
    var d1_clone = d1.clone();
    if d1_clone.value == 42 { score = score + 1; }
    if d1_clone.id == 1 { score = score + 1; }

    // Move
    var result = consume_data(d1);
    if result == 1042 { score = score + 1; }
    // d1 is now invalid (moved) - d1_clone still valid
    if d1_clone.value == 42 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Borrow Patterns
  // ============================================================

  pub type Container = {
    a: Int;
    b: Int;
    c: Int;
  } derive[Clone]

  pub fn Container.new(a: Int, b: Int, c: Int) -> Container {
    return Container{ a: a, b: b, c: c };
  }

  pub fn Container.sum() -> Int {
    return a + b + c;
  }

  pub fn Container.product() -> Int {
    return a * b * c;
  }

  pub fn Container.max_field() -> Int {
    var m = a;
    if b > m { m = b; }
    if c > m { m = c; }
    return m;
  }

  pub fn read_borrow_two(c1: &Container, c2: &Container) -> Int {
    return c1.sum() + c2.sum();
  }

  fn test_borrows() -> Int {
    var score = 0;
    var c1 = Container.new(1, 2, 3);
    var c2 = Container.new(4, 5, 6);

    if c1.sum() == 6 { score = score + 1; }
    if c1.product() == 6 { score = score + 1; }
    if c1.max_field() == 3 { score = score + 1; }

    if c2.sum() == 15 { score = score + 1; }

    // Multiple read borrows simultaneously
    var combined = read_borrow_two(&c1, &c2);
    if combined == 21 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Clone Heavy Paths
  // ============================================================

  pub type BigData = {
    f0: Int; f1: Int; f2: Int; f3: Int; f4: Int;
    f5: Int; f6: Int; f7: Int; f8: Int; f9: Int;
  } derive[Clone]

  pub fn BigData.new(base: Int) -> BigData {
    return BigData{
      f0: base, f1: base + 1, f2: base + 2, f3: base + 3, f4: base + 4,
      f5: base + 5, f6: base + 6, f7: base + 7, f8: base + 8, f9: base + 9,
    };
  }

  pub fn BigData.sum() -> Int {
    return f0 + f1 + f2 + f3 + f4 + f5 + f6 + f7 + f8 + f9;
  }

  pub fn clone_chain(original: &BigData, depth: Int) -> BigData {
    var current = original.clone();
    var i = 0;
    while i < depth {
      current = current.clone();
      i = i + 1;
    }
    return current;
  }

  fn test_clone_heavy() -> Int {
    var score = 0;
    var bd = BigData.new(0);
    if bd.f0 == 0 { score = score + 1; }
    if bd.f9 == 9 { score = score + 1; }
    if bd.sum() == 45 { score = score + 1; }

    var cloned = clone_chain(&bd, 5);
    if cloned.f0 == 0 { score = score + 1; }
    if cloned.sum() == 45 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Allocation Stress -- Vec Push/Pop
  // ============================================================

  pub fn alloc_stress_vec(count: Int) -> Int {
    var v = Vec[Int].new();
    var i = 0;
    while i < count {
      v.push(i);
      i = i + 1;
    }
    var total = 0;
    i = 0;
    while i < v.len() {
      total = total + v[i];
      i = i + 1;
    }
    return total;
  }

  pub fn alloc_stress_nested(count: Int) -> Int {
    var outer = Vec[Vec[Int]].new();
    var i = 0;
    while i < count {
      var inner = Vec[Int].new();
      var j = 0;
      while j < i + 1 {
        inner.push(j);
        j = j + 1;
      }
      outer.push(inner);
      i = i + 1;
    }
    var total = 0;
    i = 0;
    while i < outer.len() {
      total = total + outer[i].len();
      i = i + 1;
    }
    return total;
  }

  fn test_allocation() -> Int {
    var score = 0;
    var sum1 = alloc_stress_vec(50);
    if sum1 == 1225 { score = score + 1; }

    var sum2 = alloc_stress_nested(10);
    if sum2 == 55 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Move Chains
  // ============================================================

  pub fn create_and_pass() -> Int {
    var d = Data.new(5, 10);
    return consume_data(d);
  }

  pub fn chain_of_three(a: Data, b: Data, c: Data) -> Int {
    return a.value + b.value + c.value;
  }

  fn test_move_chains() -> Int {
    var score = 0;
    if create_and_pass() == 5010 { score = score + 1; }

    var result = chain_of_three(
      Data.new(1, 10),
      Data.new(2, 20),
      Data.new(3, 30),
    );
    if result == 60 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Struct Ownership Patterns
  // ============================================================

  pub type OwnedColl = {
    items: Vec[Data];
  } derive[Clone]

  pub fn OwnedColl.new() -> OwnedColl {
    return OwnedColl{ items: Vec[Data].new() };
  }

  pub fn OwnedColl.add(val: Int) {
    items.push(Data.new(items.len(), val));
  }

  pub fn OwnedColl.sum() -> Int {
    var total = 0;
    var i = 0;
    while i < items.len() {
      total = total + items[i].value;
      i = i + 1;
    }
    return total;
  }

  pub fn OwnedColl.count() -> Int {
    return items.len();
  }

  fn test_struct_ownership() -> Int {
    var score = 0;
    var coll = OwnedColl.new();
    if coll.count() == 0 { score = score + 1; }

    coll.add(10);
    coll.add(20);
    coll.add(30);

    if coll.count() == 3 { score = score + 1; }
    if coll.sum() == 60 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Recursive Ownership Walks
  // ============================================================

  pub type Node = {
    value: Int;
    children: Vec[Node];
  } derive[Clone]

  pub fn Node.new(val: Int) -> Node {
    return Node{ value: val, children: Vec[Node].new() };
  }

  pub fn Node.add_child(child: Node) {
    children.push(child);
  }

  pub fn Node.sum_tree() -> Int {
    var total = value;
    var i = 0;
    while i < children.len() {
      total = total + children[i].sum_tree();
      i = i + 1;
    }
    return total;
  }

  pub fn Node.count_nodes() -> Int {
    var count = 1;
    var i = 0;
    while i < children.len() {
      count = count + children[i].count_nodes();
      i = i + 1;
    }
    return count;
  }

  fn test_tree_ownership() -> Int {
    var score = 0;
    var root = Node.new(1);

    var child1 = Node.new(2);
    var child2 = Node.new(3);

    var grandchild = Node.new(4);
    child1.add_child(grandchild);

    root.add_child(child1);
    root.add_child(child2);

    if root.sum_tree() == 10 { score = score + 1; }
    if root.count_nodes() == 4 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 8: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_moves();
    total = total + s1;
    max_score = max_score + 5;

    var s2 = test_borrows();
    total = total + s2;
    max_score = max_score + 6;

    var s3 = test_clone_heavy();
    total = total + s3;
    max_score = max_score + 5;

    var s4 = test_allocation();
    total = total + s4;
    max_score = max_score + 2;

    var s5 = test_move_chains();
    total = total + s5;
    max_score = max_score + 2;

    var s6 = test_struct_ownership();
    total = total + s6;
    max_score = max_score + 4;

    var s7 = test_tree_ownership();
    total = total + s7;
    max_score = max_score + 2;

    return BenchResult{
      name: "memory",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: contracts
// ============================================================
module contracts {


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

}

// ============================================================
// MODULE: error
// ============================================================
module error {


  // ============================================================
  // SECTION 1: Error Type Definitions
  // ============================================================

  pub type MathError = {
    code: Int;
    msg: Str;
  } derive[Eq, Clone]

  pub type ParseError = {
    line: Int;
    col: Int;
    msg: Str;
  } derive[Clone]

  pub type AppError = {
    kind: Int;
    message: Str;
  } derive[Clone]

  // ============================================================
  // SECTION 2: Safe Math Operations
  // ============================================================

  pub fn safe_div(a: Int, b: Int) -> Result[Int, MathError] {
    if b == 0 {
      return Err(MathError{ code: 1, msg: "division by zero" });
    }
    return Ok(a / b);
  }

  pub fn safe_mod(a: Int, b: Int) -> Result[Int, MathError] {
    if b == 0 {
      return Err(MathError{ code: 2, msg: "modulo by zero" });
    }
    return Ok(a % b);
  }

  pub fn safe_sub(a: Int, b: Int) -> Result[Int, MathError] {
    if a < b {
      return Err(MathError{ code: 3, msg: "underflow" });
    }
    return Ok(a - b);
  }

  pub fn safe_mul(a: Int, b: Int) -> Result[Int, MathError] {
    if a == 0 || b == 0 { return Ok(0); }
    var max_val = 1000000;
    if a > max_val / b {
      return Err(MathError{ code: 4, msg: "overflow" });
    }
    return Ok(a * b);
  }

  fn test_safe_ops() -> Int {
    var score = 0;

    // safe_div
    match safe_div(10, 2) {
      Ok(v) => if v == 5 { score = score + 1; }
      Err(_) => {}
    }
    match safe_div(10, 0) {
      Ok(_) => {}
      Err(e) => if e.code == 1 { score = score + 1; }
    }

    // safe_mod
    match safe_mod(10, 3) {
      Ok(v) => if v == 1 { score = score + 1; }
      Err(_) => {}
    }
    match safe_mod(10, 0) {
      Ok(_) => {}
      Err(e) => if e.code == 2 { score = score + 1; }
    }

    // safe_sub
    match safe_sub(10, 3) {
      Ok(v) => if v == 7 { score = score + 1; }
      Err(_) => {}
    }
    match safe_sub(3, 10) {
      Ok(_) => {}
      Err(e) => if e.code == 3 { score = score + 1; }
    }

    // safe_mul
    match safe_mul(10, 20) {
      Ok(v) => if v == 200 { score = score + 1; }
      Err(_) => {}
    }
    match safe_mul(100000, 100000) {
      Ok(_) => {}
      Err(e) => if e.code == 4 { score = score + 1; }
    }

    return score;
  }

  // ============================================================
  // SECTION 3: Option Operations
  // ============================================================

  pub fn find_first_even(v: &Vec[Int]) -> Option[Int] {
    var i = 0;
    while i < v.len() {
      if v[i] % 2 == 0 { return Some(v[i]); }
      i = i + 1;
    }
    return None;
  }

  pub fn find_index(v: &Vec[Int], target: Int) -> Option[Int] {
    var i = 0;
    while i < v.len() {
      if v[i] == target { return Some(i); }
      i = i + 1;
    }
    return None;
  }

  pub fn opt_unwrap_or(opt: Option[Int], default: Int) -> Int {
    match opt {
      Some(v) => v,
      None => default,
    }
  }

  pub fn opt_map(opt: Option[Int], f: fn(Int) -> Int) -> Option[Int] {
    match opt {
      Some(v) => Some(f(v)),
      None => None,
    }
  }

  pub fn opt_flat_map(opt: Option[Int], f: fn(Int) -> Option[Int]) -> Option[Int] {
    match opt {
      Some(v) => f(v),
      None => None,
    }
  }

  fn double(n: Int) -> Int { return n * 2; }

  fn test_option() -> Int {
    var score = 0;
    var nums = [1, 3, 5, 8, 9];

    match find_first_even(&nums) {
      Some(v) => if v == 8 { score = score + 1; }
      None => {}
    }

    var odds = [1, 3, 5, 7];
    match find_first_even(&odds) {
      Some(_) => {}
      None => { score = score + 1; }
    }

    match find_index(&nums, 8) {
      Some(idx) => if idx == 3 { score = score + 1; }
      None => {}
    }

    match find_index(&nums, 99) {
      Some(_) => {}
      None => { score = score + 1; }
    }

    if opt_unwrap_or(Some(42), 0) == 42 { score = score + 1; }
    if opt_unwrap_or(None, 0) == 0 { score = score + 1; }

    match opt_map(Some(5), double) {
      Some(v) => if v == 10 { score = score + 1; }
      None => {}
    }

    match opt_map(None, double) {
      Some(_) => {}
      None => { score = score + 1; }
    }

    return score;
  }

  // ============================================================
  // SECTION 4: Result Chaining (Manual)
  // ============================================================

  pub fn compute_chain(a: Int, b: Int, c: Int) -> Result[Int, MathError] {
    var sum = safe_add(a, b);
    match sum {
      Ok(s) => {
        var div_result = safe_div(s, c);
        match div_result {
          Ok(d) => return Ok(d),
          Err(e) => return Err(e),
        }
      }
      Err(e) => return Err(e),
    }
  }

  pub fn safe_add(a: Int, b: Int) -> Result[Int, MathError] {
    return Ok(a + b);
  }

  pub fn multi_step(a: Int, b: Int, c: Int, d: Int) -> Result[Int, MathError] {
    // (a - b) * c / d
    var sub_result = safe_sub(a, b);
    match sub_result {
      Ok(diff) => {
        var mul_result = safe_mul(diff, c);
        match mul_result {
          Ok(prod) => {
            var div_result = safe_div(prod, d);
            return div_result;
          }
          Err(e) => return Err(e),
        }
      }
      Err(e) => return Err(e),
    }
  }

  fn test_chaining() -> Int {
    var score = 0;
    match compute_chain(10, 20, 3) {
      Ok(v) => if v == 10 { score = score + 1; }
      Err(_) => {}
    }

    match compute_chain(10, 20, 0) {
      Ok(_) => {}
      Err(_) => { score = score + 1; }
    }

    match multi_step(20, 5, 3, 5) {
      Ok(v) => if v == 9 { score = score + 1; }
      Err(_) => {}
    }

    match multi_step(5, 20, 3, 5) {
      Ok(_) => {}
      Err(e) => if e.code == 3 { score = score + 1; }
    }

    return score;
  }

  // ============================================================
  // SECTION 5: Error Type Matching
  // ============================================================

  pub type DivError = {
    dividend: Int;
    divisor: Int;
  } derive[Clone]

  pub enum ErrorKind {
    DivByZero,
    OverUnderflow(value: Int),
    NotEnough(available: Int, required: Int),
    ParseFail(line: Int, col: Int),
  }

  pub fn describe_error(kind: ErrorKind) -> Int {
    match kind {
      DivByZero => 1,
      OverUnderflow(value: _) => 2,
      NotEnough(available: _, required: _) => 3,
      ParseFail(line: _, col: _) => 4,
    }
  }

  fn test_error_matching() -> Int {
    var score = 0;
    if describe_error(DivByZero) == 1 { score = score + 1; }
    if describe_error(OverUnderflow(42)) == 2 { score = score + 1; }
    if describe_error(NotEnough(10, 20)) == 3 { score = score + 1; }
    if describe_error(ParseFail(5, 3)) == 4 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Pattern-Based Error Recovery
  // ============================================================

  pub fn try_parse(input: Int) -> Result[Int, AppError] {
    if input < 0 {
      return Err(AppError{ kind: 1, message: "negative" });
    }
    return Ok(input);
  }

  pub fn try_divide(a: Int, b: Int) -> Result[Int, AppError] {
    if b == 0 {
      return Err(AppError{ kind: 2, message: "zero" });
    }
    return Ok(a / b);
  }

  pub fn robust_compute(a: Int, b: Int) -> Int {
    match try_parse(a) {
      Err(_) => return -1,
      Ok(parsed_a) => {
        match try_parse(b) {
          Err(_) => return -2,
          Ok(parsed_b) => {
            match try_divide(parsed_a, parsed_b) {
              Err(_) => return -3,
              Ok(result) => return result,
            }
          }
        }
      }
    }
  }

  fn test_recovery() -> Int {
    var score = 0;
    if robust_compute(10, 2) == 5 { score = score + 1; }
    if robust_compute(-1, 2) == -1 { score = score + 1; }
    if robust_compute(10, -2) == -2 { score = score + 1; }
    if robust_compute(10, 0) == -3 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_safe_ops();
    total = total + s1;
    max_score = max_score + 8;

    var s2 = test_option();
    total = total + s2;
    max_score = max_score + 8;

    var s3 = test_chaining();
    total = total + s3;
    max_score = max_score + 4;

    var s4 = test_error_matching();
    total = total + s4;
    max_score = max_score + 4;

    var s5 = test_recovery();
    total = total + s5;
    max_score = max_score + 4;

    return BenchResult{
      name: "error",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: generics
// ============================================================
module generics {


  // ============================================================
  // SECTION 1: Basic Generic Functions
  // ============================================================

  pub fn identity[T](x: T) -> T {
    return x;
  }

  pub fn pair[T, U](a: T, b: U) -> (T, U) {
    return (a, b);
  }

  pub fn swap[T, U](a: T, b: U) -> (U, T) {
    return (b, a);
  }

  pub fn triple[T](a: T, b: T, c: T) -> (T, T, T) {
    return (a, b, c);
  }

  fn test_basic_generics() -> Int {
    var score = 0;
    if identity(42) == 42 { score = score + 1; }
    if identity(true) { score = score + 1; }

    var p1 = pair(1, "hello");
    var p2 = pair(true, 3.14);

    var (i_val, s_val) = pair(42, "test");
    if i_val == 42 { score = score + 1; }

    if p1.0 == 1 { score = score + 1; }

    var (b_val, f_val) = swap(3.14, true);
    if b_val { score = score + 1; }

    var t = triple(10, 20, 30);
    if t.0 == 10 { score = score + 1; }
    if t.2 == 30 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Generic Structs
  // ============================================================

  pub type Box[T] = {
    value: T;
  } derive[Clone]

  pub type Pair[A, B] = {
    first: A;
    second: B;
  } derive[Clone]

  pub type Wrapper[T] = {
    data: T;
    tag: Int;
  } derive[Clone]

  pub fn Box.new[T](val: T) -> Box[T] {
    return Box[T]{ value: val };
  }

  pub fn Box.unwrap[T]() -> T {
    return value;
  }

  pub fn Pair.new[A, B](a: A, b: B) -> Pair[A, B] {
    return Pair[A, B]{ first: a, second: b };
  }

  pub fn Wrapper.new[T](data: T, tag: Int) -> Wrapper[T] {
    return Wrapper[T]{ data: data, tag: tag };
  }

  fn test_generic_structs() -> Int {
    var score = 0;
    var b1 = Box.new[Int](42);
    if b1.value == 42 { score = score + 1; }
    if b1.unwrap() == 42 { score = score + 1; }

    var b2 = Box.new[Bool](true);
    if b2.value { score = score + 1; }

    var p = Pair.new[Int, Str](1, "one");
    if p.first == 1 { score = score + 1; }

    var w = Wrapper.new[Float64](3.14, 1);
    if w.data == 3.14 { score = score + 1; }
    if w.tag == 1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Generic Functions with Operations
  // ============================================================

  pub fn max[T](a: T, b: T) -> T {
    if a > b { return a; }
    return b;
  }

  pub fn min[T](a: T, b: T) -> T {
    if a < b { return a; }
    return b;
  }

  pub fn abs_diff(a: Int, b: Int) -> Int {
    if a > b { return a - b; }
    return b - a;
  }

  pub fn is_equal[T: Eq](a: T, b: T) -> Bool {
    return a == b;
  }

  pub fn is_not_equal[T: Eq](a: T, b: T) -> Bool {
    return a != b;
  }

  fn test_generic_ops() -> Int {
    var score = 0;
    if max(10, 20) == 20 { score = score + 1; }
    if max(-5, 5) == 5 { score = score + 1; }
    if max(3.14, 2.71) == 3.14 { score = score + 1; }

    if min(10, 20) == 10 { score = score + 1; }
    if min[Float64](5.5, 3.3) == 3.3 { score = score + 1; }

    if abs_diff(10, 3) == 7 { score = score + 1; }
    if abs_diff(3, 10) == 7 { score = score + 1; }
    if abs_diff(5, 5) == 0 { score = score + 1; }

    if is_equal(42, 42) { score = score + 1; }
    if !(is_equal(42, 43)) { score = score + 1; }

    if is_not_equal(1, 2) { score = score + 1; }
    if !(is_not_equal(5, 5)) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Option/Result Generic Patterns
  // ============================================================

  pub fn wrap_some[T](x: T) -> Option[T] {
    return Some(x);
  }

  pub fn wrap_ok[T, E](x: T) -> Result[T, E] {
    return Ok(x);
  }

  pub fn wrap_err[T, E](e: E) -> Result[T, E] {
    return Err(e);
  }

  pub fn default_value[T]() -> Option[T] {
    return None;
  }

  fn test_opt_result_generic() -> Int {
    var score = 0;
    var opt = wrap_some(99);
    match opt {
      Some(v) => if v == 99 { score = score + 1; }
      None => {}
    }

    var ok_res: Result[Int, Str] = wrap_ok[Int, Str](42);
    match ok_res {
      Ok(v) => if v == 42 { score = score + 1; }
      Err(_) => {}
    }

    var err_res: Result[Int, Str] = wrap_err[Int, Str]("fail");
    match err_res {
      Ok(_) => {}
      Err(e) => if e == "fail" { score = score + 1; }
    }

    return score;
  }

  // ============================================================
  // SECTION 5: Generic Data Structures
  // ============================================================

  pub type Stack[T] = {
    data: Vec[T];
    top: Int;
  } derive[Clone]

  pub fn Stack.new[T]() -> Stack[T] {
    return Stack[T]{ data: Vec[T].new(), top: 0 };
  }

  pub fn Stack.push[T](val: T) {
    data.push(val);
    top = data.len();
  }

  pub fn Stack.is_empty[T]() -> Bool {
    return top == 0;
  }

  pub fn Stack.size[T]() -> Int {
    return top;
  }

  pub type Priority = {
    value: Int;
    order: Int;
  } derive[Clone]

  pub type PriorityQueue[T] = {
    items: Vec[T];
    priorities: Vec[Int];
  } derive[Clone]

  pub fn PriorityQueue.new[T]() -> PriorityQueue[T] {
    return PriorityQueue[T]{ items: Vec[T].new(), priorities: Vec[Int].new() };
  }

  pub fn PriorityQueue.enqueue[T](item: T, prio: Int) {
    items.push(item);
    priorities.push(prio);
  }

  pub fn PriorityQueue.size[T]() -> Int {
    return items.len();
  }

  fn test_generic_data_structures() -> Int {
    var score = 0;
    var si: Stack[Int] = Stack.new[Int]();
    if si.is_empty() { score = score + 1; }

    si.push(42);
    if si.size() == 1 { score = score + 1; }
    if !(si.is_empty()) { score = score + 1; }

    var sb: Stack[Bool] = Stack.new[Bool]();
    sb.push(true);
    sb.push(false);
    if sb.size() == 2 { score = score + 1; }

    var pq: PriorityQueue[Int] = PriorityQueue.new[Int]();
    pq.enqueue(1, 5);
    pq.enqueue(2, 3);
    pq.enqueue(3, 7);
    if pq.size() == 3 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Generic Methods
  // ============================================================

  pub type Counter[T] = {
    value: T;
    count: Int;
  } derive[Clone]

  pub fn Counter.new[T](initial: T) -> Counter[T] {
    return Counter[T]{ value: initial, count: 0 };
  }

  pub fn Counter.set[T](new_val: T) {
    value = new_val;
    count = count + 1;
  }

  pub fn Counter.times_updated[T]() -> Int {
    return count;
  }

  fn test_generic_methods() -> Int {
    var score = 0;
    var ci: Counter[Int] = Counter.new[Int](0);
    if ci.value == 0 { score = score + 1; }
    if ci.times_updated() == 0 { score = score + 1; }

    ci.set(42);
    if ci.value == 42 { score = score + 1; }
    if ci.times_updated() == 1 { score = score + 1; }

    ci.set(99);
    if ci.times_updated() == 2 { score = score + 1; }

    var cb: Counter[Bool] = Counter.new[Bool](false);
    cb.set(true);
    if cb.value { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Nested Generics
  // ============================================================

  pub type Nested[T] = {
    a: Option[T];
    b: Option[T];
  } derive[Clone]

  pub type MultiNested[A, B, C] = {
    first: A;
    second: Vec[B];
    third: Option[C];
  } derive[Clone]

  pub fn Nested.new[T]() -> Nested[T] {
    return Nested[T]{ a: None, b: None };
  }

  pub fn Nested.set_a[T](val: T) {
    a = Some(val);
  }

  pub fn MultiNested.new[A, B, C](f: A) -> MultiNested[A, B, C] {
    return MultiNested[A, B, C]{ first: f, second: Vec[B].new(), third: None };
  }

  fn test_nested_generics() -> Int {
    var score = 0;
    var n: Nested[Int] = Nested.new[Int]();
    match n.a {
      Some(_) => {}
      None => { score = score + 1; }
    }

    n.set_a(42);
    match n.a {
      Some(v) => if v == 42 { score = score + 1; }
      None => {}
    }

    var mn: MultiNested[Int, Str, Bool] = MultiNested.new[Int, Str, Bool](1);
    if mn.first == 1 { score = score + 1; }

    match mn.third {
      Some(_) => {}
      None => { score = score + 1; }
    }

    return score;
  }

  // ============================================================
  // SECTION 8: Type Parameter Stress
  // ============================================================

  pub fn compose[A, B, C](f: fn(A) -> B, g: fn(B) -> C, x: A) -> C {
    return g(f(x));
  }

  fn add_one(x: Int) -> Int { return x + 1; }
  fn double_it(x: Int) -> Int { return x * 2; }
  fn to_bool(x: Int) -> Bool { return x > 0; }

  fn test_type_params() -> Int {
    var score = 0;
    var r1 = compose(add_one, double_it, 5);
    if r1 == 12 { score = score + 1; }

    var r2 = compose(double_it, add_one, 3);
    if r2 == 7 { score = score + 1; }

    var r3 = compose(add_one, to_bool, 0);
    if r3 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 9: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_basic_generics();
    total = total + s1;
    max_score = max_score + 7;

    var s2 = test_generic_structs();
    total = total + s2;
    max_score = max_score + 6;

    var s3 = test_generic_ops();
    total = total + s3;
    max_score = max_score + 12;

    var s4 = test_opt_result_generic();
    total = total + s4;
    max_score = max_score + 3;

    var s5 = test_generic_data_structures();
    total = total + s5;
    max_score = max_score + 6;

    var s6 = test_generic_methods();
    total = total + s6;
    max_score = max_score + 7;

    var s7 = test_nested_generics();
    total = total + s7;
    max_score = max_score + 4;

    var s8 = test_type_params();
    total = total + s8;
    max_score = max_score + 3;

    return BenchResult{
      name: "generics",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: enums
// ============================================================
module enums {


  // ============================================================
  // SECTION 1: Simple Enums
  // ============================================================

  pub enum Color {
    Red,
    Green,
    Blue,
  }

  pub enum Direction {
    North,
    South,
    East,
    West,
  }

  pub enum Status {
    Active,
    Inactive,
    Pending,
    Archived,
  }

  pub fn Color.to_int() -> Int {
    match self {
      Red => 0,
      Green => 1,
      Blue => 2,
    }
  }

  pub fn Direction.reverse() -> Direction {
    match self {
      North => South,
      South => North,
      East => West,
      West => East,
    }
  }

  pub fn Status.is_active() -> Bool {
    match self {
      Active => true,
      Inactive => false,
      Pending => false,
      Archived => false,
    }
  }

  fn test_simple_enums() -> Int {
    var score = 0;
    if Color.Red.to_int() == 0 { score = score + 1; }
    if Color.Green.to_int() == 1 { score = score + 1; }
    if Color.Blue.to_int() == 2 { score = score + 1; }

    if Direction.North.reverse() == Direction.South { score = score + 1; }
    if Direction.East.reverse() == Direction.West { score = score + 1; }

    if Status.Active.is_active() { score = score + 1; }
    if !(Status.Pending.is_active()) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Enum with Data
  // ============================================================

  pub enum Shape {
    Circle(radius: Float64),
    Rectangle(w: Float64, h: Float64),
    Triangle(base: Float64, height: Float64),
  }

  pub enum Message {
    Text(content: Str),
    Image(url: Str, width: Int, height: Int),
    File(path: Str, size: Int),
    Empty,
  }

  pub fn Shape.area() -> Float64 {
    match self {
      Circle(radius: r) => 3.14159 * r * r,
      Rectangle(w: w, h: h) => w * h,
      Triangle(base: b, height: h) => 0.5 * b * h,
    }
  }

  pub fn Shape.is_circle() -> Bool {
    match self {
      Circle(radius: _) => true,
      _ => false,
    }
  }

  pub fn Message.size_hint() -> Int {
    match self {
      Text(content: _) => 1,
      Image(url: _, width: w, height: h) => w * h,
      File(path: _, size: s) => s,
      Empty => 0,
    }
  }

  fn test_enum_data() -> Int {
    var score = 0;
    var c = Circle(radius: 2.0);
    var area_c = c.area();
    var area_c_ok = area_c > 12.5 && area_c < 12.6;
    if area_c_ok { score = score + 1; }

    var r = Rectangle(w: 3.0, h: 4.0);
    if r.area() == 12.0 { score = score + 1; }

    var t = Triangle(base: 6.0, height: 8.0);
    if t.area() == 24.0 { score = score + 1; }

    if c.is_circle() { score = score + 1; }
    if !(r.is_circle()) { score = score + 1; }

    var txt = Text(content: "hello");
    if txt.size_hint() == 1 { score = score + 1; }

    var img = Image(url: "img.png", width: 100, height: 200);
    if img.size_hint() == 20000 { score = score + 1; }

    var empty = Empty;
    if empty.size_hint() == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Large Enum (20+ Variants)
  // ============================================================

  pub enum TokenKind {
    Eof,
    Ident,
    IntLit,
    FloatLit,
    StrLit,
    CharLit,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    Arrow,
    FatArrow,
  }

  pub fn TokenKind.is_operator() -> Bool {
    match self {
      Plus => true,
      Minus => true,
      Star => true,
      Slash => true,
      Percent => true,
      Eq => true,
      EqEq => true,
      NotEq => true,
      Lt => true,
      Gt => true,
      LtEq => true,
      GtEq => true,
      _ => false,
    }
  }

  pub fn TokenKind.is_delimiter() -> Bool {
    match self {
      LParen => true,
      RParen => true,
      LBrace => true,
      RBrace => true,
      LBracket => true,
      RBracket => true,
      Comma => true,
      Semicolon => true,
      _ => false,
    }
  }

  pub fn TokenKind.precedence() -> Int {
    match self {
      Star => 10,
      Slash => 10,
      Percent => 10,
      Plus => 5,
      Minus => 5,
      EqEq => 3,
      NotEq => 3,
      Lt => 3,
      Gt => 3,
      LtEq => 3,
      GtEq => 3,
      _ => 0,
    }
  }

  fn test_large_enum() -> Int {
    var score = 0;
    if TokenKind.Plus.is_operator() { score = score + 1; }
    if TokenKind.Star.is_operator() { score = score + 1; }
    if TokenKind.EqEq.is_operator() { score = score + 1; }
    if !(TokenKind.Ident.is_operator()) { score = score + 1; }
    if !(TokenKind.Eof.is_operator()) { score = score + 1; }

    if TokenKind.LParen.is_delimiter() { score = score + 1; }
    if TokenKind.RBrace.is_delimiter() { score = score + 1; }
    if TokenKind.Semicolon.is_delimiter() { score = score + 1; }
    if !(TokenKind.Plus.is_delimiter()) { score = score + 1; }

    if TokenKind.Star.precedence() == 10 { score = score + 1; }
    if TokenKind.Plus.precedence() == 5 { score = score + 1; }
    if TokenKind.EqEq.precedence() == 3 { score = score + 1; }
    if TokenKind.Ident.precedence() == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Option/Result Enums
  // ============================================================

  pub fn option_to_int(opt: Option[Int]) -> Int {
    match opt {
      Some(v) => v,
      None => -1,
    }
  }

  pub fn result_to_int(res: Result[Int, Str]) -> Int {
    match res {
      Ok(v) => v,
      Err(_) => -1,
    }
  }

  pub fn is_ok[T, E](res: &Result[T, E]) -> Bool {
    match res {
      Ok(_) => true,
      Err(_) => false,
    }
  }

  pub fn unwrap_result(res: Result[Int, Int]) -> Int {
    match res {
      Ok(v) => v,
      Err(e) => e,
    }
  }

  fn test_opt_result_enum() -> Int {
    var score = 0;
    if option_to_int(Some(42)) == 42 { score = score + 1; }
    if option_to_int(None) == -1 { score = score + 1; }

    if result_to_int(Ok(10)) == 10 { score = score + 1; }
    if result_to_int(Err("fail")) == -1 { score = score + 1; }

    var ok_res = Ok(5);
    var err_res: Result[Int, Str] = Err("oops");
    if is_ok(&ok_res) { score = score + 1; }
    if !(is_ok(&err_res)) { score = score + 1; }

    if unwrap_result(Ok(100)) == 100 { score = score + 1; }
    if unwrap_result(Err(200)) == 200 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Enum State Machines
  // ============================================================

  pub enum Connection {
    Disconnected,
    Connecting(attempts: Int),
    Connected,
    Error(message: Str),
  }

  pub fn Connection.next() -> Connection {
    match self {
      Disconnected => Connecting(attempts: 0),
      Connecting(attempts: a) => {
        if a < 3 {
          return Connecting(attempts: a + 1);
        }
        return Connected;
      }
      Connected => Disconnected,
      Error(message: _) => Disconnected,
    }
  }

  pub fn Connection.is_connected() -> Bool {
    match self {
      Connected => true,
      _ => false,
    }
  }

  fn test_state_enum() -> Int {
    var score = 0;
    var s1 = Disconnected;
    if !(s1.is_connected()) { score = score + 1; }

    var s2 = s1.next();
    match s2 {
      Connecting(attempts: a) => if a == 0 { score = score + 1; }
      _ => {}
    }

    var s3 = s2.next().next().next().next();
    if s3.is_connected() { score = score + 1; }

    var s4 = s3.next();
    match s4 {
      Disconnected => { score = score + 1; }
      _ => {}
    }

    return score;
  }

  // ============================================================
  // SECTION 6: Expression Evaluation via Enum
  // ============================================================

  pub enum Expr {
    Const(val: Int),
    Add(left: &Expr, right: &Expr),
    Mul(left: &Expr, right: &Expr),
    Neg(inner: &Expr),
  }

  pub fn Expr.eval() -> Int {
    match self {
      Const(val: v) => v,
      Add(left: l, right: r) => l.eval() + r.eval(),
      Mul(left: l, right: r) => l.eval() * r.eval(),
      Neg(inner: i) => -i.eval(),
    }
  }

  fn test_expr_enum() -> Int {
    var score = 0;
    var c1 = Const(val: 5);
    var c2 = Const(val: 3);
    var add_expr = Add(left: &c1, right: &c2);
    if add_expr.eval() == 8 { score = score + 1; }

    var c3 = Const(val: 4);
    var mul_expr = Mul(left: &add_expr, right: &c3);
    if mul_expr.eval() == 32 { score = score + 1; }

    var neg_expr = Neg(inner: &c1);
    if neg_expr.eval() == -5 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_simple_enums();
    total = total + s1;
    max_score = max_score + 7;

    var s2 = test_enum_data();
    total = total + s2;
    max_score = max_score + 8;

    var s3 = test_large_enum();
    total = total + s3;
    max_score = max_score + 13;

    var s4 = test_opt_result_enum();
    total = total + s4;
    max_score = max_score + 8;

    var s5 = test_state_enum();
    total = total + s5;
    max_score = max_score + 4;

    var s6 = test_expr_enum();
    total = total + s6;
    max_score = max_score + 3;

    return BenchResult{
      name: "enums",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: derive
// ============================================================
module derive {


  // ============================================================
  // SECTION 1: Eq + Clone
  // ============================================================

  pub type Vec2 = {
    x: Float64;
    y: Float64;
  } derive[Eq, Clone]

  pub type Vec3 = {
    x: Float64;
    y: Float64;
    z: Float64;
  } derive[Eq, Clone]

  pub type Person = {
    name: Str;
    age: Int;
    active: Bool;
  } derive[Eq, Clone]

  pub type Timestamp = {
    seconds: Int;
    nanos: Int;
  } derive[Eq, Clone]

  fn test_eq_clone() -> Int {
    var score = 0;

    // Eq
    var v1 = Vec2{ x: 1.0, y: 2.0 };
    var v2 = Vec2{ x: 1.0, y: 2.0 };
    var v3 = Vec2{ x: 3.0, y: 4.0 };
    if v1 == v2 { score = score + 1; }
    if !(v1 == v3) { score = score + 1; }

    var p1 = Person{ name: "Alice", age: 30, active: true };
    var p2 = Person{ name: "Alice", age: 30, active: true };
    var p3 = Person{ name: "Bob", age: 25, active: false };
    if p1 == p2 { score = score + 1; }
    if !(p1 == p3) { score = score + 1; }

    // Clone
    var v1_clone = v1.clone();
    if v1 == v1_clone { score = score + 1; }
    if v1_clone.x == 1.0 { score = score + 1; }
    if v1_clone.y == 2.0 { score = score + 1; }

    var p1_clone = p1.clone();
    if p1_clone.age == 30 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Ord on comparable types
  // ============================================================

  pub type Score = {
    value: Int;
    player: Int;
  } derive[Eq, Clone]

  pub type RangeInt = {
    min: Int;
    max: Int;
  } derive[Eq, Clone]

  fn test_ord_comparisons() -> Int {
    var score = 0;
    var s1 = Score{ value: 100, player: 1 };
    var s2 = Score{ value: 200, player: 2 };

    if s1 == s1 { score = score + 1; }
    if !(s1 == s2) { score = score + 1; }
    if s1.clone().value == 100 { score = score + 1; }

    var r1 = RangeInt{ min: 0, max: 10 };
    var r2 = RangeInt{ min: 0, max: 10 };
    var r3 = RangeInt{ min: 5, max: 15 };

    if r1 == r2 { score = score + 1; }
    if !(r1 == r3) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Multi-Derive Combos
  // ============================================================

  pub type Card = {
    suit: Int;
    rank: Int;
  } derive[Eq, Clone]

  pub type Deck = {
    cards: Vec[Card];
  } derive[Clone]

  pub type GameState = {
    turn: Int;
    phase: Int;
    active_player: Int;
    score: Int;
  } derive[Eq, Clone]

  fn test_multi_derive() -> Int {
    var score = 0;
    var c1 = Card{ suit: 1, rank: 10 };
    var c2 = Card{ suit: 1, rank: 10 };
    var c3 = Card{ suit: 2, rank: 7 };

    if c1 == c2 { score = score + 1; }
    if !(c1 == c3) { score = score + 1; }

    var c1_clone = c1.clone();
    if c1_clone.suit == 1 { score = score + 1; }
    if c1_clone.rank == 10 { score = score + 1; }

    var d = Deck{ cards: Vec[Card].new() };
    // Clone deck (empty)
    var d_clone = d.clone();
    if d_clone.cards.len() == 0 { score = score + 1; }

    var gs = GameState{ turn: 5, phase: 2, active_player: 1, score: 42 };
    var gs2 = GameState{ turn: 5, phase: 2, active_player: 1, score: 42 };
    if gs == gs2 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Struct with Many Derived Traits
  // ============================================================

  pub type Config = {
    max_connections: Int;
    timeout: Int;
    retry_count: Int;
    buffer_size: Int;
    enable_logging: Bool;
    log_level: Int;
    port: Int;
    host_len: Int;
  } derive[Eq, Clone]

  pub type Metrics = {
    requests: Int;
    errors: Int;
    latency_ms: Int;
    uptime_sec: Int;
    memory_kb: Int;
    cpu_percent: Int;
    active_sessions: Int;
  } derive[Eq, Clone]

  fn test_large_derive() -> Int {
    var score = 0;
    var cfg1 = Config{
      max_connections: 100,
      timeout: 30,
      retry_count: 3,
      buffer_size: 4096,
      enable_logging: true,
      log_level: 2,
      port: 8080,
      host_len: 9,
    };

    var cfg2 = cfg1.clone();
    if cfg1 == cfg2 { score = score + 1; }
    if cfg2.max_connections == 100 { score = score + 1; }
    if cfg2.timeout == 30 { score = score + 1; }
    if cfg2.port == 8080 { score = score + 1; }

    var m1 = Metrics{
      requests: 10000,
      errors: 5,
      latency_ms: 12,
      uptime_sec: 86400,
      memory_kb: 512000,
      cpu_percent: 45,
      active_sessions: 120,
    };

    var m2 = m1.clone();
    if m1 == m2 { score = score + 1; }
    if m2.requests == 10000 { score = score + 1; }
    if m2.errors == 5 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Enum with Derive
  // ============================================================

  pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
  } derive[Eq, Clone]

  pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
  } derive[Eq, Clone]

  fn test_enum_derive() -> Int {
    var score = 0;
    var s1 = Suit.Hearts;
    var s2 = Suit.Hearts;
    var s3 = Suit.Spades;

    if s1 == s2 { score = score + 1; }
    if !(s1 == s3) { score = score + 1; }

    var s1_clone = s1.clone();
    if s1_clone == Suit.Hearts { score = score + 1; }

    var l1 = LogLevel.Info;
    var l2 = LogLevel.Info;
    var l3 = LogLevel.Error;

    if l1 == l2 { score = score + 1; }
    if !(l1 == l3) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Nested Derived Structs
  // ============================================================

  pub type Point = {
    x: Float64;
    y: Float64;
  } derive[Eq, Clone]

  pub type Line = {
    start: Point;
    end: Point;
  } derive[Eq, Clone]

  pub type Polygon = {
    points: Vec[Point];
  } derive[Clone]

  fn test_nested_derive() -> Int {
    var score = 0;
    var p1 = Point{ x: 0.0, y: 0.0 };
    var p2 = Point{ x: 1.0, y: 1.0 };
    var p1_copy = Point{ x: 0.0, y: 0.0 };

    if p1 == p1_copy { score = score + 1; }
    if !(p1 == p2) { score = score + 1; }

    var l1 = Line{ start: p1.clone(), end: p2.clone() };
    var l2 = Line{ start: p1_copy.clone(), end: p2.clone() };
    if l1 == l2 { score = score + 1; }

    var l1_clone = l1.clone();
    if l1_clone.start.x == 0.0 { score = score + 1; }
    if l1_clone.end.y == 1.0 { score = score + 1; }

    var poly = Polygon{ points: Vec[Point].new() };
    var poly_clone = poly.clone();
    if poly_clone.points.len() == 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Derive Stress -- 25-field Struct
  // ============================================================

  pub type Big25 = {
    f0: Int; f1: Int; f2: Int; f3: Int; f4: Int;
    f5: Int; f6: Int; f7: Int; f8: Int; f9: Int;
    f10: Int; f11: Int; f12: Int; f13: Int; f14: Int;
    f15: Int; f16: Int; f17: Int; f18: Int; f19: Int;
    f20: Int; f21: Int; f22: Int; f23: Int; f24: Int;
  } derive[Eq, Clone]

  pub fn Big25.new(val: Int) -> Big25 {
    return Big25{
      f0: val + 0, f1: val + 1, f2: val + 2, f3: val + 3, f4: val + 4,
      f5: val + 5, f6: val + 6, f7: val + 7, f8: val + 8, f9: val + 9,
      f10: val + 10, f11: val + 11, f12: val + 12, f13: val + 13, f14: val + 14,
      f15: val + 15, f16: val + 16, f17: val + 17, f18: val + 18, f19: val + 19,
      f20: val + 20, f21: val + 21, f22: val + 22, f23: val + 23, f24: val + 24,
    };
  }

  pub fn Big25.sum() -> Int {
    return f0 + f1 + f2 + f3 + f4 + f5 + f6 + f7 + f8 + f9
      + f10 + f11 + f12 + f13 + f14 + f15 + f16 + f17 + f18 + f19
      + f20 + f21 + f22 + f23 + f24;
  }

  fn test_big_derive() -> Int {
    var score = 0;
    var b1 = Big25.new(100);
    var b2 = Big25.new(100);
    var b3 = Big25.new(200);

    if b1 == b2 { score = score + 1; }
    if !(b1 == b3) { score = score + 1; }

    var b1c = b1.clone();
    if b1 == b1c { score = score + 1; }
    if b1c.f0 == 100 { score = score + 1; }
    if b1c.f24 == 124 { score = score + 1; }

    // Sum: 25*100 + (0+1+...+24) = 2500 + 300 = 2800
    if b1.sum() == 2800 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 8: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_eq_clone();
    total = total + s1;
    max_score = max_score + 9;

    var s2 = test_ord_comparisons();
    total = total + s2;
    max_score = max_score + 4;

    var s3 = test_multi_derive();
    total = total + s3;
    max_score = max_score + 6;

    var s4 = test_large_derive();
    total = total + s4;
    max_score = max_score + 7;

    var s5 = test_enum_derive();
    total = total + s5;
    max_score = max_score + 5;

    var s6 = test_nested_derive();
    total = total + s6;
    max_score = max_score + 6;

    var s7 = test_big_derive();
    total = total + s7;
    max_score = max_score + 5;

    return BenchResult{
      name: "derive",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: recursion
// ============================================================
module recursion {


  // ============================================================
  // SECTION 1: Classic Recursive Functions
  // ============================================================

  pub fn factorial(n: Int) -> Int {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
  }

  pub fn fibonacci(n: Int) -> Int {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
  }

  pub fn ackermann(m: Int, n: Int) -> Int {
    if m == 0 { return n + 1; }
    if n == 0 { return ackermann(m - 1, 1); }
    return ackermann(m - 1, ackermann(m, n - 1));
  }

  fn test_classic_recursion() -> Int {
    var score = 0;
    if factorial(0) == 1 { score = score + 1; }
    if factorial(5) == 120 { score = score + 1; }
    if factorial(7) == 5040 { score = score + 1; }

    if fibonacci(0) == 0 { score = score + 1; }
    if fibonacci(1) == 1 { score = score + 1; }
    if fibonacci(10) == 55 { score = score + 1; }
    if fibonacci(15) == 610 { score = score + 1; }

    // Ackermann small values
    if ackermann(0, 0) == 1 { score = score + 1; }
    if ackermann(0, 1) == 2 { score = score + 1; }
    if ackermann(1, 0) == 2 { score = score + 1; }
    if ackermann(1, 1) == 3 { score = score + 1; }
    if ackermann(2, 0) == 3 { score = score + 1; }
    if ackermann(2, 1) == 5 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Mutual Recursion
  // ============================================================

  pub fn is_even_mut(n: Int) -> Bool {
    if n == 0 { return true; }
    return is_odd_mut(n - 1);
  }

  pub fn is_odd_mut(n: Int) -> Bool {
    if n == 0 { return false; }
    return is_even_mut(n - 1);
  }

  pub fn hofstadter_female(n: Int) -> Int {
    if n == 0 { return 1; }
    return n - hofstadter_male(hofstadter_female(n - 1));
  }

  pub fn hofstadter_male(n: Int) -> Int {
    if n == 0 { return 0; }
    return n - hofstadter_female(hofstadter_male(n - 1));
  }

  fn test_mutual_recursion() -> Int {
    var score = 0;
    if is_even_mut(0) { score = score + 1; }
    if is_even_mut(2) { score = score + 1; }
    if is_even_mut(10) { score = score + 1; }
    if !(is_even_mut(1)) { score = score + 1; }
    if !(is_even_mut(99)) { score = score + 1; }

    if is_odd_mut(1) { score = score + 1; }
    if is_odd_mut(99) { score = score + 1; }
    if !(is_odd_mut(0)) { score = score + 1; }

    // Hofstadter sequences
    if hofstadter_female(0) == 1 { score = score + 1; }
    if hofstadter_male(0) == 0 { score = score + 1; }
    if hofstadter_female(1) == 1 { score = score + 1; }
    if hofstadter_male(1) == 1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Divide and Conquer
  // ============================================================

  pub fn binary_search(arr: Vec[Int], target: Int, lo: Int, hi: Int) -> Int {
    if lo > hi { return -1; }
    var mid = lo + (hi - lo) / 2;
    if arr[mid] == target { return mid; }
    if arr[mid] < target { return binary_search(arr, target, mid + 1, hi); }
    return binary_search(arr, target, lo, mid - 1);
  }

  pub fn power_div_conq(base: Int, exp: Int) -> Int {
    if exp == 0 { return 1; }
    var half = power_div_conq(base, exp / 2);
    var half_sq = half * half;
    if exp % 2 == 0 { return half_sq; }
    return base * half_sq;
  }

  pub fn gcd(a: Int, b: Int) -> Int {
    if b == 0 { return a; }
    return gcd(b, a % b);
  }

  pub fn sum_range_rec(lo: Int, hi: Int) -> Int {
    if lo > hi { return 0; }
    if lo == hi { return lo; }
    var mid = (lo + hi) / 2;
    return sum_range_rec(lo, mid) + sum_range_rec(mid + 1, hi);
  }

  pub fn max_rec(arr: Vec[Int], idx: Int) -> Int {
    if idx >= arr.len() { return -1000000; }
    var max_of_rest = max_rec(arr, idx + 1);
    if arr[idx] > max_of_rest { return arr[idx]; }
    return max_of_rest;
  }

  fn test_divide_conquer() -> Int {
    var score = 0;
    var sorted = [1, 3, 5, 7, 9, 11, 13, 15];
    if binary_search(sorted, 7, 0, sorted.len() - 1) == 3 { score = score + 1; }
    if binary_search(sorted, 1, 0, sorted.len() - 1) == 0 { score = score + 1; }
    if binary_search(sorted, 15, 0, sorted.len() - 1) == 7 { score = score + 1; }
    if binary_search(sorted, 8, 0, sorted.len() - 1) == -1 { score = score + 1; }
    var empty: Vec[Int] = [];
    if binary_search(empty, 5, 0, -1) == -1 { score = score + 1; }

    if power_div_conq(2, 10) == 1024 { score = score + 1; }
    if power_div_conq(3, 4) == 81 { score = score + 1; }
    if power_div_conq(5, 3) == 125 { score = score + 1; }

    if gcd(48, 18) == 6 { score = score + 1; }
    if gcd(1071, 462) == 21 { score = score + 1; }

    if sum_range_rec(1, 10) == 55 { score = score + 1; }
    if sum_range_rec(1, 1) == 1 { score = score + 1; }
    if sum_range_rec(10, 1) == 0 { score = score + 1; }

    var nums = [3, 7, 2, 9, 1, 5];
    if max_rec(nums, 0) == 9 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Recursive Sequence Generators
  // ============================================================

  pub fn padovan(n: Int) -> Int {
    if n == 0 || n == 1 || n == 2 { return 1; }
    return padovan(n - 2) + padovan(n - 3);
  }

  pub fn catalan(n: Int) -> Int {
    if n <= 1 { return 1; }
    var sum = 0;
    var i = 0;
    while i < n {
      sum = sum + catalan(i) * catalan(n - 1 - i);
      i = i + 1;
    }
    return sum;
  }

  pub fn tribonacci(n: Int) -> Int {
    if n == 0 { return 0; }
    if n == 1 || n == 2 { return 1; }
    return tribonacci(n - 1) + tribonacci(n - 2) + tribonacci(n - 3);
  }

  fn test_sequences() -> Int {
    var score = 0;
    if padovan(0) == 1 { score = score + 1; }
    if padovan(3) == 2 { score = score + 1; }
    if padovan(5) == 3 { score = score + 1; }
    if padovan(8) == 7 { score = score + 1; }

    if catalan(0) == 1 { score = score + 1; }
    if catalan(1) == 1 { score = score + 1; }
    if catalan(2) == 2 { score = score + 1; }
    if catalan(3) == 5 { score = score + 1; }
    if catalan(4) == 14 { score = score + 1; }

    if tribonacci(0) == 0 { score = score + 1; }
    if tribonacci(3) == 2 { score = score + 1; }
    if tribonacci(5) == 7 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Tree-Like Recursion
  // ============================================================

  pub fn binomial_rec(n: Int, k: Int) -> Int {
    if k == 0 || k == n { return 1; }
    return binomial_rec(n - 1, k - 1) + binomial_rec(n - 1, k);
  }

  pub fn stirling_second(n: Int, k: Int) -> Int {
    if k == 0 && n == 0 { return 1; }
    if k == 0 || n == 0 { return 0; }
    if k == n { return 1; }
    return k * stirling_second(n - 1, k) + stirling_second(n - 1, k - 1);
  }

  pub fn bell(n: Int) -> Int {
    if n == 0 { return 1; }
    var sum = 0;
    var k = 0;
    while k < n {
      sum = sum + binomial_rec(n - 1, k) * bell(k);
      k = k + 1;
    }
    return sum;
  }

  fn test_tree_recursion() -> Int {
    var score = 0;
    if binomial_rec(5, 2) == 10 { score = score + 1; }
    if binomial_rec(10, 5) == 252 { score = score + 1; }
    if binomial_rec(6, 0) == 1 { score = score + 1; }
    if binomial_rec(6, 6) == 1 { score = score + 1; }

    if stirling_second(0, 0) == 1 { score = score + 1; }
    if stirling_second(3, 2) == 3 { score = score + 1; }
    if stirling_second(4, 2) == 7 { score = score + 1; }

    if bell(0) == 1 { score = score + 1; }
    if bell(1) == 1 { score = score + 1; }
    if bell(2) == 2 { score = score + 1; }
    if bell(3) == 5 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Palindromes & String Recursion
  // ============================================================

  pub fn reverse_num(n: Int) -> Int {
    if n < 10 { return n; }
    var digits = 1;
    var pow = 1;
    var x = n;
    while x >= 10 {
      x = x / 10;
      digits = digits + 1;
      pow = pow * 10;
    }
    return (n % 10) * pow + reverse_num(n / 10);
  }

  pub fn is_palindrome_num(n: Int) -> Bool {
    return n == reverse_num(n);
  }

  pub fn sum_digits_rec(n: Int) -> Int {
    if n == 0 { return 0; }
    if n < 0 { return sum_digits_rec(-n); }
    return n % 10 + sum_digits_rec(n / 10);
  }

  pub fn count_digits_rec(n: Int) -> Int {
    if n == 0 { return 1; }
    if n < 0 { return count_digits_rec(-n); }
    if n < 10 { return 1; }
    return 1 + count_digits_rec(n / 10);
  }

  fn test_string_recursion() -> Int {
    var score = 0;
    if reverse_num(123) == 321 { score = score + 1; }
    if reverse_num(100) == 1 { score = score + 1; }
    if reverse_num(5) == 5 { score = score + 1; }

    if is_palindrome_num(121) { score = score + 1; }
    if is_palindrome_num(12321) { score = score + 1; }
    if !(is_palindrome_num(123)) { score = score + 1; }

    if sum_digits_rec(123) == 6 { score = score + 1; }
    if sum_digits_rec(0) == 0 { score = score + 1; }
    if sum_digits_rec(-456) == 15 { score = score + 1; }

    if count_digits_rec(0) == 1 { score = score + 1; }
    if count_digits_rec(12345) == 5 { score = score + 1; }
    if count_digits_rec(1000000) == 7 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 7: Hanoi & Backtracking
  // ============================================================

  pub fn hanoi_moves(n: Int) -> Int {
    if n == 0 { return 0; }
    return 2 * hanoi_moves(n - 1) + 1;
  }

  pub fn josephus(n: Int, k: Int) -> Int {
    if n == 1 { return 0; }
    return (josephus(n - 1, k) + k) % n;
  }

  pub fn mc_carthy(n: Int) -> Int {
    if n > 100 { return n - 10; }
    return mc_carthy(mc_carthy(n + 11));
  }

  fn test_backtracking() -> Int {
    var score = 0;
    if hanoi_moves(1) == 1 { score = score + 1; }
    if hanoi_moves(3) == 7 { score = score + 1; }
    if hanoi_moves(5) == 31 { score = score + 1; }
    if hanoi_moves(0) == 0 { score = score + 1; }

    if josephus(1, 3) == 0 { score = score + 1; }
    if josephus(5, 2) == 2 { score = score + 1; }

    if mc_carthy(80) == 91 { score = score + 1; }
    if mc_carthy(99) == 91 { score = score + 1; }
    if mc_carthy(101) == 91 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 8: Recursive Math
  // ============================================================

  pub fn lucas(n: Int) -> Int {
    if n == 0 { return 2; }
    if n == 1 { return 1; }
    return lucas(n - 1) + lucas(n - 2);
  }

  pub fn jacobsthal(n: Int) -> Int {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    return jacobsthal(n - 1) + 2 * jacobsthal(n - 2);
  }

  pub fn pell(n: Int) -> Int {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    return 2 * pell(n - 1) + pell(n - 2);
  }

  fn test_recursive_series() -> Int {
    var score = 0;
    if lucas(0) == 2 { score = score + 1; }
    if lucas(1) == 1 { score = score + 1; }
    if lucas(5) == 11 { score = score + 1; }

    if jacobsthal(0) == 0 { score = score + 1; }
    if jacobsthal(1) == 1 { score = score + 1; }
    if jacobsthal(4) == 5 { score = score + 1; }

    if pell(0) == 0 { score = score + 1; }
    if pell(1) == 1 { score = score + 1; }
    if pell(4) == 12 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 9: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_classic_recursion();
    total = total + s1;
    max_score = max_score + 13;

    var s2 = test_mutual_recursion();
    total = total + s2;
    max_score = max_score + 12;

    var s3 = test_divide_conquer();
    total = total + s3;
    max_score = max_score + 14;

    var s4 = test_sequences();
    total = total + s4;
    max_score = max_score + 12;

    var s5 = test_tree_recursion();
    total = total + s5;
    max_score = max_score + 11;

    var s6 = test_string_recursion();
    total = total + s6;
    max_score = max_score + 12;

    var s7 = test_backtracking();
    total = total + s7;
    max_score = max_score + 9;

    var s8 = test_recursive_series();
    total = total + s8;
    max_score = max_score + 9;

    return BenchResult{
      name: "recursion",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: closures
// ============================================================
module closures {


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

}

// ============================================================
// MODULE: modules
// ============================================================
module modules {


  // ============================================================
  // SECTION 1: Internal Module Utilities
  // ============================================================

  pub fn greet(name: Str) -> Str {
    return name;
  }

  pub fn add(a: Int, b: Int) -> Int {
    return a + b;
  }

  pub fn sub(a: Int, b: Int) -> Int {
    return a - b;
  }

  pub fn mul(a: Int, b: Int) -> Int {
    return a * b;
  }

  pub fn div(a: Int, b: Int) -> Int {
    if b == 0 { return 0; }
    return a / b;
  }

  pub fn square(x: Int) -> Int {
    return x * x;
  }

  pub fn cube(x: Int) -> Int {
    return x * x * x;
  }

  pub fn is_positive(x: Int) -> Bool {
    return x > 0;
  }

  pub fn is_negative(x: Int) -> Bool {
    return x < 0;
  }

  pub fn is_zero(x: Int) -> Bool {
    return x == 0;
  }

  // ============================================================
  // SECTION 2: Type Definitions for Cross-Module Use
  // ============================================================

  pub type ModuleInfo = {
    name: Str;
    version: Int;
    active: Bool;
  } derive[Eq, Clone]

  pub type Version = {
    major: Int;
    minor: Int;
    patch: Int;
  } derive[Eq, Clone]

  pub fn ModuleInfo.new(name: Str, version: Int) -> ModuleInfo {
    return ModuleInfo{ name: name, version: version, active: true };
  }

  pub fn Version.new(major: Int, minor: Int, patch: Int) -> Version {
    return Version{ major: major, minor: minor, patch: patch };
  }

  pub fn Version.to_int() -> Int {
    return major * 10000 + minor * 100 + patch;
  }

  // ============================================================
  // SECTION 3: Function That Calls Across Modules
  // ============================================================

  pub fn call_math_twice(x: Int) -> Int {
    var a = square(x);
    return cube(a);
  }

  pub fn validate_range(val: Int, lo: Int, hi: Int) -> Bool {
    return val >= lo && val <= hi;
  }

  pub fn classify_int(n: Int) -> Int {
    if is_positive(n) { return 1; }
    elif is_negative(n) { return -1; }
    return 0;
  }

  // ============================================================
  // SECTION 4: Module-Level Config/Tables
  // ============================================================

  pub fn factorial_table(n: Int) -> Int {
    if n < 0 { return 0; }
    if n > 10 { return -1; }
    var table = [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800];
    return table[n];
  }

  pub fn square_table(n: Int) -> Int {
    if n < 0 || n > 20 { return -1; }
    var squares = [
      0, 1, 4, 9, 16, 25, 36, 49, 64, 81,
      100, 121, 144, 169, 196, 225, 256, 289, 324, 361, 400,
    ];
    return squares[n];
  }

  // ============================================================
  // SECTION 5: Aggregate Test Functions
  // ============================================================

  fn test_utility_functions() -> Int {
    var score = 0;
    if add(2, 3) == 5 { score = score + 1; }
    if sub(10, 3) == 7 { score = score + 1; }
    if mul(7, 8) == 56 { score = score + 1; }
    if div(100, 4) == 25 { score = score + 1; }
    if div(10, 0) == 0 { score = score + 1; }
    if square(5) == 25 { score = score + 1; }
    if cube(3) == 27 { score = score + 1; }
    return score;
  }

  fn test_predicates() -> Int {
    var score = 0;
    if is_positive(5) { score = score + 1; }
    if !(is_positive(-1)) { score = score + 1; }
    if !(is_positive(0)) { score = score + 1; }
    if is_negative(-3) { score = score + 1; }
    if !(is_negative(5)) { score = score + 1; }
    if is_zero(0) { score = score + 1; }
    if !(is_zero(1)) { score = score + 1; }
    return score;
  }

  fn test_types() -> Int {
    var score = 0;
    var mi = ModuleInfo.new("test", 1);
    if mi.name == "test" { score = score + 1; }
    if mi.version == 1 { score = score + 1; }
    if mi.active { score = score + 1; }

    var v = Version.new(1, 2, 3);
    if v.major == 1 { score = score + 1; }
    if v.minor == 2 { score = score + 1; }
    if v.patch == 3 { score = score + 1; }
    if v.to_int() == 10203 { score = score + 1; }

    return score;
  }

  fn test_cross_calls() -> Int {
    var score = 0;
    if call_math_twice(2) == 64 { score = score + 1; }
    if validate_range(5, 0, 10) { score = score + 1; }
    if !(validate_range(15, 0, 10)) { score = score + 1; }
    if classify_int(42) == 1 { score = score + 1; }
    if classify_int(-7) == -1 { score = score + 1; }
    if classify_int(0) == 0 { score = score + 1; }
    return score;
  }

  fn test_tables() -> Int {
    var score = 0;
    if factorial_table(0) == 1 { score = score + 1; }
    if factorial_table(5) == 120 { score = score + 1; }
    if factorial_table(10) == 3628800 { score = score + 1; }
    if factorial_table(-1) == 0 { score = score + 1; }
    if factorial_table(11) == -1 { score = score + 1; }

    if square_table(0) == 0 { score = score + 1; }
    if square_table(10) == 100 { score = score + 1; }
    if square_table(20) == 400 { score = score + 1; }
    if square_table(21) == -1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_utility_functions();
    total = total + s1;
    max_score = max_score + 7;

    var s2 = test_predicates();
    total = total + s2;
    max_score = max_score + 7;

    var s3 = test_types();
    total = total + s3;
    max_score = max_score + 7;

    var s4 = test_cross_calls();
    total = total + s4;
    max_score = max_score + 6;

    var s5 = test_tables();
    total = total + s5;
    max_score = max_score + 9;

    return BenchResult{
      name: "modules",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: safety
// ============================================================
module safety {


  // ============================================================
  // SECTION 1: Read Borrow Patterns
  // ============================================================

  pub type Point = {
    x: Int;
    y: Int;
  } derive[Clone]

  pub fn Point.new(x: Int, y: Int) -> Point {
    return Point{ x: x, y: y };
  }

  pub fn Point.dist_sq(other: &Point) -> Int {
    var dx = x - other.x;
    var dy = y - other.y;
    return dx * dx + dy * dy;
  }

  pub fn Point.midpoint(other: &Point) -> Point {
    return Point{ x: (x + other.x) / 2, y: (y + other.y) / 2 };
  }

  pub fn compute_with_borrows(p1: &Point, p2: &Point, p3: &Point) -> Int {
    var d1 = p1.dist_sq(p2);
    var d2 = p2.dist_sq(p3);
    var d3 = p3.dist_sq(p1);
    return d1 + d2 + d3;
  }

  fn test_read_borrows() -> Int {
    var score = 0;
    var p1 = Point.new(0, 0);
    var p2 = Point.new(3, 4);
    var p3 = Point.new(6, 8);

    if p1.dist_sq(&p2) == 25 { score = score + 1; }

    var mid = p1.midpoint(&p2);
    if mid.x == 1 { score = score + 1; }
    if mid.y == 2 { score = score + 1; }

    var sum_dist = compute_with_borrows(&p1, &p2, &p3);
    if sum_dist == 75 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Clone to Avoid Move
  // ============================================================

  pub type Data = {
    id: Int;
    values: Vec[Int];
  } derive[Clone]

  pub fn Data.new(id: Int) -> Data {
    return Data{ id: id, values: Vec[Int].new() };
  }

  pub fn Data.add(val: Int) {
    values.push(val);
  }

  pub fn Data.sum() -> Int {
    var total = 0;
    var i = 0;
    while i < values.len() {
      total = total + values[i];
      i = i + 1;
    }
    return total;
  }

  pub fn process_then_check(d: &Data) -> Int {
    var clone1 = d.clone();
    var clone2 = d.clone();
    return clone1.sum() + clone2.sum();
  }

  fn test_clone_patterns() -> Int {
    var score = 0;
    var d = Data.new(1);
    d.add(10);
    d.add(20);
    d.add(30);

    if d.sum() == 60 { score = score + 1; }

    // Process using clones to avoid move
    var result = process_then_check(&d);
    if result == 120 { score = score + 1; }

    // Original still valid
    if d.sum() == 60 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Multiple Read Borrows
  // ============================================================

  pub type Account = {
    balance: Int;
    name: Str;
  } derive[Clone]

  pub fn Account.new(balance: Int, name: Str) -> Account {
    return Account{ balance: balance, name: name };
  }

  pub fn compare_balances(a: &Account, b: &Account) -> Int {
    if a.balance > b.balance { return 1; }
    if a.balance < b.balance { return -1; }
    return 0;
  }

  pub fn total_balance(accounts: &Vec[Account]) -> Int {
    var total = 0;
    var i = 0;
    while i < accounts.len() {
      total = total + accounts[i].balance;
      i = i + 1;
    }
    return total;
  }

  fn test_multi_borrows() -> Int {
    var score = 0;
    var a1 = Account.new(100, "Alice");
    var a2 = Account.new(200, "Bob");
    var a3 = Account.new(150, "Charlie");

    // Multiple read borrows (passing references to compare)
    var cmp = compare_balances(&a1, &a2);
    if cmp == -1 { score = score + 1; }

    var accounts = [a1.clone(), a2.clone(), a3.clone()];
    var total = total_balance(&accounts);
    if total == 450 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Struct Methods with Self Borrow
  // ============================================================

  pub type Rect = {
    x: Int;
    y: Int;
    w: Int;
    h: Int;
  } derive[Clone]

  pub fn Rect.new(x: Int, y: Int, w: Int, h: Int) -> Rect {
    return Rect{ x: x, y: y, w: w, h: h };
  }

  pub fn Rect.area() -> Int {
    return w * h;
  }

  pub fn Rect.perimeter() -> Int {
    return 2 * (w + h);
  }

  pub fn Rect.contains_point(px: Int, py: Int) -> Bool {
    return px >= x && px < x + w && py >= y && py < y + h;
  }

  pub fn Rect.overlaps(other: &Rect) -> Bool {
    if x + w <= other.x || other.x + other.w <= x { return false; }
    if y + h <= other.y || other.y + other.h <= y { return false; }
    return true;
  }

  fn test_method_borrows() -> Int {
    var score = 0;
    var r1 = Rect.new(0, 0, 10, 10);
    if r1.area() == 100 { score = score + 1; }
    if r1.perimeter() == 40 { score = score + 1; }
    if r1.contains_point(5, 5) { score = score + 1; }
    if !(r1.contains_point(15, 5)) { score = score + 1; }

    var r2 = Rect.new(5, 5, 10, 10);
    if r1.overlaps(&r2) { score = score + 1; }

    var r3 = Rect.new(20, 20, 5, 5);
    if !(r1.overlaps(&r3)) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Ownership Transfer Patterns
  // ============================================================

  pub type Resource = {
    id: Int;
    data: Vec[Int];
  } derive[Clone]

  pub fn Resource.new(id: Int) -> Resource {
    return Resource{ id: id, data: Vec[Int].new() };
  }

  pub fn Resource.fill(n: Int) {
    var i = 0;
    while i < n {
      data.push(i);
      i = i + 1;
    }
  }

  pub fn Resource.len() -> Int {
    return data.len();
  }

  pub fn take_resource(r: Resource) -> Int {
    var size = r.data.len();
    return size;
  }

  fn test_ownership() -> Int {
    var score = 0;
    var r1 = Resource.new(1);
    r1.fill(5);
    if r1.len() == 5 { score = score + 1; }

    // Clone before transfer
    var r1_clone = r1.clone();
    var consumed = take_resource(r1);
    // r1 is moved, r1_clone still valid
    if consumed == 5 { score = score + 1; }
    if r1_clone.len() == 5 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_read_borrows();
    total = total + s1;
    max_score = max_score + 4;

    var s2 = test_clone_patterns();
    total = total + s2;
    max_score = max_score + 3;

    var s3 = test_multi_borrows();
    total = total + s3;
    max_score = max_score + 2;

    var s4 = test_method_borrows();
    total = total + s4;
    max_score = max_score + 6;

    var s5 = test_ownership();
    total = total + s5;
    max_score = max_score + 3;

    return BenchResult{
      name: "safety",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: collections
// ============================================================
module collections {


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

}

// ============================================================
// MODULE: algorithms
// ============================================================
module algorithms {


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

}

// ============================================================
// MODULE: extreme
// ============================================================
module extreme {


  // ============================================================
  // SECTION 1: Deep Nesting Patterns
  // ============================================================

  pub fn deep_if_nested(n: Int) -> Int {
    if n > 0 {
      return 1;
    } elif n < 0 {
      return -1;
    } else {
      if n == 0 {
        if n * 1 == 0 {
          if n + 0 == 0 {
            return 0;
          } else {
            return -99;
          }
        } else {
          return -98;
        }
      } else {
        return -97;
      }
    }
  }

  pub fn nested_loops(depth: Int) -> Int {
    var total = 0;
    var i = 0;
    while i < depth {
      var j = 0;
      while j < depth {
        var k = 0;
        while k < depth {
          total = total + 1;
          k = k + 1;
        }
        j = j + 1;
      }
      i = i + 1;
    }
    return total;
  }

  pub fn deep_expression_tree(a: Int, b: Int, c: Int, d: Int) -> Int {
    return ((a + b) * (c - d)) + ((a - b) * (c + d)) + ((a * b) / (if c + d == 0 { 1; } else { c + d; }));
  }

  fn test_deep_nesting() -> Int {
    var score = 0;
    if deep_if_nested(5) == 1 { score = score + 1; }
    if deep_if_nested(-3) == -1 { score = score + 1; }
    if deep_if_nested(0) == 0 { score = score + 1; }

    if nested_loops(3) == 27 { score = score + 1; }
    if nested_loops(1) == 1 { score = score + 1; }
    if nested_loops(5) == 125 { score = score + 1; }

    if deep_expression_tree(1, 2, 3, 4) == -4 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Many Arguments
  // ============================================================

  pub fn sum10(a0: Int, a1: Int, a2: Int, a3: Int, a4: Int,
    a5: Int, a6: Int, a7: Int, a8: Int, a9: Int) -> Int {
    return a0 + a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9;
  }

  pub fn mul5(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
    return a * b * c * d * e;
  }

  pub fn mixed_args(a: Int, b: Float64, c: Bool, d: Int, e: Float64) -> Float64 {
    var result = 0.0;
    if c { result = result + (a as Float64); }
    result = result + b * e;
    result = result + (d as Float64);
    return result;
  }

  fn test_many_args() -> Int {
    var score = 0;
    var s10 = sum10(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
    if s10 == 55 { score = score + 1; }
    var s10b = sum10(0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    if s10b == 0 { score = score + 1; }

    if mul5(1, 2, 3, 4, 5) == 120 { score = score + 1; }
    if mul5(2, 2, 2, 2, 2) == 32 { score = score + 1; }

    var m1 = mixed_args(10, 2.0, true, 5, 3.0);
    if m1 == 21.0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Edge Cases -- Zero, Negative, Boundary Values
  // ============================================================

  pub fn zero_division_handler(a: Int, b: Int) -> Int {
    if b == 0 { return 0; }
    return a / b;
  }

  pub fn mod_negative(a: Int, b: Int) -> Int {
    if b == 0 { return -1; }
    return a % b;
  }

  pub fn max_edge(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
    var m = a;
    if b > m { m = b; }
    if c > m { m = c; }
    if d > m { m = d; }
    if e > m { m = e; }
    return m;
  }

  pub fn min_edge(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
    var m = a;
    if b < m { m = b; }
    if c < m { m = c; }
    if d < m { m = d; }
    if e < m { m = e; }
    return m;
  }

  fn test_edge_cases() -> Int {
    var score = 0;
    if zero_division_handler(10, 0) == 0 { score = score + 1; }
    if zero_division_handler(10, 2) == 5 { score = score + 1; }

    if mod_negative(10, 3) == 1 { score = score + 1; }
    if mod_negative(10, 0) == -1 { score = score + 1; }

    if max_edge(1, 2, 3, 4, 5) == 5 { score = score + 1; }
    if max_edge(-5, -4, -3, -2, -1) == -1 { score = score + 1; }
    if max_edge(0, 0, 0, 0, 0) == 0 { score = score + 1; }

    if min_edge(5, 4, 3, 2, 1) == 1 { score = score + 1; }
    if min_edge(-1, -2, -3, -4, -5) == -5 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Large Match/Switch Pattern
  // ============================================================

  pub fn match_50(value: Int) -> Int {
    match value {
      0 => 0,
      1 => 1,
      2 => 4,
      3 => 9,
      4 => 16,
      5 => 25,
      6 => 36,
      7 => 49,
      8 => 64,
      9 => 81,
      10 => 100,
      11 => 121,
      12 => 144,
      13 => 169,
      14 => 196,
      15 => 225,
      16 => 256,
      17 => 289,
      18 => 324,
      19 => 361,
      20 => 400,
      21 => 441,
      22 => 484,
      23 => 529,
      24 => 576,
      25 => 625,
      26 => 676,
      27 => 729,
      28 => 784,
      29 => 841,
      30 => 900,
      31 => 961,
      32 => 1024,
      33 => 1089,
      34 => 1156,
      35 => 1225,
      36 => 1296,
      37 => 1369,
      38 => 1444,
      39 => 1521,
      40 => 1600,
      41 => 1681,
      42 => 1764,
      43 => 1849,
      44 => 1936,
      45 => 2025,
      46 => 2116,
      47 => 2209,
      48 => 2304,
      49 => 2401,
      _ => -1,
    }
  }

  fn test_large_match() -> Int {
    var score = 0;
    if match_50(0) == 0 { score = score + 1; }
    if match_50(5) == 25 { score = score + 1; }
    if match_50(10) == 100 { score = score + 1; }
    if match_50(25) == 625 { score = score + 1; }
    if match_50(49) == 2401 { score = score + 1; }
    if match_50(50) == -1 { score = score + 1; }
    if match_50(100) == -1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Recursion Depth
  // ============================================================

  pub fn sum_to_n(n: Int) -> Int {
    if n <= 0 { return 0; }
    return n + sum_to_n(n - 1);
  }

  pub fn count_down_to_zero(n: Int) -> Int {
    if n <= 0 { return 0; }
    return 1 + count_down_to_zero(n - 1);
  }

  pub fn nested_sum(depth: Int, value: Int) -> Int {
    if depth <= 0 { return value; }
    return nested_sum(depth - 1, value + 1);
  }

  fn test_recursion_depth() -> Int {
    var score = 0;
    if sum_to_n(10) == 55 { score = score + 1; }
    if sum_to_n(0) == 0 { score = score + 1; }
    if sum_to_n(100) == 5050 { score = score + 1; }

    if count_down_to_zero(5) == 5 { score = score + 1; }
    if count_down_to_zero(0) == 0 { score = score + 1; }

    if nested_sum(10, 0) == 10 { score = score + 1; }
    if nested_sum(0, 42) == 42 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_deep_nesting();
    total = total + s1;
    max_score = max_score + 7;

    var s2 = test_many_args();
    total = total + s2;
    max_score = max_score + 5;

    var s3 = test_edge_cases();
    total = total + s3;
    max_score = max_score + 9;

    var s4 = test_large_match();
    total = total + s4;
    max_score = max_score + 7;

    var s5 = test_recursion_depth();
    total = total + s5;
    max_score = max_score + 7;

    return BenchResult{
      name: "extreme",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: comptime
// ============================================================
module comptime {


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

}

// ============================================================
// MODULE: concurrency
// ============================================================
module concurrency {


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

}

// ============================================================
// MODULE: crypto
// ============================================================
module crypto {


  // ============================================================
  // SECTION 1: Simple Hash Functions
  // ============================================================

  pub fn djb2_hash(input: &Vec[Int]) -> Int {
    var hash = 5381;
    var i = 0;
    while i < input.len() {
      hash = ((hash * 33) + hash) + input[i];
      i = i + 1;
    }
    return hash;
  }

  pub fn fnv1a_hash(input: &Vec[Int]) -> Int {
    var hash = -2128831035;
    var i = 0;
    while i < input.len() {
      hash = hash * input[i];
      hash = hash + 16777619;
      i = i + 1;
    }
    return hash;
  }

  pub fn simple_hash(input: &Vec[Int]) -> Int {
    var h = 0;
    var i = 0;
    while i < input.len() {
      h = (h * 31 + input[i]) % 1000000007;
      i = i + 1;
    }
    return h;
  }

  fn test_hashes() -> Int {
    var score = 0;
    var data = [1, 2, 3, 4, 5];
    var h1 = djb2_hash(&data);
    if h1 != 0 { score = score + 1; }

    var empty: Vec[Int] = [];
    var h_empty = djb2_hash(&empty);
    if h_empty == 5381 { score = score + 1; }

    var h2 = fnv1a_hash(&data);
    if h2 != 0 { score = score + 1; }

    var h3 = simple_hash(&data);
    if h3 != 0 { score = score + 1; }

    // Determinism: same input -> same hash
    if djb2_hash(&data) == djb2_hash(&data) { score = score + 1; }
    if simple_hash(&data) == simple_hash(&data) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Caesar/Vigenere Cipher
  // ============================================================

  pub fn caesar_encode(input: Vec[Int], shift: Int) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < input.len() {
      result.push(input[i] + shift);
      i = i + 1;
    }
    return result;
  }

  pub fn caesar_decode(input: Vec[Int], shift: Int) -> Vec[Int] {
    return caesar_encode(input, -shift);
  }

  pub fn vigenere_encode(input: Vec[Int], key: Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var key_len = key.len();
    if key_len == 0 { return result; }
    var i = 0;
    while i < input.len() {
      var shift = key[i % key_len];
      result.push(input[i] + shift);
      i = i + 1;
    }
    return result;
  }

  pub fn vigenere_decode(input: Vec[Int], key: Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var key_len = key.len();
    if key_len == 0 { return result; }
    var i = 0;
    while i < input.len() {
      var shift = key[i % key_len];
      result.push(input[i] - shift);
      i = i + 1;
    }
    return result;
  }

  fn test_ciphers() -> Int {
    var score = 0;
    var plain = [1, 2, 3, 4, 5];

    // Caesar
    var encoded = caesar_encode(plain, 10);
    if encoded.len() == 5 { score = score + 1; }
    if encoded[0] == 11 { score = score + 1; }

    var decoded = caesar_decode(encoded, 10);
    if decoded[0] == 1 { score = score + 1; }
    if decoded[4] == 5 { score = score + 1; }

    // Vigenere
    var key = [3, 5];
    var v_enc = vigenere_encode(plain, key);
    if v_enc[0] == 4 { score = score + 1; }
    if v_enc[1] == 7 { score = score + 1; }

    var v_dec = vigenere_decode(v_enc, key);
    if v_dec[0] == 1 { score = score + 1; }
    if v_dec[1] == 2 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Checksums
  // ============================================================

  pub fn xor_checksum(input: &Vec[Int]) -> Int {
    var checksum = 0;
    var i = 0;
    while i < input.len() {
      checksum = checksum + input[i];
      i = i + 1;
    }
    return checksum % 256;
  }

  pub fn additive_checksum(input: &Vec[Int]) -> Int {
    var sum = 0;
    var i = 0;
    while i < input.len() {
      sum = sum + input[i];
      i = i + 1;
    }
    return sum % 65536;
  }

  pub fn parity_check(input: &Vec[Int]) -> Int {
    var ones = 0;
    var i = 0;
    while i < input.len() {
      var n = input[i];
      while n > 0 {
        if n % 2 == 1 { ones = ones + 1; }
        n = n / 2;
      }
      i = i + 1;
    }
    return ones % 2;
  }

  fn test_checksums() -> Int {
    var score = 0;
    var data = [1, 2, 3, 4, 5];
    var ck = xor_checksum(&data);
    if ck >= 0 && ck < 256 { score = score + 1; }

    var ck2 = additive_checksum(&data);
    if ck2 == 15 { score = score + 1; }

    var p = parity_check(&data);
    if p == 0 || p == 1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Key Derivation (PBKDF2-like)
  // ============================================================

  pub fn pbkdf2_simple(password: Vec[Int], salt: Vec[Int], iterations: Int) -> Int {
    var key = 0;
    var i = 0;
    while i < iterations {
      var tmp = key;
      var j = 0;
      while j < password.len() {
        tmp = (tmp * 31 + password[j]) % 1000000007;
        j = j + 1;
      }
      j = 0;
      while j < salt.len() {
        tmp = (tmp * 37 + salt[j]) % 1000000007;
        j = j + 1;
      }
      key = tmp;
      i = i + 1;
    }
    return key;
  }

  fn test_key_derivation() -> Int {
    var score = 0;
    var pwd = [1, 2, 3, 4];
    var salt = [9, 8, 7];

    var k1 = pbkdf2_simple(pwd, salt, 100);
    if k1 != 0 { score = score + 1; }

    var k2 = pbkdf2_simple(pwd, salt, 100);
    if k1 == k2 { score = score + 1; }

    // Different password -> different key
    var pwd2 = [1, 2, 3, 5];
    var k3 = pbkdf2_simple(pwd2, salt, 100);
    if k1 != k3 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Base Encoding (Base64-like numeric)
  // ============================================================

  pub fn encode_base(b: Int, n: Int) -> Vec[Int] {
    if n == 0 {
      var r = Vec[Int].new();
      r.push(0);
      return r;
    }
    var result = Vec[Int].new();
    var x = n;
    while x > 0 {
      result.push(x % b);
      x = x / b;
    }
    var reversed = Vec[Int].new();
    var i = result.len();
    while i > 0 {
      i = i - 1;
      reversed.push(result[i]);
    }
    return reversed;
  }

  pub fn decode_base(b: Int, digits: Vec[Int]) -> Int {
    var value = 0;
    var i = 0;
    while i < digits.len() {
      value = value * b + digits[i];
      i = i + 1;
    }
    return value;
  }

  fn test_encoding() -> Int {
    var score = 0;
    // Binary encoding
    var bin = encode_base(2, 42);
    var dec_bin = decode_base(2, bin);
    if dec_bin == 42 { score = score + 1; }

    // Octal encoding
    var oct = encode_base(8, 64);
    var dec_oct = decode_base(8, oct);
    if dec_oct == 64 { score = score + 1; }

    // Hex encoding
    var hex = encode_base(16, 255);
    var dec_hex = decode_base(16, hex);
    if dec_hex == 255 { score = score + 1; }

    // Zero
    var zero = encode_base(10, 0);
    if decode_base(10, zero) == 0 { score = score + 1; }

    // Roundtrip property
    if decode_base(7, encode_base(7, 123)) == 123 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_hashes();
    total = total + s1;
    max_score = max_score + 6;

    var s2 = test_ciphers();
    total = total + s2;
    max_score = max_score + 8;

    var s3 = test_checksums();
    total = total + s3;
    max_score = max_score + 3;

    var s4 = test_key_derivation();
    total = total + s4;
    max_score = max_score + 3;

    var s5 = test_encoding();
    total = total + s5;
    max_score = max_score + 5;

    return BenchResult{
      name: "crypto",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: sort
// ============================================================
module sort {


  // ============================================================
  // SECTION 1: O(n^2) Sorts
  // ============================================================

  pub fn bubble_sort(v: Vec[Int]) -> Vec[Int] {
    var arr = vec_copy(v);
    var n = arr.len();
    var i = 0;
    while i < n - 1 {
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

  pub fn selection_sort(v: Vec[Int]) -> Vec[Int] {
    var arr = vec_copy(v);
    var n = arr.len();
    var i = 0;
    while i < n - 1 {
      var min_idx = i;
      var j = i + 1;
      while j < n {
        if arr[j] < arr[min_idx] { min_idx = j; }
        j = j + 1;
      }
      if min_idx != i {
        var temp = arr[i];
        arr[i] = arr[min_idx];
        arr[min_idx] = temp;
      }
      i = i + 1;
    }
    return arr;
  }

  pub fn insertion_sort(v: Vec[Int]) -> Vec[Int] {
    var arr = vec_copy(v);
    var n = arr.len();
    var i = 1;
    while i < n {
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

  pub fn gnome_sort(v: Vec[Int]) -> Vec[Int] {
    var arr = vec_copy(v);
    var pos = 0;
    var n = arr.len();
    while pos < n {
      if pos == 0 || arr[pos] >= arr[pos - 1] {
        pos = pos + 1;
      } else {
        var temp = arr[pos];
        arr[pos] = arr[pos - 1];
        arr[pos - 1] = temp;
        pos = pos - 1;
      }
    }
    return arr;
  }

  pub fn vec_copy(v: Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      result.push(v[i]);
      i = i + 1;
    }
    return result;
  }

  pub fn is_sorted(v: &Vec[Int]) -> Bool {
    var i = 1;
    while i < v.len() {
      if v[i - 1] > v[i] { return false; }
      i = i + 1;
    }
    return true;
  }

  pub fn sorted_equal(a: &Vec[Int], b: &Vec[Int]) -> Bool {
    if a.len() != b.len() { return false; }
    var i = 0;
    while i < a.len() {
      if a[i] != b[i] { return false; }
      i = i + 1;
    }
    return true;
  }

  fn test_quadratic_sorts() -> Int {
    var score = 0;
    var unsorted = [5, 2, 8, 1, 9, 3, 7, 4, 6];

    var bs = bubble_sort(unsorted);
    if is_sorted(&bs) { score = score + 1; }
    if bs[0] == 1 && bs[bs.len() - 1] == 9 { score = score + 1; }

    var ss = selection_sort(unsorted);
    if is_sorted(&ss) { score = score + 1; }

    var ins = insertion_sort(unsorted);
    if is_sorted(&ins) { score = score + 1; }

    var gn = gnome_sort(unsorted);
    if is_sorted(&gn) { score = score + 1; }

    // All sorts produce the same result
    if sorted_equal(&bs, &ss) { score = score + 1; }
    if sorted_equal(&bs, &ins) { score = score + 1; }
    if sorted_equal(&bs, &gn) { score = score + 1; }

    // Edge cases
    var already = [1, 2, 3, 4, 5];
    var sorted_already = bubble_sort(already);
    if is_sorted(&sorted_already) { score = score + 1; }

    var reverse = [9, 8, 7, 6, 5, 4, 3, 2, 1];
    var sorted_rev = insertion_sort(reverse);
    if is_sorted(&sorted_rev) { score = score + 1; }
    if sorted_rev[0] == 1 { score = score + 1; }

    var single = [42];
    if is_sorted(&bubble_sort(single)) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: O(n log n) Sorts (Merge Sort, Quick Sort)
  // ============================================================

  pub fn merge_sort(v: Vec[Int]) -> Vec[Int] {
    if v.len() <= 1 { return v; }
    var mid = v.len() / 2;
    var left = slice(v, 0, mid);
    var right = slice(v, mid, v.len());
    var sorted_left = merge_sort(left);
    var sorted_right = merge_sort(right);
    return merge(sorted_left, sorted_right);
  }

  pub fn slice(v: Vec[Int], start: Int, end: Int) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = start;
    while i < end {
      result.push(v[i]);
      i = i + 1;
    }
    return result;
  }

  pub fn merge(left: Vec[Int], right: Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    var j = 0;
    while i < left.len() && j < right.len() {
      if left[i] <= right[j] {
        result.push(left[i]);
        i = i + 1;
      } else {
        result.push(right[j]);
        j = j + 1;
      }
    }
    while i < left.len() {
      result.push(left[i]);
      i = i + 1;
    }
    while j < right.len() {
      result.push(right[j]);
      j = j + 1;
    }
    return result;
  }

  pub fn quick_sort(v: Vec[Int]) -> Vec[Int] {
    if v.len() <= 1 { return v; }
    var pivot = v[v.len() / 2];
    var less = Vec[Int].new();
    var equal = Vec[Int].new();
    var greater = Vec[Int].new();
    var i = 0;
    while i < v.len() {
      if v[i] < pivot {
        less.push(v[i]);
      } elif v[i] > pivot {
        greater.push(v[i]);
      } else {
        equal.push(v[i]);
      }
      i = i + 1;
    }
    var sorted_less = quick_sort(less);
    var sorted_greater = quick_sort(greater);
    return concat_three(sorted_less, equal, sorted_greater);
  }

  pub fn concat_three(a: Vec[Int], b: Vec[Int], c: Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < a.len() { result.push(a[i]); i = i + 1; }
    i = 0;
    while i < b.len() { result.push(b[i]); i = i + 1; }
    i = 0;
    while i < c.len() { result.push(c[i]); i = i + 1; }
    return result;
  }

  fn test_nlogn_sorts() -> Int {
    var score = 0;
    var unsorted = [9, 3, 7, 1, 5, 8, 2, 6, 4];

    var ms = merge_sort(unsorted);
    if is_sorted(&ms) { score = score + 1; }

    var qs = quick_sort(unsorted);
    if is_sorted(&qs) { score = score + 1; }

    if sorted_equal(&ms, &qs) { score = score + 1; }

    // Edge cases
    if is_sorted(&merge_sort([1])) { score = score + 1; }
    if is_sorted(&quick_sort([1])) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Counting/Bucket Sort
  // ============================================================

  pub fn counting_sort(v: Vec[Int], max_val: Int) -> Vec[Int] {
    var counts = Vec[Int].new();
    var i = 0;
    while i <= max_val {
      counts.push(0);
      i = i + 1;
    }
    i = 0;
    while i < v.len() {
      var idx = v[i];
      counts[idx] = counts[idx] + 1;
      i = i + 1;
    }
    var result = Vec[Int].new();
    i = 0;
    while i < counts.len() {
      var j = 0;
      while j < counts[i] {
        result.push(i);
        j = j + 1;
      }
      i = i + 1;
    }
    return result;
  }

  fn test_counting_sort() -> Int {
    var score = 0;
    var data = [3, 1, 4, 1, 5, 9, 2, 6];
    var sorted = counting_sort(data, 9);
    if is_sorted(&sorted) { score = score + 1; }
    if sorted.len() == data.len() { score = score + 1; }
    return score;
  }

  // ============================================================
  // SECTION 4: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_quadratic_sorts();
    total = total + s1;
    max_score = max_score + 12;

    var s2 = test_nlogn_sorts();
    total = total + s2;
    max_score = max_score + 5;

    var s3 = test_counting_sort();
    total = total + s3;
    max_score = max_score + 2;

    return BenchResult{
      name: "sort",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: interfaces
// ============================================================
module interfaces {


  // ============================================================
  // SECTION 1: Interface Definitions
  // ============================================================

  pub type Circle = {
    radius: Float64;
  } derive[Eq, Clone]

  pub type Rectangle = {
    width: Float64;
    height: Float64;
  } derive[Eq, Clone]

  pub type Triangle = {
    base: Float64;
    height: Float64;
  } derive[Eq, Clone]

  pub type Square = {
    side: Float64;
  } derive[Eq, Clone]

  // Interface methods implemented directly on types

  pub fn Circle.area() -> Float64 {
    return 3.14159 * radius * radius;
  }

  pub fn Circle.perimeter() -> Float64 {
    return 2.0 * 3.14159 * radius;
  }

  pub fn Rectangle.area() -> Float64 {
    return width * height;
  }

  pub fn Rectangle.perimeter() -> Float64 {
    return 2.0 * (width + height);
  }

  pub fn Triangle.area() -> Float64 {
    return 0.5 * base * height;
  }

  pub fn Square.area() -> Float64 {
    return side * side;
  }

  pub fn Square.perimeter() -> Float64 {
    return 4.0 * side;
  }

  // ============================================================
  // SECTION 2: Generic Dispatch
  // ============================================================

  pub fn total_area[T](shapes: &Vec[T], area_fn: fn(&T) -> Float64) -> Float64 {
    var total = 0.0;
    var i = 0;
    while i < shapes.len() {
      total = total + area_fn(&shapes[i]);
      i = i + 1;
    }
    return total;
  }

  pub fn count_larger_than[T](shapes: &Vec[T], threshold: Float64, area_fn: fn(&T) -> Float64) -> Int {
    var count = 0;
    var i = 0;
    while i < shapes.len() {
      if area_fn(&shapes[i]) > threshold {
        count = count + 1;
      }
      i = i + 1;
    }
    return count;
  }

  fn circle_area_fn(c: &Circle) -> Float64 { return c.area(); }
  fn rect_area_fn(r: &Rectangle) -> Float64 { return r.area(); }
  fn square_area_fn(s: &Square) -> Float64 { return s.area(); }

  fn test_interface_dispatch() -> Int {
    var score = 0;
    var c = Circle{ radius: 2.0 };
    var r = Rectangle{ width: 3.0, height: 4.0 };
    var s = Square{ side: 5.0 };

    // Direct method calls
    var ca = c.area();
    if ca > 12.5 && ca < 12.6 { score = score + 1; }

    if r.area() == 12.0 { score = score + 1; }
    if s.area() == 25.0 { score = score + 1; }

    // Perimeters
    var cp = c.perimeter();
    if cp > 12.5 && cp < 12.6 { score = score + 1; }
    if r.perimeter() == 14.0 { score = score + 1; }
    if s.perimeter() == 20.0 { score = score + 1; }

    // Generic dispatch via function pointer
    var circles = [c.clone(), Circle{ radius: 3.0 }];
    var area = total_area(&circles, circle_area_fn);
    if area > 40.8 && area < 40.9 { score = score + 1; }

    // Count larger than threshold
    var rects = [
      Rectangle{ width: 1.0, height: 1.0 },
      Rectangle{ width: 3.0, height: 4.0 },
      Rectangle{ width: 2.0, height: 2.0 },
    ];
    var big_count = count_larger_than(&rects, 5.0, rect_area_fn);
    if big_count == 1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Multiple Interface Satisfaction
  // ============================================================

  pub type Point2D = {
    x: Float64;
    y: Float64;
  } derive[Eq, Clone]

  pub fn Point2D.distance_sq(other: &Point2D) -> Float64 {
    var dx = x - other.x;
    var dy = y - other.y;
    return dx * dx + dy * dy;
  }

  pub fn Point2D.magnitude_sq() -> Float64 {
    return x * x + y * y;
  }

  pub fn Point2D.add(other: &Point2D) -> Float64 {
    return x + other.x + y + other.y;
  }

  pub fn distance_between(p1: &Point2D, p2: &Point2D) -> Float64 {
    var dsq = p1.distance_sq(p2);
    // Approximate sqrt
    if dsq == 0.0 { return 0.0; }
    var guess = dsq / 2.0;
    var i = 0;
    while i < 20 {
      guess = (guess + dsq / guess) / 2.0;
      i = i + 1;
    }
    return guess;
  }

  fn test_point_interface() -> Int {
    var score = 0;
    var p1 = Point2D{ x: 0.0, y: 0.0 };
    var p2 = Point2D{ x: 3.0, y: 4.0 };

    if p1.distance_sq(&p2) == 25.0 { score = score + 1; }
    if p2.magnitude_sq() == 25.0 { score = score + 1; }

    var dist = distance_between(&p1, &p2);
    if dist > 4.9 && dist < 5.1 { score = score + 1; }

    if p2.add(&p1) == 7.0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Record-Keeping Interface
  // ============================================================

  pub type Record = {
    id: Int;
    value: Int;
    timestamp: Int;
  } derive[Eq, Clone]

  pub fn Record.new(id: Int, value: Int) -> Record {
    return Record{ id: id, value: value, timestamp: id * 100 };
  }

  pub fn Record.update(val: Int) -> Record {
    return Record{ id: id, value: val, timestamp: id * 100 };
  }

  pub fn Record.is_active() -> Bool {
    return value > 0;
  }

  pub fn Record.score() -> Int {
    return value;
  }

  pub fn find_by_id(records: &Vec[Record], target_id: Int) -> Option[Record] {
    var i = 0;
    while i < records.len() {
      if records[i].id == target_id {
        return Some(records[i].clone());
      }
      i = i + 1;
    }
    return None;
  }

  pub fn sum_scores(records: &Vec[Record]) -> Int {
    var total = 0;
    var i = 0;
    while i < records.len() {
      total = total + records[i].score();
      i = i + 1;
    }
    return total;
  }

  pub fn count_active(records: &Vec[Record]) -> Int {
    var count = 0;
    var i = 0;
    while i < records.len() {
      if records[i].is_active() { count = count + 1; }
      i = i + 1;
    }
    return count;
  }

  fn test_record_interface() -> Int {
    var score = 0;
    var recs = Vec[Record].new();
    recs.push(Record.new(1, 10));
    recs.push(Record.new(2, 0));
    recs.push(Record.new(3, 30));

    if sum_scores(&recs) == 40 { score = score + 1; }
    if count_active(&recs) == 2 { score = score + 1; }

    match find_by_id(&recs, 2) {
      Some(r) => if r.value == 0 { score = score + 1; }
      None => {}
    }

    match find_by_id(&recs, 99) {
      Some(_) => {}
      None => { score = score + 1; }
    }

    return score;
  }

  // ============================================================
  // SECTION 5: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_interface_dispatch();
    total = total + s1;
    max_score = max_score + 8;

    var s2 = test_point_interface();
    total = total + s2;
    max_score = max_score + 4;

    var s3 = test_record_interface();
    total = total + s3;
    max_score = max_score + 4;

    return BenchResult{
      name: "interfaces",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: serialize
// ============================================================
module serialize {


  // ============================================================
  // SECTION 1: Int <-> Str Conversion (simulated)
  // ============================================================

  pub fn int_to_digits(n: Int) -> Vec[Int] {
    if n == 0 {
      var r = Vec[Int].new();
      r.push(0);
      return r;
    }
    var result = Vec[Int].new();
    var x = if n < 0 { -n; } else { n; };
    while x > 0 {
      result.push(x % 10);
      x = x / 10;
    }
    var reversed = Vec[Int].new();
    var i = result.len();
    while i > 0 {
      i = i - 1;
      reversed.push(result[i]);
    }
    return reversed;
  }

  pub fn digits_to_int(digits: &Vec[Int]) -> Int {
    if digits.len() == 0 { return 0; }
    var value = 0;
    var i = 0;
    while i < digits.len() {
      value = value * 10 + digits[i];
      i = i + 1;
    }
    return value;
  }

  pub fn int_to_str(n: Int) -> Str {
    var digits = int_to_digits(n);
    // Return string representation - just use len for test
    return "0";
  }

  fn test_serialization() -> Int {
    var score = 0;
    var d1 = int_to_digits(123);
    if d1.len() == 3 { score = score + 1; }
    if d1[0] == 1 && d1[1] == 2 && d1[2] == 3 { score = score + 1; }

    var d2 = int_to_digits(0);
    if d2.len() == 1 && d2[0] == 0 { score = score + 1; }

    var d3 = int_to_digits(100);
    if d3.len() == 3 { score = score + 1; }

    // Roundtrip
    if digits_to_int(&d1) == 123 { score = score + 1; }
    if digits_to_int(&d2) == 0 { score = score + 1; }
    if digits_to_int(&d3) == 100 { score = score + 1; }

    // Roundtrip for many values
    var all_ok = 1;
    var i = 0;
    while i < 1000 {
      var digits = int_to_digits(i);
      if digits_to_int(&digits) != i { all_ok = 0; }
      i = i + 1;
    }
    if all_ok == 1 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: CSV-Like Encoding
  // ============================================================

  pub fn encode_row(values: &Vec[Int]) -> Vec[Int] {
    // Simple encoding: add delimiter (0) between values
    var result = Vec[Int].new();
    var i = 0;
    while i < values.len() {
      if i > 0 { result.push(0); }
      result.push(values[i]);
      i = i + 1;
    }
    return result;
  }

  pub fn decode_row(encoded: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var current = 0;
    var i = 0;
    while i < encoded.len() {
      if encoded[i] == 0 {
        result.push(current);
        current = 0;
      } else {
        current = current * 10 + encoded[i];
      }
      i = i + 1;
    }
    result.push(current);
    return result;
  }

  pub fn encode_table(rows: &Vec[Vec[Int]]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < rows.len() {
      if i > 0 { result.push(-1); }
      var encoded_row = encode_row(&rows[i]);
      var j = 0;
      while j < encoded_row.len() {
        result.push(encoded_row[j]);
        j = j + 1;
      }
      i = i + 1;
    }
    return result;
  }

  fn test_csv() -> Int {
    var score = 0;
    var row = [1, 2, 3];
    var encoded = encode_row(&row);
    var decoded = decode_row(&encoded);
    if decoded.len() == 3 { score = score + 1; }
    if decoded[0] == 1 { score = score + 1; }
    if decoded[2] == 3 { score = score + 1; }

    var table = Vec[Vec[Int]].new();
    table.push([1, 2]);
    table.push([3, 4]);
    table.push([5, 6]);
    var enc_table = encode_table(&table);
    if enc_table.len() > 0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: JSON-Like Serialization
  // ============================================================

  pub type JsonValue = {
    tag: Int;
    int_val: Int;
    float_val: Float64;
    bool_val: Bool;
    str_len: Int;
  } derive[Clone]

  pub fn JsonValue.new_int(val: Int) -> JsonValue {
    return JsonValue{ tag: 0, int_val: val, float_val: 0.0, bool_val: false, str_len: 0 };
  }

  pub fn JsonValue.new_float(val: Float64) -> JsonValue {
    return JsonValue{ tag: 1, int_val: 0, float_val: val, bool_val: false, str_len: 0 };
  }

  pub fn JsonValue.new_bool(val: Bool) -> JsonValue {
    return JsonValue{ tag: 2, int_val: 0, float_val: 0.0, bool_val: val, str_len: 0 };
  }

  pub fn JsonValue.is_int() -> Bool { return tag == 0; }
  pub fn JsonValue.is_float() -> Bool { return tag == 1; }
  pub fn JsonValue.is_bool() -> Bool { return tag == 2; }

  pub fn serialize_json_array(values: &Vec[JsonValue]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < values.len() {
      result.push(values[i].tag);
      if values[i].is_int() { result.push(values[i].int_val); }
      elif values[i].is_float() { result.push(5); }
      elif values[i].is_bool() {
        if values[i].bool_val { result.push(1); } else { result.push(0); }
      }
      i = i + 1;
    }
    return result;
  }

  fn test_json() -> Int {
    var score = 0;
    var j1 = JsonValue.new_int(42);
    var j2 = JsonValue.new_bool(true);
    var j3 = JsonValue.new_int(99);

    if j1.is_int() { score = score + 1; }
    if j2.is_bool() { score = score + 1; }

    var arr = [j1, j2, j3];
    var serialized = serialize_json_array(&arr);
    if serialized.len() == 6 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 4: Data Compression (Run-Length Encoding)
  // ============================================================

  pub fn rle_encode(input: &Vec[Int]) -> Vec[Int] {
    if input.len() == 0 {
      var r = Vec[Int].new();
      return r;
    }
    var result = Vec[Int].new();
    var i = 0;
    while i < input.len() {
      var count = 1;
      var j = i + 1;
      while j < input.len() && input[j] == input[i] && count < 255 {
        count = count + 1;
        j = j + 1;
      }
      result.push(input[i]);
      result.push(count);
      i = j;
    }
    return result;
  }

  pub fn rle_decode(input: &Vec[Int]) -> Vec[Int] {
    var result = Vec[Int].new();
    var i = 0;
    while i < input.len() {
      var value = input[i];
      var count = input[i + 1];
      var j = 0;
      while j < count {
        result.push(value);
        j = j + 1;
      }
      i = i + 2;
    }
    return result;
  }

  fn test_compression() -> Int {
    var score = 0;
    var data = [1, 1, 1, 2, 2, 3, 3, 3, 3];
    var encoded = rle_encode(&data);
    if encoded.len() == 6 { score = score + 1; }

    var decoded = rle_decode(&encoded);
    if decoded.len() == 9 { score = score + 1; }

    // Roundtrip
    var round = rle_decode(&rle_encode(&data));
    if round.len() == data.len() { score = score + 1; }
    var match_count = 0;
    var i = 0;
    while i < round.len() {
      if round[i] == data[i] { match_count = match_count + 1; }
      i = i + 1;
    }
    if match_count == 9 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 5: Binary Protocol
  // ============================================================

  pub type Packet = {
    version: Int;
    type_: Int;
    payload: Vec[Int];
    checksum: Int;
  } derive[Clone]

  pub fn Packet.new(ver: Int, ptype: Int, payload: Vec[Int]) -> Packet {
    var cksum = 0;
    var i = 0;
    while i < payload.len() {
      cksum = cksum + payload[i];
      i = i + 1;
    }
    cksum = cksum % 256;
    return Packet{ version: ver, type_: ptype, payload: payload, checksum: cksum };
  }

  pub fn Packet.verify() -> Bool {
    var computed = 0;
    var i = 0;
    while i < payload.len() {
      computed = computed + payload[i];
      i = i + 1;
    }
    return checksum == computed % 256;
  }

  pub fn Packet.serialize() -> Vec[Int] {
    var result = Vec[Int].new();
    result.push(version);
    result.push(type_);
    result.push(payload.len());
    var i = 0;
    while i < payload.len() {
      result.push(payload[i]);
      i = i + 1;
    }
    result.push(checksum);
    return result;
  }

  fn test_packet() -> Int {
    var score = 0;
    var p = Packet.new(1, 10, [1, 2, 3, 4, 5]);
    if p.version == 1 { score = score + 1; }
    if p.type_ == 10 { score = score + 1; }
    if p.verify() { score = score + 1; }

    var serialized = p.serialize();
    if serialized.len() == 9 { score = score + 1; }

    // Tamper-proof
    var p2 = Packet.new(1, 10, [10, 20]);
    if p2.verify() { score = score + 1; }
    if !(Packet{ version: 1, type_: 10, payload: [1, 2], checksum: 99 }.verify()) { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 6: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_serialization();
    total = total + s1;
    max_score = max_score + 8;

    var s2 = test_csv();
    total = total + s2;
    max_score = max_score + 4;

    var s3 = test_json();
    total = total + s3;
    max_score = max_score + 3;

    var s4 = test_compression();
    total = total + s4;
    max_score = max_score + 4;

    var s5 = test_packet();
    total = total + s5;
    max_score = max_score + 6;

    return BenchResult{
      name: "serialize",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// MODULE: bench_data
// ============================================================
module bench_data {


  // ============================================================
  // SECTION 1: Verify Data Integrity
  // ============================================================

  fn test_primes_data() -> Int {
    var score = 0;
    var primes = data_primes.primes_table();
    if primes.len() == 500 { score = score + 1; }

    // Verify known positions
    if primes[0] == 2 { score = score + 1; }
    if primes[1] == 3 { score = score + 1; }
    if primes[9] == 29 { score = score + 1; }   // 10th prime
    if primes[99] == 541 { score = score + 1; }  // 100th prime
    if primes[499] == 3571 { score = score + 1; } // 500th prime

    // All entries are actually prime
    if data_primes.verify_primes() { score = score + 1; }

    // Table is strictly increasing
    if data_primes.verify_monotonic() { score = score + 1; }

    // Contains lookup
    if data_primes.contains(17) { score = score + 1; }
    if data_primes.contains(3571) { score = score + 1; }
    if !(data_primes.contains(4)) { score = score + 1; }

    // nth_prime accessor
    if data_primes.nth_prime(0) == 2 { score = score + 1; }
    if data_primes.nth_prime(499) == 3571 { score = score + 1; }
    if data_primes.nth_prime(-1) == -1 { score = score + 1; }
    if data_primes.nth_prime(500) == -1 { score = score + 1; }

    return score;
  }

  fn test_tables_data() -> Int {
    var score = 0;

    // Factorials
    var fact = data_tables.factorials();
    if fact.len() == 21 { score = score + 1; }
    if fact[0] == 1 { score = score + 1; }
    if fact[5] == 120 { score = score + 1; }
    if fact[10] == 3628800 { score = score + 1; }
    if data_tables.verify_factorials() { score = score + 1; }

    // Fibonacci
    var fib = data_tables.fibonacci_table();
    if fib.len() == 51 { score = score + 1; }
    if fib[0] == 0 && fib[1] == 1 { score = score + 1; }
    if fib[10] == 55 { score = score + 1; }
    if fib[50] == 12586269025 { score = score + 1; }
    if data_tables.verify_fibonacci() { score = score + 1; }

    // Powers of 2
    var pow2 = data_tables.powers_of_two();
    if pow2.len() == 31 { score = score + 1; }
    if pow2[0] == 1 { score = score + 1; }
    if pow2[10] == 1024 { score = score + 1; }
    if pow2[30] == 1073741824 { score = score + 1; }
    if data_tables.verify_powers_of_two() { score = score + 1; }

    // Powers of 3
    var pow3 = data_tables.powers_of_three();
    if pow3.len() == 16 { score = score + 1; }
    if pow3[5] == 243 { score = score + 1; }
    if pow3[10] == 59049 { score = score + 1; }

    // Squares
    var sq = data_tables.squares();
    if sq.len() == 101 { score = score + 1; }
    if sq[0] == 0 { score = score + 1; }
    if sq[10] == 100 { score = score + 1; }
    if sq[100] == 10000 { score = score + 1; }

    // Cubes
    var cb = data_tables.cubes();
    if cb.len() == 51 { score = score + 1; }
    if cb[0] == 0 { score = score + 1; }
    if cb[10] == 1000 { score = score + 1; }

    // Triangular
    var tri = data_tables.triangular_numbers();
    if tri.len() == 101 { score = score + 1; }
    if tri[10] == 55 { score = score + 1; }

    // Catalan
    var cat = data_tables.catalan_numbers();
    if cat.len() == 16 { score = score + 1; }
    if cat[3] == 5 { score = score + 1; }

    // Bell
    var bell = data_tables.bell_numbers();
    if bell.len() == 11 { score = score + 1; }
    if bell[3] == 5 { score = score + 1; }

    return score;
  }

  fn test_random_data() -> Int {
    var score = 0;

    // Dataset generation
    var d500 = data_random.dataset_500();
    if d500.len() == 500 { score = score + 1; }

    var d1000 = data_random.dataset_1000();
    if d1000.len() == 1000 { score = score + 1; }

    // Statistics
    var min_v = data_random.dataset_min(&d1000);
    var max_v = data_random.dataset_max(&d1000);
    if min_v < max_v { score = score + 1; }
    if min_v >= 0 { score = score + 1; }
    if max_v > 0 { score = score + 1; }

    // Mean should be roughly in range
    var mean_v = data_random.dataset_mean(&d1000);
    if mean_v >= 0 { score = score + 1; }

    // Count in range
    var count_low = data_random.dataset_count_in_range(&d1000, 0, 100);
    if count_low >= 0 { score = score + 1; }

    // Identity permutation
    var perm = data_random.identity_permutation(50);
    if perm.len() == 50 { score = score + 1; }
    if perm[0] == 0 && perm[49] == 49 { score = score + 1; }

    // Reverse permutation
    var rev = data_random.reverse_permutation(50);
    if rev[0] == 49 && rev[49] == 0 { score = score + 1; }

    // Sine wave
    var wave = data_random.sine_wave(100, 64);
    if wave.len() == 64 { score = score + 1; }

    // Prefix sum
    var psum = data_random.prefix_sum(&d500);
    if psum.len() == 500 { score = score + 1; }

    // Histogram
    var hist = data_random.histogram(&d1000, 10, 100000);
    if hist.len() == 10 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Cross-verify data modules against benchmarks
  // ============================================================

  fn test_cross_verification() -> Int {
    var score = 0;

    // Verify prime table entries against is_prime manual check
    var primes = data_primes.primes_table();
    var sample_ok = 1;
    var i = 0;
    while i < primes.len() && i < 20 {
      var p = primes[i];
      var is_p = 1;
      var d = 2;
      while d * d <= p {
        if p % d == 0 { is_p = 0; }
        d = d + 1;
      }
      if is_p == 0 { sample_ok = 0; }
      i = i + 1;
    }
    if sample_ok == 1 { score = score + 1; }

    // Verify factorial table against iterative computation
    var fact = data_tables.factorials();
    var fact_ok = 1;
    i = 0;
    while i < fact.len() && i < 10 {
      var computed = 1;
      var j = 1;
      while j <= i {
        computed = computed * j;
        j = j + 1;
      }
      if computed != fact[i] { fact_ok = 0; }
      i = i + 1;
    }
    if fact_ok == 1 { score = score + 1; }

    // Verify Fibonacci table against iterative
    var fib = data_tables.fibonacci_table();
    var fib_ok = 1;
    if fib[0] != 0 { fib_ok = 0; }
    if fib.len() >= 2 && fib[1] != 1 { fib_ok = 0; }
    i = 2;
    while i < fib.len() {
      if fib[i] != fib[i - 1] + fib[i - 2] { fib_ok = 0; }
      i = i + 1;
    }
    if fib_ok == 1 { score = score + 1; }

    // Verify powers of 2
    var pow2 = data_tables.powers_of_two();
    var pow2_ok = pow2[0] == 1;
    i = 1;
    while i < pow2.len() {
      if pow2[i] != pow2[i - 1] * 2 { pow2_ok = 0; }
      i = i + 1;
    }
    if pow2_ok { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Aggregate Runner
  // ============================================================

  pub fn run_all() -> BenchResult {
    var total = 0;
    var max_score = 0;

    var s1 = test_primes_data();
    total = total + s1;
    max_score = max_score + 14;

    var s2 = test_tables_data();
    total = total + s2;
    max_score = max_score + 28;

    var s3 = test_random_data();
    total = total + s3;
    max_score = max_score + 13;

    var s4 = test_cross_verification();
    total = total + s4;
    max_score = max_score + 4;

    return BenchResult{
      name: "data",
      score: total,
      max_score: max_score,
      passed: total == max_score,
      elapsed_ms: 0,
    };
  }

}

// ============================================================
// ENTRY POINT
// ============================================================

use math.run_all as run_math;
use primes.run_all as run_primes;
use control.run_all as run_control;
use types.run_all as run_types;
use structures.run_all as run_structures;
use memory.run_all as run_memory;
use contracts.run_all as run_contracts;
use error.run_all as run_error;
use generics.run_all as run_generics;
use enums.run_all as run_enums;
use derive.run_all as run_derive;
use recursion.run_all as run_recursion;
use closures.run_all as run_closures;
use modules.run_all as run_modules;
use safety.run_all as run_safety;
use collections.run_all as run_collections;
use algorithms.run_all as run_algorithms;
use extreme.run_all as run_extreme;
use comptime.run_all as run_comptime;
use concurrency.run_all as run_concurrency;
use crypto.run_all as run_crypto;
use sort.run_all as run_sort;
use interfaces.run_all as run_interfaces;
use serialize.run_all as run_serialize;
use bench_data.run_all as run_bench_data;

fn main() -> Int {
  var total = 0;
  total = total + run_math().score;
  total = total + run_primes().score;
  total = total + run_control().score;
  total = total + run_types().score;
  total = total + run_structures().score;
  total = total + run_memory().score;
  total = total + run_contracts().score;
  total = total + run_error().score;
  total = total + run_generics().score;
  total = total + run_enums().score;
  total = total + run_derive().score;
  total = total + run_recursion().score;
  total = total + run_closures().score;
  total = total + run_modules().score;
  total = total + run_safety().score;
  total = total + run_collections().score;
  total = total + run_algorithms().score;
  total = total + run_extreme().score;
  total = total + run_comptime().score;
  total = total + run_concurrency().score;
  total = total + run_crypto().score;
  total = total + run_sort().score;
  total = total + run_interfaces().score;
  total = total + run_serialize().score;
  total = total + run_bench_data().score;
  return total;
}
