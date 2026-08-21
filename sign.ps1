#Requires -Version 5.1
<#
.SYNOPSIS
    XIOM Binary Signing -- Authenticode code signing for Windows executables.
.DESCRIPTION
    Signs all .exe/.dll files in a directory using a code signing certificate.
    Supports self-signed certs (dev), .pfx imports (CI), and Windows certificate store.

.PARAMETER Path
    Directory containing binaries to sign (e.g., release/xiom-v0.48.9/bin/).

.PARAMETER CertificatePath
    Path to .pfx or .cer certificate file (optional -- uses cert: thumbprint or store lookup).

.PARAMETER CertificateThumbprint
    Thumbprint of a certificate already in the user's certificate store.

.PARAMETER CertificatePassword
    Password for .pfx file (use with -CertificatePath).

.PARAMETER TimestampServer
    RFC 3161 timestamp server URL (default: http://timestamp.digicert.com).

.PARAMETER CreateSelfSigned
    Creates a self-signed code signing certificate for local testing.
    Writes to .\xiom_test_cert.pfx with password "xiom".

.PARAMETER WhatIf
    Show what would be signed without actually signing.

.EXAMPLE
    # Sign with a .pfx file
    .\sign.ps1 -Path release\xiom-v0.48.9\bin -CertificatePath .\xiom_code_sign.pfx -CertificatePassword "secret"

.EXAMPLE
    # Sign using a certificate in the Windows store
    .\sign.ps1 -Path release\xiom-v0.48.9\bin -CertificateThumbprint "A1B2C3D4..."

.EXAMPLE
    # Create a self-signed cert for local testing
    .\sign.ps1 -CreateSelfSigned
#>

param(
    [string]$Path,
    [string]$CertificatePath,
    [string]$CertificateThumbprint,
    [string]$CertificatePassword,
    [string]$TimestampServer = "http://timestamp.digicert.com",
    [switch]$CreateSelfSigned,
    [switch]$WhatIf
)

$ErrorActionPreference = "Stop"

# ============================================================================
# Self-signed certificate creation (local testing)
# ============================================================================
if ($CreateSelfSigned) {
    Write-Host "  Creating self-signed code signing certificate..." -ForegroundColor Cyan
    $cert = New-SelfSignedCertificate -Type CodeSigningCert `
        -Subject "CN=XIOM Test Signing" `
        -FriendlyName "XIOM Code Signing (Test)" `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -KeyUsage DigitalSignature `
        -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3")  # Code Signing EKU

    $thumbprint = $cert.Thumbprint
    Write-Host "  Certificate created: $($cert.Subject)" -ForegroundColor Green
    Write-Host "  Thumbprint: $thumbprint" -ForegroundColor Yellow
    Write-Host ""

    # Export to .pfx for reuse
    $pfxPassword = ConvertTo-SecureString -String "xiom" -Force -AsPlainText
    $pfxPath = Join-Path (Get-Location) "xiom_test_cert.pfx"
    Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $pfxPassword | Out-Null
    Write-Host "  Exported to: $pfxPath (password: xiom)" -ForegroundColor Green
    Write-Host ""

    Write-Host "  To sign with this certificate:" -ForegroundColor Cyan
    Write-Host "    .\sign.ps1 -Path release\xiom-v0.48.9\bin -CertificateThumbprint '$thumbprint'" -ForegroundColor White
    Write-Host "    OR" -ForegroundColor DarkGray
    Write-Host "    .\sign.ps1 -Path release\xiom-v0.48.9\bin -CertificatePath '$pfxPath' -CertificatePassword 'xiom'" -ForegroundColor White
    return
}

# ============================================================================
# Resolve certificate
# ============================================================================
if (-not $Path) {
    Write-Host "ERROR: -Path is required. Use -CreateSelfSigned to generate a test certificate." -ForegroundColor Red
    Write-Host "Usage: .\sign.ps1 -Path <directory> [-CertificatePath <pfx> [-CertificatePassword <pw>] | -CertificateThumbprint <thumb>]" -ForegroundColor Yellow
    exit 1
}

$cert = $null

if ($CertificatePath) {
    Write-Host "  Loading certificate from: $CertificatePath" -ForegroundColor Cyan
    if ($CertificatePassword) {
        $securePw = ConvertTo-SecureString -String $CertificatePassword -Force -AsPlainText
        $cert = Get-PfxCertificate -FilePath $CertificatePath
    } else {
        $cert = Get-PfxCertificate -FilePath $CertificatePath
    }
} elseif ($CertificateThumbprint) {
    Write-Host "  Finding certificate in store: $CertificateThumbprint" -ForegroundColor Cyan
    $cert = Get-ChildItem -Path "Cert:\CurrentUser\My" | Where-Object { $_.Thumbprint -eq $CertificateThumbprint }
    if (-not $cert) {
        $cert = Get-ChildItem -Path "Cert:\LocalMachine\My" | Where-Object { $_.Thumbprint -eq $CertificateThumbprint }
    }
} else {
    Write-Host "ERROR: Either -CertificatePath or -CertificateThumbprint is required." -ForegroundColor Red
    Write-Host "  Run .\sign.ps1 -CreateSelfSigned to generate a test certificate." -ForegroundColor Yellow
    exit 1
}

if (-not $cert) {
    Write-Host "ERROR: Could not find or load certificate." -ForegroundColor Red
    exit 1
}

Write-Host "  Certificate: $($cert.Subject)" -ForegroundColor Green
Write-Host "  Valid until: $($cert.NotAfter)" -ForegroundColor DarkGray
Write-Host ""

# ============================================================================
# Sign binaries
# ============================================================================
if (-not (Test-Path $Path)) {
    Write-Host "ERROR: Path not found: $Path" -ForegroundColor Red
    exit 1
}

$binaries = Get-ChildItem -Path $Path -Filter "*.exe" | ForEach-Object { $_.FullName }
$binaries += Get-ChildItem -Path $Path -Filter "*.dll" | ForEach-Object { $_.FullName }

if ($binaries.Count -eq 0) {
    Write-Host "  No .exe or .dll files found in $Path" -ForegroundColor Yellow
    exit 0
}

Write-Host "  Signing $($binaries.Count) binary(s) in $Path" -ForegroundColor Cyan
Write-Host ""

$signParams = @{
    Certificate = $cert
    TimestampServer = $TimestampServer
    HashAlgorithm = "SHA256"
    Force = $true
}

foreach ($binary in $binaries) {
    $name = Split-Path $binary -Leaf
    if ($WhatIf) {
        Write-Host "    [WHATIF] Would sign: $name" -ForegroundColor DarkGray
    } else {
        try {
            Set-AuthenticodeSignature @signParams -FilePath $binary | Out-Null
            $sig = Get-AuthenticodeSignature -FilePath $binary
            if ($sig.Status -eq "Valid") {
                Write-Host "    [SIGNED] $name" -ForegroundColor Green
            } else {
                Write-Host "    [WARN] $name -- status: $($sig.Status)" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "    [FAIL] $name -- $_" -ForegroundColor Red
        }
    }
}

Write-Host ""
Write-Host "  Signing complete." -ForegroundColor Green
if ($WhatIf) {
    Write-Host "  (Use without -WhatIf to actually sign.)" -ForegroundColor DarkGray
}
