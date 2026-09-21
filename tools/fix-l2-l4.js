// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

const fs = require('fs');
const path = require('path');

const LESSON_DIRS = [
  'L2-data',
  'L3-systems',
  'L4-safety',
];

const BASE = path.resolve(__dirname, '..', 'xiom-playground', 'lessons');

// ---------------------------------------------------------------------------
// 1) ensure use xiom.io; is the first non-comment/blank line
// ---------------------------------------------------------------------------
function ensureXiomIo(code) {
  const lines = code.split('\n');
  let insertAt = 0;
  for (let i = 0; i < lines.length; i++) {
    const t = lines[i].trim();
    if (t === '' || t.startsWith('//')) { insertAt = i + 1; continue; }
    break;
  }
  const firstReal = (lines[insertAt] || '').trim();
  if (firstReal !== 'use xiom.io;') {
    lines.splice(insertAt, 0, 'use xiom.io;');
  }
  // ensure blank line after use xiom.io; if the next line is not blank and not another import
  const ioIdx = lines.findIndex(l => l.trim() === 'use xiom.io;');
  if (ioIdx !== -1 && ioIdx + 1 < lines.length) {
    const next = lines[ioIdx + 1].trim();
    if (next !== '' && !next.startsWith('use xiom.')) {
      lines.splice(ioIdx + 1, 0, '');
    }
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// 2) add use xiom.math; if math.* is used
// ---------------------------------------------------------------------------
function ensureXiomMath(code) {
  if (!code.includes('math.')) return code;
  if (code.includes('use xiom.math;')) return code;
  return code.replace(
    /(use xiom\.io;)/,
    '$1\nuse xiom.math;'
  );
}

// ---------------------------------------------------------------------------
// 3) add use xiom.string; if string.* is used
// ---------------------------------------------------------------------------
function ensureXiomString(code) {
  if (!code.includes('string.')) return code;
  if (code.includes('use xiom.string;')) return code;
  // insert after use xiom.io; (and use xiom.math; if present)
  if (code.includes('use xiom.math;')) {
    return code.replace(
      /(use xiom\.math;)/,
      '$1\nuse xiom.string;'
    );
  }
  return code.replace(
    /(use xiom\.io;)/,
    '$1\nuse xiom.string;'
  );
}

// ---------------------------------------------------------------------------
// 4) fix bare println() -> io.println()
// ---------------------------------------------------------------------------
function fixBarePrintln(code) {
  return code.replace(/(?<![a-zA-Z_.])println\s*\(/g, 'io.println(');
}

// ---------------------------------------------------------------------------
// 5) fix type annotations: : [Str] -> : Vec[Str], : [Int] -> : Vec[Int]
// ---------------------------------------------------------------------------
function fixTypeAnnotations(code) {
  code = code.replace(/:\s*\[Str\]/g, ': Vec[Str]');
  code = code.replace(/:\s*\[Int\]/g, ': Vec[Int]');
  code = code.replace(/:\s*\[Float64\]/g, ': Vec[Float64]');
  code = code.replace(/:\s*\[Bool\]/g, ': Vec[Bool]');
  return code;
}

// ---------------------------------------------------------------------------
// 6) fix double conversion: string.int_to_str(X.to_str()) -> X.to_str()
// ---------------------------------------------------------------------------
function fixDoubleConversion(code) {
  code = code.replace(
    /string\.int_to_str\((.+?)\.to_str\(\)\)/g,
    (m, inner) => inner + '.to_str()'
  );
  code = code.replace(
    /string\.float_to_str\((.+?)\.to_str\(\)\)/g,
    (m, inner) => inner + '.to_str()'
  );
  code = code.replace(
    /string\.bool_to_str\((.+?)\.to_str\(\)\)/g,
    (m, inner) => inner + '.to_str()'
  );
  return code;
}

// ---------------------------------------------------------------------------
// 7) ensure semicolons on statement lines
// ---------------------------------------------------------------------------
function ensureSemicolons(code) {
  const lines = code.split('\n');
  const result = [];
  for (const line of lines) {
    const trimmed = line.trimEnd();
    const stripped = trimmed.trim();
    if (stripped === '' ||
        stripped.startsWith('//') ||
        stripped.startsWith('/*') ||
        stripped === '{' || stripped === '}' ||
        stripped.startsWith('}') ||
        stripped.startsWith('use xiom.') ||
        stripped.endsWith(';') ||
        stripped.endsWith(',') ||
        stripped.startsWith('fn ') ||
        stripped.startsWith('type ') ||
        stripped.startsWith('enum ') ||
        stripped.startsWith('for ') ||
        stripped.startsWith('while ') ||
        stripped.startsWith('if ') ||
        stripped.startsWith('else') ||
        stripped.startsWith('elif') ||
        stripped.startsWith('match ') ||
        stripped.includes('=>') ||
        stripped.endsWith('{') ||
        stripped.endsWith('}')) {
      result.push(trimmed);
    } else if (/^[a-zA-Z_].*[^;{}]$/.test(stripped) && !stripped.startsWith('//')) {
      result.push(trimmed + ';');
    } else {
      result.push(trimmed);
    }
  }
  return result.join('\n');
}

// ---------------------------------------------------------------------------
// 8) cleanup artifacts: ,; -> ,  (comma-semicolon from bad semicolon insertion)
// ---------------------------------------------------------------------------
function cleanupCommaSemicolon(code) {
  return code.replace(/,;/g, ',');
}

// ---------------------------------------------------------------------------
// 9) fix io.println() calls for L2-L4 (.to_str() rules)
//    io.println() takes Str ONLY. L2+ uses .to_str().
// ---------------------------------------------------------------------------
function isStringLiteral(expr) {
  const t = expr.trim();
  return (t.startsWith('"') && t.endsWith('"'));
}

function hasToStringSuffix(expr) {
  return /\.to_str\(\)\s*$/.test(expr.trim());
}

function isStringApiCall(expr) {
  return /^string\.\w+/.test(expr.trim());
}

function isAlreadyStringResult(expr) {
  const t = expr.trim();
  if (isStringLiteral(t)) return true;
  if (hasToStringSuffix(t)) return true;
  if (isStringApiCall(t)) return true;
  return false;
}

// Split a + chain respecting parentheses and string literals
function splitPlusChain(expr) {
  const parts = [];
  let current = '';
  let depth = 0;
  let inString = false;
  let escape = false;
  for (let i = 0; i < expr.length; i++) {
    const ch = expr[i];
    if (escape) {
      escape = false;
      current += ch;
      continue;
    }
    if (ch === '\\') {
      escape = true;
      current += ch;
      continue;
    }
    if (ch === '"') {
      inString = !inString;
      current += ch;
      continue;
    }
    if (!inString) {
      if (ch === '(') {
        depth++;
        current += ch;
        continue;
      }
      if (ch === ')') {
        depth--;
        current += ch;
        continue;
      }
      if (ch === '+' && depth === 0) {
        parts.push(current.trim());
        current = '';
        continue;
      }
    }
    current += ch;
  }
  if (current.trim()) {
    parts.push(current.trim());
  }
  return parts;
}

// Fix the argument expression inside io.println(...)
function fixPrintArg(arg) {
  if (!arg || arg.trim() === '') return arg;

  // If no +, it's a simple expression
  if (!arg.includes('+')) {
    if (isAlreadyStringResult(arg)) return arg;
    // Numeric literal?
    if (/^\d+(\.\d+)?$/.test(arg.trim())) return arg.trim() + '.to_str()';
    // Boolean literal?
    if (/^(true|false)$/.test(arg.trim())) return arg.trim() + '.to_str()';
    // Otherwise, wrap with .to_str()
    return arg.trim() + '.to_str()';
  }

  // Has + : process each segment
  const parts = splitPlusChain(arg);
  if (parts.length <= 1) {
    // + was inside parens/strings, treat as whole
    if (isAlreadyStringResult(arg)) return arg;
    return arg.trim() + '.to_str()';
  }

  const fixedParts = parts.map(p => {
    const trimmed = p.trim();
    if (!trimmed) return trimmed;
    if (isAlreadyStringResult(trimmed)) return trimmed;
    return trimmed + '.to_str()';
  });

  return fixedParts.join(' + ');
}

function fixIoPrintln(code) {
  // Match io.println( ... ) with balanced parens (up to 2 levels)
  const regex = /io\.println\(((?:[^()]|\([^()]*(?:\([^()]*\)[^()]*)*\))*)\)/g;
  return code.replace(regex, (fullMatch, args) => {
    const fixedArgs = fixPrintArg(args);
    if (fixedArgs === args) return fullMatch;
    return `io.println(${fixedArgs})`;
  });
}

// ---------------------------------------------------------------------------
// process a single file
// ---------------------------------------------------------------------------
function processFile(filePath) {
  let raw;
  try {
    raw = fs.readFileSync(filePath, 'utf8');
  } catch (e) {
    console.error(`  SKIP (read error): ${path.basename(filePath)} - ${e.message}`);
    return { fixed: false };
  }

  let data;
  try {
    data = JSON.parse(raw);
  } catch (e) {
    console.error(`  SKIP (parse error): ${path.basename(filePath)} - ${e.message}`);
    return { fixed: false };
  }

  const fields = ['code_template', 'solution'];
  let changed = false;

  for (const field of fields) {
    if (typeof data[field] !== 'string') continue;
    let code = data[field];

    code = ensureXiomIo(code);
    code = ensureXiomMath(code);
    code = ensureXiomString(code);
    code = fixBarePrintln(code);
    code = fixTypeAnnotations(code);
    code = fixDoubleConversion(code);
    code = fixIoPrintln(code);
    code = ensureSemicolons(code);
    code = cleanupCommaSemicolon(code);

    if (code !== data[field]) {
      data[field] = code;
      changed = true;
    }
  }

  if (changed) {
    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n', 'utf8');
  }

  return { fixed: changed };
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------
let totalFixed = 0;
let totalProcessed = 0;
const fixedFiles = [];

for (const dir of LESSON_DIRS) {
  const dirPath = path.join(BASE, dir);
  if (!fs.existsSync(dirPath)) {
    console.log(`WARN: directory not found: ${dirPath}`);
    continue;
  }
  const files = fs.readdirSync(dirPath).filter(f => f.endsWith('.json'));
  console.log(`\n=== Processing ${dir} (${files.length} files) ===`);

  for (const file of files) {
    const fp = path.join(dirPath, file);
    const { fixed } = processFile(fp);
    totalProcessed++;
    if (fixed) {
      totalFixed++;
      fixedFiles.push(path.join(dir, file));
      console.log(`  FIXED: ${file}`);
    }
  }
}

console.log(`\n=== DONE ===`);
console.log(`Total processed: ${totalProcessed}`);
console.log(`Total fixed: ${totalFixed}`);

// ---------------------------------------------------------------------------
// verify: check 5 random files are valid JSON
// ---------------------------------------------------------------------------
console.log(`\n=== VERIFY (random 5) ===`);
const allDirs = LESSON_DIRS.map(d => path.join(BASE, d)).filter(d => fs.existsSync(d));
const allFiles = [];
for (const d of allDirs) {
  const files = fs.readdirSync(d).filter(f => f.endsWith('.json'));
  files.forEach(f => allFiles.push(path.join(d, f)));
}

// Deterministic "random" for reproducibility
function seededRandom(seed) {
  let s = seed;
  return function() {
    s = (s * 1664525 + 1013904223) & 0xFFFFFFFF;
    return (s >>> 0) / 0xFFFFFFFF;
  };
}
const rng = seededRandom(42);
const shuffled = [...allFiles].sort(() => rng() - 0.5);
const sample = shuffled.slice(0, 5);

for (const fp of sample) {
  try {
    const parsed = JSON.parse(fs.readFileSync(fp, 'utf8'));
    const hasTemplate = typeof parsed.code_template === 'string';
    const hasSolution = typeof parsed.solution === 'string';
    console.log(`  OK: ${path.basename(fp)} (template: ${hasTemplate}, solution: ${hasSolution})`);
  } catch (e) {
    console.error(`  FAIL: ${path.basename(fp)} - ${e.message}`);
  }
}
