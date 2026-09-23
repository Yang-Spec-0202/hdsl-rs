#requires -Version 5.1
<#
.SYNOPSIS
    Generate original placeholder graphics at the canonical asset paths.

.DESCRIPTION
    The committed repository must build from a clean checkout, so every asset
    referenced by the Slint UI exists as a simple original placeholder. During
    development, scripts/stage-reference-assets.ps1 overwrites these files with
    HMCL reference art for visual comparison; run this script again (or
    `git checkout -- crates/ui/ui/assets`) before committing.

    See docs/developer/src/ui-assets.md.
#>
[CmdletBinding()]
param(
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not $OutDir) { $OutDir = Join-Path $repoRoot 'crates\ui\ui\assets' }

$fill = [System.Drawing.Color]::FromArgb(255, 239, 237, 245)
$border = [System.Drawing.Color]::FromArgb(255, 118, 118, 131)

function New-Bitmap {
    param([string]$Rel, [int]$Width, [int]$Height, [string]$Format)
    $target = Join-Path $OutDir $Rel
    $parent = Split-Path -Parent $target
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    $bitmap = New-Object System.Drawing.Bitmap($Width, $Height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.Clear($fill)
    $pen = New-Object System.Drawing.Pen($border, [Math]::Max(1, [Math]::Round($Width / 16)))
    $graphics.DrawRectangle($pen, 1, 1, $Width - 3, $Height - 3)
    $imageFormat = if ($Format -eq 'jpg') {
        [System.Drawing.Imaging.ImageFormat]::Jpeg
    } else {
        [System.Drawing.Imaging.ImageFormat]::Png
    }
    $bitmap.Save($target, $imageFormat)
    $graphics.Dispose()
    $bitmap.Dispose()
}

function New-IconPlaceholder {
    param([string]$Rel)
    $target = Join-Path $OutDir $Rel
    $parent = Split-Path -Parent $target
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    $svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#000000" d="M5 4H19V20H5V4ZM7 6V18H17V6H7Z"/></svg>'
    Set-Content -LiteralPath $target -Value $svg -Encoding UTF8 -NoNewline
}

# Branding
New-Bitmap 'branding\icon.png'          32  32  png
New-Bitmap 'branding\icon@2x.png'       64  64  png
New-Bitmap 'branding\icon@4x.png'       128 128 png
New-Bitmap 'branding\icon@8x.png'       256 256 png
New-Bitmap 'branding\icon-mac.png'      512 512 png
New-Bitmap 'branding\icon-title.png'    24  24  png
New-Bitmap 'branding\icon-title@2x.png' 48  48  png

# Wallpapers
New-Bitmap 'wallpapers\2015-06-22.jpg' 800  480  jpg
New-Bitmap 'wallpapers\2016-02-25.jpg' 800  480  jpg
New-Bitmap 'wallpapers\2021-08-26.jpg' 1600 900  jpg

# Instance icons
$instanceIcons = 'grass', 'chest', 'chicken', 'command', 'april_fools', 'optifine',
    'craft_table', 'fabric', 'legacyfabric', 'forge', 'cleanroom', 'neoforge',
    'furnace', 'quilt'
foreach ($name in $instanceIcons) {
    New-Bitmap "instances\$name.png"    32 32 png
    New-Bitmap "instances\$name@2x.png" 64 64 png
}

# Placeholders and social
New-Bitmap 'placeholders\unknown_pack.png'   128 128 png
New-Bitmap 'placeholders\unknown_server.png' 128 128 png
New-Bitmap 'social\github.png'       32 32 png
New-Bitmap 'social\github-white.png' 32 32 png
New-Bitmap 'social\discord.png'      32 32 png

# Vector icons: derive names from the committed icons.slint, falling back to
# the staged reference folder on a fresh checkout before the first generation.
$iconNames = @()
$iconsSlint = Join-Path $repoRoot 'crates\ui\ui\icons.slint'
if (Test-Path -LiteralPath $iconsSlint) {
    foreach ($line in Get-Content -LiteralPath $iconsSlint -Encoding UTF8) {
        $match = [regex]::Match($line, 'if name == "([^"]+)"')
        if ($match.Success) { $iconNames += $match.Groups[1].Value }
    }
} else {
    $referenceIcons = Join-Path $repoRoot 'crates\ui\ui\assets\_reference\icons'
    if (Test-Path -LiteralPath $referenceIcons) {
        $iconNames = Get-ChildItem -LiteralPath $referenceIcons -Filter *.svg |
            ForEach-Object { $_.BaseName }
    }
}
if ($iconNames.Count -eq 0) {
    throw 'no icon names found; run scripts/stage-reference-assets.ps1 first'
}
foreach ($name in $iconNames) {
    New-IconPlaceholder "icons\$name.svg"
}

Write-Host "Generated placeholder assets in $OutDir ($($iconNames.Count) icons)"
