# SPDX-License-Identifier: MPL-2.0
[CmdletBinding()]
param([switch]$Offline)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'update.ps1') @PSBoundParameters
