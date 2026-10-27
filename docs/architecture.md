# Architecture

## Chosen foundation

- **UI:** React + TypeScript, built by Vite.
- **Desktop shell:** Tauri 2.
- **Native orchestration:** Rust.
- **Model experimentation:** Python workers during early development; stable pipelines should move behind a versioned local process protocol or ONNX Runtime.
- **Inference portability:** ONNX Runtime is the intended common layer. Provider selection is a gated future decision after hardware probes.

## Component boundaries

```text
React UI
  -> typed Tauri commands/events
Rust application core
  -> read-only media library
  -> cache/catalog service
  -> analysis job scheduler
  -> audio playback and microphone engine
  -> scoring engine (CPU real-time)
  -> local inference worker (GPU optional, CPU fallback)
```

The React UI never opens arbitrary source paths directly. Rust owns filesystem access and exposes safe, purpose-specific commands.

## Data safety

The media library returns read-only streams and metadata. Analysis workers receive either read-only handles or a controlled temporary copy. No code path may write beside an original song. Cache writes use a temporary path followed by an atomic rename after validation.

## Cache concept

```text
cache/<source-sha256>/<pipeline-version>/
  manifest.json
  accompaniment.*
  vocals.*
  melody.json
  lyrics.json
  waveform.bin
  logs/
```

Model files are separate from song caches and include license/source metadata. Concrete formats, codecs, and models remain decision gates.

## Performance strategy

Heavy inference is pre-playback and cancellable. During singing, the critical path is audio I/O, pitch extraction, alignment, and scoring; it must remain CPU-capable. GPU provider failures degrade to CPU rather than blocking playback of an already valid cache.

