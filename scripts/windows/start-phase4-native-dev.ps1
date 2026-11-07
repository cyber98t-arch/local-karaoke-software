[CmdletBinding()]
param(
    [switch]$VerifyOnly,
    [switch]$SkipDownload
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $repoRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found. Open a new PowerShell after installing Rust with the MSVC toolchain."
}
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "npm was not found. Install Node.js and open a new PowerShell."
}

$modelDirectory = New-Item -ItemType Directory -Force (Join-Path $repoRoot "models")
$artifacts = @(
    [pscustomobject]@{
        Label = "Spleeter ONNX model"
        Path = Join-Path $modelDirectory.FullName "spleeter-2stems-repro.onnx"
        Url = "https://github.com/cyber98t-arch/local-karaoke-software/releases/download/model-spleeter-2stems-onnx-dev-v1/spleeter-2stems-repro.onnx"
        Bytes = 78652254
        Sha256 = "8D58651B5979075C954BCBE96C6776B2DC41E7852FB6F2D054B83A696A8DB26F"
        Environment = "HIBIKI_SPLEETER_ONNX_MODEL"
    },
    [pscustomobject]@{
        Label = "CREPE ONNX model"
        Path = Join-Path $modelDirectory.FullName "crepe-tiny-research.onnx"
        Url = "https://github.com/cyber98t-arch/local-karaoke-software/releases/download/model-crepe-tiny-onnx-dev-v1/crepe-tiny-research.onnx"
        Bytes = 1956735
        Sha256 = "03FB909A5618FA9F19942E20DE749BC8E658A1AC0DF4A778B354C15B6F7A59D4"
        Environment = "HIBIKI_CREPE_ONNX_MODEL"
    },
    [pscustomobject]@{
        Label = "Whisper multilingual base model"
        Path = Join-Path $modelDirectory.FullName "ggml-base.bin"
        Url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.bin"
        Bytes = 147951465
        Sha256 = "60ED5BC3DD14EEA856493D334349B405782DDCAF0028D4B5DF4088345FBA2EFE"
        Environment = "HIBIKI_WHISPER_MODEL"
    },
    [pscustomobject]@{
        Label = "whisper.cpp Windows CLI"
        Path = Join-Path $modelDirectory.FullName "whisper-cli.exe"
        Url = "https://github.com/cyber98t-arch/local-karaoke-software/releases/download/whisper-cpp-v1.9.5-windows-x64-cpu-dev-v1/whisper-cli.exe"
        Bytes = 2115584
        Sha256 = "E8C1A1390C22EC4BB48C5A32CCD65942303C8633B09D6F897E1A1562D19BA7F2"
        Environment = "HIBIKI_WHISPER_CLI"
    }
)

function Confirm-Artifact([object]$Artifact, [string]$Path) {
    $file = Get-Item $Path -ErrorAction Stop
    if ($file.Length -ne $Artifact.Bytes) {
        throw "$($Artifact.Label) size mismatch: expected $($Artifact.Bytes), found $($file.Length)."
    }
    $actualHash = (Get-FileHash $file.FullName -Algorithm SHA256).Hash
    if ($actualHash -ne $Artifact.Sha256) {
        throw "$($Artifact.Label) SHA-256 mismatch: $actualHash"
    }
    return $file
}

foreach ($artifact in $artifacts) {
    if (-not (Test-Path $artifact.Path -PathType Leaf)) {
        if ($SkipDownload) {
            throw "$($artifact.Label) is missing at $($artifact.Path). Run again without -SkipDownload."
        }
        $megabytes = [Math]::Round($artifact.Bytes / 1MB, 1)
        Write-Host "Downloading $($artifact.Label) ($megabytes MiB development artifact)..."
        $temporaryPath = "$($artifact.Path).download"
        if (Test-Path $temporaryPath) {
            Remove-Item $temporaryPath -Force
        }
        try {
            Invoke-WebRequest -Uri $artifact.Url -OutFile $temporaryPath
            Confirm-Artifact $artifact $temporaryPath | Out-Null
            Move-Item $temporaryPath $artifact.Path
        }
        finally {
            if (Test-Path $temporaryPath) {
                Remove-Item $temporaryPath -Force
            }
        }
    }

    $verified = Confirm-Artifact $artifact $artifact.Path
    [Environment]::SetEnvironmentVariable($artifact.Environment, $verified.FullName, "Process")
    Write-Host "Verified $($artifact.Label): $($artifact.Sha256)"
}

Write-Host "All four development artifacts are verified. No source music was modified."
if ($VerifyOnly) {
    Write-Host "Verification-only run complete."
    exit 0
}

$existingListener = Get-NetTCPConnection -LocalPort 1421 -State Listen -ErrorAction SilentlyContinue |
    Select-Object -First 1
if ($null -ne $existingListener) {
    $owner = Get-Process -Id $existingListener.OwningProcess -ErrorAction SilentlyContinue
    $ownerName = if ($null -eq $owner) { "PID $($existingListener.OwningProcess)" } else { "$($owner.ProcessName) (PID $($owner.Id))" }
    throw "Port 1421 is already used by $ownerName. Stop the listed process, then run this script again."
}

Write-Host "Starting the Tauri development app with native CPU ONNX analysis enabled..."
& npm run tauri -- dev --features native-onnx
if ($LASTEXITCODE -ne 0) {
    throw "Tauri development app exited with code $LASTEXITCODE."
}
