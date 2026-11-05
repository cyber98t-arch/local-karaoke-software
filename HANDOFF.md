# Current handoff

- Updated: 2026-11-05
- Current phase: Phase 3 complete; Phase 4 in progress. Native CPU ONNX Spleeter separation is accepted on representative Windows audio. D-014 through D-016 approve the native CREPE boundary, versioned contour cache, and one development-only model publication.
- Current implementation commit: `253fafb` (`test: add representative melody probe`).
- Recent commits: `a22c1e1` implements validated identity-derived `melody.json`; `ccf2a2e` adds the generated-input native integration probe; `253fafb` adds an explicit read-only representative-vocal probe.
- Repository state: CREPE has Rust-owned 16 kHz conversion, centered normalized frames, native/injected ONNX inference, local-average decoding, and identity-derived cache-only persistence. The exact development artifact is available from GitHub prerelease `model-crepe-tiny-onnx-dev-v1`; no automatic download path uses it.
- Approved UI reference: `docs/ui-mockups/singing-screen-concept-v5.png`. Target pitch uses cyan outlined frames; correct sampled pitch uses same-sized yellow blocks; incorrect sampled pitch uses same-sized red blocks on its detected row. No smooth live-pitch line or live total score.
- Published artifact: `crepe-tiny-research.onnx`, 1,956,735 bytes, SHA-256 `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`. GitHub reported `uploaded`, and a fresh streamed download matched the pinned hash.
- Verified: All 65 default Rust library tests, `cargo check --features native-onnx`, frontend lint, 10 frontend tests, production build, default cargo check, and `git diff --check` pass. Windows loaded the exact CREPE model and completed generated-audio conversion, 201-frame CPU inference, decoding, and identity-derived `melody.json` persistence in 0.14 seconds.
- Unverified: Contour quality/performance on representative separated singing; Windows runtime hashes/notices; Whisper; GPU providers; installer; and VMware guest.
- Next safe action: On Windows, point `HIBIKI_MELODY_SOURCE` to the previously cached `vocals.flac`, set a separate `HIBIKI_MELODY_CACHE`, and run the ignored `melody_pipeline::tests::extracts_explicit_vocal_source_with_verified_native_model` probe. Review the resulting contour statistics before considering confidence thresholds or smoothing.
- Windows command queue: Run `scripts/windows/run-pending-phase4-checks.ps1` once later; `docs/windows-pending-commands.md` lists its checks. Future Windows-only commands should be appended there rather than sent piecemeal in chat.
- Blockers and decisions: Representative Windows melody evidence is deferred to the consolidated script and does not block container-side work. Confidence thresholds, Viterbi smoothing, stable model/application publishing, automatic downloads, GPU strategy, and release remain unapproved. The user's untracked `sample music/`, `screenshot/`, and attached `docs/ui-mockups/seimitsusaitenai-09.png` must never be staged.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.
