# Decision log

## D-001 — Desktop application foundation

- Status: Approved
- Decision: Tauri 2 + React/TypeScript + Rust, with Python available for early local model work.
- Source: User approval on 2026-10-27.
- Reason: lightweight UI, native performance boundary, and a path to local inference on ordinary Windows laptops.

## D-002 — Repository location

- Status: Approved
- Decision: Create the development repository on the Windows desktop.
- Source: User instruction on 2026-10-27.

## Pending decisions

### D-003 — Phase 2 catalog and metadata storage

- Status: Approved
- Decision: Use SQLite for searchable library/catalog records and job/cache state; keep versioned JSON manifests beside derived cache artifacts. Open source MP3, FLAC, and WAV files read-only, and never write tags or sidecar files into the music directory.
- Source: User approval on 2026-10-27.
- Alternative: Use only versioned JSON files under the application cache. This avoids a database dependency but makes indexed search, migrations, concurrency, and recovery more complex as the library grows.
- Implementation note: Select the smallest suitable Rust metadata-reading crate and SQLite binding after license and dependency review.
- Reason approval is required: Phase 2 changes the database and cache metadata strategy, which is a human approval gate.

### D-004 — UI reference direction

- Status: Approved
- Decision: Reduce generic AI-dashboard styling. Use Nightingale as the functional reference for local-library song selection. Use the DAM main karaoke display only for the singing and post-performance score-reveal experience; do not use the DAM Denmoku/handset UI as the song-selection reference. Keep the result visually original and do not copy proprietary assets or exact trade dress.
- Source: User direction on 2026-10-27.
- Scope: Preserve the approved React/Tauri framework and the existing privacy, read-only source, analysis, playback, and scoring boundaries.
- Detail: See `docs/ui-direction.md`.

### D-005 — Phase 3 Windows audio engine

- Status: Approved
- Decision: Use CPAL directly with its default Windows WASAPI backend as the Phase 3 audio I/O foundation for device enumeration, playback, and microphone capture.
- Source: User approval on 2026-10-29.
- Recommended option: Use CPAL directly for both input and output. It exposes devices and stream configurations, uses WASAPI by default on Windows, and keeps the real-time path under application control without introducing separate playback and capture abstractions.
- Alternative: Use Rodio for playback and CPAL for microphone capture. Playback is simpler, but Rodio currently documents an ongoing core-engine rewrite and would add another abstraction over CPAL.
- Alternative: Bind WASAPI directly through `windows-rs` or a WASAPI-specific crate. This offers maximum Windows control but increases unsafe/platform-specific code and works against the current portable CPU-fallback architecture.
- Initial scope if approved: Shared-mode device enumeration and a playback/microphone probe only. Do not enable ASIO, loopback capture, monitoring, scoring, or production playback in this decision.
- Sources reviewed: RustAudio CPAL and Rodio official repositories and current crate documentation on 2026-10-29.
- Reason approval is required: Phase 3 changes the audio engine/library and is explicitly marked `Decision required` in the roadmap.

### D-006 — Phase 4 local analysis model families and licenses

- Status: Waiting
- Research direction approved: On 2026-10-30 the user approved a conservative review limited to candidates with clear model-artifact licenses. HTDemucs weights remain excluded from adoption and download. This approval does not yet select the Phase 4 families.
- Question: Which local model families and license/provenance constraints should Phase 4 adopt for vocal separation, vocal melody extraction, and Japanese lyric transcription/alignment?
- Current recommendation after preflight: Evaluate Spleeter 2-stem for vocal separation, CREPE for monophonic vocal melody, and multilingual Whisper `base` through `whisper.cpp` for Japanese transcription. Keep every model download explicit and separate from song caches; see `docs/phase-4-model-review.md`.
- License review: Whisper and `whisper.cpp` publish MIT licenses. The archived Demucs repository publishes an MIT code license, but that alone does not clearly establish a separately redistributable license for every pretrained weight and training-data derivative. No Demucs weight may be bundled or downloaded until its exact artifact provenance and license are documented and approved.
- Alternatives: Evaluate a separation model with explicitly permissive weight terms; use a non-neural pitch tracker if CREPE latency or packaging is unsuitable; or require user-imported lyrics while postponing transcription.
- Initial scope if approved: Offline prototype jobs only, completed before playback and cached by content hash plus pipeline version. No external AI API, source-file write, live inference, automatic model download, or model redistribution.
- Reason approval is required: Phase 4 changes model families and model licenses and is marked `Decision required` in the roadmap. A model download over 500 MB or restrictive license requires a further explicit approval.
