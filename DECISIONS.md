# Decision log

## D-001 — Desktop application foundation

- Status: Approved
- Decision: Tauri 2 + React/TypeScript + Rust, with Python available for early local model work.
- Source: User approval on 2026-10-27.
- Reason: lightweight UI, native performance boundary, and a path to local inference on ordinary Windows laptops.

## D-002 — Repository location

- Status: Approved
- Decision: Create the development repository on the Windows desktop.
- Source: User instruction on 2026-10-27.

## Pending decisions

### D-003 — Phase 2 catalog and metadata storage

- Status: Approved
- Decision: Use SQLite for searchable library/catalog records and job/cache state; keep versioned JSON manifests beside derived cache artifacts. Open source MP3, FLAC, and WAV files read-only, and never write tags or sidecar files into the music directory.
- Source: User approval on 2026-10-27.
- Alternative: Use only versioned JSON files under the application cache. This avoids a database dependency but makes indexed search, migrations, concurrency, and recovery more complex as the library grows.
- Implementation note: Select the smallest suitable Rust metadata-reading crate and SQLite binding after license and dependency review.
- Reason approval is required: Phase 2 changes the database and cache metadata strategy, which is a human approval gate.

### D-004 — UI reference direction

- Status: Approved
- Decision: Reduce generic AI-dashboard styling. Use Nightingale as the functional reference for local-library song selection. Use the DAM main karaoke display only for the singing and post-performance score-reveal experience; do not use the DAM Denmoku/handset UI as the song-selection reference. Keep the result visually original and do not copy proprietary assets or exact trade dress.
- Source: User direction on 2026-10-27.
- Scope: Preserve the approved React/Tauri framework and the existing privacy, read-only source, analysis, playback, and scoring boundaries.
- Detail: See `docs/ui-direction.md`.
