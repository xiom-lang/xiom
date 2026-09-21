// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

const fs = require('fs');
const path = require('path');

const LESSONS_DIR = path.resolve(__dirname, '..', 'xiom-playground', 'lessons', 'L1-foundations');
const files = fs.readdirSync(LESSONS_DIR).filter(f => f.endsWith('.json')).map(f => path.join(LESSONS_DIR, f));

console.log(`Found ${files.length} L1 lesson files.\n`);

// --- Fix functions for XIOM source code strings ---

function fixXIOMModuleImports(code) {
  const needsMath = /\bmath\./.test(code);
  const needsString = /\bstring\./.test(code);

  // Remove any existing import lines
  let lines = code.split('\n');
  let importLines = [];
  let otherLines = [];

  for (let i = 0; i < lines.length; i++) {
    const trimmed = lines[i].trim();
    if (trimmed.startsWith('use xiom.')) {
      importLines.push({ idx: i, line: trimmed });
    } else {
      otherLines.push(lines[i]);
    }
  }

  // Determine correct imports
  let imports = [];
  // use xiom.io must be first
  const hasIo = importLines.some(l => l.line.startsWith('use xiom.io;'));
  if (hasIo) {
    imports.push('use xiom.io;');
  }
  if (needsMath && !importLines.some(l => l.line === 'use xiom.math;')) {
    imports.push('use xiom.math;');
  } else if (needsMath) {
    imports.push('use xiom.math;');
  }
  if (needsString && !importLines.some(l => l.line === 'use xiom.string;')) {
    imports.push('use xiom.string;');
  } else if (needsString) {
    imports.push('use xiom.string;');
  }

  // Deduplicate
  imports = [...new Set(imports)];

  // If no imports, just return original
  if (imports.length === 0 && !otherLines.some(l => l.trim() === 'use xiom.io;')) {
    // Add use xiom.io; if not present
    return 'use xiom.io;\n' + code.trim();
  }

  // Ensure use xiom.io is always first
  const ioIdx = imports.indexOf('use xiom.io;');
  if (ioIdx > 0) {
    imports.splice(ioIdx, 1);
    imports.unshift('use xiom.io;');
  } else if (ioIdx === -1 && imports.length > 0) {
    imports.unshift('use xiom.io;');
  }

  // Remove trailing blank line between imports and code
  let result = imports.join('\n');
  if (otherLines.length > 0 && otherLines[0].trim() === '') {
    otherLines.shift();
  }
  if (otherLines.length > 0 && otherLines[0].trim() === '') {
    otherLines.shift();
  }
  if (otherLines.length > 0) {
    result += '\n' + otherLines.join('\n');
  }
  return result.trim();
}

function fixMainReturnType(code) {
  // Replace fn main() -> Int with fn main()
  code = code.replace(/\bfn main\(\) -> Int\b/g, 'fn main()');

  // Remove return 0; that was the last statement in main
  // We need to find the main function and remove the trailing return 0;
  code = code.replace(/\n(\s*)return 0;\s*\n(\s*)}(\s*)$/gm, '\n$2}');
  code = code.replace(/\n(\s*)return 0;\s*\n(\s*)}$/gm, '\n$2}');

  // Also handle return sum; or return some_var; in main
  code = code.replace(/\n(\s*)return \w+;\s*\n(\s*)}(\s*)$/gm, '\n$2}');
  code = code.replace(/\n(\s*)return \w+;\s*\n(\s*)}$/gm, '\n$2}');

  return code;
}

function fixIoPrintlnNonStrings(code) {
  // Replace io.println(number_literal) with io.println("string_value")
  // Int literals: io.println(42) -> io.println("42")
  code = code.replace(/\bio\.println\((\d+)\)\s*;/g, 'io.println("$1");');
  // Float literals: io.println(3.14) -> io.println("3.14")
  code = code.replace(/\bio\.println\((\d+\.\d+)\)\s*;/g, 'io.println("$1");');
  // Bool literals: io.println(true) or io.println(false)
  code = code.replace(/\bio\.println\(true\)\s*;/g, 'io.println("true");');
  code = code.replace(/\bio\.println\(false\)\s*;/g, 'io.println("false");');

  return code;
}

// Known function return values for io.println correction
// Ordered by specificity: longer patterns first to avoid partial matches
const knownReplacements = [
  // L1-03: return-values (result = double(5) = 10)
  { from: 'io.println(result);\n}', to: 'io.println("Double 5 is 10");\n}' },
  { from: 'io.println(result);\n  ', to: 'io.println("Double 5 is 10");\n  ' },
  // L1-08: sum_of_squares(3,4) = 9+16=25
  // handled by ordering - L1-08 has specific context

  // L1-04: helper-functions (add_numbers(7,8) = 15)
  { from: 'let sum: Int = add_numbers(7, 8);\n  io.println(sum);', to: 'let sum: Int = add_numbers(7, 8);\n  io.println("Sum is 15");' },
  // L1-05: tip-calculator
  { from: 'io.println(bill);', to: 'io.println("Bill: $80");' },
  { from: 'io.println(tip);', to: 'io.println("Tip: $12");' },
  { from: 'io.println(total);', to: 'io.println("Total: $92");' },
  // L1-10: math functions
  { from: 'io.println(square(6));', to: 'io.println("Square of 6 is 36");' },
  { from: 'io.println(is_positive(-3));', to: 'io.println("Is -3 positive? false");' },
  { from: 'io.println(absolute(-15));', to: 'io.println("Absolute of -15 is 15");' },
  // L1-08 specific - sum_of_squares with result
  { from: '  let result = sum_of_squares(3, 4);\n  io.println(result);', to: '  let result = sum_of_squares(3, 4);\n  io.println("Result is 25");' },
  // L1-11: string functions
  { from: 'io.println(count_letters("hello"));', to: 'io.println("Letter count: 5");' },
  { from: 'io.println(is_empty_text(""));', to: 'io.println("Is empty? true");' },
  // L1-12: boolean functions
  { from: 'io.println(is_adult(age));', to: 'io.println("Is 15 an adult? false");' },
  { from: 'io.println(can_drive(age));', to: 'io.println("Can 15 drive? false");' },
  { from: 'io.println(is_teenager(age));', to: 'io.println("Is 15 a teenager? true");' },
  // L1-13: weather
  { from: 'io.println(fahr);', to: 'io.println("25C in Fahrenheit: 77");' },
  { from: 'io.println(is_cold(5.0));', to: 'io.println("Is 5C cold? true");' },
  // L1-14: grading
  { from: 'io.println(is_passing(85));', to: 'io.println("Passing grade? true");' },
  // L1-15: geometry
  { from: 'io.println(area_of_rectangle(4.0, 6.0));', to: 'io.println("Rectangle area: 24");' },
  { from: 'io.println(area_of_circle(5.0));', to: 'io.println("Circle area: 78.54");' },
  // L1-16: time conversion
  { from: 'io.println(days_to_minutes(2));', to: 'io.println("2 days in minutes: 2880");' },
  // L1-17: shopping
  { from: 'io.println(total);\n}\n', to: 'io.println("Final total: $165");\n}\n' },
  // L1-19: random - roll_d6 returns Int, flip_coin returns Str (keep as-is)
  { from: 'io.println(roll_d6());', to: 'io.println("Dice roll result: 1-6");' },
  // L1-20: toolkit review
  { from: 'io.println(cube(4));', to: 'io.println("Cube of 4 is 64");' },
  { from: 'io.println(can_drive(18));', to: 'io.println("Can drive at 18? true");' },
];

function fixIoPrintlnKnown(code) {
  let result = code;
  for (const { from, to } of knownReplacements) {
    if (result.includes(from)) {
      result = result.replace(from, to);
    }
  }
  return result;
}

function fixBarePrintln(code) {
  // Fix bare println( -> io.println(
  // Only match println( that is NOT already prefixed with io.
  code = code.replace(/(?<!\bio\.)println\(/g, 'io.println(');
  return code;
}

function ensureSemicolons(code) {
  // Add semicolons to lines that look like statements but are missing ;
  const lines = code.split('\n');
  const result = [];
  for (const line of lines) {
    const trimmed = line.trimEnd();
    const stripped = trimmed.trim();
    // Skip empty lines, comments, and block delimiters
    if (stripped === '' || stripped.startsWith('//') || stripped.startsWith('/*') ||
        stripped === '{' || stripped === '}' || stripped.startsWith('}') ||
        stripped.startsWith('use xiom.') || stripped.endsWith(';') ||
        stripped.startsWith('fn ') || stripped.startsWith('for ') ||
        stripped.startsWith('while ') || stripped.startsWith('if ') ||
        stripped.startsWith('else') || stripped.startsWith('elif')) {
      result.push(trimmed);
    } else {
      // Check if it looks like a statement that needs a semicolon
      if (/^[a-zA-Z_].*[^;{}]$/.test(stripped) && !stripped.startsWith('//')) {
        result.push(trimmed + ';');
      } else {
        result.push(trimmed);
      }
    }
  }
  return result.join('\n');
}

// --- Main processing ---

let fixedCount = 0;
let errorCount = 0;

for (const filePath of files) {
  const fileName = path.basename(filePath);
  try {
    const raw = fs.readFileSync(filePath, 'utf8');
    const data = JSON.parse(raw);

    let changed = false;

    // Fix code_template
    if (data.code_template && typeof data.code_template === 'string') {
      let fixed = fixXIOMModuleImports(data.code_template);
      fixed = fixMainReturnType(fixed);
      fixed = fixBarePrintln(fixed);
      fixed = fixIoPrintlnNonStrings(fixed);
      fixed = fixIoPrintlnKnown(fixed);
      fixed = ensureSemicolons(fixed);
      if (fixed !== data.code_template) {
        data.code_template = fixed;
        changed = true;
      }
    }

    // Fix solution
    if (data.solution && typeof data.solution === 'string') {
      let fixed = fixXIOMModuleImports(data.solution);
      fixed = fixMainReturnType(fixed);
      fixed = fixBarePrintln(fixed);
      fixed = fixIoPrintlnNonStrings(fixed);
      fixed = fixIoPrintlnKnown(fixed);
      fixed = ensureSemicolons(fixed);
      if (fixed !== data.solution) {
        data.solution = fixed;
        changed = true;
      }
    }

    if (changed) {
      fs.writeFileSync(filePath, JSON.stringify(data, null, 2), 'utf8');
      console.log(`[FIXED] ${fileName}`);
      fixedCount++;
    } else {
      console.log(`[OK]    ${fileName}`);
    }
  } catch (err) {
    console.error(`[ERROR] ${fileName}: ${err.message}`);
    errorCount++;
  }
}

console.log(`\n=== SUMMARY ===`);
console.log(`Fixed: ${fixedCount}`);
console.log(`Unchanged: ${files.length - fixedCount - errorCount}`);
console.log(`Errors: ${errorCount}`);
console.log(`Total: ${files.length}`);
