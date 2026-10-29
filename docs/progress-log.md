# Progress and verification log

## 2026-10-27 — Repository foundation and initial library UI

- Created a Tauri 2 + React/TypeScript + Rust skeleton.
- Added a Docker-based VS Code Dev Container with Node, Rust, Python, FFmpeg, Git LFS, and Codex CLI.
- Added persistent container volumes for Codex state and build caches.
- Added durable requirements, architecture, roadmap, decision, handoff, and Windows test documents.
- Added a first-pass Japanese song-selection interface and focused UI tests.
- `npm run lint`: passed.
- `npm test`: passed, 2 tests.
- `npm run build`: passed; production assets generated.
- Chrome visual inspection: song list, selection detail, analysis state, privacy note, and post-song score policy rendered correctly.
- Dev Container image: Dockerfile parsed and the base image downloaded; the full first build was stopped by request after scope was limited to preparing the environment. Post-create and Rust checks remain unverified.

## 2026-10-27 — Dev Container setup repair

- Reproduced the VS Code setup failure in the generated container image.
- Root cause: Git rejected the Windows-mounted repository because its container-visible owner differed from the `vscode` user; `git lfs install --local` therefore reported that it was not in a Git repository.
- Repair: register only the active mounted workspace as a Git `safe.directory` before running post-create setup.
- Complete post-create rerun: passed as the non-root `vscode` user. `npm ci`, Python virtual environment creation, local Git LFS initialization, Git repository detection, and `codex --version` all completed successfully.

## 2026-10-27 — Windows container launch repair

- Read the complete VS Code Dev Containers log after the container still failed before post-create.
- Root cause: VS Code automatically injected a bind mount from `\\wsl.localhost\Ubuntu\mnt\wslg\runtime-dir\wayland-0`; Docker Desktop could not reach the Ubuntu distro mount service and returned `ubuntu.sock: no such file or directory`.
- Repair: disable automatic Wayland socket mounting for this workspace. This development container does not run Linux desktop windows, so the socket is unnecessary; the Vite UI remains available through forwarded port 1420.
- Preventive follow-up: prepare Codex and Cargo volume mount points as the non-root `vscode` user inside the image, and remove the unnecessary `node_modules` named volume. Direct permission probes showed that newly created empty volumes otherwise defaulted to root ownership.

## 2026-10-27 — Phase 2 read-only discovery slice

- Added temporary-fixture tests for recursive, case-insensitive MP3, FLAC, and WAV discovery, ignoring unsupported files, and leaving source contents and directory entries unchanged.
- RED evidence: `cargo test --manifest-path src-tauri/Cargo.toml --lib library::tests` ran 2 tests; both failed at the intentional unimplemented scanner `todo!`.
- Implemented a typed scanner using only `std::fs::read_dir` and metadata inspection. It follows directories but not symbolic links, includes only regular supported files, and returns a deterministic path order. It contains no source write operation.
- GREEN evidence: `cargo test --manifest-path src-tauri/Cargo.toml --lib library::tests` passed, 2 tests.
- `rustfmt --check src-tauri/src/library.rs`: passed.
- `git diff --check`: passed.
- Full `cargo fmt --check`: failed because existing `src-tauri/build.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/main.rs` have file-ending formatting differences. No repository-wide formatting rewrite was made in this focused slice.
- The first integration-test attempt did not reach the scanner because `tauri::generate_context!()` reported missing `src-tauri/icons/icon.png`. The pure library tests were isolated from desktop startup; normal Tauri build/check remains unverified.

## 2026-10-27 — Tauri icon repair

- Generated an original transparent RGBA application icon with a gold/navy karaoke microphone and teal sound-wave accent, matching the existing UI palette, and saved it at `src-tauri/icons/icon.png`.
- Source asset inspection: PNG, 1254 x 1254, 8-bit RGBA.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed; the earlier `tauri::generate_context!()` missing-icon error is resolved.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed; 2 library tests, 0 binary tests, and 0 doc tests failed.
- `git diff --check`: passed.
- Windows icon rendering and installer packaging remain unverified. Additional target-specific icon formats may be generated when packaging requirements are exercised in Phase 8.

## 2026-10-27 — Phase 2 SQLite catalog foundation

- Added `rusqlite` with only the `bundled` feature. Cargo resolved `rusqlite` 0.40.2 and `libsqlite3-sys` 0.38.2 and updated the existing untracked lockfile.
- Added tests for creating schema version 1 at an injected application-data path and preserving an inserted source record across close/reopen.
- RED evidence: `cargo test --manifest-path src-tauri/Cargo.toml --lib catalog::tests` ran 2 catalog tests; both failed at the intentional migration `todo!`.
- Implemented an application-local `Catalog` that enables SQLite foreign keys and applies the initial `library_sources` schema in an immediate transaction. The schema stores a unique source path and a validated 64-character content SHA-256 string.
- GREEN evidence: focused catalog tests passed, 2 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 4 tests total.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `rustfmt --check src-tauri/src/catalog.rs src-tauri/src/library.rs`: passed.
- `cargo tree --manifest-path src-tauri/Cargo.toml -p rusqlite --prefix none`: completed and listed 15 packages including `rusqlite` itself on the Linux development target.
- `git diff --check`: passed.
- Windows compilation and binary-size impact remain unverified.

## 2026-10-27 — UI reference clarification and cache identity slice

- Recorded the user-approved UI direction in `docs/ui-direction.md`: Nightingale informs local-library song selection; only the DAM main karaoke display informs singing and post-performance result presentation. DAM Denmoku/handset UI is explicitly not a selection reference.
- Added `sha2` 0.10.9 as a direct dependency for content identity.
- Added tests for the known SHA-256 of a source fixture, source-content preservation, content-addressed manifest paths, and rejection of a traversal pipeline version.
- RED evidence: `cargo test --manifest-path src-tauri/Cargo.toml --lib cache::tests` ran 3 tests; all failed at the intentional hashing/path `todo!` implementations.
- Implemented streaming SHA-256 through `File::open` and a fixed-size read buffer. Implemented strict lowercase SHA-256 and safe pipeline-version validation before constructing `<cache-root>/<hash>/<pipeline-version>/manifest.json`.
- GREEN evidence: focused cache tests passed, 3 tests. Full Rust tests passed, 7 tests total.
- `npm run lint`: passed.
- `npm test`: passed, 2 tests.
- `npm run build`: passed.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- Atomic JSON manifest writing and recovery behavior remain unimplemented and unverified.

## 2026-10-27 — Phase 2 atomic cache manifest

- Defined the minimal manifest schema with schema version, source SHA-256, and pipeline version; no artifact format or model choice was introduced.
- Added tests for atomic JSON commit, retry after a stale `manifest.json.tmp`, and preservation of an existing valid manifest.
- RED evidence: focused cache tests ran 6 tests; the 3 existing identity tests passed and the 3 new manifest-write tests failed at the intentional `todo!`.
- Implemented validation of schema and cache identity, same-directory temporary writing, file synchronization, and final rename. A matching existing manifest is retained byte-for-byte and any stale temporary file is removed.
- GREEN evidence: focused cache tests passed, 6 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 10 tests total.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `rustfmt --check src-tauri/src/cache.rs src-tauri/src/catalog.rs src-tauri/src/library.rs`: passed.
- `git diff --check`: passed.
- Windows filesystem behavior remains unverified and belongs in the Windows test plan execution.

## 2026-10-27 — Phase 2 read-only audio metadata

- Generated small, original sine-wave MP3, FLAC, and WAV test fixtures with title and artist tags. `ffprobe` confirmed all three formats, tags, and nonzero durations; the fixtures contain no user or copyrighted audio.
- Added `lofty` with default features disabled and only `id3v2_compression_support` enabled. Cargo resolved version 0.25.5.
- Observed dependency issue: `lofty` 0.25.5 failed to compile with every feature disabled because its no-compression ID3v2 branch referenced unavailable error types. Enabling the read-only compression feature resolved the crate error.
- Added tests for title, artist, and duration extraction across MP3, FLAC, and WAV and for byte-for-byte source preservation.
- RED evidence: after the dependency compiled, both focused metadata tests failed at the intentional reader `todo!`.
- Implemented a read-only `lofty::Probe` boundary that copies only title, artist, and duration into the application-owned `TrackMetadata` type; no tag writer is exposed.
- GREEN evidence: focused metadata tests passed, 2 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 12 tests total.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- Targeted Rust formatting, `cargo tree -p lofty`, and `git diff --check`: passed.
- Windows metadata parsing remains unverified.

## 2026-10-27 — Phase 2 transactional library indexing

- Added service-level tests covering discovery, metadata extraction, content hashing, SQLite persistence, complete source-byte preservation, and all-or-nothing behavior when one supported file is invalid.
- Added a migration test from catalog schema version 1 to version 2 with preservation of an existing record.
- RED evidence: both service tests failed at the intentional indexing `todo!`; the migration test separately failed because the catalog remained at version 1.
- Migrated the catalog to schema version 2, adding nullable title/artist and nonnegative duration fields. New and version 1 databases migrate inside immediate SQLite transactions.
- Implemented transactional catalog replacement. The service completes all read-only source work before beginning the database transaction, so a metadata/hash failure cannot leave a partial new catalog.
- GREEN evidence: library-only tests passed, 15 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 15 tests total.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- Targeted Rust formatting and `git diff --check`: passed.
- Windows path round-tripping and Windows filesystem execution remain unverified.

## 2026-10-27 — Stop-point verification

- `npm run lint`: passed.
- `npm test`: passed, 2 tests.
- `npm run build`: passed; production assets generated.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 15 tests.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `git diff --check`: passed after the handoff update.
- No Windows-only validation was performed at this stop point.

## 2026-10-29 — Commit preparation

- Reviewed the accumulated Phase 2 diff and separated it into UI/decision documentation, the generated Tauri icon, the read-only library/catalog foundation, and final progress/handoff documentation.
- Created `39f15b0` (`docs: record phase 2 and UI direction decisions`).
- Created `2bf667c` (`build: add Tauri application icon`).
- Created `e36e99a` (`feat: add read-only library catalog foundation`).
- No push, tag, release, history rewrite, or destructive operation was performed.

## 2026-10-29 — Rust-owned library command boundary

- Added a Rust-owned library state that validates and retains the selected directory; React cannot supply an arbitrary source path to the indexing command.
- Added a purpose-specific Tauri indexing command whose serialized result exposes content identity and display metadata but no source path.
- Initialized the state with an application-data SQLite catalog path during Tauri setup. Folder-picker integration remains intentionally separate.
- RED evidence: `cargo test --manifest-path src-tauri/Cargo.toml --lib library_commands::tests` ran 3 tests; all failed at the intentional `todo!` implementations.
- GREEN evidence: the focused command-boundary tests passed, 3 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 18 tests total.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run lint`: passed.
- `npm test -- --run`: passed, 2 tests.
- `npm run build`: passed.
- Targeted Rust formatting and `git diff --check`: passed.
- Tauri launch, native folder selection, and Windows path behavior remain unverified.

## 2026-10-29 — Native library folder selection boundary

- Added the official Rust-side `tauri-plugin-dialog` 2.8.4 and registered it with the Tauri builder; no JavaScript dialog API or filesystem permission was exposed.
- License inspection found `tauri-plugin-dialog` under Apache-2.0 OR MIT and its native picker dependency `rfd` 0.17.2 under MIT.
- Added a dedicated asynchronous Tauri command that opens a single-folder picker and stores a validated local path only in `LibraryState`. Its response contains only a selection boolean.
- RED evidence: the focused command tests ran 5 tests; 3 existing tests passed and the 2 new cancel/select tests failed at the intentional `todo!`.
- GREEN evidence: focused command tests passed, 5 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 20 tests total.
- The first dependency-fetch attempt failed because the sandbox could not resolve crates.io; the approved network retry downloaded the official plugin successfully.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run lint`: passed.
- `npm test -- --run`: passed, 2 tests.
- `npm run build`: passed.
- Targeted Rust formatting and `git diff --check`: passed.
- Actual native dialog rendering, cancellation, Windows path conversion, and Tauri launch remain unverified outside the container.

## 2026-10-29 — React library import connection

- Added a typed React-side command client for native folder selection and selected-library indexing. Its types contain no source path.
- Connected the existing import button to select first and index only after successful selection. Cancellation retains the current library; failures retain it and name a safe folder-reselection recovery action.
- Successful indexing replaces mock rows with returned title, artist, duration, and content identity. Imported songs remain in the analysis-waiting state and cannot be sung before future analysis work.
- Empty indexed libraries render a safe empty state. UI row identity adds only an in-memory ordinal to the content hash so duplicate source content cannot collide; no path is introduced.
- RED evidence: `npm test -- --run src/App.test.tsx` ran 4 tests; the 2 existing tests passed and the 2 new native-import flow tests failed because the button was not connected.
- GREEN evidence: focused frontend tests passed, 4 tests. The first post-implementation run reached the correct UI state but required multiple-element assertions because the selected song appears in both the list and detail card.
- `npm run lint`: passed.
- `npm run build`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 20 tests.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `git diff --check`: passed.
- Actual Tauri command invocation and native Windows folder selection remain unverified.

## 2026-10-29 — Persisted catalog restoration

- Added a path-free Rust command that reads an existing SQLite catalog into the same display DTO used after indexing.
- A missing catalog returns an empty result without creating a database file. Existing catalog records retain source paths only inside SQLite and return content IDs, title, artist, and duration to React.
- React now requests the saved catalog at startup, briefly disables folder import to avoid a load/import race, and replaces mock rows only when persisted songs exist. A safe folder-reselection recovery action is shown if restoration fails.
- RED evidence: both new focused Rust tests failed at the intentional catalog-load `todo!`; the new React restoration test failed because no startup request existed.
- GREEN evidence: full frontend tests passed without React warnings, 5 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 22 tests total.
- `npm run lint`: passed.
- `npm run build`: passed.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- Targeted Rust formatting and `git diff --check`: passed.
- Actual Tauri startup restoration, native Windows path handling, and dialog/index/restart round trip remain unverified.

## 2026-10-29 — Linux Dev Container Japanese rendering repair

- Reproduced the environment evidence behind garbled Japanese text: `fc-list :lang=ja` returned no fonts, and `dbus-launch` was absent while the Linux Tauri application emitted a dconf warning.
- Added Debian `fonts-noto-cjk` for Japanese glyph coverage and `dbus-x11` for `dbus-launch` to the Dev Container image dependencies.
- Source files and HTML already used UTF-8, so no application text or encoding conversion was needed.
- Dockerfile inspection and `git diff --check`: passed. Dev Container rebuild, post-rebuild Japanese rendering, and dconf warning removal remain unverified.

## 2026-10-29 — Phase 2 continuation preflight

- Confirmed Phase 2 remains the only approved in-progress phase and that no decision is waiting.
- `npm run lint`: passed.
- `npm test -- --run`: passed, 5 tests.
- `npm run build`: passed; production assets generated.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 22 tests.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `git diff --check`: passed before this documentation update.
- Inspected the untracked screenshot read-only. It shows Japanese glyphs as missing boxes, so it does not verify the Dev Container font repair; whether it was captured before or after the required rebuild is unknown.
- No Windows-only validation was performed. The Phase 2 native Windows dialog/index/restart round trip remains the next required evidence.

## 2026-10-29 — Linux Dev Container manual smoke observation

- The user reported that Japanese text rendered correctly after launching the Tauri application with `npm run tauri dev`.
- The user also reported that adding files through the application appeared to work.
- This is user-observed Linux Dev Container smoke evidence. Exact imported formats, catalog restoration after restart, source immutability, dconf warning removal, and all Windows-specific behavior were not independently verified in this observation.

## 2026-10-29 — Song-selection visual simplification

- Removed the decorative `HIBIKI` lockup at the user's request.
- Reworked the selection screen toward a restrained Material Design 2-inspired visual system without adding a UI dependency or changing React/Tauri: flat neutral surfaces, modest 4 px radii, restrained status colors, no glow/gradient effects, and compact navigation and controls.
- Replaced the repeated song cards with a semantic compact table containing title, artist, format, duration, and analysis-state columns. Replaced `AI解析中` with the plain status `解析中`.
- RED evidence: the new table/wording test failed because the prior card list had no column headers and still rendered `AI解析中`.
- GREEN evidence: `npm test -- --run` passed, 6 tests. `npm run lint` and `npm run build` passed.
- The user subsequently reported that the revised UI was substantially improved and that registering the untracked `sample music/` folder containing audio files worked in the Linux Dev Container Tauri application.
- Native visual review at the target Windows size remains unverified. The contents of `sample music/` were not inspected or added to Git.

