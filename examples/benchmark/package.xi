// XIOM — Compiler Stress-Test Benchmark Suite
// Multi-file modular benchmark that pushes every language feature to its limits.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

package {
  name: "xiom-benchmark"
  version: "0.1.0"
  description: "XIOM Compiler Stress-Test Benchmark Suite — multi-module, 10K+ LOC"
  authors: ["XIOM Team"]
  deps: { "xiom-std": "0.1.0" }
  modules: [
    "benchmark.main",
    "benchmark.math",
    "benchmark.primes",
    "benchmark.control",
    "benchmark.types",
    "benchmark.structures",
    "benchmark.memory",
    "benchmark.contracts",
    "benchmark.error",
    "benchmark.generics",
    "benchmark.enums",
    "benchmark.derive",
    "benchmark.recursion",
    "benchmark.closures",
    "benchmark.modules",
    "benchmark.safety",
    "benchmark.collections",
    "benchmark.algorithms",
    "benchmark.extreme",
    "benchmark.comptime",
    "benchmark.concurrency",
    "benchmark.crypto",
    "benchmark.sort",
    "benchmark.interfaces",
    "benchmark.serialize",
    "benchmark.data_primes",
    "benchmark.data_tables",
    "benchmark.data_random",
    "benchmark.bench_data",
  ]
}
