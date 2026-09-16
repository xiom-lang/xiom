# Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Refactor compile() in lib.rs to return Result instead of process::exit."""
path = r'E:\Projects\AXIOM\crates\xiom\src\lib.rs'
with open(path, encoding='utf-8', errors='replace') as f:
    content = f.read()

# Change function signature
old_sig = 'pub fn compile(config: &CompileConfig, source_paths: &[String]) {'
new_sig = 'pub fn compile(config: &CompileConfig, source_paths: &[String]) -> Result<(), Vec<String>> {'
content = content.replace(old_sig, new_sig)

# Replace all process::exit(1) with error returns
count1 = content.count('process::exit(1);')
count2 = content.count('process::exit(1)')
content = content.replace('process::exit(1);', 'return Err(vec!["compilation failed".to_string()]);')
# Also handle std::process::exit(1) (line 1463)
content = content.replace('std::process::exit(1);', 'return Err(vec!["compilation failed".to_string()]);')

# Add Ok(()) at the end of the function (before the closing brace)
# Find the last occurrence of 'process::exit' or the function's end
# The function ends with a closing brace. Let me find the pattern.
# Actually, let me add an explicit Ok(()) return at the end.
# Search for the last '}' of the function

# Simpler: find where 'process::exit' was last in this function and add Ok after
# Or just append Ok(()) before the final }

# Find the function's closing
start = content.find(old_sig) + len(old_sig)  # skip to new sig
start = content.find(new_sig) + len(new_sig)

# Track braces to find function end
depth = 0
found_open = False
for i in range(start, len(content)):
    if content[i] == '{':
        depth += 1
        found_open = True
    elif content[i] == '}':
        depth -= 1
        if found_open and depth == 0:
            # Found compile() function's closing brace
            # Insert Ok(()) before it
            content = content[:i] + '\n    Ok(())\n' + content[i:]
            break

print(f'Replaced {count1} process::exit calls')
print(f'Added Ok(()) return at end of compile()')

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
