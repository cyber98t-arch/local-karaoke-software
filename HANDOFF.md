# Current handoff

- Updated: 2026-10-29
- Current phase: Phase 2 complete; Phase 3 approved by D-005 and in progress.
- Current implementation commit: `d48331a` (`feat: add output playback probe`).
- Recent commits: `a808957` adds the injected audio-device service; `ecc68f8` adds CPAL 0.18.2; `607cd47` exposes CPAL enumeration; `7048a2c` adds device selectors; `d48331a` adds the bounded generated-tone output probe.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and application icons for PNG and Windows ICO resources are present. The Dev Container now declares a dedicated `node_modules` volume so host Windows installs cannot replace Linux-native optional packages after the next rebuild. UI direction uses Nightingale for song-selection workflow, restrained Material Design 2 principles for the visual system, and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. The selection screen has no decorative product-name lockup and uses a compact semantic song table instead of repeated cards. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection, indexing, and catalog-read commands. React invokes selection before indexing, renders only path-free returned metadata, and restores an existing catalog on startup while preserving mock rows when no catalog exists.
- Verified: `cargo test --manifest-path src-tauri/Cargo.toml` passed 25 tests; `cargo check --manifest-path src-tauri/Cargo.toml`, targeted Rust formatting, and `git diff --check` passed. `npm test -- --run` passed 8 tests without React warnings; `npm run lint` and `npm run build` passed after restoring the lockfile-pinned Linux Rollup optional package with `npm install --no-save`. Tests cover device defaults/separation/errors, device UI, selected output invocation, and bounded generated samples. Phase 2 evidence remains complete.
- Unverified: Windows CPAL/WASAPI device enumeration and audible playback, binary-size effect, microphone capture, callback cadence/latency, GPU providers, installer, VMware guest, and future AI pipelines remain unverified. Full `cargo fmt --check` still has the documented pre-existing file-ending differences. Rebuilding the Dev Container remains advisable to activate its isolated `node_modules` volume.
- Next safe action: On Windows, run the native Tauri app, confirm the input/output selectors show expected devices and defaults, choose an output, and confirm one 500 ms test tone. Record the negotiated behavior before implementing the microphone probe.
- Blockers and decisions: No user decision is waiting. D-005 approves CPAL with default Windows WASAPI for the bounded Phase 3 probe scope. The working tree contains the user's untracked `sample music/` and `screenshot/` directories; their contents have not been inspected, modified, or staged. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

