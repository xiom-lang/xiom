// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use xiom.io;
use xiom.math;

const POOL_SIZE: Int = 1048576;
const MIN_BLOCK: Int = 64;
const MAX_ORDER: Int = 14;
const HEADER_SIZE: Int = 24;

type BuddyAllocator = {
  pool: Vec[Int];
  free_lists: Vec[Int];
  initialized: Bool;
}

fn order_for_size(size: Int) -> Int {
  var block_size: Int = MIN_BLOCK;
  var order: Int = 0;
  while block_size < size + HEADER_SIZE {
    block_size = math.shl(block_size, 1);
    order = order + 1;
  }
  if order > MAX_ORDER {
    return MAX_ORDER;
  }
  return order;
}

fn block_size_for_order(order: Int) -> Int {
  return math.shl(MIN_BLOCK, order);
}

fn header_read_free(a: &BuddyAllocator, offset: Int) -> Bool {
  if offset < 0 { return false; }
  if offset >= POOL_SIZE { return false; }
  return a.pool[offset] != 0;
}

fn header_write_free(a: &mut BuddyAllocator, offset: Int, value: Bool) {
  if offset < 0 || offset >= POOL_SIZE { return; }
  if value {
    a.pool[offset] = 1;
  } else {
    a.pool[offset] = 0;
  }
}

fn header_read_order(a: &BuddyAllocator, offset: Int) -> Int {
  if offset + 16 > POOL_SIZE { return 0; }
  var result: Int = 0;
  var i: Int = 0;
  while i < 8 {
    result = math.bit_or(result, math.shl(a.pool[offset + 8 + i], i * 8));
    i = i + 1;
  }
  return result;
}

fn header_write_order(a: &mut BuddyAllocator, offset: Int, value: Int) {
  if offset + 16 > POOL_SIZE { return; }
  var i: Int = 0;
  while i < 8 {
    a.pool[offset + 8 + i] = math.bit_and(math.shr(value, i * 8), 255);
    i = i + 1;
  }
}

fn header_read_next(a: &BuddyAllocator, offset: Int) -> Int {
  if offset + 24 > POOL_SIZE { return -1; }
  var result: Int = 0;
  var i: Int = 0;
  while i < 8 {
    result = math.bit_or(result, math.shl(a.pool[offset + 16 + i], i * 8));
    i = i + 1;
  }
  return result;
}

fn header_write_next(a: &mut BuddyAllocator, offset: Int, value: Int) {
  if offset + 24 > POOL_SIZE { return; }
  var i: Int = 0;
  while i < 8 {
    a.pool[offset + 16 + i] = math.bit_and(math.shr(value, i * 8), 255);
    i = i + 1;
  }
}

fn BuddyAllocator.split_block(order: Int) -> Int {
  if order > MAX_ORDER { return -1; }
  if free_lists[order] >= 0 {
    let offset: Int = free_lists[order];
    let next: Int = header_read_next(self, offset);
    free_lists[order] = next;
    header_write_free(self, offset, false);
    return offset;
  }
  if order == MAX_ORDER { return -1; }
  let bigger: Int = split_block(order + 1);
  if bigger < 0 { return -1; }
  let half: Int = block_size_for_order(order);
  let buddy_offset: Int = bigger + half;
  header_write_free(self, buddy_offset, true);
  header_write_order(self, buddy_offset, order);
  header_write_next(self, buddy_offset, free_lists[order]);
  free_lists[order] = buddy_offset;
  header_write_free(self, bigger, false);
  header_write_order(self, bigger, order);
  return bigger;
}

fn BuddyAllocator.init() {
  if initialized { return; }
  var i: Int = 0;
  while i < POOL_SIZE {
    pool.push(0);
    i = i + 1;
  }
  i = 0;
  while i <= MAX_ORDER {
    free_lists.push(-1);
    i = i + 1;
  }
  header_write_free(self, 0, true);
  header_write_order(self, 0, MAX_ORDER);
  header_write_next(self, 0, -1);
  free_lists[MAX_ORDER] = 0;
  initialized = true;
}

fn BuddyAllocator.alloc(size: Int) -> Int {
  if !initialized {
    init();
  }
  if size == 0 { return -1; }
  let order: Int = order_for_size(size);
  let offset: Int = split_block(order);
  if offset < 0 { return -1; }
  return offset + HEADER_SIZE;
}

fn BuddyAllocator.free(ptr: Int) {
  if ptr < HEADER_SIZE { return; }
  let block: Int = ptr - HEADER_SIZE;
  header_write_free(self, block, true);
  var current: Int = block;
  var cur_order: Int = header_read_order(self, current);
  while cur_order < MAX_ORDER {
    let bsize: Int = block_size_for_order(cur_order);
    let buddy_addr: Int = math.bit_xor(current, bsize);
    if buddy_addr + bsize > POOL_SIZE { break; }
    if buddy_addr < 0 { break; }
    let buddy_free: Bool = header_read_free(self, buddy_addr);
    let buddy_order: Int = header_read_order(self, buddy_addr);
    if !buddy_free || buddy_order != cur_order { break; }
    var prev_idx: Int = free_lists[cur_order];
    var prev: Int = -1;
    while prev_idx >= 0 {
      if prev_idx == buddy_addr {
        break;
      }
      prev = prev_idx;
      prev_idx = header_read_next(self, prev_idx);
    }
    if prev_idx == buddy_addr {
      let buddy_next: Int = header_read_next(self, buddy_addr);
      if prev < 0 {
        free_lists[cur_order] = buddy_next;
      } else {
        header_write_next(self, prev, buddy_next);
      }
    }
    if current > buddy_addr {
      current = buddy_addr;
    }
    cur_order = cur_order + 1;
    header_write_order(self, current, cur_order);
  }
  header_write_next(self, current, free_lists[cur_order]);
  free_lists[cur_order] = current;
}

fn main() {
  var allocator = BuddyAllocator{
    pool: Vec[Int].with_capacity(POOL_SIZE),
    free_lists: Vec[Int].with_capacity(MAX_ORDER + 1),
    initialized: false
  };
  allocator.init();

  let N: Int = 10000;
  var ptrs = Vec[Int].with_capacity(N);
  var i: Int = 0;
  while i < N {
    let sz: Int = 64 + ((i * 997) % 4096);
    ptrs.push(allocator.alloc(sz));
    i = i + 1;
  }
  i = 0;
  while i < N {
    allocator.free(ptrs[i]);
    i = i + 2;
  }
  i = 0;
  while i < N {
    let sz: Int = 128 + ((i * 503) % 2048);
    ptrs[i] = allocator.alloc(sz);
    i = i + 2;
  }
  i = 0;
  while i < N {
    allocator.free(ptrs[i]);
    i = i + 1;
  }
  io.println("OK");
}
