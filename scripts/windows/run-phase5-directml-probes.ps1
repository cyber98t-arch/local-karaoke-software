[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $repoRoot

$spleeterModel = Resolve-Path ".\models\spleeter-2stems-repro.onnx" -ErrorAction Stop
$crepeModel = Resolve-Path ".\models\crepe-tiny-research.onnx" -ErrorAction Stop
$env:HIBIKI_SPLEETER_ONNX_MODEL = $spleeterModel.Path
$env:HIBIKI_CREPE_ONNX_MODEL = $crepeModel.Path
$env:HIBIKI_ONNX_PROVIDER = "directml"

Write-Host "Running the generated Spleeter DirectML probe..."
& cargo test `
    --manifest-path src-tauri/Cargo.toml `
    --features native-onnx-directml `
    separation_pipeline::tests::separates_fixture_with_verified_native_model_into_cache `
    -- --ignored --exact --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "Spleeter DirectML probe failed with exit code $LASTEXITCODE"
}

Write-Host "Running the generated CREPE DirectML probe..."
& cargo test `
    --manifest-path src-tauri/Cargo.toml `
    --features native-onnx-directml `
    melody_pipeline::tests::extracts_generated_vocals_with_verified_native_model `
    -- --ignored --exact --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "CREPE DirectML probe failed with exit code $LASTEXITCODE"
}

Write-Host "Both DirectML probes selected DirectML and completed successfully."
