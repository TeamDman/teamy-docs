# SPDX-License-Identifier: MPL-2.0
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$toolLock = Get-Content -Raw -LiteralPath (Join-Path $repoRoot 'tool-lock.json') | ConvertFrom-Json
if ($toolLock.version -ne 1) { throw 'Unsupported tool lock.' }
$toolDir = Join-Path $repoRoot ('.tools/mdbook/' + $toolLock.toolVersion)
$executablePath = Join-Path $toolDir 'mdbook.exe'
$receiptPath = Join-Path $toolDir 'receipt.json'
if (-not (Test-Path -LiteralPath $executablePath) -or -not (Test-Path -LiteralPath $receiptPath)) {
    throw 'Run scripts/bootstrap-mdbook.ps1 first.'
}
$receipt = Get-Content -Raw -LiteralPath $receiptPath | ConvertFrom-Json
$binaryDigest = (Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($receipt.version -ne 1 -or $receipt.assetSha256 -ne $toolLock.assetSha256 -or $receipt.binarySha256 -ne $binaryDigest) {
    throw 'Tool receipt or executable mismatch; inspect and bootstrap the pinned release again.'
}
$toolVersion = & $executablePath --version
if ($LASTEXITCODE -ne 0 -or $toolVersion.Trim() -ne ('mdbook v' + $toolLock.toolVersion)) {
    throw 'Unexpected mdBook version.'
}
& $executablePath build $repoRoot
if ($LASTEXITCODE -ne 0) { throw 'mdBook build failed.' }
Write-Output ('Built book with ' + $toolVersion.Trim())
