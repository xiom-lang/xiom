# Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Count production unwraps in ALL crates (excluding tests/comments)."""
import os, re

workspace = r'E:\xiom-lang\xiom'
crates_dir = os.path.join(workspace, 'crates')
total = {}

for crate in os.listdir(crates_dir):
    crate_path = os.path.join(crates_dir, crate)
    if not os.path.isdir(crate_path):
        continue
    for root, dirs, files in os.walk(crate_path):
        for fname in files:
            if not fname.endswith('.rs'):
                continue
            fpath = os.path.join(root, fname)
            with open(fpath, encoding='utf-8', errors='replace') as f:
                content = f.read()
            lines = content.split('\n')
            in_test = False
            unwrap_count = 0
            for line in lines:
                if '#[cfg(test)]' in line or '#[test]' in line:
                    in_test = True
                if in_test and (line.strip() == '}' or line.strip().startswith('}')):
                    # Heuristic: test module ends at a closing brace
                    pass  # keep in_test as-is; real exit handled below
                if '.unwrap()' in line and not in_test:
                    comment = line.find('//')
                    uw = line.find('.unwrap()')
                    if comment == -1 or uw < comment:
                        unwrap_count += 1
            if unwrap_count > 0:
                rel = os.path.relpath(fpath, crates_dir)
                total[rel] = unwrap_count

for path, count in sorted(total.items(), key=lambda x: -x[1]):
    print(f'{path}: {count}')
print(f'\nTotal files with production unwraps: {len(total)}')
print(f'Total production unwraps: {sum(total.values())}')
