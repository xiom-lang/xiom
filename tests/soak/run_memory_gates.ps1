# Memory ownership gates (m375 / engine Vec leak).
#
# Each witness compiles with the working compiler and runs under a hard
# working-set cap. The pre-fix compiler exceeds the cap (or dies); the fixed
# compiler stays flat. Run from the repository root:
#
#   pwsh -File tests/soak/run_memory_gates.ps1 -Compiler target/release/xiom.exe
#
# The gate is intentionally a process-memory check, not an IR-shape check:
# the bug was "Vec memory is never freed", which only RSS can prove.

param(
    [string]$Compiler = "target/release/xiom.exe",
    [string]$Stdlib = "",
    [int]$TimeoutSec = 120
)

# Native compiler diagnostics go to stderr; do not turn them into terminating
# PowerShell errors. Each step checks $LASTEXITCODE / the process explicitly.
$ErrorActionPreference = "Continue"
$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$Compiler = (Resolve-Path $Compiler).Path
if ([string]::IsNullOrEmpty($Stdlib)) {
    $Stdlib = Join-Path $Repo "stdlib"
}
if (-not (Test-Path (Join-Path $Stdlib "package.xi"))) {
    Write-Error "stdlib checkout not found at $Stdlib (set -Stdlib)"
}
$env:XIOM_STDLIB = (Resolve-Path $Stdlib).Path
$OutDir = Join-Path $env:TEMP "xiom-memory-gates"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# name = probe stem, capMB = hard working-set cap.
$Gates = @(
    @{ Name = "vec_with_len_loop_leak";  CapMB = 128 },
    @{ Name = "vec_call_return_loop_leak"; CapMB = 128 },
    @{ Name = "read_at_loop_leak";        CapMB = 128 },
    @{ Name = "escaped_frame_vec_leak";   CapMB = 512 },
    @{ Name = "arena_scope_loop_leak";    CapMB = 512 },
    @{ Name = "vec_loop_control_leak"; CapMB = 128 },
    @{ Name = "ffi_owns_return_leak";  CapMB = 128 },
    @{ Name = "str_loop_leak";         CapMB = 24 },
    @{ Name = "vec_uint8_packed_slots"; CapMB = 24 },
    @{ Name = "vecprobe_index_loop"; Source = "vecprobe.xi"; CapMB = 128;
       Env = @{ PROBE_VEC_MODE = "0"; PROBE_VEC_N = "800000" } },
    @{ Name = "vecprobe_index_fn";   Source = "vecprobe.xi"; CapMB = 128;
       Env = @{ PROBE_VEC_MODE = "1"; PROBE_VEC_N = "800000" } }
)

function Invoke-Capped([string]$Exe, [int]$CapMB, [int]$TimeoutSec, [hashtable]$Env = @{}) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Exe
    $psi.WorkingDirectory = $Repo
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    foreach ($key in $Env.Keys) {
        $psi.EnvironmentVariables[$key] = [string]$Env[$key]
    }
    $p = [System.Diagnostics.Process]::Start($psi)
    $peak = 0
    $killed = $false
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while (-not $p.HasExited) {
        try {
            $p.Refresh()
            $ws = [int]($p.WorkingSet64 / 1MB)
            if ($ws -gt $peak) { $peak = $ws }
            if ($ws -gt $CapMB) { $p.Kill(); $killed = $true; break }
        } catch { }
        if ($sw.Elapsed.TotalSeconds -gt $TimeoutSec) { $p.Kill(); $killed = $true; break }
        Start-Sleep -Milliseconds 20
    }
    $p.WaitForExit()
    $stdout = $p.StandardOutput.ReadToEnd()
    $stderr = $p.StandardError.ReadToEnd()
    [pscustomobject]@{
        ExitCode = $p.ExitCode
        PeakMB = $peak
        Killed = $killed
        Stdout = $stdout
        Stderr = $stderr
    }
}

$failed = @()
foreach ($gate in $Gates) {
    $name = $gate.Name
    $srcName = if ($gate.Source) { [string]$gate.Source } else { "$name.xi" }
    $src = Join-Path $PSScriptRoot $srcName
    $exe = Join-Path $OutDir "$name.exe"
    if (-not (Test-Path $src)) {
        $failed += "$name (missing source)"
        continue
    }
    & $Compiler --no-cache --force --timeout 600 -o $exe $src 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $exe)) {
        $failed += "$name (compile failed)"
        Write-Output ("FAIL " + $name + " compile")
        continue
    }
    $envTable = if ($gate.Env) { $gate.Env } else { @{} }
    $r = Invoke-Capped -Exe $exe -CapMB $gate.CapMB -TimeoutSec $TimeoutSec -Env $envTable
    if ($r.Killed -or $r.ExitCode -ne 0) {
        $failed += "$name"
        Write-Output ("FAIL " + $name + " peakMB=" + $r.PeakMB + " exit=" + $r.ExitCode + " killed=" + $r.Killed)
        if ($r.Stderr) { Write-Output ("  stderr: " + $r.Stderr.Trim()) }
    } else {
        Write-Output ("PASS " + $name + " peakMB=" + $r.PeakMB + " exit=0")
    }
}

if ($failed.Count -gt 0) {
    Write-Output ("MEMORY GATES FAILED: " + ($failed -join ", "))
    exit 1
}
Write-Output "MEMORY GATES GREEN"
