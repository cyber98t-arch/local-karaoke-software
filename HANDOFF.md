# Current handoff

- Updated: 2026-10-30
- Current phase: Phase 3 complete; Phase 4 in progress. D-007 approves CPU ONNX Runtime as the Spleeter separation foundation.
- Current implementation commit: `6703660` (`feat: decode analysis audio read-only`).
- Recent commits: `3040195` defines the native inference boundary; `7a8d755` covers the injected native pipeline; `def1e51` records decoder/model-delivery approval; `6703660` adds the read-only decoder.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and application icons for PNG and Windows ICO resources are present. The Dev Container now declares a dedicated `node_modules` volume so host Windows installs cannot replace Linux-native optional packages after the next rebuild. UI direction uses Nightingale for song-selection workflow, restrained Material Design 2 principles for the visual system, and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. The selection screen has no decorative product-name lockup and uses a compact semantic song table instead of repeated cards. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection, indexing, and catalog-read commands. React invokes selection before indexing, renders only path-free returned metadata, and restores an existing catalog on startup while preserving mock rows when no catalog exists.
- Verified: The pinned Spleeter neural subgraph executes under CPU ONNX Runtime and its representative output differed from TensorFlow by at most one 16-bit PCM LSB. The Rust signal pipeline passes five focused tests. Minimal-feature Symphonia 0.6.1 now decodes repository MP3/FLAC/WAV fixtures through a read-only handle to native-rate stereo PCM; two focused decoder tests and all 49 Rust library tests pass, with source bytes and directory entries unchanged. `cargo check` and `git diff --check` pass.
- Unverified: Non-44.1-kHz behavior and resampling; native Windows decoder and ONNX linking; repeated Windows CPU timing/memory; Windows runtime hashes and final third-party notices; byte-deterministic ONNX conversion; CREPE/Whisper inference; GPU providers; installer; and VMware guest. Full `cargo fmt --check` still has the documented pre-existing file-ending differences.
- Next safe action: Add a cache-owned, atomic WAV stem writer behind an injected output directory and test it with generated PCM. Do not connect real ONNX execution or choose a resampler yet.
- Blockers and decisions: D-009 and D-010 are approved. Publishing the ONNX release asset remains gated, and development continues with local model injection. `ort` 2.0.0-rc.15 type-checks behind `native-onnx`, but its Linux prebuilt static library cannot link against the Dev Container ABI; validate the approved target natively on Windows. Sample-rate conversion for non-44.1-kHz sources is not yet selected. Windows runtime bytes/hashes, GPU strategy, publishing, and release remain unresolved or separately gated. The working tree contains the user's untracked `sample music/` and `screenshot/` directories; never stage them.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

