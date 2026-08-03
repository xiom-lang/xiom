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

function Run-Suite($pkg, $test, $label, $extraArgs) {
    $name = $label
    Write-Host "  $($name.PadRight(22))" -NoNewline
    $args = @("test", "-p", $pkg)
    if ($test) { $args += "--test", $test }
    if ($extraArgs) { $args += $extraArgs }
    $output = & cargo $args 2>&1 | Out-String

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
    @{pkg="xiom-codegen"; test="e2e_tests";                 label="e2e";                      extra=$null},
    @{pkg="xiom-codegen"; test="feature_regression_tests";   label="feature-regression";        extra=$null},
    @{pkg="xiom-codegen"; test="stdlib_execution_tests";     label="stdlib-execution";          extra=$null},
    @{pkg="xiom-codegen"; test="stdlib_tests";               label="stdlib-compile";            extra=$null},
    @{pkg="xiom-codegen"; test="integration_tests";          label="integration";               extra=$null},
    @{pkg="xiom-codegen"; test="diff_tests";                 label="diff";                      extra=$null},
    @{pkg="xiom-codegen"; test="full_diff_tests";            label="full-diff";                 extra=$null},
    @{pkg="xiom-codegen"; test="fuzz_tests";                 label="fuzz";                      extra=$null},
    @{pkg="xiom-codegen"; test="robustness_tests";           label="robustness";                extra=$null},
    @{pkg="xiom-codegen"; test=$null;                        label="codegen-unit";              extra=$null},
    @{pkg="xiom-lexer";   test=$null;                        label="lexer";                     extra=$null},
    @{pkg="xiom-parser";  test=$null;                        label="parser";                    extra=@("--","--test-threads=2")},
    @{pkg="xiom-check";   test=$null;                        label="checker";                   extra=@("--","--test-threads=2")},
    @{pkg="xiom-ctfe";    test=$null;                        label="ctfe";                      extra=$null},
    @{pkg="xiom-graph";   test=$null;                        label="graph";                     extra=$null},
    @{pkg="xiom-verify";  test="verifier_tests";             label="verifier";                  extra=$null},
    @{pkg="xiom-jit";     test=$null;                        label="jit";                       extra=$null},
    @{pkg="xiom";         test="scripting_tests";            label="scripting";                 extra=$null},
    @{pkg="xiom";         test="diff_tests";                 label="script-diff";               extra=$null}
)

$global:totalPassed = 0; $global:totalFailed = 0; $global:totalIgnored = 0
foreach ($s in $compiler) {
    Run-Suite $s.pkg $s.test $s.label $s.extra
}
$compilerPassed = $global:totalPassed
$compilerFailed = $global:totalFailed
$compilerIgnored = $global:totalIgnored
$compilerTotal = $compilerPassed + $compilerFailed

# ============================================================================
# TOOLING SUITES
# ============================================================================
$toolingPassed = 0; $toolingFailed = 0; $toolingIgnored = 0

Write-Host ""
Write-Host "TOOLING" -ForegroundColor Yellow

$tooling = @(
    @{pkg="xiom-fmt";      test=$null; label="formatter"},
    @{pkg="xiom-lsp";      test=$null; label="lsp"},
    @{pkg="xiom-pkg";      test=$null; label="package-mgr"},
    @{pkg="xiom-doc";      test=$null; label="doc-gen"},
    @{pkg="xiom-ffigen";   test=$null; label="ffi-gen"},
    @{pkg="xiom-mcp";      test=$null; label="mcp-server"},
    @{pkg="xiom-dbg";      test=$null; label="debugger"},
    @{pkg="xiom-display";  test=$null; label="display"}
)

$global:totalPassed = 0; $global:totalFailed = 0; $global:totalIgnored = 0
foreach ($s in $tooling) {
    Run-Suite $s.pkg $s.test $s.label $null
}
$toolingPassed = $global:totalPassed
$toolingFailed = $global:totalFailed
$toolingIgnored = $global:totalIgnored
$toolingTotal = $toolingPassed + $toolingFailed

# ============================================================================
# TOTALS
# ============================================================================
$grandPassed  = $compilerPassed  + $toolingPassed
$grandFailed  = $compilerFailed  + $toolingFailed
$grandIgnored = $compilerIgnored + $toolingIgnored
$grandTotal   = $compilerTotal   + $toolingTotal

Write-Host ""
Write-Host "=============================================" -ForegroundColor Magenta
Write-Host "  COMPILER  $compilerPassed/$compilerTotal" -ForegroundColor $(if ($compilerFailed -eq 0){"Green"}else{"Red"})
Write-Host "  TOOLING   $toolingPassed/$toolingTotal"  -ForegroundColor $(if ($toolingFailed -eq 0){"Green"}else{"Red"})
Write-Host "  ----------------------------------------"
if ($grandFailed -eq 0 -and $global:failedSuites.Count -eq 0) {
    Write-Host "  ALL $grandTotal TESTS PASSED" -ForegroundColor Green
    Write-Host "  TOTAL: $grandPassed/$grandTotal tests passed" -ForegroundColor Green
    if ($grandIgnored -gt 0) {
        Write-Host "  ($grandIgnored ignored)" -ForegroundColor Yellow
    }
} else {
    Write-Host "  $grandPassed passed, $grandFailed failed ($grandTotal total)" -ForegroundColor Red
    if ($global:failedSuites.Count -gt 0) {
        Write-Host "  Failures: $($global:failedSuites -join ', ')" -ForegroundColor Red
    }
}
Write-Host "=============================================" -ForegroundColor Magenta

# Final summary line for release tags
if ($grandFailed -eq 0 -and $global:failedSuites.Count -eq 0) {
    Write-Host "  Release tag: $grandPassed/$grandTotal tests" -ForegroundColor Cyan
}
