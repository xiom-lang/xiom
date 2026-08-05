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
$timings = @()
$totalPassed = 0; $totalFailed = 0; $totalIgnored = 0

function Write-Log($msg) {
    if ($Logs) { Add-Content -Path "$LogDir\session_$sessionId.txt" -Value "[$(Get-Date -Format 'HH:mm:ss.fff')] $msg" }
}

function Invoke-Cargo($pkg, $testFile, $extraFilter, $showProgress) {
    $tmpOut = "$env:TEMP\xiom_test_out.txt"
    $tmpErr = "$env:TEMP\xiom_test_err.txt"
    Remove-Item $tmpOut, $tmpErr -EA SilentlyContinue
    
    $cargs = @("test", "-p", $pkg)
    if ($testFile) { $cargs += "--test", $testFile }
    if ($extraFilter) { $cargs += $extraFilter }
    $cargs += "--", "--test-threads=$Threads"
    
    $argStr = [string]::Join(" ", $cargs)
    Write-Log "cargo $argStr"
    
    # Use Start-Process with file redirects (avoids deadlock)
    $proc = Start-Process -FilePath "cargo" -ArgumentList $cargs -NoNewWindow -PassThru -RedirectStandardOutput $tmpOut -RedirectStandardError $tmpErr
    
    # Poll for progress while process runs
    $total = 0; $passed = 0; $failed = 0; $linesRead = 0
    $testCountKnown = $false
    $lastUpdate = [DateTime]::Now
    
    while (-not $proc.HasExited) {
        Start-Sleep -Milliseconds 300
        if (Test-Path $tmpOut) {
            $lines = Get-Content $tmpOut -EA SilentlyContinue
            for ($i = $linesRead; $i -lt $lines.Count; $i++) {
                $line = $lines[$i]
                if ($line -match '^running (\d+) tests?') { $total = [int]$Matches[1]; $testCountKnown = $true }
                if ($line -match '^test (?!result:)(\S+) \.\.\. ok$') { $passed++ }
                if ($line -match '^test (?!result:)(\S+) \.\.\. FAILED$') { $failed++ }
                $linesRead++
            }
        }
        if ($showProgress -and $testCountKnown -and $total -gt 0 -and (([DateTime]::Now) - $lastUpdate).TotalSeconds -gt 0.5) {
            $finished = $passed + $failed
            $pct = [math]::Round($finished * 100 / $total)
            $bar = ""; for ($i = 0; $i -lt 20; $i++) { $bar += if ($i * 5 -lt $pct) { "=" } else { "-" } }
            Write-Host ("`r  [{0}] {1}/{2} ({3} pass, {4} fail)   " -f $bar, $finished, $total, $passed, $failed) -NoNewline
            $lastUpdate = [DateTime]::Now
        }
    }
    $proc.WaitForExit()
    
    # Read final output
    $stdout = if (Test-Path $tmpOut) { Get-Content $tmpOut -Raw -EA SilentlyContinue } else { "" }
    $stderr = if (Test-Path $tmpErr) { Get-Content $tmpErr -Raw -EA SilentlyContinue } else { "" }
    Remove-Item $tmpOut, $tmpErr -EA SilentlyContinue
    
    if ($showProgress -and $testCountKnown) {
        Write-Host ("`r" + " " * 80 + "`r") -NoNewline
    }
    
    # Parse result
    $p = 0; $f = 0; $i = 0
    $output = $stdout + "`n" + $stderr
    if ($output -match 'test result: \w+\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
        $p = [int]$Matches[1]; $f = [int]$Matches[2]; $i = [int]$Matches[3]
    } elseif ($stdout -match '(\d+) passed;\s*(\d+) failed') {
        $p = [int]$Matches[1]; $f = [int]$Matches[2]
    } elseif ($stdout -match 'failures:' -and $stdout -notmatch '0 passed') {
        $f = ([regex]::Matches($stdout, '\.\.\. FAILED')).Count
        $p = ([regex]::Matches($stdout, '\.\.\. ok')).Count
    }
    return @{ Passed=$p; Failed=$f; Ignored=$i; Output=$output }
}

function Test-Crate($pkg, $testFile, $label, $extraFilter, $showProgress) {
    Write-Log "START $label"
    Write-Host "  $($label.PadRight(25)) " -NoNewline
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-Cargo $pkg $testFile $extraFilter $showProgress
    $sw.Stop()
    
    $p = $result.Passed; $f = $result.Failed; $i = $result.Ignored
    $elapsed = $sw.Elapsed.TotalSeconds
    $elapsedStr = if ($elapsed -lt 1) { "$([math]::Round($elapsed*1000))ms" } else { "$([math]::Round($elapsed,1))s" }
    
    if ($f -gt 0) { Write-Host "FAIL ${elapsedStr} ($p/$($p+$f))" -ForegroundColor Red }
    elseif ($p -gt 0) { Write-Host "OK   ${elapsedStr} ($p)" -ForegroundColor Green }
    else { Write-Host "NONE ${elapsedStr} ($p)" -ForegroundColor Yellow }
    
    $script:totalPassed += $p; $script:totalFailed += $f; $script:totalIgnored += $i
    $script:timings += @{ Label=$label; Passed=$p; Failed=$f; Ignored=$i; Elapsed=$elapsed }
    
    Write-Log "END $label p=$p f=$f i=$i t=$elapsedStr"
    if ($f -gt 0 -and $Logs) {
        $failPath = "$LogDir\failures_$sessionId.txt"
        Add-Content -Path $failPath -Value "=== $label ($p/$($p+$f)) ==="
        ($result.Output -split "`n" | Select-String "FAILED") | ForEach-Object { Add-Content -Path $failPath -Value $_ }
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
Write-Host " OK ($([math]::Round($sw.Elapsed.TotalSeconds,1))s)" -ForegroundColor Green
Write-Log "BUILD OK"
Write-Host ""

# ---- Run ----
Write-Host "RUN - $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Yellow

Write-Host " [UNITS]" -ForegroundColor Cyan
Test-Crate "xiom-lexer"   $null "lexer"            $null $false
Test-Crate "xiom-parser"  $null "parser"           $null $false
Test-Crate "xiom-check"   $null "checker"          $null $false
Test-Crate "xiom-ctfe"    $null "ctfe"             $null $false
Test-Crate "xiom-graph"   $null "graph"            $null $false
Test-Crate "xiom-codegen" $null "codegen-unit"     $null $false
Test-Crate "xiom-verify"  "verifier_tests" "verifier"    $null $false
Test-Crate "xiom-jit"     $null "jit"              $null $false

Write-Host " [TOOLS]" -ForegroundColor Cyan
Test-Crate "xiom-fmt"     $null "formatter"        $null $false
Test-Crate "xiom-lsp"     $null "lsp"              $null $false
Test-Crate "xiom-pkg"     $null "package-mgr"      $null $false
Test-Crate "xiom-doc"     $null "doc-gen"          $null $false
Test-Crate "xiom-ffigen"  $null "ffi-gen"          $null $false
Test-Crate "xiom-mcp"     $null "mcp-server"       $null $false
Test-Crate "xiom-dbg"     $null "debugger"         $null $false
Test-Crate "xiom-display" $null "display"          $null $false
Test-Crate "xiom" "scripting_tests" "scripting"    $null $false
Test-Crate "xiom" "diff_tests" "script-diff"       $null $false

Write-Host " [SUITES]" -ForegroundColor Cyan
Test-Crate "xiom-codegen" "integration_tests" "integration"     $null $false
Test-Crate "xiom-codegen" "diff_tests" "diff"                  $null $false
if (-not $Fast) { Test-Crate "xiom-codegen" "full_diff_tests" "full-diff"    $null $false }
if (-not $Fast) { Test-Crate "xiom-codegen" "fuzz_tests" "fuzz"              $null $false }
Test-Crate "xiom-codegen" "robustness_tests" "robustness"       $null $false
Test-Crate "xiom-codegen" "stdlib_execution_tests" "stdlib-exec" $null $false
Test-Crate "xiom-codegen" "stdlib_tests" "stdlib-compile"        $null $false
if (-not $Fast) { Test-Crate "xiom-codegen" "feature_regression_tests" "feature-reg" $null $false }

if (-not $Fast) {
    Write-Host " [E2E]" -ForegroundColor Cyan
    if ($E2EOnly) {
        Test-Crate "xiom-codegen" "e2e_tests" "e2e-gate" @("e2e_p0","e2e_p1","e2e_p2","e2e_never_type","e2e_asm","e2e_spawn_basic","e2e_i2") $true
    } else {
        Test-Crate "xiom-codegen" "e2e_tests" "e2e (all 2231)" $null $true
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
    foreach ($r in $timings) { if ($r.Failed -gt 0) { Write-Host "  $($r.Label): $($r.Failed) failed / $($r.Passed+$r.Failed) total" -ForegroundColor Red } }
    if ($Logs) { Write-Host "  Failure details: $LogDir\failures_$sessionId.txt" -ForegroundColor Yellow }
}
if ($Logs) { Write-Host "  Session log: $LogDir\session_$sessionId.txt" -ForegroundColor Yellow }
if ($totalFailed -gt 0) { exit 1 } else { exit 0 }
