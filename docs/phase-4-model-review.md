# Phase 4 model-family preflight

Reviewed 2026-10-30. This review does not approve a model family, add a dependency, or download a model.

## Constraints

- All analysis stays local and finishes before playback.
- The original song is opened read-only; generated stems, contours, and transcripts belong only in the application cache.
- CPU execution on an ordinary Windows integrated-GPU laptop must remain possible.
- Model artifacts need an exact source, version, checksum, size, and license record before download.
- A download over 500 MB or a restrictive/unclear license requires another explicit user approval.

## Vocal separation

### Spleeter 2-stem — recommended for a bounded CPU feasibility probe

- The official repository provides a vocals/accompaniment model and an MIT code license.
- The official project paper states that both source code and pretrained models are distributed under MIT. This is clearer artifact evidence than the Demucs repository currently provides.
- The implementation uses TensorFlow and FFmpeg/libsndfile. Its packaging and installed size may be unsuitable for the desktop application even if inference itself is acceptable.
- The standard released models were trained to 11 kHz; alternate inference configurations extend the spectrogram but may introduce artifacts. Separation quality must therefore be measured on representative Japanese songs before adoption.
- No model was downloaded. Exact artifact URL, checksum, compressed size, installed dependency size, and Windows CPU duration remain unverified.

### Rejected or deferred candidates

- HTDemucs: deferred because the archived repository's MIT code license does not clearly establish the terms and provenance of each pretrained weight artifact.
- Open-Unmix `umxl`: rejected for the current permissive-license shortlist because its official documentation marks the weights CC BY-NC-SA 4.0. Other Open-Unmix checkpoints still need artifact-specific terms before consideration.
- Spleeter is not yet selected merely because its terms are clearer; dependency weight and output quality remain acceptance risks.

## Vocal melody

### CREPE — recommended family

- The official project describes CREPE as a monophonic pitch tracker and publishes it under MIT.
- It operates on 16 kHz audio and offers smaller-capacity variants for speed at a possible accuracy cost.
- This matches the separated-vocal contour use case better than a polyphonic audio-to-MIDI model.
- The original package depends on TensorFlow. A converted runtime must not be assumed to carry the same artifact terms without its own review.

### Basic Pitch — fallback only

- The official project is Apache-2.0, supports Windows ONNX inference, and is designed as a lightweight polyphonic transcription model.
- Its note/MIDI-oriented output is useful as a comparison, but it is not the first choice for a continuous monophonic singing contour.

## Japanese transcription

### Whisper via `whisper.cpp` — recommended family

- OpenAI Whisper and `whisper.cpp` publish MIT licenses; the official converted-model repository is marked MIT.
- The multilingual `base` artifact is approximately 142 MiB and is the recommended first CPU feasibility target. It remains below the 500 MB approval threshold.
- Multilingual `small` is approximately 466 MiB and may be evaluated later only if `base` Japanese accuracy is insufficient. It is still below 500 MB, but its runtime cost needs separate measurement.
- Whisper produces transcription timing, not karaoke-quality lyric alignment by itself. Alignment confidence, repeated phrases, instrumental gaps, and user correction remain separate product work.

## Recommended gated sequence

1. Ask for explicit approval of Spleeter 2-stem, CREPE, and multilingual Whisper `base` as evaluation families only.
2. Before any download, pin exact upstream revisions and record artifact URLs, sizes, hashes, and license files.
3. Download only artifacts below 500 MB after approval; do not bundle or redistribute them at this stage.
4. Build cache-only command-line feasibility probes, starting with one generated/public-domain fixture rather than the user's music.
5. Measure Windows CPU duration, peak memory, installed size, and output usefulness before selecting production runtimes.

## Official sources reviewed

- Deezer Spleeter repository, paper, and wiki: <https://github.com/deezer/spleeter>
- Open-Unmix official repository: <https://github.com/sigsep/open-unmix-pytorch>
- CREPE official repository: <https://github.com/marl/crepe>
- Spotify Basic Pitch official repository: <https://github.com/spotify/basic-pitch>
- OpenAI Whisper repository: <https://github.com/openai/whisper>
- `whisper.cpp` repository and official converted models: <https://github.com/ggml-org/whisper.cpp> and <https://huggingface.co/ggerganov/whisper.cpp>
