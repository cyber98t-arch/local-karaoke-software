# Product requirements

## Product goal

Provide a polished, offline karaoke experience on Windows 11, including ordinary laptops with integrated graphics. Use local computation for analysis and scoring; online access is allowed only for explicitly approved model downloads.

## Functional requirements

1. Select music through a polished library UI inspired by the clarity and density of modern desktop music players, without copying proprietary assets.
2. Import MP3, FLAC, and WAV by referencing a user-selected source folder.
3. Treat every original source as immutable and read-only.
4. Generate an accompaniment with local vocal separation.
5. Extract the original vocal melody/pitch contour locally.
6. Obtain, generate, or import Japanese lyrics, align them automatically, and allow manual timing/text correction.
7. Finish and cache separation, pitch, waveform, lyric, and metadata processing before playback.
8. Capture a microphone with low latency and perform real-time CPU-capable pitch scoring.
9. Use Intel Arc, AMD Radeon, or NVIDIA GPU acceleration for pre-playback inference when available, with a CPU fallback.
10. Show pitch, timing, lyrics, and feedback while singing, but never the total score.
11. Reveal the total score and breakdown only after singing ends.
12. Show logs, stage progress, cache state, errors, and model-download progress in understandable language.
13. Support CPU-only installation and smoke tests in a VMware Windows 11 Home guest.

## Non-functional requirements

- No external AI API and no upload of music, microphone audio, lyrics, or scores.
- Graceful operation without a discrete GPU.
- A failed analysis must be resumable without corrupting a valid cache.
- Cache identity includes source content hash, pipeline version, and relevant model versions.
- User-facing errors must name the failed stage and a safe recovery action.

## Acceptance boundary for the skeleton

- Repository opens in a VS Code Dev Container.
- Codex CLI is installed in the container and its home persists across rebuilds.
- Song-selection UI starts, searches, and selects mock songs.
- UI communicates local-only processing, analysis state, and post-performance score policy.
- Frontend tests and production build pass.
- Windows-only validation has an explicit test plan.

