# Current handoff

- Updated: 2026-10-29
- Current phase: Phase 1 complete; Phase 2 approved by D-003 and in progress.
- Current implementation commit: `f7f580d` (`feat: connect library import UI`).
- Session commits: `ead46c6` adds the official Rust-side native folder picker; `9e316f1` records that stop point; `f7f580d` connects React selection, indexing, cancellation, and path-free metadata rendering. Earlier command-boundary work remains in `633ac8d`.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and an original Tauri application icon are present. UI direction uses Nightingale for song selection and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection and indexing commands. React now invokes selection before indexing and renders only path-free returned metadata while preserving the current library on cancellation or failure.
- Verified: At the final stop point, `npm run lint`, focused `npm test -- --run src/App.test.tsx` (4 tests), `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml` (20 tests), `cargo check --manifest-path src-tauri/Cargo.toml`, and `git diff --check` passed. Native-import ordering and cancellation, Rust-only selected-path retention, path-free results, source-byte preservation, catalog migration, and all-or-nothing indexing passed. Each Phase 2 slice has a recorded TDD RED run before its GREEN implementation.
- Unverified: Binary-size effect and Windows compilation remain unmeasured. Full `cargo fmt --check` reports existing file-ending differences in three Rust files. Windows metadata parsing, icon rendering, Tauri launch, microphone, GPU providers, installer, VMware guest, and all future AI pipelines remain unverified.
- Next safe action: Add a path-free Rust command for reading the existing catalog and initialize React from it, so reopening the application does not fall back to mock rows after a catalog has been created.
- Blockers and decisions: No user decision is waiting and no Phase 2 approval blocker remains. Windows path round-tripping and filesystem behavior must be verified later under `docs/windows-test-plan.md`. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

