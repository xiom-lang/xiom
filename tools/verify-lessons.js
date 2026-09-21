// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

const fs = require('fs');
const path = require('path');

const BASE = path.resolve(__dirname, '..', 'xiom-playground', 'lessons');
const DIRS = ['L5-patterns', 'L6-engineering', 'L7-mastery', 'L8-ecosystem'];

// Known string-returning function/method names (same as fix script)
const STR_RETURNING_FUNCS = new Set([
  'error_message', 'get_user_name', 'display', 'greet', 'execute', 'name',
  'label', 'read', 'summary', 'describe', 'to_text',
  'biggest_expense', 'caesar_decrypt', 'caesar_encrypt', 'cat_name',
  'cell_display', 'check_budget', 'class_name', 'condition_icon',
  'decode_morse', 'decode_word', 'encode_word', 'find_warmest',
  'format_price', 'format_timer', 'generate_password', 'get_grade',
  'go_north', 'goodbye', 'handle_command', 'hash_password', 'hello',
  'letter_grade', 'letter_grade_from_avg', 'login', 'look',
  'mad_libs', 'piece_char', 'play_random', 'reverse', 'scramble',
  'show_hint', 'to_acronym', 'to_roman', 'trim_spaces', 'truncate',
  'int_to_str', 'float_to_str', 'bool_to_str', 'from_char',
  'str_concat', 'str_len',
]);

function isStringReturningCall(arg) {
  const t = arg.trim();
  const m = t.match(/\.(\w+)\s*\([^)]*\)$/);
  if (m && STR_RETURNING_FUNCS.has(m[1])) return true;
  const s = t.match(/^(\w+)\s*\([^)]*\)$/);
  if (s && STR_RETURNING_FUNCS.has(s[1])) return true;
  return false;
}

let allFiles = [];
for (const dir of DIRS) {
  const dp = path.join(BASE, dir);
  const files = fs.readdirSync(dp).filter(f => f.endsWith('.json'));
  files.forEach(f => allFiles.push(path.join(dir, f)));
}

let broken = 0;
let jsonErrors = 0;

for (const rel of allFiles) {
  const fp = path.join(BASE, rel);
  let data;
  try {
    data = JSON.parse(fs.readFileSync(fp, 'utf8'));
  } catch (e) {
    console.log(`JSON ERROR: ${rel}`);
    jsonErrors++;
    continue;
  }

  for (const field of ['code_template', 'solution']) {
    const code = data[field];
    if (!code) continue;

    // check use xiom.io is first non-blank line
    const lines = code.split('\n');
    let firstLine = '';
    for (let i = 0; i < lines.length; i++) {
      const t = lines[i].trim();
      if (t === '') continue;
      firstLine = t;
      break;
    }
    if (firstLine !== 'use xiom.io;') {
      console.log(`MISSING use xiom.io: ${rel} [${field}] first=${firstLine}`);
      broken++;
    }

    // check io.println calls (use same balanced-paren regex as fix script)
    const ioRegex = /io\.println\(((?:[^()]|\([^()]*\))*)\)/g;
    let m;
    while ((m = ioRegex.exec(code)) !== null) {
      const arg = m[1];
      if (arg.startsWith('"')) continue;
      if (arg.includes('+')) continue;
      if (/\.to_str\(\)$/.test(arg.trim())) continue;
      if (/\bstring\.\w+_to_str\(/.test(arg)) continue;
      if (/\bstring\.str_concat\(/.test(arg)) continue;
      if (isStringReturningCall(arg)) continue;
      console.log(`UNFIXED io.println: ${rel} [${field}] => io.println(${arg})`);
      broken++;
    }

    // check string.int_to_str(X.to_str()) double conversion
    if (/string\.int_to_str\(.+?\.to_str\(\)\)/.test(code)) {
      console.log(`DOUBLE CONVERSION: ${rel} [${field}]`);
      broken++;
    }
    if (/string\.float_to_str\(.+?\.to_str\(\)\)/.test(code)) {
      console.log(`DOUBLE CONVERSION: ${rel} [${field}]`);
      broken++;
    }
  }
}

console.log(`\nTotal files: ${allFiles.length}`);
console.log(`JSON errors: ${jsonErrors}`);
console.log(`Issues found: ${broken}`);
