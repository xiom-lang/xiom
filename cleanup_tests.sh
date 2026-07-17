#!/usr/bin/env bash
# ============================================================================
# XIOM Test Artifact Cleanup — macOS & Linux
# ============================================================================
# Cleans up e2e test artifacts from project root
# Usage: ./cleanup_tests.sh
# ============================================================================
set -euo pipefail

echo "========================================"
echo " XIOM Test Artifact Cleanup"
echo "========================================"

# 1. Delete e2e test executables (keep xiom toolchain binaries)
echo ""
echo "--- Deleting e2e test executables ---"
count=0
for f in *.exe; do
    if [[ "$f" != "xiomc.exe" && "$f" != "xiom-fmt.exe" && \
          "$f" != "xiom-doc.exe" && "$f" != "xiom-ffigen.exe" && \
          "$f" != "xiom-pkg.exe" && "$f" != "xiom-lsp.exe" ]]; then
        rm -f "$f"
        echo "  DEL: $f"
        ((count++)) || true
    fi
done 2>/dev/null
echo "  ($count exe files deleted)"

# 2. Delete LLVM IR dumps
echo ""
echo "--- Deleting LLVM IR dumps ---"
count=0
for f in *.ll; do
    rm -f "$f"
    echo "  DEL: $f"
    ((count++)) || true
done 2>/dev/null
echo "  ($count .ll files deleted)"

# 3. Delete WASM artifacts
echo ""
echo "--- Deleting WASM artifacts ---"
count=0
for f in *.wasm; do
    rm -f "$f"
    echo "  DEL: $f"
    ((count++)) || true
done 2>/dev/null
echo "  ($count .wasm files deleted)"

# 4. Delete PDB debug symbol files
echo ""
echo "--- Deleting PDB symbol files ---"
count=0
for f in *.pdb; do
    rm -f "$f"
    echo "  DEL: $f"
    ((count++)) || true
done 2>/dev/null
echo "  ($count .pdb files deleted)"

# 5. Delete object, assembly, and text artifacts
echo ""
echo "--- Deleting .o/.obj/.s/.txt artifacts ---"
for f in *.o *.obj *.s *.txt; do
    rm -f "$f"
    echo "  DEL: $f"
done 2>/dev/null

echo ""
echo "Done."
