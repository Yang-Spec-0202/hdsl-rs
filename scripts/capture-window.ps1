#requires -Version 5.1
<#
.SYNOPSIS
    Capture a screen region to a PNG file for README and manual screenshots.

.DESCRIPTION
    Grabs the given rectangle from the primary screen. Use the bounds reported by
    the window inspector for the launcher window.

    pwsh -File scripts/capture-window.ps1 -X 100 -Y 100 -W 1080 -H 700 -Out docs/images/hdsl-home.png
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][int]$X,
    [Parameter(Mandatory = $true)][int]$Y,
    [Parameter(Mandatory = $true)][int]$W,
    [Parameter(Mandatory = $true)][int]$H,
    [Parameter(Mandatory = $true)][string]$Out
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$parent = Split-Path -Parent $Out
if ($parent -and -not (Test-Path -LiteralPath $parent)) {
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
}

$bitmap = New-Object System.Drawing.Bitmap($W, $H)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($X, $Y, 0, 0, (New-Object System.Drawing.Size($W, $H)))
$bitmap.Save((Resolve-Path -LiteralPath $parent).Path + '\' + (Split-Path -Leaf $Out), [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()
Write-Host "Saved $Out ($W x $H)"
