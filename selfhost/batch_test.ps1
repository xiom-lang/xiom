# Batch test selfhost against multiple AXIOM examples
param([switch]$Quick)

$examples = @(
    "demo_float.ax",
    "diff_test.ax",
    "phase1_ownership.ax",
    "phase1_async.ax",
    "phase1_async_spawn.ax",
    "phase1_generics.ax",
    "phase1_interface.ax",
    "phase1_error.ax",
    "stress_borrow_10level.ax",
    "stress_float_matrix.ax",
    "stress_generic_5chain.ax"
)

$skip = @(
    "phase1_error.ax"  # Uses match/Result/derive - body parser can't handle
)

$pass = 0
$fail = 0
$skipCount = 0
$results = @()

foreach ($ex in $examples) {
    Write-Host "Testing: $ex " -ForegroundColor Cyan -NoNewline

    if ($skip -contains $ex) {
        Write-Host "SKIP" -ForegroundColor Yellow
        $skipCount++
        $results += "$ex : SKIP (uses match/Result/derive - unsupported)"
        continue
    }

    # Create a temp selfhost variant targeting this example
    $selfhostSrc = Get-Content "selfhost\axiomc_v10.ax" -Raw
    $selfhostSrc = $selfhostSrc -replace 'selfhost\\\\axiomc_v10\.ax', "examples\\$ex"
    $stem = $ex.Replace('.ax', '')
    $tempFile = "selfhost\test_temp_$stem.ax"
    Set-Content -Path $tempFile -Value $selfhostSrc

    Write-Host "." -NoNewline
    # Compile the test variant
    $compile = cargo run -p axiomc -- -o "test_temp_$stem.exe" $tempFile 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host " FAIL (compile)" -ForegroundColor Red
        $fail++
        $results += "$ex : COMPILE FAILED"
        Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
        continue
    }

    # Run and check output
    $exePath = ".\test_temp_$stem.exe"
    $output = & $exePath 2>&1 | Out-String
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0 -and $output -match "define i64 @main") {
        Write-Host " PASS (exit $exitCode)" -ForegroundColor Green
        $pass++
        $results += "$ex : PASS"
    } else {
        Write-Host " FAIL (exit $exitCode)" -ForegroundColor Red
        if ($output.Length -gt 200) {
            $snippet = $output.Substring(0, [Math]::Min(200, $output.Length))
        } else {
            $snippet = $output
        }
        $results += "$ex : FAIL (exit $exitCode)`n  output: $snippet"
    }

    Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    Remove-Item "test_temp_$stem.exe" -Force -ErrorAction SilentlyContinue
    if (Test-Path "test_temp_$stem.exe.ll") { Remove-Item "test_temp_$stem.exe.ll" -Force }
}

Write-Host ""
Write-Host "=== RESULTS ==="
foreach ($r in $results) { Write-Host $r }
Write-Host ""
Write-Host "PASS: $pass / FAIL: $fail / SKIP: $skipCount / TOTAL: $($examples.Count)"
