# CREPE tiny ONNX feasibility results

Status: feasibility demonstrated on Linux CPU; production runtime/artifact
adoption is awaiting D-014.

## Pinned input

- Family: CREPE tiny, capacity multiplier 4
- Upstream model revision: `bb29b8d99a89924476d112d27ed470e4ac5617c0`
- Compressed H5 bytes: `1,307,310`
- Compressed H5 SHA-256:
  `9c1d76d405674d2327f5636347d07f551dfd11c6acc0475b7fd85082cdab0add`
- Decompressed H5 bytes: `2,011,144`
- Decompressed H5 SHA-256:
  `409254d6a537cc432061112b7ec5f6fd283658f108fc2abc24db9c25858b42df`
- License: MIT

The already approved and installed disposable Spleeter conversion environment
was reused. No dependency or model download occurred in this probe.

## Conversion

`research/onnx/convert_crepe_tiny.py` recreates the official tiny architecture,
loads the pinned H5 weights, and converts it with TensorFlow 2.12.1,
tf2onnx 1.16.1, and ONNX opset 13.

- Parameters: `487,096`
- Input: float32 `[batch, 1024]`, name `input`
- Output: float32 `[batch, 360]`, name `classifier`
- First ONNX bytes: `1,956,735`
- First ONNX SHA-256:
  `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`
- Second ONNX bytes: `1,956,738`
- Second ONNX SHA-256:
  `ba92f87c660e9bdc78fbe425c29d7e35f2926fa5b439c3a40e39c15879c61dab`

Both conversions produced identical numerical results but different bytes,
again because tf2onnx assigns varying internal identifiers. Any adopted
artifact must pin its exact byte size and SHA-256; conversion is not currently
byte-deterministic.

On eight deterministic normalized input frames, TensorFlow versus the first
CPU ONNX artifact measured RMSE `4.4096e-8` and maximum absolute difference
`6.1095e-7`.

## Generated-tone pitch probe

The official preprocessing contract was reproduced: mono 16 kHz audio,
zero-centered 1024-sample windows, 160-sample/10 ms hop, per-window mean/std
normalization, 360 bins, and local weighted averaging over the peak ±4 bins.

| Expected | Median estimate | Absolute error | Median confidence | CPU inference for 201 frames |
|---:|---:|---:|---:|---:|
| 110 Hz | 110.1008 Hz | 1.586 cents | 0.8741 | 0.0891 s |
| 220 Hz | 220.8825 Hz | 6.931 cents | 0.8161 | 0.0884 s |
| 440 Hz | 440.3964 Hz | 1.559 cents | 0.9269 | 0.0526 s |

These generated tones establish basic pitch usefulness and substantial
faster-than-real-time inference in the Linux container. They do not replace a
Windows probe or evaluation on separated singing voice.

## Proposed production boundary

Reuse the approved native Rust ONNX Runtime CPU boundary. Application-owned
Rust code should downmix the cached vocal stem, resample to 16 kHz, create and
normalize centered frames, decode salience to frequency/confidence, and commit
a versioned contour beneath the identity-derived cache. TensorFlow, Python,
NumPy, and hmmlearn must remain research-only.

Viterbi smoothing is not required for the initial contour probe; it changes
temporal decoding behavior and should be evaluated separately against the
unsmoothed local-average output.
