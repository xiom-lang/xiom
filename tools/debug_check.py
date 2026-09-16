# Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
# SPDX-License-Identifier: MIT OR Apache-2.0

import subprocess, tempfile, os
f = tempfile.NamedTemporaryFile(suffix='.xi', delete=False, mode='w')
f.write('io.println("hello");\n')
f.close()
result = subprocess.run([r'E:\Projects\AXIOM\target\debug\xiom.exe', '--check', f.name], capture_output=True, text=True)
print('exit:', result.returncode)
print('stdout:', result.stdout[:200])
print('stderr:', result.stderr[:200])
os.unlink(f.name)
