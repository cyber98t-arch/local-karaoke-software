# Current handoff

- Updated: 2026-10-30
- Current phase: Phase 3 complete. Phase 4 has not started and is waiting on D-006.
- Current implementation commit: `7fbd1ab` (`fix: distinguish silent microphone probes`); documentation commits after it record Windows evidence and phase closure.
- Recent commits: `6415294` adds the microphone probe; `05e8367` shows callback diagnostics; `7fbd1ab` fixes zero-peak reporting; `91a3592` records the first Windows callback evidence.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and application icons for PNG and Windows ICO resources are present. The Dev Container now declares a dedicated `node_modules` volume so host Windows installs cannot replace Linux-native optional packages after the next rebuild. UI direction uses Nightingale for song-selection workflow, restrained Material Design 2 principles for the visual system, and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. The selection screen has no decorative product-name lockup and uses a compact semantic song table instead of repeated cards. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection, indexing, and catalog-read commands. React invokes selection before indexing, renders only path-free returned metadata, and restores an existing catalog on startup while preserving mock rows when no catalog exists.
- Verified: `cargo test --manifest-path src-tauri/Cargo.toml` passed 26 tests; `cargo check --manifest-path src-tauri/Cargo.toml` and `git diff --check` passed. `npm test -- --run` passed 10 tests without React warnings; `npm run lint` and `npm run build` passed. Windows enumeration, default selection, selected-device output, audible test tone, microphone callback delivery, nonzero speech response, and absence of audible monitoring passed. A Logitech C270 input stream delivered 147 callbacks at a reported 10.2 ms average without a visible error; zero-peak handling is covered by a regression test. Phase 2 evidence remains complete.
- Unverified: Binary-size effect, formal end-to-end latency, GPU providers, installer, VMware guest, and all future AI pipelines remain unverified. Full `cargo fmt --check` still has the documented pre-existing file-ending differences.
- Next safe action: Resolve D-006 before starting Phase 4. After approval, document exact model artifact sources, hashes, licenses, expected sizes, and CPU feasibility before downloading or integrating any model.
- Blockers and decisions: D-006 is waiting for the user's model-family and license decision. In particular, the Demucs code license does not by itself resolve pretrained-weight redistribution/provenance. The working tree contains the user's untracked `sample music/` and `screenshot/` directories; do not stage them. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

