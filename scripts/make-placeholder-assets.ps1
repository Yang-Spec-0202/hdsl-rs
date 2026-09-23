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
    if (Test-Path -LiteralPath $target) {
        Write-Verbose "keep existing $Rel"
        return
    }
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
    if (Test-Path -LiteralPath $target) {
        Write-Verbose "keep existing $Rel"
        return
    }
    $parent = Split-Path -Parent $target
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    $svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#000000" d="M5 4H19V20H5V4ZM7 6V18H17V6H7Z"/></svg>'
    Set-Content -LiteralPath $target -Value $svg -Encoding UTF8 -NoNewline
}

# Branding and wallpapers are original artwork committed in the repository;
# only generate a placeholder if one is missing.

# Instance icons: simple original colored tiles, one hue per name.
function New-InstanceIcon {
    param([string]$Rel, [int]$Size, [string]$Seed)
    $target = Join-Path $OutDir $Rel
    if (Test-Path -LiteralPath $target) {
        Write-Verbose "keep existing $Rel"
        return
    }
    $parent = Split-Path -Parent $target
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    $palette = @(
        @(120, 170, 90),  @(170, 120, 80),  @(230, 210, 120), @(110, 130, 170),
        @(200, 120, 150), @(90, 160, 170),  @(180, 150, 90),  @(120, 140, 200),
        @(150, 110, 180), @(140, 140, 150), @(100, 170, 130), @(200, 160, 90),
        @(160, 130, 110), @(130, 150, 190)
    )
    $hash = 0
    foreach ($ch in $Seed.ToCharArray()) { $hash = ($hash * 31 + [int][char]$ch) % $palette.Count }
    $rgb = $palette[$hash]
    $fillColor = [System.Drawing.Color]::FromArgb(255, $rgb[0], $rgb[1], $rgb[2])
    $bitmap = New-Object System.Drawing.Bitmap($Size, $Size)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.Clear([System.Drawing.Color]::Transparent)
    $inset = [Math]::Max(1, [Math]::Round($Size / 16))
    $brush = New-Object System.Drawing.SolidBrush($fillColor)
    $graphics.FillRectangle($brush, $inset, $inset, $Size - $inset * 2, $Size - $inset * 2)
    $inner = [Math]::Round($Size / 3)
    $offset = [Math]::Round(($Size - $inner) / 2)
    $light = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(90, 255, 255, 255))
    $graphics.FillRectangle($light, $offset, $offset, $inner, $inner)
    $bitmap.Save($target, [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose(); $bitmap.Dispose()
}

$instanceIcons = 'grass', 'chest', 'chicken', 'command', 'april_fools', 'optifine',
    'craft_table', 'fabric', 'legacyfabric', 'forge', 'cleanroom', 'neoforge',
    'furnace', 'quilt'
foreach ($name in $instanceIcons) {
    New-InstanceIcon "instances\$name.png"    32 $name
    New-InstanceIcon "instances\$name@2x.png" 64 $name
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
