[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $repoRoot

$modelDirectory = Join-Path $repoRoot "models"
$modelPath = Join-Path $modelDirectory "crepe-tiny-research.onnx"
$modelUrl = "https://github.com/cyber98t-arch/local-karaoke-software/releases/download/model-crepe-tiny-onnx-dev-v1/crepe-tiny-research.onnx"
$expectedBytes = 1956735
$expectedSha256 = "03FB909A5618FA9F19942E20DE749BC8E658A1AC0DF4A778B354C15B6F7A59D4"

New-Item -ItemType Directory -Force $modelDirectory | Out-Null
if (-not (Test-Path $modelPath -PathType Leaf)) {
    Write-Host "Downloading the approved 1.96 MB CREPE development model..."
    $temporaryModel = "$modelPath.download"
    Remove-Item $temporaryModel -Force -ErrorAction SilentlyContinue
    Invoke-WebRequest -Uri $modelUrl -OutFile $temporaryModel
    Move-Item $temporaryModel $modelPath
}

$model = Get-Item $modelPath
if ($model.Length -ne $expectedBytes) {
    throw "CREPE model size mismatch: expected $expectedBytes, found $($model.Length)"
}
$modelHash = (Get-FileHash $modelPath -Algorithm SHA256).Hash
if ($modelHash -ne $expectedSha256) {
    throw "CREPE model SHA-256 mismatch: $modelHash"
}
Write-Host "Verified CREPE model: $modelHash"

$env:HIBIKI_CREPE_ONNX_MODEL = $model.FullName
$generatedProbe = @(
    "test",
    "--manifest-path", "src-tauri/Cargo.toml",
    "--features", "native-onnx",
    "melody_pipeline::tests::extracts_generated_vocals_with_verified_native_model",
    "--", "--ignored", "--exact", "--nocapture"
)
& cargo @generatedProbe
if ($LASTEXITCODE -ne 0) {
    throw "Generated-vocal CREPE probe failed with exit code $LASTEXITCODE"
}

$separationRoot = Join-Path $repoRoot "cache\native-separation"
$vocalSource = Get-ChildItem $separationRoot -Recurse -Filter "vocals.flac" -File -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if ($null -eq $vocalSource) {
    Write-Warning "No cached vocals.flac was found. Representative-vocal probe skipped."
    Write-Host "All currently runnable Phase 4 Windows checks passed."
    exit 0
}

$melodyCache = New-Item -ItemType Directory -Force (Join-Path $repoRoot "cache\native-melody")
$env:HIBIKI_MELODY_SOURCE = $vocalSource.FullName
$env:HIBIKI_MELODY_CACHE = $melodyCache.FullName
Write-Host "Representative vocal source: $($vocalSource.FullName)"

$representativeProbe = @(
    "test",
    "--manifest-path", "src-tauri/Cargo.toml",
    "--features", "native-onnx",
    "melody_pipeline::tests::extracts_explicit_vocal_source_with_verified_native_model",
    "--", "--ignored", "--exact", "--nocapture"
)
& cargo @representativeProbe
if ($LASTEXITCODE -ne 0) {
    throw "Representative-vocal CREPE probe failed with exit code $LASTEXITCODE"
}

Write-Host "All currently pending Phase 4 Windows checks passed."
