# Apinox Installer for Windows
# Usage: irm https://apinox.denisetiya.site/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "denisetiya/apinox"
$InstallDir = if ($env:APINOX_INSTALL_DIR) { $env:APINOX_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }
$Version = if ($env:APINOX_VERSION) { $env:APINOX_VERSION } else { "latest" }

# Detect architecture
$Arch = if ([System.Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }
} else {
    Write-Host "✗ 32-bit Windows is not supported" -ForegroundColor Red
    exit 1
}

$Platform = "windows-$Arch.exe"
$Binary = "apinox.exe"

# Get latest version
if ($Version -eq "latest") {
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
        $Version = $release.tag_name
    } catch {
        Write-Host "! Could not determine latest version, trying first release..." -ForegroundColor Yellow
        $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases" -UseBasicParsing
        $Version = $releases[0].tag_name
    }
}

Write-Host ""
Write-Host "  ╔══════════════════════════════════╗" -ForegroundColor Blue
Write-Host "  ║       Apinox Installer           ║" -ForegroundColor Blue
Write-Host "  ╚══════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""

# Download
$url = "https://github.com/$Repo/releases/download/$Version/apinox-$Platform"
$tmpFile = Join-Path $env:TEMP "apinox-$Binary"

Write-Host "▸ Downloading apinox $Version for $Platform..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $url -OutFile $tmpFile -UseBasicParsing
} catch {
    Write-Host "✗ Download failed: $url" -ForegroundColor Red
    exit 1
}

# Install
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$destFile = Join-Path $InstallDir $Binary
Move-Item -Path $tmpFile -Destination $destFile -Force
Write-Host "✓ Installed to $destFile" -ForegroundColor Green

# Add to PATH if needed
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$currentPath", "User")
    $env:Path = "$InstallDir;$env:Path"
    Write-Host "! Added $InstallDir to PATH" -ForegroundColor Yellow
    Write-Host "  Restart your terminal for changes to take effect." -ForegroundColor Yellow
}

# Verify
Write-Host ""
try {
    $ver = & $destFile --version 2>&1
    Write-Host "✓ Installed: $ver" -ForegroundColor Green
} catch {
    Write-Host "! Binary installed but could not verify version" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  Done! Run " -NoNewline
Write-Host "apinox --help" -ForegroundColor Blue -NoNewline
Write-Host " to get started."
Write-Host ""
