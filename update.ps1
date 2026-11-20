# SPDX-License-Identifier: MPL-2.0
[CmdletBinding()]
param([switch]$Offline)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath($PSScriptRoot)
$manifestPath = Join-Path $repoRoot 'Cargo.toml'
if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw 'Cargo.toml is required for local installation.'
}
$lockPath = Join-Path $repoRoot 'Cargo.lock'
if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) {
    throw 'Cargo.lock is required for locked installation.'
}
if (-not (Test-Path -LiteralPath (Join-Path $repoRoot 'book.toml') -PathType Leaf)) {
    throw 'book.toml is required to register this repository as the default book.'
}
$lockBefore = (Get-FileHash -LiteralPath $lockPath -Algorithm SHA256).Hash
$cargoArguments = @('install', '--path', $repoRoot, '--locked')
if ($Offline) { $cargoArguments += '--offline' }
& cargo @cargoArguments
if ($LASTEXITCODE -ne 0) { throw ('Cargo install failed with exit code ' + $LASTEXITCODE) }
if ((Get-FileHash -LiteralPath $lockPath -Algorithm SHA256).Hash -ne $lockBefore) {
    throw 'Cargo.lock changed during locked installation; review the change.'
}
# Rust owns platform config paths; this uses the same locked source as the installed binary.
$configArguments = @('run', '--manifest-path', $manifestPath, '--bin', 'teamy-docs', '--locked', '--release', '--quiet')
if ($Offline) { $configArguments += '--offline' }
$configArguments += @('--', 'book-root', 'set', '--path', $repoRoot)
& cargo @configArguments
if ($LASTEXITCODE -ne 0) { throw ('Cargo install succeeded, but default book registration failed with exit code ' + $LASTEXITCODE) }
if ((Get-FileHash -LiteralPath $lockPath -Algorithm SHA256).Hash -ne $lockBefore) {
    throw 'Cargo.lock changed during default book registration; review the change.'
}
