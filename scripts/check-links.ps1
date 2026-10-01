# SPDX-License-Identifier: MPL-2.0
# Validate local generated targets and fragments without network requests.
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$bookRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot 'book'))
$bookPrefix = $bookRoot.TrimEnd([char[]]'\/') + [System.IO.Path]::DirectorySeparatorChar
if (-not (Test-Path -LiteralPath (Join-Path $bookRoot 'index.html'))) { throw 'Build the book first.' }
$problems = [System.Collections.Generic.List[string]]::new()
$idsByPath = @{}
$checked = 0
$documents = @(Get-ChildItem -LiteralPath $bookRoot -Filter '*.html' -Recurse -File)
foreach ($document in $documents) {
    $html = [System.IO.File]::ReadAllText($document.FullName)
    foreach ($match in [regex]::Matches($html, '<(?:a|link|img|script)\b[^>]*?\b(?:href|src)\s*=\s*["'']([^"'']+)["'']', 'IgnoreCase')) {
        $reference = [System.Net.WebUtility]::HtmlDecode($match.Groups[1].Value)
        if ($reference -match '^(?:[a-z][a-z0-9+.-]*:|//)' -or $reference -eq '') { continue }
        $parts = $reference.Split('#', 2)
        $pathPart = [System.Uri]::UnescapeDataString($parts[0].Split('?', 2)[0])
        $targetPath = if ($pathPart -eq '') { $document.FullName } else {
            [System.IO.Path]::GetFullPath((Join-Path $document.DirectoryName $pathPart))
        }
        $label = [System.IO.Path]::GetRelativePath($bookRoot, $document.FullName)
        if (-not $targetPath.StartsWith($bookPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            [void]$problems.Add($label + ': target escapes build output: ' + $reference)
            continue
        }
        if (-not (Test-Path -LiteralPath $targetPath -PathType Leaf)) {
            [void]$problems.Add($label + ': missing target: ' + $reference)
            continue
        }
        $checked++
        if ($parts.Length -eq 2 -and $parts[1] -ne '' -and $targetPath.EndsWith('.html', [System.StringComparison]::OrdinalIgnoreCase)) {
            if (-not $idsByPath.ContainsKey($targetPath)) {
                $ids = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
                $targetHtml = [System.IO.File]::ReadAllText($targetPath)
                foreach ($id in [regex]::Matches($targetHtml, '\bid\s*=\s*["'']([^"'']+)["'']', 'IgnoreCase')) {
                    [void]$ids.Add([System.Net.WebUtility]::HtmlDecode($id.Groups[1].Value))
                }
                $idsByPath[$targetPath] = $ids
            }
            $fragment = [System.Uri]::UnescapeDataString($parts[1])
            if (-not $idsByPath[$targetPath].Contains($fragment)) {
                [void]$problems.Add($label + ': missing fragment: ' + $reference)
            }
        }
    }
}
if ($problems.Count -gt 0) { throw ($problems -join [Environment]::NewLine) }
Write-Output ('Validated ' + $checked + ' local references in ' + $documents.Count + ' HTML documents; external URLs were not requested.')
