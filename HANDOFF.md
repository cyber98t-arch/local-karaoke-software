# Current handoff

- Updated: 2026-10-29
- Current phase: Phase 1 complete; Phase 2 approved by D-003 and in progress.
- Current implementation commit: `e36e99a` (`feat: add read-only library catalog foundation`).
- Session commits: `39f15b0` records the approved Phase 2/UI direction, `2bf667c` adds the Tauri icon, and `e36e99a` adds the Phase 2 Rust foundation. The remaining roadmap, progress-log, and handoff updates are intended for the final documentation commit.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, mock song-selection UI, and an original Tauri application icon are present. UI direction uses Nightingale for song selection and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery.
- Verified: At the final stop point, `npm run lint`, `npm test` (2 tests), `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml` (15 tests), `cargo check --manifest-path src-tauri/Cargo.toml`, and `git diff --check` passed. MP3, FLAC, and WAV fixture metadata, source-byte preservation, catalog migration, and all-or-nothing indexing passed. Targeted Rust formatting checks passed earlier. Each Phase 2 slice has a recorded TDD RED run before its GREEN implementation.
- Unverified: Binary-size effect and Windows compilation remain unmeasured. Full `cargo fmt --check` reports existing file-ending differences in three Rust files. Windows metadata parsing, icon rendering, Tauri launch, microphone, GPU providers, installer, VMware guest, and all future AI pipelines remain unverified.
- Next safe action: Define and test the Rust-owned Tauri command/state boundary for selecting and indexing a library without exposing arbitrary source-file access to React; keep folder-picker integration separate until the command contract is proven.
- Blockers and decisions: No user decision is waiting and no Phase 2 approval blocker remains. Windows path round-tripping and filesystem behavior must be verified later under `docs/windows-test-plan.md`. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

