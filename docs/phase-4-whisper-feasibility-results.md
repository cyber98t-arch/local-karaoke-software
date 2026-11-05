# Whisper base CPU feasibility results

Status: bounded development-only feasibility completed on 2026-11-05. No
runtime, transcript cache format, model publication, or application download
path is adopted by this report.

## Pinned inputs

- `whisper.cpp`: tag `v1.9.5`, commit
  `d1be6fde11ac6e0407606b4e42fe72d34add8037`
- Model: multilingual `ggml-base.bin`, converted-model revision
  `5359861c739e955e79d9a303bcbc70fb988958b1`
- Model bytes: `147,951,465`
- Model SHA-256:
  `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`
- Runtime configuration: CPU only, four threads, Japanese language fixed,
  no CUDA/HIP/Vulkan/BLAS, `GGML_NATIVE=OFF`
- Build tools: GCC/G++ 12.2.0, CMake 4.2.0, Ninja 1.13.0

All source, build, model, audio, and output files remained under
`/tmp/hibiki-whisper-feasibility`. Peak total research storage was about
293 MB. Nothing was copied into Git or application distribution.

## Evaluation audio

The probe used Wikimedia Commons
[`Ja-pronunciation-practice5.ogg`](https://commons.wikimedia.org/wiki/File:Ja-pronunciation-practice5.ogg),
recorded by a native Japanese speaker and dedicated to the public domain by its
author. The 30.93-second, 92,184-byte file had SHA-256
`317ced938950ed677e2ed6063417a784c670e8e620ac642d9b9f55c4ca651c70`.

The reference word sequence published by the associated Wikibooks exercise is:

`天気 練習 残業 安心 すんなり 電波 煎餅 玄米 千 本 専用 本屋 三円 単位`

## Observed CPU result

- Wall time: `1.483 s` for `30.928 s` of audio
- Approximate real-time factor: `0.048`
- Peak process RSS sampled from `/proc`: `296,304 KiB` (about 289 MiB)
- Output segments: `14`
- Transcript:
  `電気 練習 残業 安心 すんなり 電波 専米 現買 1000 本 専用 本や 3円 タイム!`

Segment boundaries followed the exercise's approximately two-second word
spacing closely. Several forms were useful or semantically equivalent (`1000`
for `千`, `3円` for `三円`), but material recognition errors remained:
`天気` became `電気`, `煎餅`/`玄米` became `専米`/`現買`, and `単位` became
`タイム`. Therefore base is fast and lightweight enough for the target CPU,
but its raw transcript must be treated as a draft requiring alignment,
confidence handling, and user correction rather than authoritative lyrics.

## Reproducibility hashes

- `whisper-cli`:
  `b47fcb064a2b9cd0c52b478871a8965fda5576697909c9a14df6c25f36bcb38b`
- `libwhisper.so.1.9.5`:
  `22c21ad831eaf2514a562ce19b50268e880aff8c1c1a85511b680fcfe8892210`
- `libggml.so.0.26.0`:
  `15187cd7a7364f1c24dd016038fae34abd94c9399cf84dbc66380a3d59039da8`
- `libggml-cpu.so.0.26.0`:
  `aa008b9f2c379ae57460395aba82b4b1e07b99b7d69cfff8a925a6fd66545620`

These Linux build hashes describe this feasibility environment only and are
not portable Windows distribution hashes.

## License inventory

- OpenAI Whisper code and model family: MIT
- `whisper.cpp` and GGML code at the pinned revision: MIT
- Wikimedia evaluation audio: public-domain dedication by the author
- CMake: BSD-3-Clause; Ninja: Apache-2.0; build tools are not redistributed

## Recommended next boundary

If approved, use a versioned local process protocol around a pinned CPU-only
`whisper.cpp` executable for the prototype. This keeps C++ runtime failure and
memory outside the Tauri process and avoids selecting a Rust FFI wrapper before
Windows packaging evidence exists. Transcript schema, alignment, confidence
policy, model acquisition UX, and production packaging remain separate gates.
