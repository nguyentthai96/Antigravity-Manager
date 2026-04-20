# Antigravity Account Switcher Helper Script (PowerShell)
# Usage: .\switch-account.ps1 <email> [project_id]

param(
    [Parameter(Mandatory=$true)]
    [string]$Email,
    
    [Parameter(Mandatory=$false)]
    [string]$ProjectId = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$AccountsFile = Join-Path $ScriptDir "antigravity_accounts.json"
$CliBin = Join-Path $ScriptDir "target\release\antigravity-cli.exe"

# Check if accounts file exists
if (-not (Test-Path $AccountsFile)) {
    Write-Host "Error: Accounts file not found: $AccountsFile" -ForegroundColor Red
    exit 1
}

# Check if CLI binary exists
if (-not (Test-Path $CliBin)) {
    Write-Host "CLI binary not found. Building..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
}

# Build command
$Args = @(
    "--accounts-file", $AccountsFile,
    "--email", $Email
)

if ($ProjectId) {
    $Args += "--project-id", $ProjectId
}

Write-Host "Switching to account: $Email" -ForegroundColor Green
if ($ProjectId) {
    Write-Host "Using project ID: $ProjectId" -ForegroundColor Green
}
Write-Host ""

# Execute
& $CliBin @Args

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✓ Account switch completed successfully!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "✗ Account switch failed" -ForegroundColor Red
    exit 1
}
