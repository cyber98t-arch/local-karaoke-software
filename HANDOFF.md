# Current handoff

- Updated: 2026-11-05
- Current phase: Phase 3 complete; Phase 4 in progress. Native CPU ONNX Spleeter and CREPE paths are established. Whisper base feasibility and the D-018 typed local-process boundary are complete.
- Current implementation commit: `a2cdeae` (`feat: define Whisper worker protocol`).
- Recent commits: `6ab9670` records the bounded Whisper feasibility result; `a2cdeae` adds the typed protocol-v2 local-process boundary and response validation.
- Repository state: CREPE has Rust-owned 16 kHz conversion, centered normalized frames, native/injected ONNX inference, local-average decoding, and identity-derived cache-only persistence. The exact development artifact is available from GitHub prerelease `model-crepe-tiny-onnx-dev-v1`; no automatic download path uses it.
- Approved UI reference: `docs/ui-mockups/singing-screen-concept-v5.png`. Target pitch uses cyan outlined frames; correct sampled pitch uses same-sized yellow blocks; incorrect sampled pitch uses same-sized red blocks on its detected row. No smooth live-pitch line or live total score.
- Published artifact: `crepe-tiny-research.onnx`, 1,956,735 bytes, SHA-256 `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`. GitHub reported `uploaded`, and a fresh streamed download matched the pinned hash.
- Verified: All 66 default Rust library tests, frontend lint, production build, default cargo check, and `git diff --check` pass. The latest container frontend test run stalled after Vitest startup, so the prior 10-test pass remains the latest completed frontend-test evidence. Windows loaded the exact CREPE model and completed generated-audio conversion, 201-frame CPU inference, decoding, and identity-derived `melody.json` persistence in 0.14 seconds.
- Unverified: Contour quality/performance on representative separated singing; Windows runtime hashes/notices; production Whisper worker execution; GPU providers; installer; and VMware guest.
- Whisper feasibility: Pinned multilingual base processed 30.93 seconds of public-domain Japanese speech in 1.483 seconds with about 289 MiB sampled peak RSS. It produced useful timestamps but several material word errors; see `docs/phase-4-whisper-feasibility-results.md`.
- Whisper worker boundary: Protocol v2 now defines a Japanese transcription request with verified source/model identities and identity-derived output, plus validated timestamped response segments. Focused Rust and Python tests pass; no production executable is connected.
- Next safe action: Obtain D-019 approval before implementing versioned, cache-only `transcript.json` persistence. Do not implement the new cache format while its status is `Waiting`.
- Windows command queue: The user ran `scripts/windows/run-pending-phase4-checks.ps1`; exact-model verification and the generated-vocal probe passed, while the optional representative-vocal probe was safely skipped because no cached `vocals.flac` existed. Future Windows-only commands should be appended to `docs/windows-pending-commands.md` rather than sent piecemeal in chat.
- Blockers and decisions: D-019 transcript cache format is waiting; alignment remains a later major gate. Confidence thresholds, Viterbi smoothing, stable model/application publishing, automatic downloads, GPU strategy, and release remain unapproved. The user's untracked `sample music/`, `screenshot/`, and attached `docs/ui-mockups/seimitsusaitenai-09.png` must never be staged.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.
