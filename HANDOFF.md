# Current handoff

- Updated: 2026-10-29
- Current phase: Phase 1 complete; Phase 2 approved by D-003 and in progress.
- Current implementation commit: `c8c406e` (`build: add Japanese fonts to dev container`).
- Session commits: `a37dba0` adds non-creating catalog reads and React startup restoration; `4155915` records that stop point; `c8c406e` adds Noto CJK fonts and `dbus-launch` support to the Dev Container. React import remains in `f7f580d` and native selection in `ead46c6`.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and an original Tauri application icon are present. UI direction uses Nightingale for song selection and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection, indexing, and catalog-read commands. React invokes selection before indexing, renders only path-free returned metadata, and restores an existing catalog on startup while preserving mock rows when no catalog exists.
- Verified: At the final stop point, `npm run lint`, `npm test -- --run` (5 tests, no React warnings), `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml` (22 tests), `cargo check --manifest-path src-tauri/Cargo.toml`, targeted Rust formatting, and `git diff --check` passed. Non-creating missing-catalog reads, path-free restoration, native-import ordering and cancellation, source-byte preservation, catalog migration, and all-or-nothing indexing passed. Each Phase 2 slice has a recorded TDD RED run before its GREEN implementation.
- Unverified: Binary-size effect and Windows compilation remain unmeasured. Full `cargo fmt --check` reports existing file-ending differences in three Rust files. Windows metadata parsing, icon rendering, Tauri launch, microphone, GPU providers, installer, VMware guest, and all future AI pipelines remain unverified.
- Next safe action: Run the Phase 2 native Windows round-trip cases in `docs/windows-test-plan.md`: select a folder, index supported files, confirm source bytes and directory entries are unchanged, restart the app, and confirm the catalog restores without exposing paths.
- Blockers and decisions: No user decision is waiting and no Phase 2 approval blocker remains. Rebuild the Dev Container to verify the newly added Noto CJK font and `dbus-launch` repair Linux-only Japanese rendering and the dconf warning. Windows path round-tripping and filesystem behavior must still be verified under `docs/windows-test-plan.md`. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

