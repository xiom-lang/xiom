// XIOM -- Benchmark Suite Entry Point
// Orchestrates all 20 stress-test modules and reports results.
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module benchmark.main

use benchmark.math;
use benchmark.primes;
use benchmark.control;
use benchmark.types;
use benchmark.structures;
use benchmark.memory;
use benchmark.contracts;
use benchmark.error;
use benchmark.generics;
use benchmark.enums;
use benchmark.derive;
use benchmark.recursion;
use benchmark.closures;
use benchmark.modules;
use benchmark.safety;
use benchmark.collections;
use benchmark.algorithms;
use benchmark.extreme;
use benchmark.comptime;
use benchmark.concurrency;
use benchmark.crypto;
use benchmark.sort;
use benchmark.interfaces;
use benchmark.serialize;
use benchmark.bench_data;
use benchmark.ownership;
use benchmark.borrow;
use benchmark.contracts_hard;
use benchmark.generics_hard;
use benchmark.monomorph;

// --- Benchmark Result Type ---
pub type BenchResult = {
  name: Str;
  score: Int;
  max_score: Int;
  passed: Bool;
  elapsed_ms: Int;
} derive[Clone]

// --- Result Aggregation ---
pub type SuiteResult = {
  results: Vec[BenchResult];
  total_score: Int;
  total_max: Int;
  total_elapsed: Int;
} derive[Clone]

pub fn SuiteResult.new() -> SuiteResult {
  return SuiteResult{
    results: Vec[BenchResult].new(),
    total_score: 0,
    total_max: 0,
    total_elapsed: 0,
  };
}

pub fn SuiteResult.add_result(result: BenchResult) {
  results.push(result);
  total_score = total_score + result.score;
  total_max = total_max + result.max_score;
  total_elapsed = total_elapsed + result.elapsed_ms;
}

// --- Progress Reporter ---
fn report_start(name: Str) {
  var marker = "[";
  marker = marker;
  name.len();
}

// --- Main ---
fn main() -> Int {
  var suite = SuiteResult.new();

  // ---- MODULE 1: Math Stress ----
  report_start("math");
  var r1 = math.run_all();
  suite.add_result(r1);

  // ---- MODULE 2: Primes Stress ----
  report_start("primes");
  var r2 = primes.run_all();
  suite.add_result(r2);

  // ---- MODULE 3: Control Flow Stress ----
  report_start("control");
  var r3 = control.run_all();
  suite.add_result(r3);

  // ---- MODULE 4: Type System Stress ----
  report_start("types");
  var r4 = types.run_all();
  suite.add_result(r4);

  // ---- MODULE 5: Data Structures Stress ----
  report_start("structures");
  var r5 = structures.run_all();
  suite.add_result(r5);

  // ---- MODULE 6: Memory/Ownership Stress ----
  report_start("memory");
  var r6 = memory.run_all();
  suite.add_result(r6);

  // ---- MODULE 7: Contracts Stress ----
  report_start("contracts");
  var r7 = contracts.run_all();
  suite.add_result(r7);

  // ---- MODULE 8: Error Handling Stress ----
  report_start("error");
  var r8 = error.run_all();
  suite.add_result(r8);

  // ---- MODULE 9: Generics Stress ----
  report_start("generics");
  var r9 = generics.run_all();
  suite.add_result(r9);

  // ---- MODULE 10: Enums/Match Stress ----
  report_start("enums");
  var r10 = enums.run_all();
  suite.add_result(r10);

  // ---- MODULE 11: Derive Stress ----
  report_start("derive");
  var r11 = derive.run_all();
  suite.add_result(r11);

  // ---- MODULE 12: Recursion Stress ----
  report_start("recursion");
  var r12 = recursion.run_all();
  suite.add_result(r12);

  // ---- MODULE 13: Closures Stress ----
  report_start("closures");
  var r13 = closures.run_all();
  suite.add_result(r13);

  // ---- MODULE 14: Module Interop Stress ----
  report_start("modules");
  var r14 = modules.run_all();
  suite.add_result(r14);

  // ---- MODULE 15: Borrow Safety Stress ----
  report_start("safety");
  var r15 = safety.run_all();
  suite.add_result(r15);

  // ---- MODULE 16: Collections Stress ----
  report_start("collections");
  var r16 = collections.run_all();
  suite.add_result(r16);

  // ---- MODULE 17: Algorithms Stress ----
  report_start("algorithms");
  var r17 = algorithms.run_all();
  suite.add_result(r17);

  // ---- MODULE 18: Extreme Edge Cases ----
  report_start("extreme");
  var r18 = extreme.run_all();
  suite.add_result(r18);

  // ---- MODULE 19: Compile-Time Stress ----
  report_start("comptime");
  var r19 = comptime.run_all();
  suite.add_result(r19);

  // ---- MODULE 20: Concurrency Patterns ----
  report_start("concurrency");
  var r20 = concurrency.run_all();
  suite.add_result(r20);

  // ---- MODULE 21: Cryptographic Patterns ----
  report_start("crypto");
  var r21 = crypto.run_all();
  suite.add_result(r21);

  // ---- MODULE 22: Sorting Algorithms ----
  report_start("sort");
  var r22 = sort.run_all();
  suite.add_result(r22);

  // ---- MODULE 23: Interface Dispatch ----
  report_start("interfaces");
  var r23 = interfaces.run_all();
  suite.add_result(r23);

  // ---- MODULE 24: Serialization/I/O ----
  report_start("serialize");
  var r24 = serialize.run_all();
  suite.add_result(r24);

  // ---- MODULE 25: Data Processing ----
  report_start("data");
  var r25 = bench_data.run_all();
  suite.add_result(r25);

  // ---- MODULE 26: Deep Ownership Chains ----
  report_start("ownership");
  var r26 = ownership.run_all();
  suite.add_result(r26);

  // ---- MODULE 27: Borrow Checker Torture ----
  report_start("borrow");
  var r27 = borrow.run_all();
  suite.add_result(r27);

  // ---- MODULE 28: Massive Contract Verification ----
  report_start("contracts_hard");
  var r28 = contracts_hard.run_all();
  suite.add_result(r28);

  // ---- MODULE 29: Heavy Generics + Comptime ----
  report_start("generics_hard");
  var r29 = generics_hard.run_all();
  suite.add_result(r29);

  // ---- MODULE 30: Huge Monomorphisation Stress ----
  report_start("monomorph");
  var r30 = monomorph.run_all();
  suite.add_result(r30);

  // Final summary
  return suite.total_score;
}
