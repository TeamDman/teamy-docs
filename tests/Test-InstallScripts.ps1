# SPDX-License-Identifier: MPL-2.0
# Mock Cargo only: no installation, config, network, PATH, or security changes.
[CmdletBinding()]
param([Parameter(Mandatory)][string]$ArtifactRoot)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$artifact = [IO.Path]::GetFullPath($ArtifactRoot)
[IO.Directory]::CreateDirectory($artifact) | Out-Null
$lockPath = Join-Path $repoRoot 'Cargo.lock'
$lockBefore = (Get-FileHash -LiteralPath $lockPath -Algorithm SHA256).Hash
$global:TeamyDocsMockCalls = [Collections.Generic.List[object]]::new()
$global:TeamyDocsMockInstallExit = 0
$global:TeamyDocsMockConfigExit = 0
function cargo {
    $global:TeamyDocsMockCalls.Add([string[]]$args)
    if ($args[0] -eq 'run') { $global:LASTEXITCODE = $global:TeamyDocsMockConfigExit }
    else { $global:LASTEXITCODE = $global:TeamyDocsMockInstallExit }
}
$originalLocation = Get-Location
try {
    Set-Location -LiteralPath $artifact
    foreach ($name in @('update.ps1', 'install.ps1')) {
        $global:TeamyDocsMockCalls.Clear()
        & (Join-Path $repoRoot $name) -Offline
        $expectedInstall = @('install', '--path', $repoRoot, '--locked', '--offline')
        $expectedConfig = @('run', '--manifest-path', (Join-Path $repoRoot 'Cargo.toml'), '--bin', 'teamy-docs', '--locked', '--release', '--quiet', '--offline', '--', 'book-root', 'set', '--path', $repoRoot)
        if ($global:TeamyDocsMockCalls.Count -ne 2 -or
            ($global:TeamyDocsMockCalls[0] -join [char]0) -ne ($expectedInstall -join [char]0) -or
            ($global:TeamyDocsMockCalls[1] -join [char]0) -ne ($expectedConfig -join [char]0)) {
            throw ('Incorrect locked install/registration arguments from another CWD: ' + $name)
        }
    }
    $global:TeamyDocsMockCalls.Clear()
    $global:TeamyDocsMockInstallExit = 42
    $failed = $false
    try { & (Join-Path $repoRoot 'install.ps1') }
    catch { $failed = $_.Exception.Message -eq 'Cargo install failed with exit code 42' }
    if (-not $failed -or $global:TeamyDocsMockCalls.Count -ne 1) {
        throw 'Native install failure must stop before registration.'
    }
    $global:TeamyDocsMockInstallExit = 0
    $global:TeamyDocsMockConfigExit = 43
    $failed = $false
    try { & (Join-Path $repoRoot 'install.ps1') }
    catch { $failed = $_.Exception.Message -eq 'Cargo install succeeded, but default book registration failed with exit code 43' }
    if (-not $failed) { throw 'Native registration failure was not reported.' }
    $global:TeamyDocsMockConfigExit = 0
    foreach ($missing in @('Cargo.lock', 'book.toml')) {
        $guardRepo = Join-Path $artifact ('missing-' + $missing)
        [IO.Directory]::CreateDirectory($guardRepo) | Out-Null
        Copy-Item -LiteralPath (Join-Path $repoRoot 'update.ps1') -Destination (Join-Path $guardRepo 'update.ps1')
        [IO.File]::WriteAllText((Join-Path $guardRepo 'Cargo.toml'), "[package]" + [Environment]::NewLine + "name = 'synthetic'" + [Environment]::NewLine)
        if ($missing -ne 'Cargo.lock') { [IO.File]::WriteAllText((Join-Path $guardRepo 'Cargo.lock'), '# synthetic lock') }
        $global:TeamyDocsMockCalls.Clear()
        $failed = $false
        try { & (Join-Path $guardRepo 'update.ps1') }
        catch { $failed = $_.Exception.Message -eq $(if ($missing -eq 'Cargo.lock') { 'Cargo.lock is required for locked installation.' } else { 'book.toml is required to register this repository as the default book.' }) }
        if (-not $failed -or $global:TeamyDocsMockCalls.Count -ne 0) {
            throw ('Missing ' + $missing + ' must fail before Cargo is called.')
        }
    }
    if ((Get-FileHash -LiteralPath $lockPath -Algorithm SHA256).Hash -ne $lockBefore) {
        throw 'Lockfile changed during mocked installation checks.'
    }
} finally {
    Set-Location -LiteralPath $originalLocation.Path
    Remove-Variable -Name TeamyDocsMockCalls,TeamyDocsMockInstallExit,TeamyDocsMockConfigExit -Scope Global
}
Write-Output 'Install/update checks passed: unrelated CWD, both locked/offline calls, installation and registration failures, missing lock/book guards; Cargo.lock unchanged.'
