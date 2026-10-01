# SPDX-License-Identifier: MPL-2.0
# Download a pinned release archive; extract only the reviewed executable.
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$toolLock = Get-Content -Raw -LiteralPath (Join-Path $repoRoot 'tool-lock.json') | ConvertFrom-Json
if ($toolLock.version -ne 1 -or $toolLock.tool -ne 'mdbook') { throw 'Unsupported tool lock.' }
if (-not $IsWindows -or [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') {
    throw 'This bootstrap supports Windows x86-64 only; use the pinned official asset for your platform.'
}
$toolDir = Join-Path $repoRoot ('.tools/mdbook/' + $toolLock.toolVersion)
$archivePath = Join-Path $toolDir 'release.zip'
$executablePath = Join-Path $toolDir 'mdbook.exe'
$receiptPath = Join-Path $toolDir 'receipt.json'
[void][System.IO.Directory]::CreateDirectory($toolDir)
if (-not (Test-Path -LiteralPath $archivePath)) {
    Invoke-WebRequest -Uri $toolLock.assetUrl -OutFile $archivePath
}
$archiveDigest = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($archiveDigest -ne $toolLock.assetSha256) {
    throw 'Release archive checksum mismatch. Nothing was extracted or executed; inspect the local archive.'
}
Add-Type -AssemblyName System.IO.Compression
$archive = [System.IO.Compression.ZipFile]::OpenRead($archivePath)
try {
    $entries = @($archive.Entries | Where-Object { $_.FullName -eq 'mdbook.exe' })
    if ($entries.Count -ne 1) { throw 'Expected exactly one mdbook.exe at archive root.' }
    $inputStream = $entries[0].Open()
    try {
        $outputStream = [System.IO.File]::Open($executablePath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
        try { $inputStream.CopyTo($outputStream) } finally { $outputStream.Dispose() }
    } finally { $inputStream.Dispose() }
} finally { $archive.Dispose() }
$binaryDigest = (Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash.ToLowerInvariant()
$toolVersion = & $executablePath --version
if ($LASTEXITCODE -ne 0 -or $toolVersion.Trim() -ne ('mdbook v' + $toolLock.toolVersion)) {
    throw 'Downloaded executable did not report the pinned version.'
}
[ordered]@{
    version = 1
    toolVersion = $toolLock.toolVersion
    assetSha256 = $archiveDigest
    binarySha256 = $binaryDigest
} | ConvertTo-Json | Set-Content -LiteralPath $receiptPath -Encoding utf8
Write-Output ('Verified project-local ' + $toolVersion.Trim())
