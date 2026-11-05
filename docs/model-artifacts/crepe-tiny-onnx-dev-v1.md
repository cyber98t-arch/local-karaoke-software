# CREPE tiny ONNX development artifact v1

This is a development-only model asset for native Windows CPU melody tests. It
is not an application release or installer and is not used by an automatic
download path.

## Published artifact

- File: `crepe-tiny-research.onnx`
- Release: <https://github.com/cyber98t-arch/local-karaoke-software/releases/tag/model-crepe-tiny-onnx-dev-v1>
- Download: <https://github.com/cyber98t-arch/local-karaoke-software/releases/download/model-crepe-tiny-onnx-dev-v1/crepe-tiny-research.onnx>
- Bytes: `1,956,735`
- SHA-256: `03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4`
- ONNX opset: 13
- Input tensor: `input`, float32 `[batch, 1024]`
- Output tensor: `classifier`, float32 `[batch, 360]`

Always verify both byte size and SHA-256 before loading the artifact.

## Provenance

- Source family: CREPE tiny, capacity multiplier 4
- Source revision: `bb29b8d99a89924476d112d27ed470e4ac5617c0`
- Compressed H5 bytes: `1,307,310`
- Compressed H5 SHA-256:
  `9c1d76d405674d2327f5636347d07f551dfd11c6acc0475b7fd85082cdab0add`
- Decompressed H5 bytes: `2,011,144`
- Decompressed H5 SHA-256:
  `409254d6a537cc432061112b7ec5f6fd283658f108fc2abc24db9c25858b42df`
- Conversion environment: TensorFlow 2.12.1, tf2onnx 1.16.1, ONNX opset 13
- Conversion script: `research/onnx/convert_crepe_tiny.py`

The conversion reconstructs the pinned official tiny architecture and loads the
pinned upstream weights. Application code owns 16 kHz conversion, centered
framing, normalization, local-average pitch decoding, and contour persistence.

The conversion is numerically reproducible from pinned inputs but is not
byte-deterministic because tf2onnx assigns varying internal identifiers. This
publication pins the exact bytes and hash above rather than claiming that a
fresh conversion will produce the same file hash.

## License

CREPE code and the selected upstream model artifact are documented under MIT:

https://github.com/marl/crepe/blob/c9b71ce61491454125a0693f584f7244f29d9884/LICENSE

Conversion tooling is not embedded in the model file. tf2onnx and ONNX use
Apache-2.0; TensorFlow uses Apache-2.0. Runtime packaging notices are handled
separately and are not part of this model-only prerelease.

## Validation status

The converted output agreed with TensorFlow within RMSE `4.4096e-8` and maximum
absolute difference `6.1095e-7` on eight deterministic normalized frames.
Generated 110, 220, and 440 Hz tones were estimated within 6.94 cents by the
local-average decoder. Native Windows model loading, full contour persistence,
and representative singing quality remain explicit follow-up checks.
