# SPDX-License-Identifier: MPL-2.0
# Synthetic contract validation only; never runs a model.
[CmdletBinding()]
param([Parameter(Mandatory)][string]$ReportPath)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$mapped = @(& (Join-Path $PSScriptRoot 'New-SectionDecideRequests.ps1') -ReportPath $ReportPath -Query '--help')
if ($mapped.Count -lt 1) { throw 'Expected synthetic sections.' }
foreach ($envelope in $mapped) {
 $json = $envelope.request | ConvertTo-Json -Depth 8 -Compress
 $request = $json | ConvertFrom-Json -AsHashtable
 if (((@($request.Keys | Sort-Object)) -join ',') -ne 'input,model,options,question,version') { throw 'Unexpected request fields.' }
 if ($request.version -ne 1 -or $request.model -ne 'julia-1') { throw 'Invalid request version/model.' }
 if ($request.input.visuals.Count -ne 0 -or [string]::IsNullOrEmpty($request.input.text)) { throw 'Invalid input.' }
 if (($request.options.id -join ',') -ne 'relevant,irrelevant') { throw 'Invalid option IDs.' }
 foreach ($option in $request.options) {
  if ($option.content.visuals.Count -ne 0 -or [string]::IsNullOrEmpty($option.content.text)) { throw 'Invalid option content.' }
 }
 if ([Text.Encoding]::UTF8.GetByteCount($json) -gt 2097152) { throw 'Oversized payload.' }
 if ([string]::IsNullOrEmpty($envelope.section_id)) { throw 'Missing provenance.' }
}
$failed = $false
try { & (Join-Path $PSScriptRoot 'New-SectionDecideRequests.ps1') -ReportPath $ReportPath -Query '--help' -MaxSectionCharacters 1 | Out-Null }
catch { $failed = $_.Exception.Message -like 'Section requires explicit chunking:*' }
if (-not $failed) { throw 'Oversized section rejection failed.' }
$mapper = Join-Path $PSScriptRoot 'New-SectionDecideRequests.ps1'
$failed = $false
try { & $mapper -ReportPath $ReportPath -Query '--help' -MaxReportBytes 1 | Out-Null }
catch { $failed = $_.Exception.Message -eq 'Section report exceeds byte limit.' }
if (-not $failed) { throw 'Bounded report rejection failed.' }
$temporaryFixture = [IO.Path]::GetTempFileName()
try {
 $negative = [IO.File]::ReadAllText($ReportPath) | ConvertFrom-Json
 $negative.sections[0].id = $null
 [IO.File]::WriteAllText($temporaryFixture, ($negative | ConvertTo-Json -Depth 12), [Text.UTF8Encoding]::new($false))
 $failed = $false
 try { & $mapper -ReportPath $temporaryFixture -Query '--help' | Out-Null }
 catch { $failed = $_.Exception.Message -eq 'Invalid section string: id' }
 if (-not $failed) { throw 'Invalid provenance rejection failed.' }
 [IO.File]::WriteAllBytes($temporaryFixture, [byte[]]@(0xC3,0x28))
 $failed = $false
 try { & $mapper -ReportPath $temporaryFixture -Query '--help' | Out-Null }
 catch { $failed = $true }
 if (-not $failed) { throw 'Invalid UTF8 rejection failed.' }
} finally { Remove-Item -LiteralPath $temporaryFixture }
Write-Output ('Validated ' + $mapped.Count + ' version-1 Decide requests; length, report-byte, provenance, UTF8 rejection; no inference.')
