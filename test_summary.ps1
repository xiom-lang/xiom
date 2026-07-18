#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Full Test Suite — compiler + tooling. Single-line release tag.
.DESCRIPTION
    Run: .\test_summary.ps1
    Sections: Compiler, Tooling, Total
    Output last line: "Release tag: NNN/NNN tests" (ready for release tagging)
#>

$ErrorActionPreference = "Continue"

function Run-Suite($pkg, $test, $label) {
    $name = $label
    Write-Host "  $($name.PadRight(22))" -NoNewline
    if ($test) {
        $output = & cargo test -p $pkg --test $test 2>&1 | Out-String
    } else {
        $output = & cargo test -p $pkg 2>&1 | Out-String
    }

    $matched = $false
    $passed = 0; $failed = 0; $ignored = 0; $status = "FAIL"
    foreach ($line in ($output -split "`n")) {
        if ($line -match 'test result: (\w+)\.\s*(\d+) passed;\s*(\d+) failed;\s*(\d+) ignored') {
            $status = $Matches[1]
            $passed = [int]$Matches[2]
            $failed = [int]$Matches[3]
            $ignored = [int]$Matches[4]
            $matched = $true
            break
        }
    }

    if ($matched) {
        if ($failed -gt 0) {
            Write-Host "FAIL ($passed/$($passed + $failed))" -ForegroundColor Red
            $global:totalFailed += $failed
        } else {
            Write-Host "OK ($passed/$($passed + $failed))" -ForegroundColor Green
        }
        $global:totalPassed += $passed
        $global:totalIgnored += $ignored
    } else {
        Write-Host "CRASH" -ForegroundColor Red
        $global:failedSuites += "$label (no result)"
    }
}

$global:totalPassed = 0
$global:totalFailed = 0
$global:totalIgnored = 0
$global:failedSuites = @()

# ============================================================================
# COMPILER SUITES
# ============================================================================
$compilerPassed = 0; $compilerFailed = 0; $compilerIgnored = 0

Write-Host "XIOM Test Suite" -ForegroundColor Magenta
Write-Host "==============" -ForegroundColor Magenta
Write-Host ""
Write-Host "COMPILER" -ForegroundColor Yellow

$compiler = @(
    @{pkg="xiom-codegen"; test="e2e_tests";                 label="e2e"},
    @{pkg="xiom-codegen"; test="feature_regression_tests";   label="feature-regression"},
    @{pkg="xiom-codegen"; test="stdlib_execution_tests";     label="stdlib-execution"},
    @{pkg="xiom-codegen"; test="diff_tests";                 label="diff"},
    @{pkg="xiom-codegen"; test="full_diff_tests";            label="full-diff"},
    @{pkg="xiom-codegen"; test="fuzz_tests";                 label="fuzz"},
    @{pkg="xiom-codegen"; test="integration_tests";          label="integration"},
    @{pkg="xiom-codegen"; test="robustness_tests";           label="robustness"},
    @{pkg="xiom-codegen"; test="stdlib_tests";               label="stdlib-compile"}
)

$global:totalPassed = 0; $global:totalFailed = 0; $global:totalIgnored = 0
foreach ($s in $compiler) {
    Run-Suite $s.pkg $s.test $s.label
}
$compilerPassed = $global:totalPassed
$compilerFailed = $global:totalFailed
$compilerIgnored = $global:totalIgnored
$compilerTotal = $compilerPassed + $compilerFailed + $compilerIgnored

# ============================================================================
# TOOLING SUITES
# ============================================================================
$toolingPassed = 0; $toolingFailed = 0; $toolingIgnored = 0

Write-Host ""
Write-Host "TOOLING" -ForegroundColor Yellow

$tooling = @(
    @{pkg="xiom-check";    test=$null; label="checker"},
    @{pkg="xiom-parser";   test=$null; label="parser"},
    @{pkg="xiom-fmt";      test=$null; label="formatter"},
    @{pkg="xiom-lsp";      test=$null; label="lsp"},
    @{pkg="xiom-pkg";      test=$null; label="package-mgr"},
    @{pkg="xiom-doc";      test=$null; label="doc-gen"},
    @{pkg="xiom-ffigen";   test=$null; label="ffi-gen"},
    @{pkg="xiom-mcp";      test=$null; label="mcp-server"},
    @{pkg="xiom-verify";   test=$null; label="verifier"}
)

$global:totalPassed = 0; $global:totalFailed = 0; $global:totalIgnored = 0
foreach ($s in $tooling) {
    Run-Suite $s.pkg $s.test $s.label
}
$toolingPassed = $global:totalPassed
$toolingFailed = $global:totalFailed
$toolingIgnored = $global:totalIgnored
$toolingTotal = $toolingPassed + $toolingFailed + $toolingIgnored

# ============================================================================
# TOTALS
# ============================================================================
$grandPassed  = $compilerPassed  + $toolingPassed
$grandFailed  = $compilerFailed  + $toolingFailed
$grandIgnored = $compilerIgnored + $toolingIgnored
$grandTotal   = $compilerTotal   + $toolingTotal

Write-Host ""
Write-Host "=============================================" -ForegroundColor Magenta
Write-Host "  COMPILER  $compilerTotal/$compilerTotal" -ForegroundColor $(if ($compilerFailed -eq 0){"Green"}else{"Red"})
Write-Host "  TOOLING   $toolingTotal/$toolingTotal"  -ForegroundColor $(if ($toolingFailed -eq 0){"Green"}else{"Red"})
Write-Host "  ----------------------------------------"
if ($grandFailed -eq 0 -and $global:failedSuites.Count -eq 0) {
    Write-Host "  ALL $grandTotal TESTS PASSED" -ForegroundColor Green
    Write-Host "  TOTAL: $grandTotal/$grandTotal tests passed" -ForegroundColor Green
} else {
    Write-Host "  $grandPassed passed, $grandFailed failed ($grandTotal total)" -ForegroundColor Red
    if ($global:failedSuites.Count -gt 0) {
        Write-Host "  Failures: $($global:failedSuites -join ', ')" -ForegroundColor Red
    }
}
Write-Host "=============================================" -ForegroundColor Magenta

# Final summary line for release tags
if ($grandFailed -eq 0 -and $global:failedSuites.Count -eq 0) {
    Write-Host "  Release tag: $grandTotal/$grandTotal tests" -ForegroundColor Cyan
}
