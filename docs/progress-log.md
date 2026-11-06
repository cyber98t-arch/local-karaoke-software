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

## 2026-10-30 — Native tensor and mask boundary

- Added a typed spectrogram-inference trait, pinned Spleeter input/output tensor names and shapes, and an `ort` session implementation behind the explicit `native-onnx` Cargo feature.
- An actual Linux test link exposed ABI incompatibility in the rc.15 prebuilt static runtime (`__isoc23_strtol` and newer libstdc++ symbols). Normal Linux tests now exclude the optional native link; `cargo check --features native-onnx` and default `cargo check` both pass. Windows linking remains unverified and no version was silently changed.
- Added application-owned squared ratio masks with Spleeter's epsilon behavior and zero high-frequency extension. Four focused Spleeter tests pass, covering tensor contract, STFT round trip, segment padding, and processed-band reconstruction.

## 2026-10-30 — Injected native separation pipeline

- Added one typed `separate_waveform` path from generated stereo PCM through Rust STFT, ONNX-shaped magnitude input, injected inference, ratio masks, and Rust inverse STFT.
- A dependency-free fake inference implementation routes the generated mixture to vocals and silence to accompaniment. The focused end-to-end test passed and verified output length plus expected stem-energy separation.
- Five focused Spleeter tests pass. Actual ONNX execution remains feature-gated and awaits native Windows linking; no user audio or model was opened by this step.

## 2026-10-30 — Decoder and model acquisition approved

- The user approved D-009: pin Symphonia 0.6.1 with only MP3, FLAC, PCM, and WAV/RIFF support for Rust-owned read-only analysis decoding. MPL-2.0 notices and obligations must be retained.
- The user approved D-010: the converted ONNX model will use an explicit one-time app-managed download into a separate model cache, with exact size/hash/provenance/license validation and atomic commit. Development remains local-injected until a release asset is separately approved for publication.
- No dependency, model, or release asset was downloaded or published in this approval step.

## 2026-10-30 — Read-only native analysis decoding

- Pinned Symphonia 0.6.1 with default features disabled and only FLAC, MP3, PCM, and WAV support enabled. The decoder opens sources through a Rust-owned read-only file handle and produces native-rate stereo `f32` frames in memory.
- Mono input is duplicated to stereo, stereo input is retained, and inputs with more than two channels fail rather than being silently remixed. No resampler, cache write, UI command, model download, or playback connection was added.
- RED evidence: the first focused test failed at the intentional decoder `todo!`. GREEN evidence: both focused decoder tests passed for MP3, FLAC, and WAV fixtures; all 49 Rust library tests passed.
- The tests observed 44.1 kHz preservation, finite/nonempty output, exact mono-to-stereo duplication, byte-for-byte source preservation, and an unchanged source-directory entry list. `cargo check` and `git diff --check` also passed.
- Symphonia is MPL-2.0 and unmodified. Final installer notices/source-availability materials and native Windows validation remain packaging work; see `docs/phase-4-analysis-decoder-review.md`.

## 2026-10-30 — Read-only decoding connected to injected separation

- Added a typed path from a Rust-owned decoded source to the existing injected Spleeter spectrogram boundary and Rust signal transforms. It performs no cache write and does not load a real model.
- RED evidence: the focused test failed to compile before the source-separation function existed. GREEN evidence: all three focused inference tests passed, including byte-for-byte preservation of the WAV fixture.
- The connection currently accepts 44.1 kHz only. Other native rates fail with an explicit resampling-not-configured error rather than being silently processed incorrectly.
- Opened D-011 as `Waiting` after reviewing Rubato's current official repository/docs. No resampling dependency was added and no sample-rate strategy was selected.

## 2026-10-30 — Offline analysis resampling approved and implemented

- The user approved D-011. Pinned Rubato 5.0.1 with only its FFT-resampler feature; no playback, microphone, or real-time path was changed.
- RED evidence: the 96 kHz test failed to compile before the application-owned resampling function existed. GREEN evidence: it converted a generated one-second stereo signal to 44.1 kHz and exactly 44,100 finite frames while preserving distinct left/right content.
- `separate_source` now decodes read-only, applies the fixed-ratio conversion when needed, and then enters the existing injected spectrogram-inference boundary. Native-rate 44.1 kHz input takes an identity path.
- Three focused decoder/resampler tests and three focused separation tests passed. `cargo check --features native-onnx` and `git diff --check` passed. Native Windows and representative-source performance remain unverified.
- The phase handoff baseline also passed: frontend lint, 10 frontend tests, production build, all 51 Rust library tests, default `cargo check`, and `git diff --check`.

## 2026-11-05 — Lossless two-stem cache approved and implemented

- The user approved D-012. Pinned pure-Rust `flacenc` 0.5.1 with default features disabled and implemented stereo 44.1 kHz signed 16-bit FLAC output beneath an injected analysis cache only.
- RED evidence: the first focused test failed at the intentional cache-writer `todo!`. Initial encoded files were valid to FFmpeg but exposed fixed-block STREAMINFO incompatibility with Symphonia; explicitly recording equal fixed block sizes resolved application-owned decoding.
- Both stems are encoded and synced as temporary files, decoded through Symphonia for frame/rate validation, renamed to fixed artifact names, then committed by renaming a synced JSON marker containing format, frame count, byte sizes, and SHA-256 values.
- Three cache tests pass: valid two-stem commit/decode, preservation of an existing valid cache, and rejection of nonfinite samples without artifacts. All 55 Rust library tests and `cargo check --features native-onnx` passed after the identity-derived coordinator was added.
- Added an injected end-to-end boundary from read-only source decoding through resampling, fake spectrogram inference, Rust inverse transforms, and FLAC cache commit. Its RED `todo!` failure and GREEN test confirm source-byte preservation and cache-only output.
- The public coordinator now derives the output directory from `AnalysisIdentity` plus the injected application-cache root; callers cannot provide an arbitrary final artifact directory.
- The handoff baseline passed: frontend lint, 10 frontend tests, production build, all 55 Rust library tests, default and `native-onnx` cargo checks, and `git diff --check`.

## 2026-11-05 — Native Windows ONNX Runtime link verified

- The user ran `cargo test --manifest-path src-tauri/Cargo.toml --features native-onnx --lib` from Windows PowerShell with the MSVC toolchain.
- Cargo downloaded 17 crates (3.0 MiB), compiled `ort` and `ort-sys` 2.0.0-rc.15, linked the native test executable, and passed all 53 Windows-applicable library tests in 0.44 seconds.
- This resolves the earlier Linux-container ABI link blocker for the approved Windows target. It does not yet prove model loading, real inference, output quality, runtime binary hashes, or installer packaging.
- The run reported two unused test-import warnings because Unix-only process-worker tests are excluded on Windows. The imports are now guarded by `cfg(unix)`.

## 2026-11-05 — Development ONNX artifact published

- After explicit D-013 approval, created GitHub prerelease `model-spleeter-2stems-onnx-dev-v1`; this is a model-only development artifact, not an application release or installer.
- Uploaded only `spleeter-2stems-repro.onnx`. GitHub reports state `uploaded` and exactly 78,652,254 bytes. A fresh streamed download, without retaining another copy, produced SHA-256 `8d58651b5979075c954bcbe96c6776b2dc41e7852fb6f2d054b83a696a8db26f`, matching the pinned local artifact.
- Provenance, conversion versions, tensor contract, non-byte-determinism warning, and MIT license evidence are recorded in `docs/model-artifacts/spleeter-2stems-onnx-dev-v1.md` and the prerelease body.
- Added an ignored, feature-gated Windows probe that requires an explicit `HIBIKI_SPLEETER_ONNX_MODEL` path, verifies the published SHA-256 before loading, and runs one generated zero-spectrogram inference. Normal tests never download or load the model.

## 2026-11-05 — Native Windows model inference verified

- The user explicitly downloaded the development prerelease model into the ignored local `models/` directory. PowerShell SHA-256 was `8D58651B5979075C954BCBE96C6776B2DC41E7852FB6F2D054B83A696A8DB26F`, exactly matching the published manifest.
- The ignored native probe compiled on Windows, loaded the verified ONNX file, ran one generated `[1, 512, 1024, 2]` zero-spectrogram input through CPU ONNX Runtime, validated both output shapes and finite values, and passed in 1.35 seconds.
- This verifies Windows model loading and direct neural-subgraph inference. It does not yet verify waveform transforms plus cache output on Windows; a separate ignored fixture-to-cache test is added for that next step.

## 2026-11-05 — Native Windows fixture-to-cache separation verified

- The user ran the ignored integration probe with the verified local model. Native Windows completed read-only WAV decoding, CPU ONNX inference, Rust signal transforms, two 16-bit FLAC encodes, validation, and identity-derived cache commit in 1.41 seconds.
- The probe passed and confirmed the repository WAV fixture SHA-256 was unchanged. Both FLAC stems and the commit manifest existed only in the disposable cache directory.
- Added a separate ignored full-source probe requiring explicit model, source, and cache environment variables. It preserves output under the supplied cache root for listening, prints elapsed time and artifact hashes, and rechecks source SHA-256 after completion.

## 2026-11-05 — Native Windows representative-source separation completed

- The user ran the explicit-source native probe against a selected file under the ignored `sample music/` directory. The complete CPU separation path finished in 128.96 seconds; total test time was 132.29 seconds.
- The test passed, which includes exact source SHA-256 equality before and after processing. Derived output was written only under the identity-derived `cache/native-separation/` path.
- The committed vocals FLAC SHA-256 is `b5022fbb9b03804ca8130a4c97cbab4e0ccff244c711fc58bae1b631d3d9abba`; accompaniment is `5d25ee278f1c7dc8e9a0b3b6884ed43a9ca0c7706e278af5d4d2a081378e8348`.
- Native Windows representative-source execution is now verified. Output listening quality and source duration/real-time factor remain to be recorded; peak memory was not measured by this probe.

## 2026-11-05 — Native Windows stem listening accepted

- The user confirmed that both cached FLAC stems played normally and that the separated audio sounded acceptable.
- This completes the representative-source usefulness check for the native Rust/CPU-ONNX Spleeter path. No source modification, external API, GPU provider, or application release was involved.
- Source duration/real-time factor and peak memory were not captured, so the 128.96-second observation is retained without claiming a speed ratio.

## 2026-11-05 — CREPE tiny ONNX feasibility demonstrated

- Reused the existing isolated TensorFlow/tf2onnx research environment with no new downloads. The pinned 1,307,310-byte compressed H5 passed bzip2 validation and expanded to 2,011,144 bytes with SHA-256 `409254d6a537cc432061112b7ec5f6fd283658f108fc2abc24db9c25858b42df`.
- Reconstructed the official 487,096-parameter tiny architecture and converted it to an approximately 1.96 MB opset-13 ONNX model. Eight normalized frames matched TensorFlow at RMSE `4.4096e-8` and maximum absolute difference `6.1095e-7`.
- Two conversions had different bytes/hashes but identical numerical results, so byte-determinism remains unresolved and an adopted artifact must be individually pinned.
- CPU ONNX generated-tone probes estimated 110, 220, and 440 Hz within 1.586, 6.931, and 1.559 cents; each 2-second/201-frame inference took 0.053–0.089 seconds in the Linux container.
- Added reproducible conversion and pitch-probe scripts plus `docs/phase-4-crepe-feasibility-results.md`. No model or generated artifact is committed.
- Opened D-014 as `Waiting`; no CREPE production runtime, converted artifact, publication, or Viterbi strategy was adopted.

## 2026-11-05 — Native CREPE signal and inference boundary implemented

- The user approved D-014. Added Rust-owned stereo downmix and Rubato FFT conversion to 16 kHz mono, centered 1024-sample frames at 160-sample hops, per-frame normalization, local nine-bin weighted pitch decoding, and confidence/timestamp output.
- RED evidence: the framing/decoder tests first failed at their intentional implementation placeholders; the injected inference tests likewise failed before tensor validation/decoding existed; the 44.1 kHz conversion test failed before downmix/resampling was implemented.
- GREEN evidence: four focused signal tests and two injected inference tests passed. The full default Rust library suite passed with 61 tests, and `cargo check --features native-onnx` passed.
- Added a native `ort` session behind `native-onnx` using the converted model's pinned `input` and `classifier` tensor names and `[frames, 1024]` / `[frames, 360]` contracts. Normal tests use an injected fake and never load or download a model.
- Added an ignored real-model probe requiring explicit `HIBIKI_CREPE_ONNX_MODEL` and exact SHA-256 `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`. The Linux Dev Container could not link the rc.15 prebuilt static ONNX Runtime because it requires newer glibc/libstdc++ symbols; this is the already documented container ABI limitation. No real native CREPE inference result is claimed.
- Opened D-015 for the concrete melody-contour cache format. No contour artifact, model publication, Viterbi smoothing, GPU provider, or source-directory write was added.

## 2026-11-05 — Versioned melody contour cache implemented

- The user approved D-015's recommended JSON format. Added a schema-versioned `melody.json` containing the 16 kHz rate, 160-sample hop, model artifact ID and SHA-256, decoder name/version, and ordered time/frequency/confidence points.
- RED evidence: three cache tests failed at the intentional commit placeholder, and the identity-derived pipeline test failed at its separate placeholder. GREEN evidence: all four focused tests passed after implementation.
- The writer rejects invalid model identities, empty/nonfinite/out-of-range/unordered contours, syncs a request-specific temporary file, atomically renames it, validates and preserves an existing compatible cache, and refuses to replace an incompatible one.
- The public pipeline accepts only an application cache root plus validated `AnalysisIdentity`; it derives the final directory before 16 kHz conversion, framing, injected inference, decoding, and cache commit. It receives in-memory vocals and never opens or writes a source file.
- All 65 default Rust library tests and `cargo check --features native-onnx` passed. Added an ignored end-to-end native probe that requires an explicit verified model path; no model is loaded by normal tests.
- Opened D-016 for optional publication of the exact 1,956,735-byte development artifact. No model or release was published.

## 2026-11-05 — Development CREPE ONNX artifact published

- After explicit D-016 approval, created GitHub prerelease `model-crepe-tiny-onnx-dev-v1`. It is a model-only development artifact, not an application release, installer, stable artifact, or automatic-download endpoint.
- Uploaded only `crepe-tiny-research.onnx`. GitHub reported state `uploaded` and exactly 1,956,735 bytes.
- A fresh streamed download produced SHA-256 `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`, exactly matching the pinned local conversion.
- The release and repository metadata record the pinned H5 provenance, TensorFlow/tf2onnx versions, tensor contract, MIT upstream license, numerical comparison, and non-byte-deterministic conversion warning.
- Native Windows CREPE model loading and full generated-vocal contour persistence remain the next user-run verification; no user audio is required for that probe.

## 2026-11-05 — Singing-screen visual direction accepted

- The user accepted `docs/ui-mockups/singing-screen-concept-v5.png` as the implementation reference for the singing screen.
- The approved behavior uses cyan outlined frames for target notes, same-sized yellow sampled blocks for correct performed pitch, and same-sized red blocks on the detected row for incorrect pitch. It does not use a smooth or connected live-pitch line.
- The concept retains large synchronized lyrics, a fixed timing playhead, compact technique feedback, and secondary progress/microphone/key-shift/tempo controls. It shows no live total score or rank.
- This is visual-direction approval only. Phase 6 scoring weights and implementation remain gated and were not started.

## 2026-11-05 — Native Windows CREPE integration verified

- The user explicitly downloaded `crepe-tiny-research.onnx`. PowerShell reported exactly 1,956,735 bytes and SHA-256 `03FB909A5618FA9F19942E20DE749BC8E658A1AC0DF4A778B354C15B6F7A59D4`, matching the published artifact.
- The ignored native integration probe passed on Windows in 0.14 seconds. It loaded the verified model, converted two seconds of generated 44.1 kHz stereo vocals to 16 kHz mono, created 201 normalized frames, ran CPU ONNX inference, decoded pitch/confidence points, and committed validated `melody.json` under a disposable identity-derived cache.
- This verifies the complete native melody path on generated input. It does not establish accuracy or confidence behavior on separated singing voice.
- Added a separate ignored representative-vocal probe. It accepts explicit model, read-only vocal-stem, and cache-root paths; verifies the model and input hashes; decodes without modifying the stem; commits `melody.json` only beneath the identity-derived cache; and prints duration, hash, point count, and output path.

## 2026-11-05 — Windows checks consolidated

- Added `scripts/windows/run-pending-phase4-checks.ps1` so pending native checks can be run as one batch instead of requiring repeated chat commands.
- The script downloads only the already-approved 1.96 MB CREPE development artifact when absent, verifies exact size and SHA-256, runs the generated-input probe, discovers the newest cached `vocals.flac`, and runs the representative read-only probe when available.
- Added `docs/windows-pending-commands.md` as the durable queue for future Windows-only checks. The Dev Container has no PowerShell runtime, so script execution remains Windows-unverified; all 65 default Rust tests and `git diff --check` passed after adding it.

## 2026-11-05 — Whisper base CPU feasibility completed

- The user approved D-017 for development-environment-only transfer and temporary storage. Pinned `whisper.cpp` v1.9.5 commit `d1be6fde11ac6e0407606b4e42fe72d34add8037` and downloaded the already pinned 147,951,465-byte multilingual base model into `/tmp`; its SHA-256 matched `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`.
- Installed CMake 4.2.0 and Ninja 1.13.0 only under the disposable research directory and built a CPU-only CLI with GPU and BLAS providers disabled. Total research storage was approximately 293 MB, below the approved 1 GB budget.
- Evaluated a 30.93-second public-domain native-Japanese pronunciation exercise. CPU inference finished in 1.483 seconds with sampled peak RSS 296,304 KiB and produced 14 timestamped segments.
- The output recognized much of the word sequence but included material errors such as `天気→電気`, `煎餅/玄米→専米/現買`, and `単位→タイム`. Raw base output is therefore a draft, not authoritative lyrics.
- Recorded input/runtime hashes, licenses, timings, memory, transcript comparison, and the recommended process boundary in `docs/phase-4-whisper-feasibility-results.md`. Opened D-018 as `Waiting`; no runtime, transcript schema, model distribution, or alignment method was adopted.

## 2026-11-05 — Whisper local-process protocol approved and typed

- The user approved D-018's supervised local-process recommendation. Bumped the analysis-worker protocol to version 2 and added a typed Japanese transcription operation carrying input/model paths, expected source/model SHA-256 values, fixed `ja` language, request identity, and an identity-derived cache output directory.
- Added typed response segments with millisecond bounds and Rust validation for language, duration, monotonic nonoverlapping timestamps, range bounds, and nonempty text. Successful responses without transcription metadata are rejected.
- The existing timeout, bounded stdout/stderr, no-shell process launch, response-ID matching, and cache-path derivation remain in force. Research workers were updated to protocol v2 without adding a production Whisper executable.
- Six focused Rust worker tests passed, including the new cache-owned Japanese request contract. Two existing Python separation-worker tests passed from their intended `workers/` directory. Transcript persistence, alignment, confidence policy, executable packaging, and model acquisition remain unselected.

## 2026-11-05 — Consolidated Windows Phase 4 checks passed

- The user ran `scripts/windows/run-pending-phase4-checks.ps1` from Windows PowerShell. It verified the exact CREPE model SHA-256 and the generated-vocal native integration probe passed in 0.19 seconds.
- No cached `vocals.flac` was present, so the script safely skipped the optional representative-vocal probe and reported all currently runnable Phase 4 Windows checks passed.
- No additional Windows command is pending from this batch. Representative-vocal contour quality remains unverified until a cache contains a suitable vocal stem.

## 2026-11-05 — Versioned transcript draft cache implemented

- The user approved D-019. Added atomic, schema-versioned `transcript.json` persistence beneath the existing content- and pipeline-identity cache directory.
- Each artifact is explicitly marked `rawDraft` and records fixed Japanese language, duration, Whisper model artifact ID and SHA-256, worker protocol version, and ordered millisecond transcript segments.
- Validation rejects unsafe identities, invalid hashes, zero durations, empty transcripts, blank text, overlaps, reversed intervals, and out-of-duration segments. A compatible existing artifact is preserved rather than replaced.
- An injected pipeline test verifies worker output commits only below the derived cache and leaves the input audio bytes unchanged. Four focused transcript tests passed; no production Whisper executable, alignment, correction UI, confidence policy, or download path was added.

## 2026-11-05 — Whisper distribution boundary approved and implemented

- The user approved D-020: the eventual Windows application will bundle a pinned CPU-only `whisper.cpp` executable and notices, while the 147,951,465-byte multilingual base model requires an explicit user action and exact size/SHA-256 verification before model-cache commit.
- Added a network-independent Rust boundary that validates a staged model against its manifest, copies it to a temporary file below the separate model cache, revalidates it, syncs it, and atomically commits it. Existing valid models are preserved and invalid content leaves the model cache empty.
- Added a development-only Python protocol adapter that receives an explicit `whisper.cpp` CLI path, verifies input and model hashes, invokes fixed Japanese JSON-output arguments without a shell, and converts timestamped JSON into protocol-v2 segments. The Rust process wrapper now accepts explicitly injected arguments for this adapter.
- Three focused Rust model-cache tests and two Python parser tests passed. No Whisper binary or model was downloaded, bundled, published, or connected to the application UI.
- Reused the existing disposable research CLI, pinned model, and public-domain Japanese fixture to exercise the adapter without a download. The complete adapter invocation succeeded in 1.8 seconds and returned 14 validated protocol-v2 segments; all writes stayed under a new `/tmp` output directory.

## 2026-11-05 — Japanese lyric alignment foundation implemented

- The user approved D-021's lightweight Pure Rust direction. Added deterministic character-level edit-distance alignment between authoritative lyrics and timestamped Whisper draft segments without another model or dependency.
- Original lyric bytes and display characters are retained. Whitespace and Japanese/ASCII punctuation are ignored only during comparison, while matched comparison characters receive interpolated segment timing.
- Substitutions, deletions, and other unmatched characters remain explicitly un-timed instead of inheriting a fabricated timestamp. Invalid or unordered transcript segments are rejected.
- Four focused tests passed for exact Japanese timing, a `天気→電気` recognition error, original punctuation/spacing preservation, and invalid inputs. No alignment cache, confidence threshold, correction UI, or scoring connection was selected.

## 2026-11-05 — Versioned aligned-lyrics cache implemented

- The user approved D-022. Added atomic `aligned-lyrics.json` persistence below the content/pipeline identity cache with schema and alignment-algorithm versions.
- The manifest records SHA-256 of the exact authoritative UTF-8 lyric text, preserves the original display text, and stores each character's UTF-8 byte bounds, match status, and optional timing.
- Validation requires complete contiguous reconstruction of the original text, valid UTF-8 boundaries, ordered positive timing for matched characters, and no fabricated timing for ignored/unmatched characters. Existing compatible artifacts are preserved.
- Three focused cache tests and one identity-derived pipeline test passed. Confidence thresholds, correction UI, lyric-provider behavior, scoring, and live display remain outside this decision.

## 2026-11-05 — Resumable Phase 4 orchestration foundation added

- Added an injected coordinator that drives separation, melody, transcription, and alignment in order while appending the existing atomic typed job snapshots.
- A fresh job starts queued; interrupted/running or retryable failed jobs resume at their recorded stage; completed jobs return without repeating work. Stage failures retain an actionable message and retryable stage identity.
- Cancellation is checked between stages and passed into each long-running stage for cooperative interruption. Cancelled and non-retryable terminal jobs cannot be resumed silently.
- Four focused tests passed for ordered completion, retry from transcription, cancellation between stages, and cancellation returned by a running stage. No Tauri command, UI, model download, or source-directory write was added.

## 2026-11-05 — Local authoritative lyric input implemented

- The user approved D-023. Added a typed local-only boundary for pasted/edited lyrics and read-only UTF-8 `.txt` imports; no online lyric service is contacted.
- The boundary preserves exact display text and hashes its UTF-8 bytes, removes only an optional file BOM, rejects empty/NUL/non-UTF-8/non-`.txt`/over-one-MiB input, and never writes to the imported file.
- Three focused tests passed for exact pasted-text identity, BOM-aware read-only import with byte-for-byte source preservation, and invalid/unsafe inputs. Native picker/UI wiring remains separate.

## 2026-11-05 — Path-free lyric input connected to the song screen

- Added Rust-managed per-song lyric state plus Tauri commands for pasted text and native `.txt` selection. Song identities must be lowercase content SHA-256 values.
- Native file paths and lyric bodies remain in Rust. React receives only selection status, exact-text SHA-256, and byte size; cancellation does not change stored state.
- The selected-song card now supports paste/edit submission and native `.txt` selection with local-only success/error states. Indexed song IDs are no longer decorated in React, preserving their catalog content identity.
- Three focused Rust command tests, frontend lint, and production build passed. Two frontend interaction tests were added, but the container Vitest process again stalled immediately after startup even with one fork worker, so they are not reported as passed.

## 2026-11-06 — Windows whisper.cpp development workflow added

- The user approved D-024. Added a manual `windows-2022` workflow that fetches only pinned `whisper.cpp` commit `d1be6fde11ac6e0407606b4e42fe72d34add8037`, configures an x64 CPU-only static Release build, and builds `whisper-cli`.
- The seven-day development artifact is configured to contain only `whisper-cli.exe`, the upstream MIT license, and JSON provenance with revision, build options, toolchain/generator, byte size, and SHA-256. It does not download the Whisper model or publish a release.
- The workflow file was accepted by GitHub on push. It has not been dispatched: this development environment has neither the GitHub CLI nor a GitHub API token, so it cannot start a manual workflow without new credentials/tool installation or a user click.
- The user dispatched run `42135598802`. All build and upload steps succeeded in 1 minute 28 seconds. GitHub reported one 901,923-byte artifact with archive SHA-256 `548da9d9a17b52211ebeb7fc79eba5962fb6cf564b1c54ef8bcadafbca3de992`, expiring 2026-11-13.
- GitHub requires authentication to download even a public repository's Actions artifact and detailed step log, while this environment has no API token. Updated the workflow to duplicate provenance into the public Job Summary so a second run can be inspected without downloading or publishing the artifact.

## 2026-11-06 — Windows whisper.cpp executable provenance verified

- The user manually dispatched the updated workflow as run `42136432004` from commit `9a92ec6ba94c32c39dff5630bd86c7af82f27d64`. The Windows build and seven-day artifact upload completed successfully.
- The Job Summary reports `whisper-cli.exe` at exactly 2,115,584 bytes with SHA-256 `e8c1a1390c22ec4bb48c5a32ccd65942303c8633b09d6f897e1a1562d19ba7f2`.
- Provenance matches the approved boundary: upstream `ggml-org/whisper.cpp` tag `v1.9.5` at commit `d1be6fde11ac6e0407606b4e42fe72d34add8037`; `windows-2022`; x86_64 Release; Visual Studio 17 2022; CMake 3.31.6; static libraries; no GPU providers; MIT license.
- GitHub reported artifact ID `13466262062`, archive SHA-256 `976d2a0644da6669856b4804501bfe85e3ba27e8e4af6d32018bbcf8339e8aae`, and a seven-day development retention window. The archive digest is distinct from the executable digest above.
- This verifies reproducible development-build provenance only. The executable was not published, bundled into the application, or adopted as a release artifact, and no model was downloaded by the workflow.

## 2026-11-06 — Windows whisper.cpp development publication approved

- The user approved D-025 to publish only the already verified CPU-only executable, upstream MIT license, and provenance as a clearly labeled development prerelease.
- Added a manual publication workflow that downloads artifact `whisper-cpp-v1.9.5-windows-x64-cpu-dev` from source run `42136432004` rather than rebuilding it. Before publication it requires the approved 2,115,584-byte size and SHA-256 `e8c1a1390c22ec4bb48c5a32ccd65942303c8633b09d6f897e1a1562d19ba7f2`, and validates the pinned CPU-only provenance.
- The workflow will create prerelease tag `whisper-cpp-v1.9.5-windows-x64-cpu-dev-v1`. It does not publish an application, bundle the executable, download a model, or change the GPU strategy.
- Initial publication run `42138628748` successfully downloaded and revalidated the exact artifact, then stopped before creating a release because GitHub CLI could not infer a repository without a checkout. The workflow now passes the explicit `GITHUB_REPOSITORY` value; no partial release or tag was created.
- Corrected publication run `42142219937` succeeded from commit `da8bb2d`. GitHub prerelease `whisper-cpp-v1.9.5-windows-x64-cpu-dev-v1` contains exactly `whisper-cli.exe`, `LICENSE-whisper.cpp.txt`, and `provenance.json`.
- GitHub reports the executable as 2,115,584 bytes with asset digest SHA-256 `e8c1a1390c22ec4bb48c5a32ccd65942303c8633b09d6f897e1a1562d19ba7f2`, matching the approved build. The release remains development-only; it is not bundled into the application or an installer.
- PowerShell interpreted a Markdown backtick before the leading `e` of the hash as an escape character in the human-readable release body. The uploaded executable and provenance are correct; the workflow source now uses a literal here-string and plain text so future release notes cannot acquire that control character.

## 2026-11-06 — Read-only analysis source resolution added

- Added a catalog lookup by content SHA-256 and a Rust-only library boundary that resolves an indexed song to its source path for analysis without returning that path to React.
- Before analysis use, the boundary verifies that the source is still a file and recomputes its SHA-256 read-only. Missing or externally changed sources fail with an actionable re-index message instead of analyzing stale catalog identity.
- Three focused tests passed for catalog identity lookup, unchanged source resolution with byte-for-byte preservation, and rejection after external mutation. No analysis process, model acquisition, or source-directory write was started by this step.

## 2026-11-06 — Validated analysis-stage cache handoff added

- Added readers that revalidate the stem manifest and hashes before decoding cached `vocals.flac` for melody extraction, and revalidate transcript identity, protocol, language, duration, and segments before lyric alignment.
- Two focused tests passed for the separation-to-melody and transcription-to-alignment handoffs. Downstream stages consume only cache-owned artifacts; no source path is exposed to React.

## 2026-11-06 — Concrete four-stage local analysis runner added

- Added a typed runner that executes separation, reloads validated cached vocals for melody extraction, transcribes the cache-owned vocal stem, reloads the validated transcript, and aligns authoritative lyrics.
- A full injected integration test completed all four stages and produced stems, `melody.json`, `transcript.json`, and `aligned-lyrics.json` beneath one identity-derived cache while preserving the source bytes exactly.
- The runner uses injected inference/process backends so normal tests do not load models. Native runtime construction, application state, and Tauri/UI commands remain the next layer.

## 2026-11-06 — Path-free analysis command state added

- Added a Rust-owned per-song job registry with typed state, duplicate-active-job rejection, cooperative atomic cancellation, and safe retry replacement after terminal state.
- Registered path-free Tauri commands for status lookup and cancellation. Two focused tests passed; no source/model/cache path is serialized to React.
- The start command remains intentionally unregistered until native model/runtime preflight is connected, so the UI cannot report fake analysis progress.

## 2026-11-06 — Cache-owned Whisper PCM input added

- The concrete runner now converts the validated cached vocal stem to an atomic 16-bit stereo PCM WAV below the identity cache before invoking transcription, avoiding an unsupported assumption that the CPU CLI accepts FLAC directly.
- A round-trip decode test and the full four-stage injected integration test passed. Temporary output is removed by atomic rename and no file is created beside the original music.

## 2026-11-06 — Direct Rust whisper.cpp backend added

- Added a production-oriented Rust backend that verifies the pinned CLI byte size and SHA-256, verifies the exact cache-owned WAV and model hashes, launches `whisper-cli` without a shell using fixed Japanese JSON arguments, enforces a timeout, and parses timestamped output into the typed protocol response.
- Corrected transcription request identity to hash the actual cache-owned WAV rather than incorrectly reusing the original song hash. The original content identity remains part of the cache path.
- Two backend tests, the typed request-contract test, the four-stage integration test, and default `cargo check` passed. The development Python adapter remains research-only and is no longer required by this concrete runtime path.

## 2026-11-06 — Native analysis artifact preflight added

- Added one development configuration boundary for the exact published Spleeter ONNX, CREPE ONNX, Whisper base model, and Windows `whisper-cli` artifacts.
- Every file must be explicitly configured and match its pinned byte size and SHA-256 before analysis can start. A focused test covers exact acceptance plus size/hash rejection, and default `cargo check` passed.
- This is a local development preflight only; it does not bundle or automatically download any artifact.

## 2026-11-06 — Background native analysis start command added

- Registered a typed `start_analysis` Tauri command that resolves and rehashes the source internally, requires authoritative local lyrics, verifies all four native artifacts off the UI thread, rejects duplicate active work, and starts the concrete runner on a blocking task.
- Each stage updates the Rust-owned path-free job registry before work begins. Cooperative cancellation is passed into the existing pipeline, while terminal results and actionable failures are retained for status/retry handling.
- Builds without `native-onnx` return an explicit unsupported-build error rather than simulated progress. Default and `native-onnx` cargo checks passed, along with the command-state tests.

## 2026-11-06 — Selected-song analysis controls connected

- Added a typed React/Tauri analysis client for queued, four running stages, completed, retryable/non-retryable failure, and cancelled states; no filesystem path crosses this boundary.
- The selected-song card now requires accepted authoritative lyrics before starting, polls every active song, shows stage-specific Japanese progress, updates the library row and real queue count, supports cooperative cancellation, and exposes retry only for retryable failures.
- The completed state enables singing. Cancelled work remains visibly terminal rather than claiming it can resume through the existing terminal cache snapshot.
- All 15 song-library UI tests passed, including new start/stage, cancel, and retry cases. Frontend lint, production build, default cargo check, native-ONNX cargo check, and `git diff --check` also passed.

## 2026-11-06 — One-command Windows native development launch added

- Added `scripts/windows/start-phase4-native-dev.ps1` to acquire only missing approved development artifacts, verify all four exact byte sizes and SHA-256 values, set process-local paths, and start Tauri with the `native-onnx` feature.
- A clean machine downloads approximately 220 MiB into temporary names below the Git-ignored `models` directory. Existing invalid files cause a hard failure and are never overwritten; `-SkipDownload` and `-VerifyOnly` provide stricter modes.
- HTTP redirect checks returned 200 for the published Spleeter ONNX, CREPE ONNX, pinned Whisper base model, and CPU-only Windows `whisper-cli` URLs. No artifact body was downloaded during this container check.
- The script does not bundle models, select a GPU provider, publish an application, or write beside source music. Actual PowerShell execution and the first UI-driven four-stage run remain Windows-only verification.

