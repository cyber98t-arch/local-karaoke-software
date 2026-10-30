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

## 2026-10-29 — Windows Tauri launch prerequisite repair

- The first Windows launch reached Rust after installing/finding the MSVC Rust toolchain, then failed because the repository contained only `icon.png` while `tauri-build` requires `src-tauri/icons/icon.ico` for a Windows resource.
- Derived a six-size Windows ICO (256, 128, 64, 48, 32, and 16 px) from the existing application icon without changing its design.
- `cargo check --manifest-path src-tauri/Cargo.toml` and `git diff --check` passed after adding the ICO.
- `npm run build` in the Linux Dev Container could not run because the Windows-host `npm ci` replaced the shared `node_modules` with platform-specific dependencies and left the Linux Rollup optional package absent. Added a dedicated Dev Container `node_modules` volume to prevent future Windows/Linux dependency collisions; a container rebuild is required to verify it.
- After the ICO addition, the user successfully launched the native application on Windows, selected a folder, and saw its songs in the library.
- The user then closed and relaunched the Windows application and confirmed that the previously indexed song list restored without selecting the folder again.
- A simultaneously open Dev Container forwarded port 1420 and repeatedly opened VS Code preview tabs during diagnosis. Changed the port behavior from `openPreview` to `notify`; Windows-native testing should not run concurrently with a container dev server on the same host port.
- The user selected a Windows folder named `夜　遊び` containing the same song and confirmed the library continued to display the song normally, providing native evidence for Japanese characters and whitespace in the selected path.
- Windows source-directory immutability remains unverified.

## 2026-10-29 — Phase 2 Windows completion evidence

- The user confirmed that selecting the Windows source folder did not create, modify, rename, move, or delete anything in it.
- Combined with the automated byte-preservation and transactional tests, Windows launch, folder selection, song display, restart restoration, and Japanese/whitespace path evidence, all Phase 2 acceptance criteria are now covered.
- Marked Phase 2 complete. Phase 3 implementation did not begin.
- Reviewed current official audio-library information for the Phase 3 gate. CPAL directly over its default Windows WASAPI backend is the recommended option; Rodio plus CPAL and direct WASAPI bindings remain documented alternatives in D-005.
- Phase-close `cargo test --manifest-path src-tauri/Cargo.toml` passed, 22 tests; `cargo check --manifest-path src-tauri/Cargo.toml` and `git diff --check` passed. The frontend baseline awaits the documented Dev Container rebuild because host Windows `npm ci` replaced the shared Linux-native optional dependency.

## 2026-10-29 — Phase 3 audio-engine approval

- The user approved D-005: direct CPAL audio I/O using the default Windows WASAPI backend.
- Approval is limited to shared-mode device enumeration plus playback and microphone probes. ASIO, loopback capture, audible microphone monitoring, scoring, and production playback remain outside this decision.
- Added explicit Phase 3 acceptance criteria and a test-first sequence. No audio dependency or implementation was added at this stop point.

## 2026-10-29 — Phase 3 CPAL dependency review

- Selected CPAL 0.18.2 with default features only. Its published crate metadata and included license declare Apache-2.0, and its Rust 1.85 minimum is below the current Rust 1.99.0 toolchain.
- Confirmed that Windows uses the WASAPI implementation through `windows`/`windows-core` 0.62 and that the Dev Container already includes the Linux ALSA development package required for local compilation.
- Explicitly excluded ASIO and every optional CPAL feature. No dependency was added and no audio stream was opened during this review.
- Detailed evidence and integration constraints are in `docs/phase-3-dependency-review.md`.

## 2026-10-29 — Phase 3 typed audio-device service

- Added application-owned audio direction, backend-device, and serialized device value types behind an injected `AudioDeviceBackend` trait. No CPAL type crosses the service boundary.
- RED evidence: both focused tests failed at the intentional device-enumeration `todo!`.
- Implemented the smallest service that combines input and output devices, marks defaults by stable ID rather than display name, preserves duplicate display names with distinct IDs, and returns stage-specific safe-retry errors.
- GREEN evidence: focused audio-device tests passed, 2 tests. Full `cargo test --manifest-path src-tauri/Cargo.toml` passed, 24 tests total; `cargo check --manifest-path src-tauri/Cargo.toml`, targeted Rust formatting, and `git diff --check` passed.
- CPAL has not yet been added, no Tauri command exposes this service, and no audio device or stream was opened.

## 2026-10-29 — CPAL enumeration and output probe

- Added CPAL 0.18.2 with its empty default feature set; no optional feature such as ASIO was enabled. The resolved Linux CPAL subtree contains four direct/transitive audio packages and all inspected licenses are Apache-2.0, MIT, or both.
- Implemented a CPAL adapter that reads stable device IDs and names from input/output enumerators and system defaults, then exposed it through `list_audio_devices`. Enumeration does not open a stream.
- Added typed React invocation and compact input/output selectors. The system default is selected by ID; empty and backend-error states provide recovery text.
- RED evidence: the frontend device test failed because no accessible input/output selectors existed.
- Added a selected-output probe that negotiates the default format, generates a 440 Hz sine at 0.15 amplitude for 500 ms, supports CPAL's integer and floating sample formats, and reports stream failures without reading a source file.
- RED evidence: the oscillator test failed at its intentional `todo!`. GREEN evidence: the focused oscillator test and full Rust suite passed; full Rust count is 25 tests.
- Frontend tests passed, 8 tests without React warnings. `npm run lint`, `npm run build`, `cargo check --manifest-path src-tauri/Cargo.toml`, targeted Rust formatting, and `git diff --check` passed.
- Windows WASAPI device enumeration and audible test-tone confirmation remain pending user observation. Microphone capture has not been implemented.

## 2026-10-29 — Windows WASAPI enumeration and playback observation

- The user confirmed on Windows that input/output selectors populated, defaults were identified, output selection worked, the 500 ms test tone was audible, and no error or application stop occurred.
- This completes native validation of Phase 3 device enumeration and the bounded shared-mode output probe. Microphone capture and callback-cadence evidence remain pending.

## 2026-10-29 — Bounded microphone-level probe

- Added a selected-input Tauri probe that accepts only IDs found through CPAL's input-device iterator, negotiates the default input format, and runs for 1.5 seconds.
- The real-time callback computes a bounded peak and increments an atomic callback counter. It performs no filesystem, database, network, model, UI, output-stream, loopback, monitoring, or sample-retention work.
- The result reports device name, sample rate, channels, duration, callback count, average callback interval, and peak level. No captured samples cross the Rust boundary.
- Added a UI action and safe microphone-permission/reconnection errors; the UI displays peak percentage, callback count, and average callback interval without exposing samples.
- Automated tests cover bounded peak calculation and selected-input invocation. Full Rust tests passed, 26 tests; frontend tests passed, 9 tests. `cargo check`, lint, production build, targeted formatting, and `git diff --check` passed.
- Windows microphone permission, visible peak response to speech, absence of audible monitoring, and callback cadence remain pending user observation.

## 2026-10-30 — Windows microphone callback observation

- Inspected the user's latest Windows screenshot read-only. The selected Logitech C270 microphone completed 147 callbacks over 1.5 seconds with a reported 10.2 ms average interval and no visible error.
- The screenshot reported a 0% rounded peak. Callback delivery is verified, but audible-input response is not yet proven by this evidence.
- Corrected the UI so a zero peak reports `音声レベルは検出されませんでした` instead of claiming microphone input was detected. Added a regression test; frontend tests now pass 10 tests, with lint and production build passing.
- The screenshot remains untracked and was not modified or committed.

## 2026-10-30 — Phase 3 Windows completion evidence

- The user reran the selected-input probe while speaking and confirmed it worked, completing the requested check for a nonzero microphone response without audible monitoring.
- Native Windows evidence now covers device enumeration and defaults, input/output selection, audible generated output, microphone callback cadence, nonzero level activity, and loopback-free/no-monitoring behavior.
- Phase 3 is complete. The phase-close baseline passed: frontend lint, 10 frontend tests, production build, 26 Rust tests, Rust check, and `git diff --check`.
- Opened D-006 as `Waiting`. Phase 4 implementation and model downloads remain blocked until the local model families and license/provenance constraints are explicitly approved.

## 2026-10-30 — Phase 4 permissive-model preflight

- The user approved continuing a conservative model review while excluding HTDemucs weights whose artifact terms are not sufficiently clear. This was research approval, not model-family adoption.
- Reviewed official upstream sources for Spleeter, Open-Unmix, CREPE, Basic Pitch, Whisper, and `whisper.cpp`; no dependency or model was downloaded.
- Spleeter 2-stem is the current separation evaluation recommendation because its official paper explicitly describes the pretrained models and code as MIT-distributed. Its TensorFlow packaging cost and 11 kHz model bandwidth remain material risks.
- CREPE is the current MIT-licensed monophonic-pitch recommendation. Multilingual Whisper `base` through `whisper.cpp` is the current MIT-licensed Japanese transcription recommendation at approximately 142 MiB.
- D-006 remains `Waiting` for explicit adoption of the evaluation families. Exact revisions, artifacts, hashes, installed sizes, Windows CPU performance, and output quality remain unverified.

## 2026-10-30 — Phase 4 model-family approval

- The user approved D-006: evaluate Spleeter 2-stem, CREPE, and multilingual Whisper `base` through `whisper.cpp`; HTDemucs weights remain excluded.
- Added Phase 4 acceptance criteria and a test-first sequence centered on read-only sources, cache-only atomic artifacts, model provenance, resumable jobs, and a required Windows CPU path.
- No model, runtime, or dependency was downloaded or added during this approval step.

## 2026-10-30 — Validated model artifact manifest

- Added an application-owned serialized model manifest covering family, stable ID, version/revision, HTTPS artifact source, SHA-256, exact byte size, SPDX license identifier, and HTTPS license evidence.
- RED evidence: the focused tests failed at the intentional validation `todo!` for both valid and unsafe metadata cases.
- GREEN evidence: the focused suite passed 3 tests. The validator rejects traversal-capable IDs, non-HTTPS sources/evidence, malformed hashes, zero-byte artifacts, missing license identifiers, and unsupported schema versions.
- Phase-step baseline passed: frontend lint, 10 frontend tests, production build, 29 Rust tests, Rust check, targeted Rust formatting, and `git diff --check`.
- No network access, model download, runtime dependency, source-audio access, or cache write was added by this implementation.

## 2026-10-30 — Content-addressed analysis job boundary

- Added an analysis identity derived only from source SHA-256, pipeline version, ordered model IDs, and analysis-parameter SHA-256. Model IDs are length-delimited and hashed into one path component to avoid ambiguous joins and path disclosure.
- Added typed queued, running-stage, completed, failed, and cancelled states. Completed/cancelled jobs are terminal; actionable failures retain their stage and only explicitly retryable failures may restart.
- RED evidence: cache identity tests failed at the intentional `todo!`; state tests then failed to compile before transition methods existed.
- GREEN evidence: 5 focused analysis-job tests passed, covering cache containment, unsafe/colliding identity rejection, path-free progress serialization, cancellation, retry, completion, and terminal-state protection.
- Full baseline passed: frontend lint, 10 frontend tests, production build, 34 Rust tests, Rust check, targeted Rust formatting, and `git diff --check`.
- No model/runtime dependency, download, process launch, original-audio write, or cache write was added.

## 2026-10-30 — Pinned evaluation artifact metadata

- Added validated JSON metadata for Spleeter 2-stem v1.4.0 (73,109,797 bytes), CREPE tiny at models commit `bb29b8d` (1,307,310 bytes compressed), and multilingual Whisper `base` at converted-model commit `5359861` (147,951,465 bytes).
- Recorded immutable HTTPS artifact URLs, SHA-256 values, exact byte sizes, SPDX MIT identifiers, and upstream license evidence. Tests load all three bundled manifests and validate the typed boundary.
- Spleeter's SHA-256 came from its official `checksum.json`; Whisper's size and SHA-256 came from the official converted-model repository. CREPE publishes a Git blob hash but not SHA-256, so its 1.3 MB `tiny` artifact was fetched to a temporary file and measured as `9c1d76d405674d2327f5636347d07f551dfd11c6acc0475b7fd85082cdab0add`.
- No model binary was copied into the repository or application cache. Spleeter and Whisper model binaries were not downloaded.
- Full baseline passed: frontend lint, 10 frontend tests, production build, 35 Rust tests, Rust check, targeted Rust formatting, and `git diff --check`.

## 2026-10-30 — Atomic analysis state persistence

- Added cache-owned append-only state snapshots. Every transition writes and syncs a new temporary JSON file, then renames it to a previously unused sequence path; existing snapshots are never overwritten.
- Recovery removes the matching stale temporary file before retry. Persisted transition validation requires an initial queued state, permits active progress/failure/cancellation, permits restart only after retryable failure, and keeps completed, cancelled, and non-retryable failure states terminal.
- RED evidence: the persistence tests failed to compile before `append_job_state` existed. GREEN evidence: 7 focused analysis-job tests passed, including stale-temporary recovery and preservation of completed state.
- Full baseline passed: frontend lint, 10 frontend tests, production build, 37 Rust tests, Rust check, targeted Rust formatting, and `git diff --check`.
- No process was launched and no model/runtime dependency or original source file was opened.

## 2026-10-30 — Spleeter runtime dependency preflight

- Reviewed the official Spleeter 2.4.2 dependency declaration and Windows guidance without installing packages. The supported range is Python 3.8–3.11 with TensorFlow 2.12.1 and NumPy below 2.0.
- TensorFlow dominates the expected dependency/installed size. Windows CPU functionality is claimed upstream but remains unmeasured on the target machine class.
- Defined a bounded first probe that uses an isolated Python 3.11 CPU worker and a generated PCM array through the Spleeter API, avoiding user music and an external FFmpeg executable.
- Deferred real source decoding, FFmpeg build/license selection, production Python bundling, and ONNX conversion. None was silently chosen or installed.

## 2026-10-30 — Versioned local analysis-worker protocol

- Added a protocol-versioned Rust request/response boundary behind an injected backend. The synthetic probe request contains only a safe request ID, fixed operation, and cache-owned output directory; it has no source-audio field.
- The service rejects traversal-capable IDs, protocol/request mismatches, failed responses without actionable errors, missing metrics, and zero-valued metrics.
- Added a Python-standard-library fake worker that reads one JSON request from stdin, writes one cache-owned probe manifest by temporary-file replacement, and emits one JSON response. It has no model, audio decoder, network, or user-source access.
- RED evidence: 2 focused Rust tests failed at the intentional service `todo!`. GREEN evidence: both focused tests passed, and a manual fake-worker invocation returned the expected 16 kHz stereo/16,000-frame metrics.
- Full baseline passed: frontend lint, 10 frontend tests, production build, 39 Rust tests, Rust check, targeted Rust formatting, Python bytecode compilation, and `git diff --check`.

## 2026-10-30 — Bounded local worker subprocess

- Added a concrete worker backend that launches only an injected executable and script via the operating-system process API; it never invokes a command shell or accepts arbitrary worker arguments.
- The adapter writes one JSON request to stdin, closes it, drains stdout/stderr concurrently, retains only a configured maximum, polls a fixed deadline, and kills/waits for a worker that times out.
- RED evidence: focused tests failed to compile before `LocalProcessWorker` existed. GREEN evidence: 4 worker tests passed, including a real fake-worker subprocess and termination of a deliberately hanging Python fixture.
- Unix subprocess behavior is verified in the container. Native Windows process launch/termination remains unverified and is intentionally separate from model inference evidence.
- Full baseline passed: frontend lint, 10 frontend tests, production build, 41 Rust tests, Rust check, targeted Rust formatting, Python bytecode compilation, and `git diff --check`.

## 2026-10-30 — TensorFlow runtime feasibility stop

- Started a read-only dependency download probe for Spleeter 2.4.2 into an isolated `/tmp` directory; no package was installed and no project/application cache path was used.
- Resolution selected the official TensorFlow 2.12.1 Linux CPython 3.11 wheel at 585.9 MB before transitive dependencies. The unconstrained closure then downloaded multiple alternative gRPC wheels while backtracking.
- Cancelled the resolver rather than consume more network/storage or pretend the environment was reproducible. Pip removed the incomplete temporary artifacts; the probe directory contained no retained wheelhouse afterward.
- This is negative feasibility evidence for shipping the official TensorFlow worker on ordinary laptops. Opened D-007 as `Waiting`, recommending a controlled Spleeter-to-ONNX/CPU Runtime investigation while preserving model provenance and numerical comparison requirements.

## 2026-10-30 — D-007 ONNX feasibility research approval

- The user approved investigation of Spleeter-to-ONNX conversion and CPU ONNX Runtime only. Formal engine adoption, bundling, publishing, and distribution remain explicitly unapproved.
- Required evidence includes TensorFlow/ONNX numerical and listening comparison, elapsed time, peak memory, future Intel/AMD/NVIDIA feasibility, complete license review, and SHA-256 provenance.
- Any large dependency/model download or installation requires another advance approval with its expected capacity and purpose. No new download or installation occurred in this step.
- If conversion is infeasible or materially worse, the research must present permissively licensed lightweight alternatives and return to the user rather than changing model family silently.

## 2026-10-30 — Download-free ONNX feasibility preflight

- Reviewed official tf2onnx conversion support, ONNX Runtime CPU/provider documentation, current package metadata, and licenses without downloading or installing anything.
- Confirmed that checkpoint conversion requires explicit tensor names and that Spleeter publishes no official ONNX artifact/recipe. Current tf2onnx primarily tests TensorFlow 2.13–2.15 while Spleeter pins 2.12.1, so conversion compatibility is an experiment rather than an assumption.
- Defined reproducible numerical, listening, timing, memory, and reconstruction comparisons plus separate Intel Arc/OpenVINO, cross-vendor WinML/DirectML, and NVIDIA/CUDA future paths. No GPU strategy was selected.
- Recorded MIT/Apache-2.0 license preflight and the need for exact transitive notices and converted-artifact provenance before distribution.
- Estimated a conservative research envelope of up to 1.0 GB download and 3.0 GB temporary disk. Per the user's condition, execution is stopped pending explicit capacity/purpose approval.

## 2026-10-30 — Spleeter ONNX CPU feasibility evidence

- After user approval, created a disposable Python environment and downloaded the pinned Spleeter artifact. Actual transfer stayed below 1.0 GB. Temporary disk briefly reached approximately 3.2 GB while the wheelhouse and installed environment coexisted; this exceeded the 3.0 GB estimate by about 0.2 GB, was reported immediately, and the research-owned 789 MB wheelhouse was removed before continuing.
- Full waveform-to-waveform conversion failed because tf2onnx 1.16.1 could not convert Spleeter's RFFT layout or IRFFT nodes. Conversion of the neural spectrogram subgraph succeeded at opset 13; NumPy performs the STFT, ratio masks, and inverse STFT.
- CPU ONNX Runtime successfully separated a generated stereo signal. Against TensorFlow, final vocals waveform RMSE was `3.4119e-8` and accompaniment waveform RMSE was `6.0138e-7`; maximum absolute differences were `1.7862e-7` and `2.2948e-6` respectively.
- One generated 10-second cold-process observation measured TensorFlow at 0.368 s and 919,064 KiB peak RSS, versus the ONNX path at 0.257 s and 439,984 KiB peak RSS. These are preliminary Linux-container observations, not Windows acceptance evidence.
- Two conversions were semantically valid but produced different hashes because tf2onnx generated different internal temporary node identifiers. The second artifact was 78,652,254 bytes with SHA-256 `8d58651b5979075c954bcbe96c6776b2dc41e7852fb6f2d054b83a696a8db26f`; byte-deterministic conversion remains unresolved.
- Added reproducible inspection, conversion, CPU runtime, signal-transform test, and installed-environment inventory scripts. The 71-distribution inventory records versions, declared license metadata, and per-distribution installed-file hashes. No model, converted ONNX binary, generated audio, or temporary environment is committed.
- Remaining evidence is representative-music A/B listening, repeated and Windows CPU measurements, Windows wheel hashes/notices, and an explicit post-report adoption decision. ONNX adoption, bundling, distribution, and GPU-provider selection remain unapproved.

## 2026-10-30 — Representative-music ONNX comparison prepared

- The user placed a 44.1 kHz, 16-bit stereo PCM WAV in the sample directory for the approved comparison. Read a 30-second excerpt from 60–90 seconds and wrote the excerpt plus four stem outputs only below a new `/tmp` directory.
- The source WAV remained byte-for-byte and metadata-identical: its 44,331,086-byte size, modification timestamp, mode, and SHA-256 were unchanged after the probe.
- After 16-bit PCM encoding, TensorFlow and ONNX outputs differed by at most one PCM least-significant bit for both stems. RMSE was `4.6389e-6` for vocals and `6.1870e-6` for accompaniment.
- This run measured TensorFlow at 0.751 s and 1,288,380 KiB peak RSS. ONNX plus NumPy transforms measured 1.063 s and 925,896 KiB peak RSS; model execution alone was 0.507 s. The prototype reduced peak memory but was slower overall on this run.
- Numerical comparison is complete for this excerpt. Listening quality remains pending the user's judgment; no audible-equivalence conclusion has been recorded yet.

## 2026-10-30 — CPU ONNX separation adopted

- The user reported no audible difference between the TensorFlow and ONNX stem pairs and chose the lighter option.
- D-007 now adopts the pinned Spleeter neural spectrogram subgraph on CPU ONNX Runtime. Application-owned CPU signal processing remains part of the approved path; TensorFlow is retained only as a research reference and will not be a production dependency.
- The choice prioritizes lower peak memory and a smaller production dependency surface. The representative 30-second prototype was slower overall because the current Python/NumPy transforms are not optimized; no speed advantage is claimed.
- GPU provider selection, Windows packaging hashes/notices, automatic model-download behavior, publishing, and release remain separate gates.

## 2026-10-30 — Production two-stem worker contract

- Extended the versioned local-worker boundary with a typed two-stem separation request. Rust supplies existing native-owned input/model paths and expected source/model SHA-256 values; React receives none of these paths.
- The output directory is derived from the validated analysis identity beneath the injected application-cache root. Successful responses must identify fixed `vocals.wav` and `accompaniment.wav` artifacts with nonzero byte sizes and lowercase SHA-256 values.
- RED evidence: the focused test failed to compile before the separation types, response field, and service method existed. GREEN evidence: the new focused test passed and confirmed the input fixture remained unchanged.
- All 42 Rust tests and `cargo check` passed; `git diff --check` passed. No model runtime, dependency, download, decoder, UI command, or actual inference launch was added.

## 2026-10-30 — Worker-side separation commit boundary

- Added a dependency-free Python boundary that validates the versioned request, existing input/model files, exact SHA-256 values, safe request ID, and fresh cache output directory before invoking an injected inference function.
- The worker writes only request-specific cache temporary files, requires two nonempty outputs and valid stereo metrics, renames them to fixed artifact names, and returns their byte sizes and SHA-256 values. Hash mismatch, stale temporary output, and existing valid artifacts fail without overwrite.
- RED evidence: both Python tests initially failed because the worker module did not exist. GREEN evidence: 2 Python tests and 5 focused Rust worker tests passed; Python bytecode compilation and `git diff --check` passed.
- Actual inference remains intentionally disconnected. Opened D-008 as `Waiting` because native Rust ONNX versus bundled Python materially changes installed size and Windows packaging.

## 2026-10-30 — Native Rust ONNX boundary approved

- The user approved D-008's recommended native Rust ONNX Runtime plus Rust signal-processing direction. Production separation will not bundle Python or NumPy.
- Reviewed current primary metadata without downloading dependencies: `ort` 2.0.0-rc.15 targets ONNX Runtime 1.31, uses `MIT OR Apache-2.0`, requires Rust 1.92, and defaults to binary download/copy plus extra features. `realfft` 3.5.0 is MIT and supports the required real FFT path.
- The current Rust 1.99 toolchain satisfies both MSRVs. CPU-only minimal features, exact version pins, artifact hashes, notices, and offline behavior are required; no GPU provider was selected.
- Stopped before dependency resolution/download. The proposed bounded probe is up to 150 MB network transfer and 600 MB additional temporary build/cache storage for crates, one CPU runtime archive, hash/license capture, compilation, and a generated-input test.

## 2026-10-30 — Native ONNX dependencies pinned

- After capacity approval, pinned `ort`/`ort-sys` 2.0.0-rc.15 and `realfft` 3.5.0 with CPU-only features; Cargo.lock records the complete crates.io closure and checksums. No GPU feature was enabled.
- `cargo check` passed using a dedicated target. The cached Linux CPU static ONNX Runtime library is 111,701,892 bytes with SHA-256 `184360cecb39400bc59f98269ed179f4925bf75dd11be084604f4070edbf1493`.
- Cargo registry growth was approximately 35.4 MB. The dedicated target rebuilt all Tauri dependencies and temporarily grew to approximately 1.38 GB, causing total growth of about 1.53 GB—roughly 930 MB over the approved estimate.
- Stopped immediately, reported the overage, and removed only the research-owned temporary target. Remaining added caches total approximately 147 MB. Windows runtime bytes/hashes and native inference remain unverified.

## 2026-10-30 — Native Spleeter signal transforms

- Added application-owned Rust STFT/inverse-STFT using pinned `realfft`: 4096-point periodic Hann windows, 1024-sample hop, stereo channels, tail padding, overlap-add, and Spleeter's `2/3` compensation.
- RED evidence: the focused generated-stereo test failed at the intentional STFT `todo!`. GREEN evidence: the fully overlapped region round-tripped within `2e-5` per sample/channel.
- Added magnitude partitioning into ONNX input shape `[segments, 512, 1024, 2]` with zero padding after the final real frame. Both focused signal tests passed.
- No source audio, model artifact, GPU provider, decoder, UI command, or cache output was accessed by these tests.

