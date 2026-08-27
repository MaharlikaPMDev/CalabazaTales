param([string]$Version = '0.1.2')

$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root 'dist'
$stage = Join-Path $dist 'bundle'

if (Test-Path -LiteralPath $dist) { Remove-Item -LiteralPath $dist -Recurse -Force }
New-Item -ItemType Directory -Force -Path $dist, $stage | Out-Null

& (Join-Path $PSScriptRoot 'build_assets.ps1') -Root $root
if (-not $?) { throw 'HUD asset generation failed.' }

$cargo = Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
& $cargo +stable-x86_64-pc-windows-gnu build --release --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$wasm = Join-Path $root 'target/wasm32-wasip2/release/calabaza_tales.wasm'
Copy-Item -LiteralPath $wasm -Destination (Join-Path $dist 'CalabazaTales.wasm')

# Stable timestamps make the resource-pack archives and server hashes reproducible.
$stableTime = [DateTime]::SpecifyKind([DateTime]'2026-01-01T00:00:00', [DateTimeKind]::Utc)
$packRoots = @((Join-Path $root 'resource-pack-java'), (Join-Path $root 'resource-pack-bedrock'))
Get-ChildItem -LiteralPath $packRoots -Recurse -File | ForEach-Object { $_.LastWriteTimeUtc = $stableTime }
Get-ChildItem -LiteralPath $packRoots -Recurse -Directory | Sort-Object FullName -Descending |
    ForEach-Object { $_.LastWriteTimeUtc = $stableTime }
Get-Item -LiteralPath $packRoots | ForEach-Object { $_.LastWriteTimeUtc = $stableTime }

$javaZip = Join-Path $dist "CalabazaTales-Java-26.2-v$Version.zip"
& python (Join-Path $PSScriptRoot 'make_zip.py') (Join-Path $root 'resource-pack-java') $javaZip
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$bedrockPack = Join-Path $dist "CalabazaTales-Bedrock-v$Version.mcpack"
& python (Join-Path $PSScriptRoot 'make_zip.py') (Join-Path $root 'resource-pack-bedrock') $bedrockPack
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Copy-Item -LiteralPath (Join-Path $dist 'CalabazaTales.wasm'), $javaZip, $bedrockPack -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'README.md'), (Join-Path $root 'ROADMAP.md'), (Join-Path $root 'LICENSE') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'config') -Destination $stage -Recurse
Copy-Item -LiteralPath (Join-Path $root 'docs') -Destination $stage -Recurse
$bundle = Join-Path $dist "CalabazaTales-v$Version.zip"
& python (Join-Path $PSScriptRoot 'make_zip.py') $stage $bundle
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Remove-Item -LiteralPath $stage -Recurse -Force

$artifacts = Get-ChildItem -LiteralPath $dist -File | Sort-Object Name
$checksums = foreach ($artifact in $artifacts) {
    $sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $artifact.FullName).Hash.ToLowerInvariant()
    "$sha256  $($artifact.Name)"
}
$checksums | Set-Content -LiteralPath (Join-Path $dist 'SHA256SUMS.txt') -Encoding utf8

$javaSha1 = (Get-FileHash -Algorithm SHA1 -LiteralPath $javaZip).Hash.ToLowerInvariant()
Write-Output "Java SHA-1: $javaSha1"
Get-ChildItem -LiteralPath $dist -File | Select-Object Name, Length
