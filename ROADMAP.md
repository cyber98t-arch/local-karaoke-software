# Roadmap

| Phase | Status | Outcome | Gate |
|---|---|---|---|
| 0. Repository foundation | Complete | Git, Dev Container, Codex continuity, requirements and verification baseline | Approved: Tauri 2 + React + Rust + Python |
| 1. Song-selection UI skeleton | Complete | Searchable/selectable mock library with analysis states | No new gate |
| 2. Read-only library and cache catalog | Complete | Scan MP3/FLAC/WAV without changing sources; content-addressed catalog | Approved: D-003 SQLite catalog + JSON artifact manifests |
| 3. Playback and microphone probe | In progress | Windows audio device selection, playback, loopback-free mic monitoring | Approved: D-005 CPAL + default Windows WASAPI |
| 4. Analysis pipeline prototype | Not started | Local separation, melody extraction, Japanese lyric alignment, progress reporting | Decision required: model families and licenses |
| 5. GPU provider matrix | Not started | DirectML/CUDA/other supported providers with measured CPU fallback | Decision required after hardware probes |
| 6. Singing and scoring UI | Not started | Low-latency live pitch/timing feedback; no live total score | Scoring weights require approval |
| 7. Result and lyric correction | Not started | Post-song total/breakdown and editable synced Japanese lyrics | UX acceptance review |
| 8. Packaging and VMware qualification | Not started | Windows installer and CPU-only Windows 11 Home VM evidence | Release approval |

Every phase begins with a read-only preflight, explicit acceptance criteria, and a small test-first implementation plan. Do not start a gated phase until its decision is recorded as approved in `DECISIONS.md`.

## Phase 2 acceptance criteria

- A user-selected directory can be scanned for supported MP3, FLAC, and WAV files through a Rust-owned read-only library boundary.
- Scanning and metadata extraction never create, modify, rename, tag, or delete anything in the selected source directory.
- Catalog records are stored in an application-local SQLite database; derived artifact manifests are written only below the application cache.
- Song and cache identity includes the source content SHA-256 and pipeline version.
- Interrupted or failed catalog/cache updates do not invalidate a previously valid record or artifact manifest.
- Tests cover supported-file discovery, ignored files, source immutability, stable content identity, and safe catalog updates.

## Phase 2 initial test-first steps

1. Completed 2026-10-27: reviewed candidate Rust metadata and SQLite crates and selected `rusqlite` (`bundled`) plus read-only use of `lofty`; see `docs/phase-2-dependency-review.md`. Binary-size and Windows measurements remain pending.
2. Completed 2026-10-27: added Rust tests using temporary fixtures for recursive MP3, FLAC, and WAV discovery, ignored files, and source immutability; observed both tests fail against the initial `todo!` implementation.
3. Completed 2026-10-27: implemented the smallest typed, deterministic, read-only library scanner needed to pass those tests, without metadata parsing or persistence.
4. Completed 2026-10-27: added catalog schema/migration tests and introduced `rusqlite` with bundled SQLite under an injected application-data path. The current version 2 schema creates `library_sources` atomically, migrates version 1 without data loss, and preserves records on reopen.
5. Completed 2026-10-27: added read-only content hashing and versioned JSON manifest behavior behind cache-owned paths. Tests cover validated identity paths, atomic commit, stale temporary-file recovery, and preservation of an existing valid manifest.

Additional Phase 2 progress: read-only `lofty` metadata extraction now covers generated MP3, FLAC, and WAV fixtures and returns only title, artist, and duration value fields.

Additional Phase 2 progress: a service layer now combines discovery, metadata extraction, content hashing, and transactional catalog replacement. Catalog schema version 2 adds metadata fields and migrates version 1 records without loss.

Additional Phase 2 progress: a Rust-owned Tauri state validates and retains the selected library root, and a purpose-specific indexing command returns content IDs and display metadata without exposing source paths to React. Native folder-picker integration remains pending.

Additional Phase 2 progress: an official Rust-side Tauri dialog command now selects a single local folder and retains it only in Rust state; cancellation and path-free responses are covered by tests. React invocation and native Windows verification remain pending.

Additional Phase 2 progress: the React import action now invokes native selection followed by indexing, preserves the current library on cancellation/failure, and renders only path-free display metadata returned by Rust. Native Windows end-to-end verification remains pending.

Additional Phase 2 progress: React now restores an existing application-local catalog through a path-free Rust command on startup; a missing catalog remains non-creating. Native Windows end-to-end verification was completed at Phase 2 closure below.

Phase 2 completed 2026-10-29: automated tests cover read-only discovery, metadata, hashing, transactional catalog updates, path-free command responses, and restoration. Windows native testing confirmed application launch, folder selection, song display, restart restoration, Japanese/whitespace path handling, and no observed source-folder changes.

## Phase 3 acceptance criteria

- Rust owns audio-device access and exposes typed, purpose-specific commands to React; the UI does not open audio streams directly.
- Users can enumerate and select Windows input and output devices, including clearly identified system defaults.
- A short local playback probe reports the selected device, negotiated stream configuration, completion, and actionable errors.
- A microphone probe captures input without loopback capture or audible monitoring and reports negotiated configuration, level activity, and measured callback cadence.
- Device loss, permission denial, unsupported configurations, and stream failures identify the failed stage and a safe retry action.
- The real-time callback path performs no filesystem, database, network, model, or UI work and remains CPU-capable.
- Automated tests use an injected audio boundary; Windows-only device, playback, microphone, and latency evidence follows `docs/windows-test-plan.md`.

## Phase 3 initial test-first steps

1. Review the selected CPAL version, license, dependency impact, Windows WASAPI behavior, and Linux build prerequisites before adding it.
2. Add failing tests for a typed audio-device service using an injected fake backend; cover defaults, input/output separation, duplicate display names, and backend errors.
3. Implement the smallest read-only CPAL adapter and path-free Tauri command needed to enumerate devices and supported/default configurations; do not open streams yet.
4. Connect device selection in React and verify safe empty/error states without starting playback or capture.
5. Add a generated-tone playback probe behind the injected boundary, then validate it on Windows shared-mode WASAPI.
6. Add a bounded microphone-level probe with no loopback or monitoring, then measure callback cadence on Windows.

