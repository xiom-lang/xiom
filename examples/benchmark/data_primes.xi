// XIOM -- Primes Data Module
// Large static table of prime numbers for benchmark processing.
// First 500 primes -- used to verify prime generation algorithms.
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.data_primes

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
