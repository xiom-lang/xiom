// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use xiom.io;
use xiom.math;
use xiom.time;

type TcpHeader = {
  src_port: Int;
  dst_port: Int;
  seq_num: Int;
  ack_num: Int;
  data_offset: Int;
  reserved: Int;
  ns: Int;
  flags: Int;
  window: Int;
  checksum: Int;
  urgent_ptr: Int;
}

fn read_u16_be(data: &Vec[Int], offset: Int) -> Int {
  return math.bit_or(math.shl(data[offset], 8), data[offset + 1]);
}

fn read_u32_be(data: &Vec[Int], offset: Int) -> Int {
  let hi: Int = math.bit_or(
    math.shl(data[offset], 24),
    math.shl(data[offset + 1], 16)
  );
  let lo: Int = math.bit_or(
    math.shl(data[offset + 2], 8),
    data[offset + 3]
  );
  return math.bit_or(hi, lo);
}

fn parse_packet(data: &Vec[Int], len: Int) -> TcpHeader {
  if len < 20 {
    return TcpHeader{
      src_port: 0, dst_port: 0, seq_num: 0, ack_num: 0,
      data_offset: 0, reserved: 0, ns: 0, flags: 0,
      window: 0, checksum: 0, urgent_ptr: 0
    };
  }
  let bo: Int = data[12];
  return TcpHeader{
    src_port: read_u16_be(data, 0),
    dst_port: read_u16_be(data, 2),
    seq_num: read_u32_be(data, 4),
    ack_num: read_u32_be(data, 8),
    data_offset: math.bit_and(math.shr(bo, 4), 15),
    reserved: math.bit_and(math.shr(bo, 1), 7),
    ns: math.bit_and(bo, 1),
    flags: data[13],
    window: read_u16_be(data, 14),
    checksum: read_u16_be(data, 16),
    urgent_ptr: read_u16_be(data, 18)
  };
}

fn compute_checksum(data: &Vec[Int], len: Int) -> Int {
  var sum: Int = 0;
  var i: Int = 0;
  while i < len - 1 {
    sum = sum + read_u16_be(data, i);
    i = i + 2;
  }
  if math.bit_and(len, 1) != 0 {
    sum = sum + math.shl(data[len - 1], 8);
  }
  while math.shr(sum, 16) != 0 {
    sum = math.bit_and(sum, 65535) + math.shr(sum, 16);
  }
  let inverted: Int = math.bit_xor(sum, 65535);
  return math.bit_and(inverted, 65535);
}

fn main() -> Int {
  var packet = Vec[Int].with_capacity(40);
  var idx: Int = 0;
  while idx < 40 {
    packet.push(0);
    idx = idx + 1;
  }
  packet[0] = 31;
  packet[1] = 144;
  packet[2] = 0;
  packet[3] = 80;
  packet[4] = 0;
  packet[5] = 0;
  packet[6] = 0;
  packet[7] = 1;
  packet[8] = 0;
  packet[9] = 0;
  packet[10] = 0;
  packet[11] = 0;
  packet[12] = 80;
  packet[13] = 2;
  packet[14] = 127;
  packet[15] = 255;
  packet[16] = 0;
  packet[17] = 0;
  packet[18] = 0;
  packet[19] = 0;

  var i: Int = 0;
  while i < 1000000 {
    packet[4] = math.bit_and(math.shr(i, 24), 255);
    packet[5] = math.bit_and(math.shr(i, 16), 255);
    packet[6] = math.bit_and(math.shr(i, 8), 255);
    packet[7] = math.bit_and(i, 255);
    let csum: Int = compute_checksum(&packet, 20);
    packet[16] = math.bit_and(math.shr(csum, 8), 255);
    packet[17] = math.bit_and(csum, 255);
    let _ = parse_packet(&packet, 20);
    i = i + 1;
  }

  io.println("OK");
  return 0;
}
