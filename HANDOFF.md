# Current handoff

- Updated: 2026-11-05
- Current phase: Phase 3 complete; Phase 4 in progress. Native CPU ONNX Spleeter separation is accepted on representative Windows audio. D-014 and D-015 approve and implement the native CREPE boundary plus versioned melody contour cache.
- Current implementation commit: `ccf2a2e` (`test: add native melody pipeline probe`).
- Recent commits: `a22c1e1` adds the validated identity-derived `melody.json` cache and injected end-to-end melody pipeline; `ccf2a2e` adds the opt-in native model integration probe.
- Repository state: CREPE now has Rust-owned 16 kHz mono conversion, centered normalized frames, injected/native ONNX inference, local-average decoding, and cache-only persistence. `melody.json` records schema/rate/hop/model provenance/decoder metadata and ordered time/frequency/confidence points. Existing valid contours are preserved and invalid contours commit nothing.
- Verified: Four focused cache/pipeline tests pass. All 65 default Rust library tests pass, as does `cargo check --features native-onnx` and `git diff --check`. The pipeline test confirms output only below the `AnalysisIdentity`-derived application cache path. Earlier Windows Spleeter native inference, representative separation, unchanged source hash, playable stems, and accepted listening quality remain valid.
- Partially verified: The ignored native CREPE integration probe is implemented and pins the local research artifact SHA-256, but the Linux container cannot link the rc.15 static ONNX Runtime due its newer glibc/libstdc++ ABI. No native CREPE result is claimed.
- Unverified: Native Windows CREPE inference, contour contents/quality/performance on singing, representative-source melody usefulness, Windows runtime hashes/notices, Whisper, GPU providers, installer, and VMware guest.
- Next safe action: Await D-016. If publication is approved, publish only the exact development CREPE artifact with provenance/license notes, verify a fresh streamed download hash, then provide a Windows probe command. If rejected, use the documented manual-copy alternative.
- Blockers and decisions: D-016 development CREPE artifact publication is `Waiting`. Stable model/application publishing, automatic downloads, GPU strategy, and release remain unapproved. The user's untracked `sample music/` and `screenshot/` directories must never be staged.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.
