<#
Build script for Windows (PowerShell).

Runs a release build for the Rust workspace, installs selected crates (including `hielements-mcp`),
and builds the VS Code extension if present.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
Push-Location $scriptDir

Write-Host "[hielements] Starting full Windows build..."

# If invoked from scripts/, go up one level so we run in repo root
if (-not (Test-Path "Cargo.toml")) {
    Push-Location ..
}

# Prepare output directories and log paths
$repoRoot = Get-Location
$outDir = Join-Path $repoRoot 'build-output'
New-Item -ItemType Directory -Path $outDir -Force | Out-Null

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$logPath = Join-Path $outDir ("build-$timestamp.log")
$htmlPath = Join-Path $outDir ("build-$timestamp.html")

Start-Transcript -Path $logPath -Force

$buildSucceeded = $true

try {
    Write-Host "[hielements] Building Rust workspace (release)..."
    & cargo build --workspace --release

Write-Host "[hielements] Installing selected crates (will include mcp)..."
$crates = @('crates/hielements-cli','crates/hielements-mcp')
foreach ($c in $crates) {
    if (Test-Path $c) {
        Write-Host "[hielements] Installing $c..."
        & cargo install --path $c --force
    } else {
        Write-Host "[hielements] Skipping $c (not found)"
    }
}

$vsCodeDir = 'vscode-extension'
if (Test-Path $vsCodeDir) {

        Write-Host "[hielements] Building VS Code extension..."
        Push-Location $vsCodeDir
        if (Test-Path 'package.json') {
            Write-Host "[hielements] Installing Node dependencies (npm install)..."

            $npmCmd = $null
            $npmCmdObj = Get-Command npm.cmd -ErrorAction SilentlyContinue
            if ($npmCmdObj) { $npmCmd = $npmCmdObj.Source } else { $npmCmd = (Get-Command npm -ErrorAction SilentlyContinue).Source }

            if (-not $npmCmd) {
                Write-Host "[hielements] npm not found in PATH; skipping npm steps."
            } else {
                try {
                    & $npmCmd install
                } catch {
                    Write-Host "[hielements] npm install failed: $_. Exception.Message"
                    Write-Host "[hielements] Continuing without failing the whole build."
                }

                Write-Host "[hielements] Running npm build (if available)..."
                try {
                    & $npmCmd run build
                } catch {
                    try {
                        & $npmCmd run compile
                    } catch {
                        Write-Host "[hielements] No build/compile script in package.json or step failed; skipping build step."
                    }
                }
            }
        } else {
            Write-Host "[hielements] No package.json in vscode-extension; skipping npm steps."
        }
        Pop-Location
    } else {
        Write-Host "[hielements] No vscode-extension directory found; skipping."
    }

    } catch {
        Write-Host "[hielements] ERROR: $_"
        $buildSucceeded = $false
    } finally {
        try { Stop-Transcript } catch {}

        # Create a simple rendered HTML file that includes the log and a small summary
        try {
            $logContent = Get-Content -Path $logPath -Raw -ErrorAction SilentlyContinue
            if (-not $logContent) { $logContent = "(no log captured)" }

            $statusText = if ($buildSucceeded) { 'Success' } else { 'Failed' }
            $html = @"
    <!doctype html>
    <html>
      <head>
        <meta charset="utf-8" />
        <title>hielements build - $timestamp</title>
        <style>body { font-family: system-ui, Arial; padding: 1rem; } pre { background:#f6f8fa; padding:1rem; overflow:auto; }</style>
      </head>
      <body>
        <h1>hielements build — $timestamp</h1>
        <p><strong>Status:</strong> $statusText</p>
        <h2>Raw Log</h2>
        <pre>$([System.Web.HttpUtility]::HtmlEncode($logContent))</pre>
      </body>
    </html>
    "@

            $html | Out-File -FilePath $htmlPath -Encoding utf8
            Write-Host "[hielements] Log written to: $logPath"
            Write-Host "[hielements] Rendered HTML written to: $htmlPath"
        } catch {
            Write-Host "[hielements] Failed to write rendered HTML: $_"
        }

        # Ensure we return to original locations
        while ((Get-Location).Path -ne $scriptDir) { try { Pop-Location } catch { break } }

        if ($buildSucceeded) {
            Write-Host "[hielements] Build finished successfully."
            Exit 0
        } else {
            Write-Host "[hielements] Build finished with errors. See log and HTML in build-output."
            Exit 1
        }
    }
