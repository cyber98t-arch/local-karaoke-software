# Current handoff

- Updated: 2026-10-29
- Current phase: Phase 2 complete; Phase 3 approved by D-005 and in progress.
- Current implementation commit: `6415294` (`feat: add microphone level probe`).
- Recent commits: `607cd47` exposes CPAL enumeration; `7048a2c` adds device selectors; `d48331a` adds the bounded output probe; `0b41ec8` records its Windows success; `6415294` adds the no-storage/no-monitoring microphone-level probe.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and application icons for PNG and Windows ICO resources are present. The Dev Container now declares a dedicated `node_modules` volume so host Windows installs cannot replace Linux-native optional packages after the next rebuild. UI direction uses Nightingale for song-selection workflow, restrained Material Design 2 principles for the visual system, and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. The selection screen has no decorative product-name lockup and uses a compact semantic song table instead of repeated cards. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection, indexing, and catalog-read commands. React invokes selection before indexing, renders only path-free returned metadata, and restores an existing catalog on startup while preserving mock rows when no catalog exists.
- Verified: `cargo test --manifest-path src-tauri/Cargo.toml` passed 26 tests; `cargo check --manifest-path src-tauri/Cargo.toml`, targeted Rust formatting, and `git diff --check` passed. `npm test -- --run` passed 9 tests without React warnings; `npm run lint` and `npm run build` passed. Tests cover device defaults/separation/errors, device UI, output invocation, bounded generated samples, microphone peak calculation, and selected input invocation. Windows enumeration and the output test tone passed user observation. Phase 2 evidence remains complete.
- Unverified: Windows microphone permission/capture, visible speech response, absence of audible monitoring, callback cadence/latency, binary-size effect, GPU providers, installer, VMware guest, and future AI pipelines remain unverified. Full `cargo fmt --check` still has the documented pre-existing file-ending differences. Rebuilding the Dev Container remains advisable to activate its isolated `node_modules` volume.
- Next safe action: On Windows, select the intended microphone and run the 1.5 second probe while speaking; confirm a nonzero peak, no audible monitoring, no file creation, and no error. Record the reported callback behavior before extending audio work.
- Blockers and decisions: No user decision is waiting. D-005 approves CPAL with default Windows WASAPI for the bounded Phase 3 probe scope. The working tree contains the user's untracked `sample music/` and `screenshot/` directories; their contents have not been inspected, modified, or staged. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

