#requires -Version 5.1
<#
.SYNOPSIS
    Stage HMCL reference assets for local UI development only.

.DESCRIPTION
    Copies the bitmap assets used by the launcher shell from a read-only HMCL
    checkout, and extracts the vector icon enum (SVG.java) into standalone SVG
    files. The output directory is gitignored and must never be committed or
    pushed: see AGENTS.md and docs/developer/src/ui-assets.md.

    Run from anywhere:
        pwsh -File scripts/stage-reference-assets.ps1
#>
[CmdletBinding()]
param(
    [string]$ReferenceRoot,
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not $ReferenceRoot) { $ReferenceRoot = Join-Path $repoRoot '..\reference\HMCL' }
if (-not $OutDir) { $OutDir = Join-Path $repoRoot 'crates\ui\ui\assets' }

$assetRoot = Join-Path $ReferenceRoot 'HMCL\src\main\resources\assets'
$svgJava = Join-Path $ReferenceRoot 'HMCL\src\main\java\org\jackhuang\hmcl\ui\SVG.java'

if (-not (Test-Path -LiteralPath $assetRoot)) {
    throw "HMCL asset root not found: $assetRoot"
}

function Copy-Asset {
    param([string]$From, [string]$To)
    $source = Join-Path $assetRoot $From
    if (-not (Test-Path -LiteralPath $source)) {
        Write-Warning "missing reference asset: $From"
        return
    }
    $target = Join-Path $OutDir $To
    $parent = Split-Path -Parent $target
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    Copy-Item -LiteralPath $source -Destination $target -Force
}

# Branding
Copy-Asset 'img\icon.png'          'branding\icon.png'
Copy-Asset 'img\icon@2x.png'       'branding\icon@2x.png'
Copy-Asset 'img\icon@4x.png'       'branding\icon@4x.png'
Copy-Asset 'img\icon@8x.png'       'branding\icon@8x.png'
Copy-Asset 'img\icon-mac.png'      'branding\icon-mac.png'
Copy-Asset 'img\icon-title.png'    'branding\icon-title.png'
Copy-Asset 'img\icon-title@2x.png' 'branding\icon-title@2x.png'

# Wallpapers
foreach ($name in '2015-06-22', '2016-02-25', '2021-08-26') {
    Copy-Asset "img\wallpapers\$name.jpg" "wallpapers\$name.jpg"
}

# Instance icon set (default + alternatives)
$instanceIcons = 'grass', 'chest', 'chicken', 'command', 'april_fools', 'optifine',
    'craft_table', 'fabric', 'legacyfabric', 'forge', 'cleanroom', 'neoforge',
    'furnace', 'quilt'
foreach ($name in $instanceIcons) {
    Copy-Asset "img\$name.png"    "instances\$name.png"
    Copy-Asset "img\$name@2x.png" "instances\$name@2x.png"
}

# Placeholders and social
Copy-Asset 'img\unknown_pack.png'   'placeholders\unknown_pack.png'
Copy-Asset 'img\unknown_server.png' 'placeholders\unknown_server.png'
Copy-Asset 'img\github.png'         'social\github.png'
Copy-Asset 'img\github-white.png'   'social\github-white.png'
Copy-Asset 'img\discord.png'        'social\discord.png'

# Vector icons extracted from the Java enum.
if (-not (Test-Path -LiteralPath $svgJava)) {
    throw "SVG.java not found: $svgJava"
}
$iconDir = Join-Path $OutDir 'icons'
if (-not (Test-Path -LiteralPath $iconDir)) {
    New-Item -ItemType Directory -Path $iconDir -Force | Out-Null
}

$pattern = '^\s*([A-Z][A-Z0-9_]*)\(\s*"([^"]*)"\s*\)'
$count = 0
foreach ($line in Get-Content -LiteralPath $svgJava -Encoding UTF8) {
    $match = [regex]::Match($line, $pattern)
    if (-not $match.Success) { continue }
    $name = $match.Groups[1].Value
    $path = $match.Groups[2].Value
    if ([string]::IsNullOrWhiteSpace($path)) { continue }
    $file = ($name.ToLowerInvariant() -replace '_', '-') + '.svg'
    $svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#000000" d="' + $path + '"/></svg>'
    Set-Content -LiteralPath (Join-Path $iconDir $file) -Value $svg -Encoding UTF8 -NoNewline
    $count++
}

Write-Host "Staged reference assets into $OutDir"
Write-Host "  vector icons: $count"
Write-Host "These overwrite the committed placeholders. Restore them with"
Write-Host "scripts/make-placeholder-assets.ps1 before committing."
