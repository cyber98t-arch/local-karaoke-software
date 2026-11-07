# Current handoff

- Updated: 2026-11-07
- Current phase: Phase 3 complete; Phase 4 native four-stage execution is verified on Windows. Cancellation responsiveness is implemented and awaits Windows confirmation; corrected-lyrics cache identity needs an explicit decision.
- Current implementation: D-024 and D-025 are complete. Phase 4 now has a hash-revalidating source resolver, concrete four-stage native runner, direct Rust `whisper-cli` backend, pinned artifact preflight, background Tauri start/status/cancel/retry commands, and selected-song controls with path-free typed state.
- Recent commits: `f90f880` connects selected-song analysis controls; `8ca1370` adds the Windows native launcher; `66cc2ae` exposes explicit retry.
- Repository state: CREPE has Rust-owned 16 kHz conversion, centered normalized frames, native/injected ONNX inference, local-average decoding, and identity-derived cache-only persistence. The exact development artifact is available from GitHub prerelease `model-crepe-tiny-onnx-dev-v1`; no automatic download path uses it.
- Approved UI reference: `docs/ui-mockups/singing-screen-concept-v5.png`. Target pitch uses cyan outlined frames; correct sampled pitch uses same-sized yellow blocks; incorrect sampled pitch uses same-sized red blocks on its detected row. No smooth live-pitch line or live total score.
- Published artifact: `crepe-tiny-research.onnx`, 1,956,735 bytes, SHA-256 `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`. GitHub reported `uploaded`, and a fresh streamed download matched the pinned hash.
- Verified: All 91 default Rust library tests and four Python worker tests previously passed. The current selected-song analysis work passed all 15 App UI tests, frontend lint, production build, default cargo check, native-ONNX cargo check, and `git diff --check`. Windows loaded the exact CREPE model and completed generated-audio conversion, 201-frame CPU inference, decoding, and identity-derived `melody.json` persistence in 0.14 seconds.
- Unverified: Responsive Windows cancellation, corrected-lyrics reanalysis identity, contour quality/performance review of the generated melody, GPU providers, installer, and VMware guest.
- Whisper feasibility: Pinned multilingual base processed 30.93 seconds of public-domain Japanese speech in 1.483 seconds with about 289 MiB sampled peak RSS. It produced useful timestamps but several material word errors; see `docs/phase-4-whisper-feasibility-results.md`.
- Whisper worker boundary: Protocol v2 now defines a Japanese transcription request with verified source/model identities and identity-derived output, plus validated timestamped response segments. Focused Rust and Python tests pass; no production executable is connected.
- Transcript cache: D-019 is approved and implemented as atomic, validated, raw-draft `transcript.json` below the identity-derived cache. Four focused tests pass and input audio bytes remain unchanged.
- Whisper distribution: D-020 is approved. A staged model can be verified and atomically committed, and an explicit-CLI development adapter exists. No actual executable/model is bundled and no network acquisition/UI is connected yet.
- Development adapter evidence: The existing disposable `whisper.cpp` CLI, pinned base model, and public-domain Japanese fixture produced 14 protocol-v2 segments in 1.8 seconds after source/model hash verification. Writes stayed under `/tmp`.
- Lyric alignment: D-021 uses deterministic character-level edit distance, preserves original text, and leaves recognition errors un-timed. Four focused alignment tests pass.
- Aligned lyrics cache: D-022 records exact lyric identity, algorithm provenance, original display text, and validated per-character timing/status below the identity-derived cache. Four focused tests pass.
- Analysis orchestration: Existing atomic job snapshots now drive all four stages in order, resume retryable work, and expose cooperative cancellation. Four focused tests pass.
- Authoritative lyrics: D-023 accepts bounded pasted text or read-only UTF-8 `.txt`, hashes exact display text, and makes no lyric-network request. Three focused tests pass.
- Lyric UI boundary: Rust owns native paths and lyric bodies; React receives acceptance metadata only. The analysis UI likewise exchanges song hashes and typed state only.
- Analysis UI: Accepted lyrics unlock background analysis; all active jobs are polled across song selection changes; the four stages, completion, actionable failure, cancellation, and retryable failure are represented without simulated progress.
- Native ORT lifecycle: Spleeter and CREPE share one process-wide CPU `Environment` while keeping separate sessions; the completed Windows run verifies this fix.
- Whisper CLI boundary: Fixed arguments match pinned v1.9.5 and verified paths use Windows long-path canonicalization; the completed Windows run verifies transcription and JSON handoff.
- Windows native E2E: Verified through the UI's enabled `この曲を歌う` state after all four native stages. Whisper cancellation now terminates the child process at 20 ms polling intervals and the UI immediately shows `中止処理中…`; Windows cancellation rerun is pending.
- Windows native launch: The user verified all four exact artifacts, compiled with `native-onnx`, and launched `hibiki-karaoke.exe` successfully using `scripts/windows/start-phase4-native-dev.ps1`.
- VS Code coexistence: Verified on Windows. Native Vite/Tauri use coordinated port 1421 and npm explicitly selects tracked `vite.config.ts`; the user confirmed the app works while VS Code remains open, without rebuilding the container.
- Next safe action: Commit/push responsive cancellation, have the user verify it on Windows, and request a decision before changing cache identity for corrected lyrics. Do not yet bundle artifacts or enable a GPU provider.
- Windows command queue: The user ran `scripts/windows/run-pending-phase4-checks.ps1`; exact-model verification and the generated-vocal probe passed, while the optional representative-vocal probe was safely skipped because no cached `vocals.flac` existed. Future Windows-only commands should be appended to `docs/windows-pending-commands.md` rather than sent piecemeal in chat.
- Blockers and decisions: The Windows Whisper development executable is published and verified, but application bundling and application release remain unapproved. Confidence thresholds, full lyric correction UX, Viterbi smoothing, stable model/application publishing, GPU strategy, and release remain unapproved. The user's untracked `sample music/`, `screenshot/`, and attached `docs/ui-mockups/seimitsusaitenai-09.png` must never be staged.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.
