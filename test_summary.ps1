#Requires -Version 5.1
param([switch]$Fast, [int]$Threads = 8, [switch]$E2EOnly, [switch]$Logs, [switch]$CleanBuild, [switch]$CleanLogs)

$ErrorActionPreference = "Continue"
if ($CleanBuild) { if (Test-Path ".test_build") { Remove-Item ".test_build\*" -Recurse -Force -EA SilentlyContinue }; Write-Host "Cleaned .test_build"; exit 0 }
if ($CleanLogs)  { if (Test-Path ".testlogs")  { Remove-Item ".testlogs\*"  -Recurse -Force -EA SilentlyContinue }; Write-Host "Cleaned .testlogs";  exit 0 }

$BuildDir = ".test_build"; $LogDir = ".testlogs"
if (-not (Test-Path $BuildDir)) { New-Item -ItemType Directory -Force $BuildDir | Out-Null }
if (-not (Test-Path $LogDir))   { New-Item -ItemType Directory -Force $LogDir   | Out-Null }

$startTime = Get-Date
$sessionId = $startTime.ToString("yyyyMMdd_HHmmss")
$logPath = "$LogDir\session_$sessionId.txt"
$timings = @()

function Write-Log($msg) {
    if ($Logs) { Add-Content -Path $logPath -Value "[$(Get-Date -Format 'HH:mm:ss.fff')] $msg" }
}

$totalPassed = 0; $totalFailed = 0; $totalIgnored = 0

function Invoke-CargoTest($pkg, $testFile, $extraFilter) {
    $cargs = @("test", "-p", $pkg)
    if ($testFile) { $cargs += "--test", $testFile }
    if ($extraFilter) { $cargs += $extraFilter }
    $cargs += "--", "--test-threads=$Threads"
    $cargs += "2>&1"
    
    # Use a temp file for output to avoid hanging
    $tmp = "$env:TEMP\xiom_test_${pkg}_$((Get-Date).Ticks).txt"
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = "cargo"
    $psi.Arguments = [string]::Join(" ", $cargs)
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true
    
    $proc = [System.Diagnostics.Process]::Start($psi)
    
    # Wait with timeout (5 min per crate)
    $timeout = 300000
    $finished = $proc.WaitForExit($timeout)
    if (-not $finished) {
        $proc.Kill()
        return "TIMEOUT"
    }
    
    $stdout = $proc.StandardOutput.ReadToEnd()
    $stderr = $proc.StandardError.ReadToEnd()
    return $stdout + "`n" + $stderr
}

function Test-Crate($pkg, $testFile, $label, $extraFilter) {
    Write-Log "START $label"
    Write-Host "  $($label.PadRight(25)) " -NoNewline
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = Invoke-CargoTest $pkg $testFile $extraFilter
    $sw.Stop()
    
    if ($output -eq "TIMEOUT") {
        Write-Host "HANG  (timeout)" -ForegroundColor Magenta
        Write-Log "END $label TIMEOUT"
        return
    }
    
    $p = 0; $f = 0; $i = 0
    # Try multiple regex patterns for different cargo output formats
    if ($output -match 'test result: \w+\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
        $p = [int]$Matches[1]; $f = [int]$Matches[2]; $i = [int]$Matches[3]
    } elseif ($output -match '(\d+) passed;\s*(\d+) failed') {
        $p = [int]$Matches[1]; $f = [int]$Matches[2]
    } elseif ($output -match 'test result: (\w+)') {
        # Tests ran but no count — check for "ok" or "FAILED"
        if ($output -match '0 passed; 0 failed') { $p = 0; $f = 0 }
        elseif ($output -match 'failures:' -and $output -notmatch '0 passed') {
            # Some failures, extract from individual test lines
            $f = ([regex]::Matches($output, '\.\.\. FAILED')).Count
            $p = ([regex]::Matches($output, '\.\.\. ok')).Count
        }
    }
    
    $elapsed = $sw.Elapsed.TotalSeconds
    $elapsedStr = if ($elapsed -lt 1) { "$([math]::Round($elapsed*1000))ms" } else { "$([math]::Round($elapsed,1))s" }
    
    if ($f -gt 0) { Write-Host "FAIL ${elapsedStr} ($p/$($p+$f))" -ForegroundColor Red }
    elseif ($p -gt 0) { Write-Host "OK   ${elapsedStr} ($p)" -ForegroundColor Green }
    else { Write-Host "NONE ${elapsedStr} ($p)" -ForegroundColor Yellow }
    
    $r = @{ Label=$label; Passed=$p; Failed=$f; Ignored=$i; Elapsed=$elapsed }
    $script:timings += $r
    $script:totalPassed += $p; $script:totalFailed += $f; $script:totalIgnored += $i
    
    Write-Log "END $label p=$p f=$f i=$i t=$($elapsedStr)"
    if ($f -gt 0 -and $Logs) {
        $failPath = "$($script:LogDir)\failures_$sessionId.txt"
        Add-Content -Path $failPath -Value "=== $label ==="
        ($output -split "`n" | Select-String "FAILED") | ForEach-Object { Add-Content -Path $failPath -Value $_ }
        Add-Content -Path $failPath -Value ""
    }
}

# ---- Build ----
Write-Host "XIOM Test Suite v3" -ForegroundColor Magenta
Write-Host "===================" -ForegroundColor Magenta
Write-Host ""
Write-Host "BUILD (parallel)..." -ForegroundColor Yellow -NoNewline
Write-Log "BUILD START"
$sw = [System.Diagnostics.Stopwatch]::StartNew()
cargo test --workspace --no-run 2>&1 | Out-Null
$sw.Stop()
if ($LASTEXITCODE -ne 0) {
    Write-Host " FAILED" -ForegroundColor Red
    Write-Log "BUILD FAILED"
    exit 1
}
Write-Host " OK ($([math]::Round($sw.Elapsed.TotalSeconds,1))s)" -ForegroundColor Green
Write-Log "BUILD OK"
Write-Host ""

# ---- Run ----
Write-Host "RUN - $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow

Write-Host " [UNITS]" -ForegroundColor Cyan
Test-Crate "xiom-lexer"   $null "lexer"
Test-Crate "xiom-parser"  $null "parser"
Test-Crate "xiom-check"   $null "checker"
Test-Crate "xiom-ctfe"    $null "ctfe"
Test-Crate "xiom-graph"   $null "graph"
Test-Crate "xiom-codegen" $null "codegen-unit"
Test-Crate "xiom-verify"  "verifier_tests" "verifier"
Test-Crate "xiom-jit"     $null "jit"

Write-Host " [TOOLS]" -ForegroundColor Cyan
Test-Crate "xiom-fmt"     $null "formatter"
Test-Crate "xiom-lsp"     $null "lsp"
Test-Crate "xiom-pkg"     $null "package-mgr"
Test-Crate "xiom-doc"     $null "doc-gen"
Test-Crate "xiom-ffigen"  $null "ffi-gen"
Test-Crate "xiom-mcp"     $null "mcp-server"
Test-Crate "xiom-dbg"     $null "debugger"
Test-Crate "xiom-display" $null "display"
Test-Crate "xiom" "scripting_tests" "scripting"
Test-Crate "xiom" "diff_tests" "script-diff"

Write-Host " [SUITES]" -ForegroundColor Cyan
Test-Crate "xiom-codegen" "integration_tests" "integration"
Test-Crate "xiom-codegen" "diff_tests" "diff"
if (-not $Fast) { Test-Crate "xiom-codegen" "full_diff_tests" "full-diff" }
if (-not $Fast) { Test-Crate "xiom-codegen" "fuzz_tests" "fuzz" }
Test-Crate "xiom-codegen" "robustness_tests" "robustness"
Test-Crate "xiom-codegen" "stdlib_execution_tests" "stdlib-exec"
Test-Crate "xiom-codegen" "stdlib_tests" "stdlib-compile"
if (-not $Fast) { Test-Crate "xiom-codegen" "feature_regression_tests" "feature-reg" }

if (-not $Fast) {
    Write-Host " [E2E]" -ForegroundColor Cyan
    if ($E2EOnly) {
        Test-Crate "xiom-codegen" "e2e_tests" "e2e-gate" @("e2e_p0","e2e_p1","e2e_p2","e2e_never_type","e2e_asm","e2e_spawn_basic","e2e_i2")
    } else {
        Test-Crate "xiom-codegen" "e2e_tests" "e2e (all 2231)"
    }
} else {
    Write-Host " [E2E] SKIPPED" -ForegroundColor Yellow
}

# ---- Summary ----
$totalTime = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 1)
$total = $totalPassed + $totalFailed
Write-Host ""
Write-Host "============================================" -ForegroundColor Magenta
Write-Host "  TEST SUMMARY" -ForegroundColor Cyan
Write-Host "  Time:    ${totalTime}s" -ForegroundColor White
Write-Host "  Passed:  $totalPassed" -ForegroundColor Green
if ($totalFailed -gt 0) { Write-Host "  Failed:  $totalFailed" -ForegroundColor Red }
else { Write-Host "  Failed:  0" -ForegroundColor Green }
if ($totalIgnored -gt 0) { Write-Host "  Ignored: $totalIgnored" -ForegroundColor Yellow }
Write-Host "  Total:   $total" -ForegroundColor White
Write-Host "============================================" -ForegroundColor Magenta

Write-Log "SUMMARY p=$totalPassed f=$totalFailed i=$totalIgnored t=$totalTime"

if ($totalFailed -gt 0) {
    Write-Host "`nFAILURES:" -ForegroundColor Red
    foreach ($r in $timings) {
        if ($r.Failed -gt 0) {
            Write-Host "  $($r.Label): $($r.Failed) failed / $($r.Passed + $r.Failed) total" -ForegroundColor Red
        }
    }
    if ($Logs) { Write-Host "  Failure details: $LogDir\failures_$sessionId.txt" -ForegroundColor Yellow }
}
if ($Logs) { Write-Host "  Session log: $LogDir\session_$sessionId.txt" -ForegroundColor Yellow }
if ($totalFailed -gt 0) { exit 1 } else { exit 0 }
