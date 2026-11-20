# SPDX-License-Identifier: MPL-2.0
# Fixture-only renderer comparison; no downloads or inference.
[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$CliPath,
 [Parameter(Mandatory)][string]$MdBookPath,
 [Parameter(Mandatory)][string]$ArtifactRoot
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$fixture = Join-Path $PSScriptRoot 'fixtures/book'
$artifact = [IO.Path]::GetFullPath($ArtifactRoot)
[IO.Directory]::CreateDirectory($artifact) | Out-Null
if ((& $MdBookPath --version).Trim() -ne 'mdbook v0.5.4') { throw 'Expected pinned mdBook 0.5.4.' }
$rendered = Join-Path $artifact 'synthetic-book'
& $MdBookPath build $fixture --dest-dir $rendered
if ($LASTEXITCODE -ne 0) { throw 'Synthetic mdBook build failed.' }
$reportJson = & $CliPath sections list --book-root $fixture --output-format json
if ($LASTEXITCODE -ne 0) { throw 'Synthetic section list failed.' }
$reportPath = Join-Path $artifact 'synthetic-sections.json'
[IO.File]::WriteAllText($reportPath, ($reportJson -join [Environment]::NewLine), [Text.UTF8Encoding]::new($false))
$report = $reportJson | ConvertFrom-Json
$checked = 0
foreach ($page in $report.pages) {
 $htmlPath = Join-Path $rendered ([IO.Path]::ChangeExtension($page.path, '.html'))
 $html = [IO.File]::ReadAllText($htmlPath)
 $htmlAnchors = @([regex]::Matches($html, '<h[1-6]\b[^>]*\bid="([^"]*)"') |
  ForEach-Object { [Net.WebUtility]::HtmlDecode($_.Groups[1].Value) })
 $sourceAnchors = @($report.sections | Where-Object {
  $_.page_path -eq $page.path -and $_.kind -eq 'heading'
 } | ForEach-Object { $_.anchor })
 if (($htmlAnchors -join [char]0) -ne ($sourceAnchors -join [char]0)) {
  throw ('Anchor mismatch for ' + $page.path + ': source=' + ($sourceAnchors -join ',') + '; renderer=' + ($htmlAnchors -join ','))
 }
 $checked += $sourceAnchors.Count
}
Write-Output ('Verified ' + $checked + ' synthetic heading anchors against mdBook 0.5.4.')
