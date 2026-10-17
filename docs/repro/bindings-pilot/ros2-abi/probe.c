// ros2-abi probe: does the XIOM fn-cast call implement the MS x64 ABI for
// records passed/returned BY VALUE? Build: clang -shared -o probe.dll probe.c
// No CRT dependency: pure integer/pointer arithmetic.
//
// m351 (REL-20261016-2031-bindings-3, B-33): in-repo reproduction bundle.
// This file is compiled by the test (do not commit probe.dll).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifdef _WIN32
#define XIOM_PROBE_EXPORT __declspec(dllexport)
#else
#define XIOM_PROBE_EXPORT __attribute__((visibility("default")))
#endif

typedef struct Alloc5_s {
  void * allocate;
  void * deallocate;
  void * reallocate;
  void * zero_allocate;
  void * state;
} Alloc5;

typedef struct Pair2_s {
  long long a;
  long long b;
} Pair2;

/* 8-byte aggregate: MS x64 passes/returns this in a SINGLE integer register
   (i64). LLVM's naked %struct lowering would split it into ecx/edx -- the
   m351 small-aggregate coercion loads/stores the raw i64 bytes. */
typedef struct Pair2i_s {
  int a;
  int b;
} Pair2i;

XIOM_PROBE_EXPORT long long probe_sum5(Alloc5 al) {
  return (long long)al.allocate + (long long)al.deallocate +
         (long long)al.reallocate + (long long)al.zero_allocate +
         (long long)al.state;
}

XIOM_PROBE_EXPORT Alloc5 probe_make5(long long base) {
  Alloc5 al;
  al.allocate = (void *)(base + 1);
  al.deallocate = (void *)(base + 2);
  al.reallocate = (void *)(base + 3);
  al.zero_allocate = (void *)(base + 4);
  al.state = (void *)(base + 5);
  return al;
}

XIOM_PROBE_EXPORT long long probe_sum2(Pair2 p, long long bias) {
  return p.a * 1000 + p.b + bias;
}

XIOM_PROBE_EXPORT Pair2 probe_make2(long long base) {
  Pair2 p;
  p.a = base + 7;
  p.b = base + 9;
  return p;
}

XIOM_PROBE_EXPORT long long probe_mul(long long x, long long y) {
  return x * y;
}

XIOM_PROBE_EXPORT long long probe_sum2i(Pair2i p, long long bias) {
  return (long long)p.a * 1000 + (long long)p.b + bias;
}

XIOM_PROBE_EXPORT Pair2i probe_make2i(long long base) {
  Pair2i p;
  p.a = (int)base + 7;
  p.b = (int)base + 9;
  return p;
}

XIOM_PROBE_EXPORT long long probe_sum5p(Alloc5 * al) {
  return (long long)al->allocate + (long long)al->deallocate + (long long)al->reallocate + (long long)al->zero_allocate + (long long)al->state;
}


XIOM_PROBE_EXPORT long long probe_ws9(void * ws, long long a, long long b, long long c, long long d, long long e, long long f, void * ctx, Alloc5 al) {
  long long s = (long long)ws + a + b + c + d + e + f + (long long)ctx;
  s += (long long)al.allocate + (long long)al.deallocate + (long long)al.reallocate + (long long)al.zero_allocate + (long long)al.state;
  return s;
}
XIOM_PROBE_EXPORT long long probe_t7(void * t, void * cl, void * x, long long p, long long cb, Alloc5 al, long long au) {
  long long s = (long long)t + (long long)cl + (long long)x + p + cb + au;
  s += (long long)al.allocate + (long long)al.state;
  return s;
}
XIOM_PROBE_EXPORT long long probe_c3(long long ty, void * cl, Alloc5 al) {
  return ty + (long long)cl + (long long)al.allocate + (long long)al.state;
}
