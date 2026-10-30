# Phase 4 Spleeter runtime preflight

Reviewed 2026-10-30. This review did not install Python packages, launch inference, or add an audio decoder.

## Upstream runtime

- Spleeter 2.4.2 declares Python `>=3.8,<3.12`, TensorFlow `2.12.1`, `tensorflow-io-gcs-filesystem` `0.32.0`, NumPy `<2.0.0`, pandas, norbert, typer, httpx, and the MIT `ffmpeg-python` wrapper.
- The upstream package classifiers include Windows and Python 3.8 through 3.11. The README recommends `python -m spleeter` when the Windows console launcher fails.
- The official package and wheel are small, but TensorFlow and its native dependency closure dominate download, installed size, startup time, and compatibility risk.
- Upstream documentation says CPU separation is supported. It does not provide evidence for the project's target integrated-GPU Windows laptops, so duration and memory remain measurement tasks.

## Bounded first probe

- Use an isolated Python 3.11 worker environment during development, consistent with the approved architecture for early model experimentation.
- Pin Spleeter 2.4.2 and TensorFlow 2.12.1. Record hashes for every downloaded distribution before treating the environment as reproducible.
- Force CPU-only behavior for this probe. GPU provider selection remains Phase 5 and is not part of D-006.
- Generate a short synthetic stereo PCM array in the worker and call the Spleeter Python API directly. Do not open user music and do not require an external FFmpeg executable for the first runtime/load probe.
- Write probe output only to a temporary/cache-owned directory and validate bounded duration/channel/sample-rate metadata. Do not judge separation quality from a synthetic signal.

## Deferred integration decisions

- Actual MP3/FLAC/WAV decoding for analysis is not selected. Adding or distributing an FFmpeg binary requires an exact build/configuration and license review; `ffmpeg-python` alone is only a wrapper.
- Bundling Python/TensorFlow into the Windows application is not selected. Installed size, startup, antivirus behavior, updates, and licensing notices must be measured first.
- ONNX conversion or another inference runtime would change the runtime/model artifact strategy and requires a new decision rather than being introduced during this probe.

## Official sources reviewed

- Spleeter `pyproject.toml`: <https://github.com/deezer/spleeter/blob/master/pyproject.toml>
- Spleeter README and Windows guidance: <https://github.com/deezer/spleeter>
- Spleeter 2.4.2 release metadata: <https://pypi.org/project/spleeter/2.4.2/>
