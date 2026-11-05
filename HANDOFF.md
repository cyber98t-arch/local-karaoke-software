# Current handoff

- Updated: 2026-11-05
- Current phase: Phase 3 complete; Phase 4 in progress. Native CPU ONNX Spleeter separation is accepted on representative Windows audio. D-014 through D-016 approve the native CREPE boundary, versioned contour cache, and one development-only model publication.
- Current implementation commit: `ccf2a2e` (`test: add native melody pipeline probe`).
- Recent commits: `a22c1e1` implements validated identity-derived `melody.json`; `ccf2a2e` adds the opt-in native integration probe; `6782d76` records D-016 approval and artifact provenance.
- Repository state: CREPE has Rust-owned 16 kHz conversion, centered normalized frames, native/injected ONNX inference, local-average decoding, and identity-derived cache-only persistence. The exact development artifact is available from GitHub prerelease `model-crepe-tiny-onnx-dev-v1`; no automatic download path uses it.
- Published artifact: `crepe-tiny-research.onnx`, 1,956,735 bytes, SHA-256 `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`. GitHub reported `uploaded`, and a fresh streamed download matched the pinned hash.
- Verified: All 65 default Rust library tests, `cargo check --features native-onnx`, frontend lint, 10 frontend tests, production build, default cargo check, and `git diff --check` pass. The injected pipeline writes only under the `AnalysisIdentity`-derived cache.
- Unverified: Native Windows CREPE model loading and generated-vocal `melody.json` persistence; contour quality/performance on representative separated singing; Windows runtime hashes/notices; Whisper; GPU providers; installer; and VMware guest.
- Next safe action: On Windows, explicitly download and hash the development model, set `HIBIKI_CREPE_ONNX_MODEL`, and run the ignored `melody_pipeline::tests::extracts_generated_vocals_with_verified_native_model` probe. This uses generated audio and a disposable cache only.
- Blockers and decisions: The next step requires the user to run the Windows command. Stable model/application publishing, automatic downloads, GPU strategy, and release remain unapproved. The user's untracked `sample music/` and `screenshot/` directories must never be staged.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.
