# Phase 5 GPU-provider preflight

Status: **D-027 development prototype approved; Windows DirectML verification pending**

D-027 was approved on 2026-11-07 for a development-only DirectML prototype.
Provider-binary distribution and application adoption remain unapproved.

## Current boundary

- Phase 4 uses one pinned CPU ONNX Runtime environment for Spleeter and CREPE.
- GPU acceleration is optional and pre-playback only. CPU correctness and cache
  compatibility must remain intact.
- No GPU provider, binary package, driver installer, model change, or cache
  format is approved by this preflight.

## Current official provider direction

- DirectML supports DirectX 12 hardware across Intel, AMD, and NVIDIA, and fits
  the existing ONNX Runtime session boundary. It remains supported but is in
  sustained engineering.
- Microsoft recommends Windows ML for new Windows deployments, especially on
  Windows 11 24H2 and newer, where it can select providers automatically.
  Moving to its WinRT/C API distribution needs a separate integration and
  packaging proof because the application currently uses the Rust `ort` crate.
- The built-in OpenVINO provider is deprecated in favor of the standalone
  OpenVINO Plugin EP. It remains an Intel-specific comparison candidate, not a
  cross-vendor baseline.
- CUDA remains an NVIDIA-specific candidate and brings strict CUDA/cuDNN
  runtime compatibility and distribution requirements.

## Recommended bounded first step after approval

1. Keep CPU as the mandatory final provider and unchanged correctness path.
2. Add an optional DirectML session preference behind a development feature.
3. Probe provider/session construction explicitly and report the selected path;
   never claim GPU use merely because session creation succeeded.
4. Run identical Spleeter and CREPE fixtures on CPU and available GPU hardware.
5. Compare output differences, elapsed time, peak memory, initialization cost,
   provider DLL size, and fallback behavior.
6. Do not accelerate Whisper, bundle provider binaries, or change cache identity
   until results and packaging evidence receive another review.

## Hardware evidence

Run the checked-in read-only PowerShell probe from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\probe-phase5-hardware.ps1
```

It reads Windows, CPU, memory, display-adapter, and driver metadata through CIM
and prints JSON. It does not install drivers, download dependencies, initialize
a model, modify the registry, or write a report file.

Received 2026-11-07:

- Windows 11 Home build 26200, 64-bit
- AMD Ryzen 9 9950X3D, 32 logical processors
- Approximately 32 GB system memory
- AMD Radeon RX 9070 XT, driver `32.0.32015.7029`, status OK
- AMD Radeon integrated graphics, driver `32.0.21045.16010`, status OK

The `Win32_VideoController.AdapterRAM` field is legacy-width metadata and is not
used as authoritative VRAM capacity evidence. Provider measurements must name
the selected adapter explicitly so the discrete and integrated GPUs cannot be
confused.

## Development runtime artifact

The pinned `ort-sys` 2.0.0-rc.15 distribution table resolves the Windows x64
DirectML build to:

- URL: `https://cdn.pyke.io/0/pyke:ort-rs/ms@1.31.0/x86_64-pc-windows-msvc+directml.tar.lzma2`
- Compressed bytes observed by HTTP HEAD: `32,457,673` (30.95 MiB)
- SHA-256 recorded by `ort-sys`: `f99aaef23f12986abb05924c944d8433521263ad756b462df762a5a982c98405`

Cargo downloads it only when the `native-onnx-directml` development feature is
built. This is not approval to bundle or redistribute its extracted DLLs.

## Primary references

- ONNX Runtime execution providers: <https://onnxruntime.ai/docs/execution-providers/>
- Windows and Windows ML direction: <https://onnxruntime.ai/docs/get-started/with-windows.html>
- DirectML provider: <https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html>
- OpenVINO provider/plugin transition: <https://onnxruntime.ai/docs/execution-providers/OpenVINO-ExecutionProvider.html>
- CUDA provider requirements: <https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html>
- Rust `ort` provider behavior: <https://github.com/pykeio/ort/blob/main/docs/content/perf/execution-providers.mdx>
