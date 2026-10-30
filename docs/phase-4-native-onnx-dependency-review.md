# Phase 4 native ONNX dependency review

Date: 2026-10-30

Status: D-008 approved the native Rust boundary. This document is a pre-download
review; no crate or runtime binary was downloaded while producing it.

## Recommended minimal stack

### `ort` 2.0.0-rc.15

- Purpose: safe Rust binding over ONNX Runtime for the approved CPU inference
  subgraph.
- Runtime/API target: ONNX Runtime 1.31.
- License: `MIT OR Apache-2.0` for `ort` and `ort-sys`; ONNX Runtime is MIT.
- MSRV: Rust 1.92; the current development toolchain is Rust 1.99.
- Stability risk: the crate remains a 2.0 release candidate. Pin the exact
  release and treat upgrades as reviewed changes.
- Default-feature risk: defaults include `download-binaries`, `copy-dylibs`,
  `ndarray`, native TLS, and tracing. Production configuration should disable
  defaults and enable only the CPU inference/API and tensor features actually
  required. GPU execution-provider features remain out of scope.
- Binary provenance: the project publishes attested prebuilt binaries, but a
  release build must record the exact archive URL, size, SHA-256, attestation
  result when available, extracted DLL/static-library hashes, and notices.

### `realfft` 3.5.0

- Purpose: application-owned 4096-point real FFT/inverse FFT matching the
  verified Spleeter preprocessing and reconstruction path.
- License: MIT. Its RustFFT dependency is `MIT OR Apache-2.0`.
- MSRV: Rust 1.61.
- GPU/network behavior: none; signal processing remains CPU-capable.

## Windows and privacy constraints

- Official ONNX Runtime Windows builds require the Visual C++ 2019 runtime or
  newer. This must be checked on the Windows 11/VMware test target.
- Inference loads the model and PCM input locally. Runtime initialization and
  inference must be tested with outbound network access unavailable.
- Only the CPU execution provider is enabled in Phase 4. DirectML, WinML,
  OpenVINO, CUDA, and other providers remain Phase 5 decisions.
- Original audio remains read-only. Rust writes stems only to a new
  request-specific application-cache directory and records output hashes.

## Download/build envelope requiring approval

The exact `ort` prebuilt archive size is not exposed by the reviewed crate
metadata. A conservative first probe reserves up to 150 MB of network transfer
for crates and one CPU runtime archive, and up to 600 MB of additional temporary
build/cache storage. The purpose is only to resolve/pin the dependency closure,
record hashes/licenses, compile a minimal CPU session, and run a generated-input
test. No model will be added to Git and no GPU package will be fetched.

Stop and request explicit capacity approval before executing that probe.

## Probe result

The user approved the envelope and the dependency probe completed on
2026-10-30.

- Resolved `ort`/`ort-sys` exactly at 2.0.0-rc.15 and `realfft` exactly at
  3.5.0. Cargo.lock records registry checksums for the full closure.
- CPU-only Linux static runtime:
  - size: 111,701,892 bytes
  - SHA-256: `184360cecb39400bc59f98269ed179f4925bf75dd11be084604f4070edbf1493`
- Cargo registry cache grew by approximately 35.4 MB.
- `cargo check` passed with the dedicated target and pinned lockfile.

The dedicated target unexpectedly rebuilt the complete Tauri dependency graph
and reached approximately 1.38 GB. Together with the registry and ORT caches,
temporary growth was about 1.53 GB, exceeding the approved 600 MB estimate by
about 930 MB. Work stopped, the overage was reported, and only the
research-owned `/tmp/hibiki-native-onnx-target` was removed. Persistent added
cache was reduced to approximately 147 MB. No source, model, lockfile, or user
data was deleted.

The Linux static-library hash is development evidence only. Windows x86-64
runtime bytes, size, SHA-256, attestation, and bundling behavior remain
unverified.

## Primary references reviewed

- `ort` repository and its MIT/Apache-2.0 license files:
  <https://github.com/pykeio/ort>
- `ort` 2.0.0-rc.15 feature metadata:
  <https://docs.rs/crate/ort/2.0.0-rc.15/features>
- `ort-sys` 2.0.0-rc.15 package metadata:
  <https://github.com/pykeio/ort/blob/v2.0.0-rc.15/ort-sys/Cargo.toml>
- ONNX Runtime installation and Windows requirements:
  <https://onnxruntime.ai/docs/install/>
- `realfft` 3.5.0 API/license documentation:
  <https://docs.rs/realfft/3.5.0/realfft/>
