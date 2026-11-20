# SPDX-License-Identifier: MPL-2.0
# Pure mapping only: never starts a server or invokes inference.
[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$ReportPath,
 [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$Query,
 [ValidateRange(1,10000)][int]$MaxSections = 10000,
 [ValidateRange(1,2097152)][int]$MaxSectionCharacters = 12000,
 [ValidateRange(1,134217728)][long]$MaxReportBytes = 134217728
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$stream = [IO.File]::OpenRead($ReportPath)
$buffer = [byte[]]::new(8192)
$bytes = [IO.MemoryStream]::new()
try {
 while (($count = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
  if ($bytes.Length + $count -gt $MaxReportBytes) { throw 'Section report exceeds byte limit.' }
  $bytes.Write($buffer, 0, $count)
 }
 $report = [Text.UTF8Encoding]::new($false, $true).GetString($bytes.ToArray()) | ConvertFrom-Json
} finally { $stream.Dispose(); $bytes.Dispose() }
if ($report.schema_version -ne 1 -or $report.content_basis -ne 'markdown-source') { throw 'Unsupported report.' }
$sections = @($report.sections)
if ($sections.Count -gt $MaxSections) { throw 'Section count exceeds limit.' }
# Validate all requests first, avoiding successful partial work.
$envelopes = [Collections.Generic.List[object]]::new()
$index = 0
foreach ($section in $sections) {
 foreach ($name in @('id', 'page_path')) {
  if ($section.$name -isnot [string] -or [string]::IsNullOrWhiteSpace($section.$name)) { throw ('Invalid section string: ' + $name) }
 }
 foreach ($name in @('title', 'anchor', 'text')) {
  if ($section.$name -isnot [string]) { throw ('Invalid section string: ' + $name) }
 }
 if ($section.heading_hierarchy -isnot [Array]) { throw 'Invalid heading hierarchy.' }
 foreach ($heading in $section.heading_hierarchy) {
  if ($null -eq $heading -or $heading.title -isnot [string]) { throw 'Invalid hierarchy title.' }
 }
 if ($section.text.Length -gt $MaxSectionCharacters) { throw ('Section requires explicit chunking: ' + $section.id) }
 $headingPath = (@($section.heading_hierarchy) | ForEach-Object { $_.title }) -join ' > '
 $context = 'Page: ' + $section.page_path + [Environment]::NewLine +
  'Heading path: ' + $headingPath + [Environment]::NewLine +
  'Section: ' + $section.title + [Environment]::NewLine + [Environment]::NewLine + $section.text
 $request = [ordered]@{
  version = 1
  model = 'julia-1'
  question = 'Is this documentation section relevant to ' + $Query + '?'
  input = [ordered]@{ text = $context; visuals = @() }
  options = @(
   [ordered]@{ id = 'relevant'; content = [ordered]@{ text = 'Relevant'; visuals = @() } },
   [ordered]@{ id = 'irrelevant'; content = [ordered]@{ text = 'Not relevant'; visuals = @() } }
  )
 }
 $payload = $request | ConvertTo-Json -Depth 8 -Compress
 if ([Text.Encoding]::UTF8.GetByteCount($payload) + [Text.Encoding]::UTF8.GetByteCount([Environment]::NewLine) -gt 2097152) { throw ('Decide request exceeds 2 MiB: ' + $section.id) }
 $envelopes.Add([pscustomobject][ordered]@{
  index = $index; section_id = $section.id; page_path = $section.page_path
  anchor = $section.anchor; request = $request
 })
 $index++
}
$envelopes
