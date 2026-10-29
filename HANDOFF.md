# Current handoff

- Updated: 2026-10-29
- Current phase: Phase 1 complete; Phase 2 approved by D-003 and in progress.
- Current implementation commit: `633ac8d` (`feat: add Rust-owned library command boundary`).
- Session commits: `633ac8d` adds the tested Rust-owned selected-library state and path-free Tauri indexing command. Earlier Phase 2 foundation and handoff commits remain `e36e99a` and `a15ddc8`.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, mock song-selection UI, and an original Tauri application icon are present. UI direction uses Nightingale for song selection and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state now backs a purpose-specific Tauri indexing command; its DTO does not expose source paths to React.
- Verified: At the final stop point, `npm run lint`, `npm test -- --run` (2 tests), `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml` (18 tests), `cargo check --manifest-path src-tauri/Cargo.toml`, targeted Rust formatting, and `git diff --check` passed. MP3, FLAC, and WAV fixture metadata, source-byte preservation, catalog migration, all-or-nothing indexing, required Rust-side library selection, directory validation, and path-free command results passed. Each Phase 2 slice has a recorded TDD RED run before its GREEN implementation.
- Unverified: Binary-size effect and Windows compilation remain unmeasured. Full `cargo fmt --check` reports existing file-ending differences in three Rust files. Windows metadata parsing, icon rendering, Tauri launch, microphone, GPU providers, installer, VMware guest, and all future AI pipelines remain unverified.
- Next safe action: Add native folder-picker integration that calls `LibraryState::select_library`, then connect React to the path-free indexing command without returning the selected absolute path.
- Blockers and decisions: No user decision is waiting and no Phase 2 approval blocker remains. Windows path round-tripping and filesystem behavior must be verified later under `docs/windows-test-plan.md`. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

