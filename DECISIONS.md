# Decision log

## D-001 — Desktop application foundation

- Status: Approved
- Decision: Tauri 2 + React/TypeScript + Rust, with Python available for early local model work.
- Source: User approval on 2026-10-27.
- Reason: lightweight UI, native performance boundary, and a path to local inference on ordinary Windows laptops.

## D-002 — Repository location

- Status: Approved
- Decision: Create the development repository on the Windows desktop.
- Source: User instruction on 2026-10-27.

## Pending decisions

### D-003 — Phase 2 catalog and metadata storage

- Status: Approved
- Decision: Use SQLite for searchable library/catalog records and job/cache state; keep versioned JSON manifests beside derived cache artifacts. Open source MP3, FLAC, and WAV files read-only, and never write tags or sidecar files into the music directory.
- Source: User approval on 2026-10-27.
- Alternative: Use only versioned JSON files under the application cache. This avoids a database dependency but makes indexed search, migrations, concurrency, and recovery more complex as the library grows.
- Implementation note: Select the smallest suitable Rust metadata-reading crate and SQLite binding after license and dependency review.
- Reason approval is required: Phase 2 changes the database and cache metadata strategy, which is a human approval gate.

### D-004 — UI reference direction

- Status: Approved
- Decision: Reduce generic AI-dashboard styling. Use Nightingale as the functional reference for local-library song selection. Use the DAM main karaoke display only for the singing and post-performance score-reveal experience; do not use the DAM Denmoku/handset UI as the song-selection reference. Keep the result visually original and do not copy proprietary assets or exact trade dress.
- Source: User direction on 2026-10-27.
- Scope: Preserve the approved React/Tauri framework and the existing privacy, read-only source, analysis, playback, and scoring boundaries.
- Detail: See `docs/ui-direction.md`.

### D-005 — Phase 3 Windows audio engine

- Status: Approved
- Decision: Use CPAL directly with its default Windows WASAPI backend as the Phase 3 audio I/O foundation for device enumeration, playback, and microphone capture.
- Source: User approval on 2026-10-29.
- Recommended option: Use CPAL directly for both input and output. It exposes devices and stream configurations, uses WASAPI by default on Windows, and keeps the real-time path under application control without introducing separate playback and capture abstractions.
- Alternative: Use Rodio for playback and CPAL for microphone capture. Playback is simpler, but Rodio currently documents an ongoing core-engine rewrite and would add another abstraction over CPAL.
- Alternative: Bind WASAPI directly through `windows-rs` or a WASAPI-specific crate. This offers maximum Windows control but increases unsafe/platform-specific code and works against the current portable CPU-fallback architecture.
- Initial scope if approved: Shared-mode device enumeration and a playback/microphone probe only. Do not enable ASIO, loopback capture, monitoring, scoring, or production playback in this decision.
- Sources reviewed: RustAudio CPAL and Rodio official repositories and current crate documentation on 2026-10-29.
- Reason approval is required: Phase 3 changes the audio engine/library and is explicitly marked `Decision required` in the roadmap.

### D-006 — Phase 4 local analysis model families and licenses

- Status: Approved
- Decision: Evaluate Spleeter 2-stem for vocals/accompaniment separation, CREPE for monophonic vocal melody, and multilingual Whisper `base` through `whisper.cpp` for Japanese transcription. HTDemucs weights are excluded. Evaluation remains local, cache-only, CPU-capable, and pre-playback.
- Source: User approval on 2026-10-30.
- Research direction approved: On 2026-10-30 the user approved a conservative review limited to candidates with clear model-artifact licenses. HTDemucs weights remain excluded from adoption and download. This approval does not yet select the Phase 4 families.
- Question: Which local model families and license/provenance constraints should Phase 4 adopt for vocal separation, vocal melody extraction, and Japanese lyric transcription/alignment?
- Selected evaluation stack: Spleeter 2-stem for vocal separation, CREPE for monophonic vocal melody, and multilingual Whisper `base` through `whisper.cpp` for Japanese transcription. Keep every model download explicit and separate from song caches; see `docs/phase-4-model-review.md`.
- License review: Whisper and `whisper.cpp` publish MIT licenses. The archived Demucs repository publishes an MIT code license, but that alone does not clearly establish a separately redistributable license for every pretrained weight and training-data derivative. No Demucs weight may be bundled or downloaded until its exact artifact provenance and license are documented and approved.
- Alternatives: Evaluate a separation model with explicitly permissive weight terms; use a non-neural pitch tracker if CREPE latency or packaging is unsuitable; or require user-imported lyrics while postponing transcription.
- Initial scope if approved: Offline prototype jobs only, completed before playback and cached by content hash plus pipeline version. No external AI API, source-file write, live inference, automatic model download, or model redistribution.
- Reason approval is required: Phase 4 changes model families and model licenses and is marked `Decision required` in the roadmap. A model download over 500 MB or restrictive license requires a further explicit approval.

### D-007 — Spleeter inference runtime after TensorFlow feasibility failure

- Status: Approved
- Decision: Adopt the pinned Spleeter 2-stem neural spectrogram subgraph with CPU ONNX Runtime as the production separation foundation. Keep STFT, ratio-mask construction, and inverse STFT in application-owned CPU-capable code. TensorFlow remains a research reference only and is not a production dependency.
- Source: The user approved the lighter option on 2026-10-30 after reporting no audible difference between the TensorFlow and ONNX outputs.
- Scope: This selects the CPU inference runtime. It does not select a GPU provider, approve a release, or waive the requirements for cache-only derived output, read-only source access, artifact hashes, notices, Windows validation, and explicit model download behavior.
- Required evidence: successful CPU separation; TensorFlow-versus-ONNX numerical output difference, listening-quality review, processing time, and peak memory; future Intel Arc/AMD Radeon/NVIDIA provider feasibility; licenses for ONNX Runtime, converted weights, conversion script, and every dependency; SHA-256 records for models and dependencies; and lightweight alternative candidates if conversion is infeasible or materially degrades quality.
- Download gate: Before any large dependency/model download or installation, report the expected size and purpose and obtain explicit user approval. The earlier incomplete TensorFlow resolver probe does not authorize another large download.
- Adoption evidence: On a representative 30-second excerpt, the encoded outputs differed by at most one 16-bit PCM LSB and the user could not hear a difference. The ONNX prototype used less peak memory, although its application-owned Python/NumPy signal processing made that particular run slower overall. The user's priority was the lighter option.
- Remaining gates: GPU strategy is deferred to Phase 5. Packaging, Windows wheel/binary hashes, notices, model acquisition UX, installer contents, publishing, and release still require their normal reviews and approvals.
- Alternative: Continue with the official Python 3.11, Spleeter 2.4.2, and TensorFlow 2.12.1 worker. The Linux TensorFlow wheel alone is 585.9 MB, and an unpinned dependency resolution began repeatedly backtracking through gRPC releases before it was stopped. This conflicts with the ordinary-laptop and lightweight-packaging mission.
- Alternative: Reopen D-006 and select a different separation model with official permissive ONNX weights. This changes the approved model family and needs a separate candidate/license review.
- Evidence: The 2026-10-30 download probe used an isolated `/tmp` destination, did not install anything, and was cancelled during dependency backtracking. The incomplete resolver removed its temporary wheel files; nothing was added to the repository or application cache.
- Reason approval is required: Choosing ONNX Runtime changes the inference runtime/model artifact strategy; continuing TensorFlow would commit the application to a materially heavy dependency closure.

### D-008 — Production ONNX Runtime packaging boundary

- Status: Approved
- Question: Should the approved CPU ONNX separation path run through a native Rust ONNX Runtime binding with Rust signal processing, or through a bundled Python/NumPy/ONNX Runtime worker?
- Recommended option: Native Rust. It best matches the user's lightweight priority, avoids shipping a Python interpreter and NumPy, and keeps the stable pipeline behind the existing Rust orchestration boundary. It requires more implementation and Windows binary/provider packaging work.
- Alternative: Bundled Python worker. It can reuse the numerically verified prototype with less immediate engineering, but retains a Python interpreter, NumPy, and wheel/native-library packaging overhead.
- Not recommended: Require a user-managed Python installation. This reduces installer contents but harms offline reliability and ordinary-user setup.
- Scope held until decision: The dependency-free request validation and cache commit boundary may remain, but do not add a production ONNX binding/runtime package or choose its binary acquisition strategy.
- Reason approval is required: This selects the production inference packaging boundary and materially affects installed size, dependency licensing/notices, Windows deployment, and maintenance.
- Decision: Use a native Rust ONNX Runtime binding and application-owned Rust signal processing. Do not bundle a Python interpreter or NumPy for production separation.
- Source: User approval on 2026-10-30 after the native Rust option was explicitly recommended.
- Initial dependency direction: Evaluate pinned `ort` 2.0.0-rc.15 with CPU-only minimal features and `realfft` 3.5.0. Do not enable a GPU execution provider under this decision. Do not rely on an unrecorded build-time binary download for release artifacts.

### D-009 — Read-only analysis audio decoder

- Status: Approved
- Question: Which native decoder should convert indexed MP3, FLAC, and WAV sources to stereo PCM for pre-playback analysis?
- Recommended option: Pin pure-Rust Symphonia 0.6.1 with only MP3, FLAC, PCM, and WAV/RIFF features. Upstream rates these codecs/formats as Excellent, supports gapless decoding, uses safe Rust with minimal dependencies, and avoids shipping FFmpeg.
- License: MPL-2.0 for Symphonia. The application must retain notices and comply with source-availability obligations for any modified MPL-covered files; application-owned files remain under their own terms.
- Alternative: Bundle an audio-only FFmpeg build. It covers more formats but adds a larger native binary, build-configuration provenance, and LGPL/GPL feature-selection risk.
- Scope: Decode through a Rust-owned read-only file handle into memory/cache-owned PCM. Never write beside, tag, rename, or otherwise modify the source.
- Reason approval is required: This selects the production decoding library and its license boundary within the audio pipeline.
- Decision: Use Symphonia 0.6.1 with only MP3, FLAC, PCM, and WAV/RIFF support for analysis decoding. Preserve the read-only source invariant and include MPL-2.0 notices/compliance materials.
- Source: User approval on 2026-10-30.

### D-010 — Converted ONNX model acquisition

- Status: Approved
- Question: How should the 78.7 MB converted Spleeter ONNX artifact reach the application model cache?
- Recommended option: An explicit, one-time app-managed download from a future approved release asset, pinned by exact byte size, SHA-256, source-checkpoint provenance, conversion-tool versions, and MIT license evidence. Development uses an injected local artifact until publishing is separately approved.
- Alternative: Bundle the model in every installer. This improves first-run offline behavior but makes every installer roughly 79 MB larger and couples model updates to app releases.
- Alternative: Convert locally from the TensorFlow checkpoint. This avoids distributing the converted file but requires the very large TensorFlow conversion environment rejected by the lightweight objective.
- Scope: Store models separately from song caches; never download silently; verify before atomic commit; keep the previous valid model on failure.
- Reason approval is required: This changes model acquisition, application storage, download UX, provenance handling, and eventual release contents.
- Decision: Use an explicit one-time, app-managed download into the separate model cache, with exact size/SHA-256/provenance/license verification and atomic commit. Development may inject a local artifact; publishing the future release asset remains separately gated.
- Source: User approval on 2026-10-30.

### D-011 — Offline analysis sample-rate conversion

- Status: Approved
- Question: How should MP3/FLAC/WAV sources that are not 44.1 kHz be converted to Spleeter's required 44.1 kHz stereo input?
- Recommended option: Pin Rubato 5.0.1 and use its synchronous FFT resampler for fixed-ratio, pre-playback conversion. Upstream specifically recommends `Fft` plus whole-clip `process_all()` for offline file-rate conversion; it is pure Rust, CPU-capable, and licensed `MIT OR Apache-2.0`. The existing pipeline already uses `realfft` 3.5.0, which Rubato's FFT feature also pins.
- Alternative: Implement application-owned windowed-sinc resampling. This reduces third-party API exposure but adds substantial DSP correctness, anti-aliasing, delay-trimming, and test responsibility without a product advantage.
- Rejected shortcut: Linear or polynomial-only interpolation for arbitrary file conversion. It is lighter but can alias during downsampling such as 96 kHz to 44.1 kHz and is unsuitable for the model's analysis input.
- Scope if approved: Resample decoded in-memory stereo only before separation, preserve the original read-only file and native-rate decoded data, record the conversion parameters in analysis identity, and test duration/channel/finite-sample/source-immutability behavior. This does not change playback or microphone resampling.
- Reason approval is required: This selects an audio-processing dependency and conversion strategy that affects model input, analysis reproducibility, performance, and output quality.
- Decision: Use Rubato 5.0.1's synchronous FFT resampler for fixed-ratio, pre-playback conversion to 44.1 kHz. Keep playback and microphone resampling outside this scope.
- Source: User approval on 2026-10-30.

### D-012 — Derived two-stem cache encoding

- Status: Approved
- Question: Which on-disk encoding should store cache-owned vocals and accompaniment stems after separation?
- Recommended option: Stereo 44.1 kHz, signed 16-bit FLAC using pure-Rust `flacenc` 0.5.1 with default features disabled. It is lossless relative to the selected 16-bit cache precision, is already decodable by the approved Symphonia feature set, reduces ordinary-laptop cache usage compared with WAV, and is Apache-2.0 licensed.
- Alternative: Signed 16-bit PCM WAV using Hound 3.5.1 (Apache-2.0). This is simpler and fast, but two one-hour stereo stems consume about 1.27 GB before filesystem overhead.
- Alternative: 32-bit float WAV. It avoids quantizing model output but raises the same two-stem hour to about 2.54 GB and conflicts with the lightweight ordinary-laptop goal.
- Precision note: The representative TensorFlow/ONNX equivalence test already used 16-bit PCM encoding and differed by at most one least-significant bit. Quantization must clamp safely and be deterministic; analysis that needs full precision should consume in-memory floats before cache encoding.
- Scope if approved: Write only request-specific temporary files beneath the application cache, finalize and validate both stems, record sizes/hashes/format parameters in the manifest, then atomically commit without replacing a valid cache. Never write beside the source.
- Reason approval is required: This selects the persistent derived-audio cache format, precision, encoder dependency, storage footprint, and compatibility boundary.
- Decision: Store both derived stems as stereo 44.1 kHz signed 16-bit FLAC using pure-Rust `flacenc` 0.5.1 with default features disabled. Commit only after both temporary artifacts validate.
- Source: User approval on 2026-10-30.

### D-013 — Development ONNX artifact publication

- Status: Approved
- Question: May the verified converted Spleeter ONNX artifact be published as a GitHub prerelease asset so the native Windows checkout can load and test it?
- Recommended option: Publish one clearly labeled development-only prerelease asset, not an application release. Use `spleeter-2stems-repro.onnx`, exactly 78,652,254 bytes with SHA-256 `8d58651b5979075c954bcbe96c6776b2dc41e7852fb6f2d054b83a696a8db26f`, and attach its MIT license/provenance/conversion-version notes. Windows development then downloads it explicitly and verifies size/hash before local injection.
- Alternative: The user manually copies the same 78.7 MB file from the Dev Container path `/tmp/hibiki-onnx-feasibility.OrMRC1/spleeter-2stems-repro.onnx` to the Windows checkout. This avoids publication but is machine-specific and less reproducible.
- Scope if approved: Create a prerelease/development asset only; do not publish an installer, stable release, application update, or automatic-download endpoint. Do not claim byte-deterministic conversion; pin this exact artifact instead.
- Reason approval is required: Uploading the model creates a public release asset and distributes a converted model artifact; publishing is an explicit human approval gate.
- Decision: Publish the exact verified artifact as a clearly labeled GitHub development prerelease with license and provenance notes. This does not approve a stable application release, installer, or automatic download endpoint.
- Source: User approval on 2026-11-05.

### D-014 — CREPE tiny production inference boundary

- Status: Approved
- Question: Should the approved CREPE tiny melody family use a converted ONNX artifact with the existing native Rust CPU ONNX Runtime and application-owned Rust preprocessing/postprocessing?
- Recommended option: Adopt native ONNX plus Rust signal processing. The 487,096-parameter model converts successfully to about 1.96 MB, agrees with TensorFlow within `6.11e-7`, and estimated 110/220/440 Hz generated tones within 6.94 cents. This avoids adding TensorFlow, Python, NumPy, resampy, or hmmlearn to the application.
- Alternative: Bundle the original TensorFlow/Keras runtime. This preserves the upstream implementation directly but reintroduces the dependency size and memory strategy rejected for Spleeter.
- Alternative if singing accuracy is insufficient: Evaluate CREPE small or Basic Pitch under a new artifact/license/performance comparison rather than silently changing capacity or family.
- Scope if approved: Implement and test Rust-owned 16 kHz mono conversion, centered 1024-sample frames at 10 ms hops, normalization, native CPU inference, local-average cents decoding, confidence, and cache-only contour output. Do not publish an artifact, enable GPU providers, or adopt Viterbi smoothing under this decision.
- Evidence: `docs/phase-4-crepe-feasibility-results.md`.
- Reason approval is required: This selects the stable runtime and converted model-artifact strategy for the melody stage and excludes the upstream TensorFlow runtime from production.
- Decision: Use a converted CREPE tiny ONNX artifact through the existing native Rust CPU ONNX Runtime boundary with application-owned Rust preprocessing and local-average decoding. Keep TensorFlow/Python research-only; do not add Viterbi smoothing, GPU providers, or artifact publication under this decision.
- Source: User approval to continue on 2026-11-05 after the feasibility report and recommended boundary were presented.

### D-015 — Melody contour cache format

- Status: Approved
- Question: Which concrete cache format should persist the pre-playback CREPE melody contour?
- Recommended option: Commit one versioned `melody.json` beneath the analysis-identity-derived cache. Store the schema version, 16 kHz input rate, 10 ms hop, model artifact identity and SHA-256, decoder name/version, and ordered points containing time, frequency, and confidence. Write a request-specific temporary file, validate finite values and monotonic timestamps, sync it, then atomically rename it without replacing an existing valid contour.
- Alternative: Store a compact binary array plus a separate JSON manifest. This reduces size and parse time, but adds a second format and migration surface before contour size is demonstrated to be a problem.
- Scope if approved: Derived cache only; never write beside a source. Do not add smoothing, voiced/unvoiced thresholding, lyrics, scoring weights, model publication, or a UI contract under this decision.
- Reason approval is required: This selects a persistent derived-artifact cache format and validation/migration boundary.
- Decision: Store one versioned, validated `melody.json` under the analysis-identity-derived cache, including model provenance and ordered time/frequency/confidence points. Commit it atomically without replacing an existing valid contour.
- Source: User approval on 2026-11-05.

### D-016 — Development CREPE ONNX artifact publication

- Status: Approved
- Question: May the verified converted CREPE tiny ONNX artifact be published as a GitHub development prerelease asset for the native Windows melody probe?
- Recommended option: Publish exactly one clearly labeled development-only asset, `crepe-tiny-research.onnx`, with byte size `1,956,735` and SHA-256 `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`. Include the MIT upstream license, pinned H5 provenance, conversion versions, tensor contract, and non-byte-deterministic conversion warning.
- Alternative: Manually copy the same file from the Dev Container temporary path `/tmp/hibiki-crepe-onnx.9BrlHI/crepe-tiny.onnx` into the ignored Windows `models/` directory. This avoids publication but is machine-specific and less reproducible.
- Scope if approved: A model-only development prerelease, not an application release, installer, stable artifact, automatic-download endpoint, GPU strategy, or approval of representative-source quality.
- Reason approval is required: Uploading the converted model creates a public release asset and is an explicit publishing gate.
- Decision: Publish the exact verified ONNX file as one clearly labeled GitHub development prerelease asset with pinned provenance, hash, size, tensor contract, license, and non-determinism notice.
- Source: User approval to continue on 2026-11-05.

### D-017 — Bounded Whisper base feasibility download

- Status: Approved
- Question: May Phase 4 download and build the pinned CPU-only `whisper.cpp` feasibility environment and download multilingual Whisper `base` for a bounded Japanese transcription probe?
- Recommended option: Allow a research-only disposable probe using the already approved MIT model family. Download exactly the pinned 147,951,465-byte `ggml-base.bin` (SHA-256 `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`), a pinned `whisper.cpp` source revision, and CPU build dependencies only. Budget up to 200 MB network transfer and 1 GB temporary source/build/model storage; keep all binaries and models out of Git and application distribution.
- Initial probe: Use generated/public-domain or repository-safe Japanese speech only, open input read-only, write derived transcripts only to a disposable cache, record elapsed time and peak memory, and assess timestamp/text usefulness. Do not use user music for the first probe.
- Scope exclusion: This does not adopt a production FFI/process boundary, transcript cache schema, alignment strategy, automatic download UX, GPU provider, model publication, or installer contents.
- Reason approval is required: The probe requires a substantial model/dependency download and temporary build footprint under the user's explicit capacity-approval condition.
- Decision: Permit the research-only disposable CPU probe within the stated transfer and temporary-storage budget. All downloaded and built artifacts remain in the development environment only and outside Git/application distribution.
- Source: User approval on 2026-11-05, with explicit confirmation that the capacity applies only to the development environment.

### D-018 — Whisper prototype runtime boundary

- Status: Approved
- Question: Should the Phase 4 Japanese transcription prototype invoke pinned CPU-only `whisper.cpp` through a versioned local process protocol?
- Recommended option: Use a supervised local process boundary. It isolates C++ crashes and peak memory from Tauri, keeps input/output typed and cache-scoped, and avoids committing to a Rust FFI wrapper before Windows packaging evidence. The executable and model remain local-only and pre-playback.
- Alternative: Add a Rust `whisper.cpp` binding. This gives direct calls but couples the application build, C++ ABI, native library lifecycle, and failure containment to a third-party wrapper.
- Evidence: `docs/phase-4-whisper-feasibility-results.md`; multilingual base processed 30.93 seconds in 1.483 seconds with about 289 MiB peak RSS, but produced several material word errors.
- Scope if approved: Define and test an injected typed request/response protocol and a development worker command. Do not yet select transcript cache format, alignment algorithm, confidence threshold, automatic model download, GPU provider, or installer packaging.
- Reason approval is required: This selects the transcription engine integration boundary and native process strategy.
- Decision: Use a supervised, versioned local process protocol for the Whisper transcription prototype. Keep C++ execution and failure outside Tauri and validate all identities, paths, bounds, and response segments in Rust.
- Source: User approval to continue on 2026-11-05 after the feasibility report and recommended boundary were presented.

### D-019 — Whisper transcript cache format

- Status: Approved
- Question: Should Phase 4 persist validated Whisper drafts as a versioned `transcript.json` beneath the identity-derived application cache?
- Recommended option: Store an atomic, validated JSON artifact containing schema version, fixed `ja` language, source duration, model artifact ID and SHA-256, worker protocol version, and ordered `{ startMs, endMs, text }` segments. Mark it as a raw draft so later alignment or user correction cannot be confused with authoritative lyrics.
- Alternative: Keep transcription only in memory until lyric alignment is selected. This avoids one migration boundary, but prevents resumable analysis and needlessly repeats the expensive transcription stage.
- Scope if approved: Add the cache type, validation, atomic commit, preservation of an existing compatible artifact, and injected pipeline tests. Do not select alignment, correction UI, confidence policy, automatic model download, executable packaging, or a GPU provider.
- Reason approval is required: This establishes a new persistent cache format and therefore crosses the repository's explicit cache-format approval gate.
- Decision: Persist validated Japanese transcription drafts as atomic, versioned `transcript.json` artifacts only beneath the identity-derived application cache, using the recommended provenance and raw-draft fields.
- Source: User approval on 2026-11-05.

### D-020 — Production Whisper executable and model acquisition

- Status: Approved
- Question: How should the Windows application obtain the pinned CPU-only `whisper.cpp` executable and multilingual base model?
- Recommended option: Bundle a pinned, CPU-only `whisper.cpp` executable and required notices with the application, while downloading the 147,951,465-byte multilingual base model only after an explicit in-app action. Verify exact byte size and SHA-256 before atomic commit to the separate model cache; never download silently.
- Alternative: Bundle both executable and model in every installer. This improves first-run offline availability but increases every installer by about 148 MB and couples model updates to application releases.
- Rejected alternative: Require users to install or locate `whisper.cpp` themselves. This reduces installer size but conflicts with reliable ordinary-user setup.
- Scope if approved: Implement an injected development adapter and typed acquisition boundary with generated/public-domain tests. Do not publish binaries/models, download the model during tests, create a release, select alignment, or add GPU execution.
- Reason approval is required: This selects native executable packaging, installer contents, and model acquisition behavior.
- Decision: Bundle the pinned CPU-only `whisper.cpp` executable and notices with the application. Acquire the 147,951,465-byte multilingual base model only after explicit user action, verify its pinned size and SHA-256, and commit it atomically to the separate model cache.
- Source: User approval on 2026-11-05.
