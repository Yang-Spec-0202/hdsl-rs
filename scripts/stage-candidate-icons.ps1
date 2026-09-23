#requires -Version 5.1
<#
.SYNOPSIS
    Overlay the candidate icon set onto the committed placeholders for local UI review.

.DESCRIPTION
    The candidate sets in ../ui-assets-original (Reicon and Tabler) are third-party
    MIT artwork. Per the workspace AGENTS.md the shipped graphics must be original, so
    these files must NOT be committed. This script copies them over the committed
    placeholders in crates/ui/ui/assets/icons so the UI can be reviewed with real
    glyphs, and maps a couple of names.

    Restore the committed placeholders before committing:
        git checkout -- crates/ui/ui/assets/icons

    See docs/developer/src/ui-assets.md.
#>
[CmdletBinding()]
param(
    [ValidateSet('reicon', 'tabler')]
    [string]$Set = 'reicon',
    [string]$SourceRoot,
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not $SourceRoot) { $SourceRoot = Join-Path $repoRoot '..\ui-assets-original' }
if (-not $OutDir) { $OutDir = Join-Path $repoRoot 'crates\ui\ui\assets\icons' }

$srcDir = Join-Path $SourceRoot $(if ($Set -eq 'tabler') { 'icons-tabler' } else { 'icons' })
if (-not (Test-Path -LiteralPath $srcDir)) {
    throw "candidate icon set not found: $srcDir"
}
if (-not (Test-Path -LiteralPath $OutDir)) {
    throw "committed icon directory not found: $OutDir"
}

$count = 0
foreach ($icon in Get-ChildItem -LiteralPath $srcDir -Filter *.svg) {
    Copy-Item -LiteralPath $icon.FullName -Destination (Join-Path $OutDir $icon.Name) -Force
    $count++
}

Write-Host "Staged $count candidate icons ($Set) into $OutDir"
Write-Host "These are third-party MIT assets; run 'git checkout -- crates/ui/ui/assets/icons' before committing."
