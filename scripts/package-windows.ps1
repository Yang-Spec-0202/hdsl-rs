param(
    [string]$Mdbook = 'mdbook',
    [string]$MakeNsis = 'makensis',
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$dist = Join-Path $repo 'dist'

function Reset-Stage([string]$target) {
    $resolved = [IO.Path]::GetFullPath($target)
    $allowed = [IO.Path]::GetFullPath((Join-Path $dist 'stage')) + [IO.Path]::DirectorySeparatorChar
    if (-not $resolved.StartsWith($allowed, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to reset path outside dist/stage: $resolved"
    }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $resolved | Out-Null
}

Push-Location $repo
try {
    if (-not $SkipBuild) {
        & cargo build --release --locked -p hdsl
        if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
        & $Mdbook build docs/user
        if ($LASTEXITCODE -ne 0) { throw 'mdbook build failed' }
    }
    $metadata = & cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' }
    $version = ($metadata.packages | Where-Object name -eq 'hdsl' | Select-Object -First 1).version
    if (-not $version) { throw 'HDSL package version is missing' }
    $binary = Join-Path $repo 'target/release/hdsl.exe'
    $book = Join-Path $repo 'docs/user/book'
    if (-not (Test-Path -LiteralPath $binary) -or -not (Test-Path -LiteralPath (Join-Path $book 'index.html'))) {
        throw 'Release binary or offline help is missing'
    }
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    $portable = Join-Path $dist 'stage/windows-portable'
    $installer = Join-Path $dist 'stage/windows-installer'
    Reset-Stage $portable
    Reset-Stage $installer
    foreach ($stage in @($portable, $installer)) {
        Copy-Item -LiteralPath $binary -Destination (Join-Path $stage 'hdsl.exe')
        Copy-Item -LiteralPath (Join-Path $repo 'LICENSE') -Destination $stage
        Copy-Item -LiteralPath (Join-Path $repo 'NOTICE') -Destination $stage
        Copy-Item -LiteralPath (Join-Path $repo 'crates/ui/ui/assets/hdsl-mark.svg') -Destination $stage
        Copy-Item -LiteralPath $book -Destination (Join-Path $stage 'help') -Recurse
    }
    New-Item -ItemType File -Force -Path (Join-Path $portable 'portable.flag') | Out-Null
    $zip = Join-Path $dist "hdsl-$version-windows-x64-portable.zip"
    if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
    Compress-Archive -Path (Join-Path $portable '*') -DestinationPath $zip -CompressionLevel Optimal
    $setup = Join-Path $dist "hdsl-$version-windows-x64-setup.exe"
    & $MakeNsis "/DSOURCE_DIR=$installer" "/DOUTPUT_FILE=$setup" (Join-Path $repo 'packaging/windows/hdsl.nsi')
    if ($LASTEXITCODE -ne 0) { throw 'NSIS installer build failed' }
    $checksums = @($zip, $setup) | ForEach-Object {
        $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_).Hash.ToLowerInvariant()
        "$hash  $([IO.Path]::GetFileName($_))"
    }
    $checksums | Set-Content -LiteralPath (Join-Path $dist 'SHA256SUMS-windows-x64.txt') -Encoding ascii
    Write-Output "Created $zip and $setup"
} finally {
    Pop-Location
}
