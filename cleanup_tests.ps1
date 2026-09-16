# Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
# SPDX-License-Identifier: MIT OR Apache-2.0

# XIOM Test Artifact Cleanup
# Cleans up e2e test artifacts from project root
## Run from: E:\Projects\AXIOM
# Usage: .\cleanup_tests.ps1

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " XIOM Test Artifact Cleanup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Delete e2e test executables (keep xiom toolchain binaries)
Write-Host ""
Write-Host "--- Deleting e2e test executables ---" -ForegroundColor Yellow
$count = 0
Get-ChildItem -Path . -Filter *.exe | Where-Object {
    $_.Name -notmatch '^(xiom|xiom-fmt|xiom-doc|xiom-ffigen|xiom-pkg|xiom-lsp|xiom-mcp|xiom-dbg|xiom-verify)\.exe$'
} | ForEach-Object {
    Remove-Item $_.FullName -Force
    Write-Host "  DEL: $($_.Name)"
    $count++
}
Write-Host "  ($count exe files deleted)"

# 2. Delete LLVM IR dumps
Write-Host ""
Write-Host "--- Deleting LLVM IR dumps ---" -ForegroundColor Yellow
$count = 0
Get-ChildItem -Path . -Filter *.ll | ForEach-Object {
    Remove-Item $_.FullName -Force
    Write-Host "  DEL: $($_.Name)"
    $count++
}
Write-Host "  ($count .ll files deleted)"

# 3. Delete WASM artifacts
Write-Host ""
Write-Host "--- Deleting WASM artifacts ---" -ForegroundColor Yellow
$count = 0
Get-ChildItem -Path . -Filter *.wasm | ForEach-Object {
    Remove-Item $_.FullName -Force
    Write-Host "  DEL: $($_.Name)"
    $count++
}
Write-Host "  ($count .wasm files deleted)"

# 4. Delete PDB debug symbol files
Write-Host ""
Write-Host "--- Deleting PDB symbol files ---" -ForegroundColor Yellow
$count = 0
Get-ChildItem -Path . -Filter *.pdb | ForEach-Object {
    Remove-Item $_.FullName -Force
    Write-Host "  DEL: $($_.Name)"
    $count++
}
Write-Host "  ($count .pdb files deleted)"

# 5. Delete object, assembly, and text artifacts
Write-Host ""
Write-Host "--- Deleting .obj/.s/.txt artifacts ---" -ForegroundColor Yellow
Get-ChildItem -Path . -Filter *.obj | ForEach-Object { Remove-Item $_.FullName -Force; Write-Host "  DEL: $($_.Name)" }
Get-ChildItem -Path . -Filter *.s   | ForEach-Object { Remove-Item $_.FullName -Force; Write-Host "  DEL: $($_.Name)" }
Get-ChildItem -Path . -Filter *.txt | ForEach-Object { Remove-Item $_.FullName -Force; Write-Host "  DEL: $($_.Name)" }

Write-Host ""
Write-Host "Done." -ForegroundColor Green
