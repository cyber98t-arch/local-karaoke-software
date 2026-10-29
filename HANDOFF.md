# Current handoff

- Updated: 2026-10-30
- Current phase: Phase 3 complete; Phase 4 approved by D-006 and in progress.
- Current implementation commit: `2e4f84b` (`feat: add validated model artifact manifest`).
- Recent commits: `aba7674` closes Phase 3; `08fdd69` reviews permissive model candidates; `5f029dd` approves Phase 4; `2e4f84b` adds the validated model artifact boundary.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, song-selection UI, and application icons for PNG and Windows ICO resources are present. The Dev Container now declares a dedicated `node_modules` volume so host Windows installs cannot replace Linux-native optional packages after the next rebuild. UI direction uses Nightingale for song-selection workflow, restrained Material Design 2 principles for the visual system, and the DAM main display only for singing/results, explicitly excluding Denmoku as a selection reference. The selection screen has no decorative product-name lockup and uses a compact semantic song table instead of repeated cards. Phase 2 includes a typed read-only recursive scanner, read-only MP3/FLAC/WAV metadata extraction, streaming SHA-256, a version 2 application-local SQLite catalog with lossless version 1 migration, transactional library indexing, validated content-addressed manifest paths, and atomic minimal JSON manifest commits with stale-temporary recovery. A Rust-owned selected-library state backs purpose-specific selection, indexing, and catalog-read commands. React invokes selection before indexing, renders only path-free returned metadata, and restores an existing catalog on startup while preserving mock rows when no catalog exists.
- Verified: `cargo test --manifest-path src-tauri/Cargo.toml` passed 29 tests; `cargo check --manifest-path src-tauri/Cargo.toml`, targeted formatting for the new model module, and `git diff --check` passed. `npm test -- --run` passed 10 tests without React warnings; `npm run lint` and `npm run build` passed. The model-manifest tests cover complete metadata, stable serialization, and rejection of unsafe IDs, non-HTTPS evidence, invalid SHA-256, zero size, and missing license identifiers. Windows Phase 3 evidence and Phase 2 evidence remain complete.
- Unverified: Binary-size effect, formal end-to-end latency, GPU providers, installer, VMware guest, and all future AI pipelines remain unverified. Full `cargo fmt --check` still has the documented pre-existing file-ending differences.
- Next safe action: Define the cache-owned analysis-job identity/state boundary with tests, including source hash, pipeline version, ordered model identities, parameters, cancellation, and atomic completion. Then pin exact upstream revisions and artifact metadata before downloading anything.
- Blockers and decisions: D-006 approves only evaluation of Spleeter 2-stem, CREPE, and multilingual Whisper `base`; HTDemucs weights remain excluded. No model or analysis dependency has been downloaded. Downloads over 500 MB, restrictive terms, a GPU-strategy change, or a different model family still require approval. The working tree contains the user's untracked `sample music/` and `screenshot/` directories; do not stage them. Full `cargo fmt --check` still reports pre-existing file-ending differences in `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs`; targeted checks for the new Rust modules pass.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

