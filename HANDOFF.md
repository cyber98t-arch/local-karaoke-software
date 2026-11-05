# Current handoff

- Updated: 2026-11-05
- Current phase: Phase 3 complete; Phase 4 in progress. CPU ONNX Spleeter separation is accepted on representative Windows audio, and D-014 approves native CPU ONNX CREPE tiny with Rust pre/postprocessing.
- Current implementation commit: `58077e5` (`test: add native CREPE model probe`).
- Recent commits: `509354f` adds CREPE framing/local-average decoding and records D-014 approval; `616ff57` adds the injected/native inference boundary; `4fe7098` adds 16 kHz mono conversion; `58077e5` adds the opt-in verified-model probe.
- Repository state: The read-only library/catalog, Windows audio probes, and native two-stem separation path remain intact. CREPE now has Rust-owned stereo downmix, Rubato FFT conversion to 16 kHz mono, centered normalized 1024-sample frames at 10 ms hops, local-average pitch/confidence decoding, an injected inference trait, and an optional native `ort` session. No CREPE model is committed or published, and no melody contour is persisted yet.
- Verified: Four focused CREPE signal tests and two injected inference tests pass. The default Rust library suite passes all 61 tests. `cargo check --features native-onnx` and `git diff --check` pass. The earlier Windows Spleeter evidence remains: native model inference, complete cache-only representative-source separation in 128.96 seconds, unchanged source hash, playable FLAC stems, and user-accepted listening quality.
- Partially verified: The ignored CREPE probe checks the exact local research-model hash and native tensor path, but the Linux container cannot link the rc.15 static ONNX Runtime because its binary requires newer glibc/libstdc++ symbols. This is an environment ABI limitation, not a successful native CREPE inference run. Windows CREPE execution remains unverified.
- Unverified: CREPE native Windows inference/quality/performance; melody cache persistence; representative-source melody usefulness; source duration/real-time factor and peak memory; Windows runtime hashes and final notices; byte-deterministic model conversion; Whisper; GPU providers; installer; and VMware guest.
- Next safe action: Await D-015. If approved, implement a validated, atomic, identity-derived `melody.json` cache writer with failing tests first. Model publication and a Windows native probe remain separately gated.
- Blockers and decisions: D-015 melody-contour cache format is `Waiting`. CREPE artifact publication, GPU strategy, stable publishing, and application release are not approved. The working tree contains the user's untracked `sample music/` and `screenshot/` directories; never stage them.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.
