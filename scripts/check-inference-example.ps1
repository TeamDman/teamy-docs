# SPDX-License-Identifier: MPL-2.0
[CmdletBinding()]
param([Parameter(Mandatory)][string]$ServiceRoot)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$docsRoot = Split-Path -Parent $PSScriptRoot
$servicePath = (Resolve-Path -LiteralPath $ServiceRoot).Path
$coreRoot = Join-Path $servicePath 'crates/teamy_llm_core'
$sourcePath = Join-Path $coreRoot 'examples/finite_choice_contract.rs'
$manifestPath = Join-Path $coreRoot 'Cargo.toml'
$chapterPath = Join-Path $docsRoot 'src/finite-choice-models.md'
if (-not (Test-Path -LiteralPath $sourcePath) -or -not (Test-Path -LiteralPath $manifestPath)) {
    throw 'Use the service checkout containing the finite-choice core example. The older published revision does not contain it.'
}
$manifest = Get-Content -Raw -LiteralPath $manifestPath
if ($manifest -notmatch 'name\s*=\s*"teamy_llm_core"') { throw 'Unexpected service core package.' }
$chapter = Get-Content -Raw -LiteralPath $chapterPath
$match = [regex]::Match($chapter, '(?s)<!-- checked-choice-example:start -->\s*```rust\r?\n(.*?)\r?\n```\s*<!-- checked-choice-example:end -->')
if (-not $match.Success) { throw 'The chapter has no complete checked-choice example.' }
$source = Get-Content -Raw -LiteralPath $sourcePath
$normalise = { param([string]$text) $text.Replace("`r`n", "`n").TrimEnd() }
if ((& $normalise $source) -cne (& $normalise $match.Groups[1].Value)) {
    throw 'The book example differs from the actual core example. Review and update both before validation.'
}
$targetPath = Join-Path $coreRoot 'target'
Push-Location -LiteralPath $servicePath
try {
    & cargo test --offline --locked --manifest-path $manifestPath -p teamy_llm_core -j 2 --target-dir $targetPath
    if ($LASTEXITCODE -ne 0) { throw 'Core contract tests failed. Stop here; do not claim the example is checked.' }
    & cargo run --offline --locked --manifest-path $manifestPath -p teamy_llm_core --example finite_choice_contract -j 2 --target-dir $targetPath
    if ($LASTEXITCODE -ne 0) { throw 'Core example assertions failed.' }
} finally { Pop-Location }
$receiptDirectory = Join-Path $docsRoot '.validation'
$null = New-Item -ItemType Directory -Force -Path $receiptDirectory
$receipt = [ordered]@{
    version = 1
    checkedAtUtc = [DateTimeOffset]::UtcNow.ToString('o')
    package = 'teamy_llm_core'
    example = 'finite_choice_contract'
    sourceSha256 = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    validation = 'core-tests-and-executable-assertions'
    modelInference = $false
    pagesCi = $false
}
$receipt | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $receiptDirectory 'choice-example.json') -Encoding utf8
Write-Output 'Checked the exact book example against the service core. No model inference was performed.'
