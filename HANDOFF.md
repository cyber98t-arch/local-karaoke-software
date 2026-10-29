# Current handoff

- Updated: 2026-10-29
- Current phase: Phase 2 complete; Phase 3 approved by D-005 and in progress.
- Current implementation commit: `7fbd1ab` (`fix: distinguish silent microphone probes`).
- Recent commits: `d48331a` adds the bounded output probe; `0b41ec8` records its Windows success; `6415294` adds the microphone probe; `05e8367` shows callback diagnostics; `7fbd1ab` fixes zero-peak reporting.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and application icons for PNG and Windows ICO resources are present. The Dev Container now declares a dedicated `node_modules` volume so host Windows installs cannot replace Linux-native optional packages after the next rebuild. UI direction uses Nightingale for song-selection workflow, restrained Material Design 2 principles for the visual system, and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. The selection screen has no decorative product-name lockup and uses a compact semantic song table instead of repeated cards. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection, indexing, and catalog-read commands. React invokes selection before indexing, renders only path-free returned metadata, and restores an existing catalog on startup while preserving mock rows when no catalog exists.
- Verified: `cargo test --manifest-path src-tauri/Cargo.toml` passed 26 tests; `cargo check --manifest-path src-tauri/Cargo.toml`, targeted Rust formatting, and `git diff --check` passed. `npm test -- --run` passed 10 tests without React warnings; `npm run lint` and `npm run build` passed. Windows enumeration and output passed. A Windows Logitech C270 input stream delivered 147 callbacks at a reported 10.2 ms average without a visible error; zero-peak handling is covered by a regression test. Phase 2 evidence remains complete.
- Unverified: A nonzero Windows microphone response to speech, absence of audible monitoring, binary-size effect, GPU providers, installer, VMware guest, and future AI pipelines remain unverified. Callback cadence has been observed but is not yet a formal latency measurement. Full `cargo fmt --check` still has the documented pre-existing file-ending differences.
- Next safe action: On Windows, rerun the microphone probe while speaking close to the selected microphone and confirm a nonzero peak plus no audible monitoring. If it remains zero, inspect Windows input level/privacy and the selected endpoint before changing capture code.
- Blockers and decisions: No user decision is waiting. D-005 approves CPAL with default Windows WASAPI for the bounded Phase 3 probe scope. The working tree contains the user's untracked `sample music/` and `screenshot/` directories; their contents have not been inspected, modified, or staged. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

