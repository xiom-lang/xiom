const fs = require('fs');
const path = require('path');

const LESSON_DIRS = [
  'L5-patterns',
  'L6-engineering',
  'L7-mastery',
  'L8-ecosystem',
];

const BASE = path.resolve(__dirname, '..', 'xiom-playground', 'lessons');

// ---------------------------------------------------------------------------
// 1) ensure use xiom.io; is the first non-comment line
// ---------------------------------------------------------------------------
function ensureXiomIo(code) {
  if (code.includes('use xiom.io;')) return code;
  const lines = code.split('\n');
  // strip leading blank lines
  let firstCode = 0;
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].trim() === '') { firstCode = i + 1; continue; }
    break;
  }
  lines.splice(firstCode, 0, 'use xiom.io;');
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// 2) add use xiom.string; if string.* is used
// ---------------------------------------------------------------------------
function ensureXiomString(code) {
  if (!code.includes('string.')) return code;
  if (code.includes('use xiom.string;')) return code;
  // insert after use xiom.io;
  return code.replace(
    /(use xiom\.io;)/,
    '$1\n\nuse xiom.string;'
  );
}

// ---------------------------------------------------------------------------
// 3) fix double conversion: string.int_to_str(X.to_str())  →  X.to_str()
//    and string.float_to_str(X.to_str()) → X.to_str()
// ---------------------------------------------------------------------------
function fixDoubleConversion(code) {
  // This regex captures string.int_to_str( ... .to_str() ) patterns
  // where the inner expression can be anything.
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
// 4) fix io.println(expr) where expr is non-Str and has no .to_str()
//    We skip: string literals, already .to_str(), string concatenation,
//    string.*_to_str wrappers.
// ---------------------------------------------------------------------------
function hasToString(expr) {
  return /\.to_str\(\)\s*$/.test(expr.trim());
}

function isStringLiteral(expr) {
  const t = expr.trim();
  return t.startsWith('"') || t.startsWith("'");
}

// Known string-returning function/method names (extracted from all 150 lesson files)
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

function isStringReturningCall(expr) {
  const t = expr.trim();
  // Check if expression ends with .funcName() or .funcName(args)
  const m = t.match(/\.(\w+)\s*\([^)]*\)$/);
  if (m && STR_RETURNING_FUNCS.has(m[1])) return true;
  // Check for standalone function calls that return Str
  const s = t.match(/^(\w+)\s*\([^)]*\)$/);
  if (s && STR_RETURNING_FUNCS.has(s[1])) return true;
  return false;
}

function isStringExpr(expr) {
  if (hasToString(expr)) return true;
  if (isStringLiteral(expr)) return true;
  if (expr.includes('+')) return true;
  if (/\bstring\.\w+_to_str\(/.test(expr)) return true;
  if (/\bstring\.str_concat\(/.test(expr)) return true;
  if (isStringReturningCall(expr)) return true;
  return false;
}

function needsParenWrap(expr) {
  // compound expressions containing arithmetic/comparison/logical operators
  // need parentheses so .to_str() binds correctly
  const trimmed = expr.trim();
  // if there are no spaces, it's a simple term — no parens needed
  if (!/\s/.test(trimmed)) return false;
  // if it looks like a method chain: x.y().z() etc
  if (/^[\w.]+\(\)$/.test(trimmed)) return false;
  // contains these operators (outside parens) → compound
  return /[*/%]|==|!=|<=|>=|&&|\|\||[&|](?!=)/.test(trimmed);
}

function fixIoPrintln(code) {
  // Match io.println( ... ) handling balanced parens (single level)
  // We match io.println(  then capture until the matching close paren
  const regex = /io\.println\(((?:[^()]|\([^()]*\))*)\)/g;
  return code.replace(regex, (fullMatch, args) => {
    if (isStringExpr(args)) return fullMatch;
    if (needsParenWrap(args)) {
      return `io.println((${args}).to_str())`;
    }
    return `io.println(${args}.to_str())`;
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
    console.error(`  SKIP (read error): ${filePath}`);
    return { fixed: false };
  }

  let data;
  try {
    data = JSON.parse(raw);
  } catch (e) {
    console.error(`  SKIP (parse error): ${filePath}`);
    return { fixed: false };
  }

  const fields = ['code_template', 'solution'];
  let changed = false;

  for (const field of fields) {
    if (typeof data[field] !== 'string') continue;
    let code = data[field];

    // ensure use xiom.io;
    code = ensureXiomIo(code);

    // ensure use xiom.string; if needed
    code = ensureXiomString(code);

    // fix double conversions
    code = fixDoubleConversion(code);

    // fix io.println() calls
    code = fixIoPrintln(code);

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
    }
  }
}

console.log(`\n=== DONE ===`);
console.log(`Total processed: ${totalProcessed}`);
console.log(`Total fixed: ${totalFixed}`);
if (fixedFiles.length > 0) {
  console.log(`Fixed files:`);
  fixedFiles.forEach(f => console.log(`  ${f}`));
}

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

const sample = allFiles.sort(() => Math.random() - 0.5).slice(0, 5);
for (const fp of sample) {
  try {
    JSON.parse(fs.readFileSync(fp, 'utf8'));
    console.log(`  OK: ${path.basename(fp)}`);
  } catch (e) {
    console.error(`  FAIL: ${path.basename(fp)} — ${e.message}`);
  }
}
