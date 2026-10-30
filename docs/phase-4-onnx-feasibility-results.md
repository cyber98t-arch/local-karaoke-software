# D-007 ONNX feasibility results

Status: **CPU ONNX Runtime adopted on 2026-10-30; GPU strategy, publishing, and release remain unapproved**.

## Scope and environment

- Date: 2026-10-30
- Host: Linux development container, CPU execution only
- Input: generated 44.1 kHz stereo tones; no user music was read
- Spleeter: 2.4.2, pinned official 2-stem checkpoint
- TensorFlow: 2.12.1
- tf2onnx: 1.16.1, ONNX opset 13
- ONNX Runtime: 1.16.3, `CPUExecutionProvider`

The disposable environment used approximately 2.6 GB after the wheelhouse was
removed. Actual network transfer remained below the approved 1.0 GB envelope.
The environment briefly reached approximately 3.2 GB, about 0.2 GB above the
approved temporary-disk estimate, while both the 789 MB wheelhouse and 2.4 GB
installed environment coexisted. Work stopped, this was reported, and only the
research-owned wheelhouse was removed before continuing.

## Conversion result

Direct conversion of the complete waveform-to-waveform graph failed. tf2onnx
could not convert Spleeter's RFFT layout and reported unsupported IRFFT nodes.
The neural spectrogram subgraph converted successfully instead:

- input: `Abs:0`
- outputs: `vocals_spectrogram/mul:0`,
  `accompaniment_spectrogram/mul:0`
- STFT, ratio-mask construction, and inverse STFT: small NumPy implementation
- ONNX model size: 78,652,254 bytes on the recorded second conversion
- ONNX SHA-256: `8d58651b5979075c954bcbe96c6776b2dc41e7852fb6f2d054b83a696a8db26f`

Two conversions produced different byte hashes (the first was
`90cf4c80561aad8bb99cd34aa1780f91b264b73b06f19a460b95c05007ce00e9`).
tf2onnx generated different internal temporary node identifiers. Therefore the
current conversion is reproducible by pinned inputs and tooling, but not yet
byte-deterministic. Every produced artifact must retain its own SHA-256.

Checkpoint files:

| File | Bytes | SHA-256 |
|---|---:|---|
| `model.data-00000-of-00001` | 78,614,080 | `7747f9fd2c782306dbec1504360fbb645a097a48446f23486c3ff9c89bc11788` |
| `model.index` | 5,244 | `55661a09f79c86071fc7077b44b4cf1f01b6d82bbbb83ed94cb68a2f9944e378` |
| `model.meta` | 805,575 | `6e1f6d86a22bb452a58cb20e3de87b416f8a79e626ca223e20f763ab2d21ec95` |

The downloaded `2stems.tar.gz` was 73,109,797 bytes with SHA-256
`f3a90b39dd2874269e8b05a48a86745df897b848c61f3958efc80a39152bd692`.

## Numerical comparison

On a generated two-second stereo mixture, the ONNX neural outputs versus the
same TensorFlow tensors were:

| Output | RMSE | Maximum absolute difference |
|---|---:|---:|
| vocals spectrogram | 6.4448e-8 | 1.1683e-5 |
| accompaniment spectrogram | 1.2023e-8 | 7.6294e-6 |
| vocals waveform | 3.4119e-8 | 1.7862e-7 |
| accompaniment waveform | 6.0138e-7 | 2.2948e-6 |

This establishes close numerical agreement for the generated probe. It does
not replace evaluation on representative music.

## Preliminary CPU measurement

One cold-process run used the same generated 10-second stereo mixture:

| Engine | Wall time | Peak RSS |
|---|---:|---:|
| TensorFlow 2.12.1 | 0.368 s | 919,064 KiB |
| ONNX Runtime plus NumPy transforms | 0.257 s | 439,984 KiB |

These are single-container observations, not Windows laptop acceptance
measurements. Repeated runs, installed-size accounting for a production-only
dependency closure, and Windows CPU measurements remain required.

## Listening quality

A user-supplied 44.1 kHz, 16-bit stereo PCM WAV was sampled from 60–90 seconds
without modifying the source. Its byte size, modification timestamp, mode, and
SHA-256 were identical before and after the probe. All derived files were
written beneath `/tmp` and were not committed.

After both outputs were encoded as 16-bit PCM, the maximum TensorFlow-versus-
ONNX difference was exactly one PCM least-significant bit (`3.0517578125e-5`)
for both stems. PCM RMSE was `4.6389e-6` for vocals and `6.1870e-6` for
accompaniment. This is strong numerical evidence that the two engines should
be audibly equivalent. The subsequent user judgment independently reported no
audible difference.

The user subsequently reported that no difference was audible and selected the
lighter option. D-007 therefore adopts the CPU ONNX path as the production
separation foundation. This is a runtime decision, not permission to publish a
release or silently download model/runtime artifacts.

The same 30-second run measured TensorFlow at 0.751 s and 1,288,380 KiB peak
RSS, versus the ONNX/NumPy path at 1.063 s and 925,896 KiB peak RSS. ONNX model
execution used 0.507 s; the remaining signal-processing and allocation cost
made this prototype slower overall. Optimization and repeated Windows
measurement are required before any performance claim.

## Licenses and dependency inventory

- Spleeter code and published model package: MIT metadata
- ONNX Runtime: MIT
- tf2onnx: Apache-2.0
- ONNX: Apache-2.0
- TensorFlow reference/conversion environment: Apache-2.0
- NumPy: BSD-3-Clause

`docs/research/onnx-python-environment.json` records all 71 installed
distributions, versions, declared license metadata, and a SHA-256 over each
distribution's installed RECORD files (excluding generated `.pyc`). This is an
installed-environment integrity record, not a substitute for a future locked
Windows wheel manifest and third-party-notice audit before distribution.

## Future GPU feasibility (not selected)

- Intel Arc: Windows ML/DirectML, or the ONNX Runtime OpenVINO execution provider
- AMD Radeon: Windows ML/DirectML
- NVIDIA: CUDA execution provider, or Windows ML/DirectML

Microsoft recommends Windows ML for new Windows ONNX applications while the
DirectML execution provider is in sustained engineering. Provider packaging,
operator coverage, numerical results, and actual hardware performance are not
verified. Selecting one is a Phase 5 decision gate.

## Lightweight alternatives if this path is rejected

The partial-graph conversion works, so an alternative is not required for
technical failure. Still, the 78.7 MB model and custom signal path should be
weighed against models with official ONNX artifacts and explicit weight
licenses. Candidate model families must receive a new D-006 review; no family
is silently substituted here. Candidates for that future review include
MDX-Net-derived compact ONNX models and other permissively licensed
waveform/spectrogram separators, provided the exact weight license and source
are verified before download.

## Remaining gates

1. User judgment from the prepared representative-music A/B listening files.
2. Repeated TensorFlow/ONNX benchmarks and Windows CPU/peak-memory evidence.
3. Windows-compatible wheel hashes and complete third-party notices.
4. A deliberate decision on whether non-byte-deterministic conversion is
   acceptable or needs normalization.
5. Separate approval before GPU-provider selection, publishing, or release.
