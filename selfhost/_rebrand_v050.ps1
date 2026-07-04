# To process axiomc_v050.ax, run this PowerShell script:
# powershell -ExecutionPolicy Bypass -File "E:\Projects\AXIOM\selfhost\_rebrand.ps1"
#
# Or manually: this file differs from the original in exactly 2 lines:
#   Line 1:   AXIOM -> XIOM
#   Line 666: axiomc.ax -> xiomc.xi

$selfhost = "E:\Projects\AXIOM\selfhost"
$content = Get-Content -Path "$selfhost\axiomc_v050.ax" -Raw
$content = $content -replace 'AXIOM', 'XIOM'
$content = $content -replace 'axiomc\.ax', 'xiomc.xi'
[System.IO.File]::WriteAllText("$selfhost\xiomc_v050.xi", $content)
Write-Host "Processed: axiomc_v050.ax -> xiomc_v050.xi"
