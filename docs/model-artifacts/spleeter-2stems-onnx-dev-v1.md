# Spleeter 2-stem ONNX development artifact v1

This is a development-only model asset for native Windows CPU inference tests.
It is not an application release or installer and is not yet used by an
automatic download path.

## Published artifact

- File: `spleeter-2stems-repro.onnx`
- Bytes: `78,652,254`
- SHA-256: `8d58651b5979075c954bcbe96c6776b2dc41e7852fb6f2d054b83a696a8db26f`
- ONNX opset: 13
- Input tensor: `Abs:0`
- Output tensors: `vocals_spectrogram/mul:0`,
  `accompaniment_spectrogram/mul:0`

Always verify both byte size and SHA-256 before loading the artifact.

## Provenance

- Source family: Deezer Spleeter 2-stem
- Source release: v1.4.0 `2stems.tar.gz`
- Source archive bytes: `73,109,797`
- Source archive SHA-256:
  `f3a90b39dd2874269e8b05a48a86745df897b848c61f3958efc80a39152bd692`
- Conversion environment: Spleeter 2.4.2, TensorFlow 2.12.1,
  tf2onnx 1.16.1
- Conversion script: `research/onnx/convert_spleeter_spectrogram.py`

The conversion extracts only the neural spectrogram subgraph. Application code
owns STFT, ratio-mask construction, and inverse STFT.

The conversion is reproducible from pinned inputs and tools but is not currently
byte-deterministic because tf2onnx generates varying internal temporary node
identifiers. This publication pins the exact bytes and hash above; it does not
claim that a fresh conversion will have the same hash.

## License

Spleeter code and the selected upstream model release are documented under the
MIT license. Preserve the upstream license and attribution when redistributing
this converted artifact:

https://github.com/deezer/spleeter/blob/v1.4.0/LICENSE

Conversion tooling is not embedded in the model file. tf2onnx and ONNX use
Apache-2.0; TensorFlow uses Apache-2.0. Runtime packaging notices are handled
separately and are not part of this model-only prerelease.

## Validation status

CPU ONNX Runtime executed this neural subgraph successfully. On the recorded
representative 30-second comparison, the TensorFlow and ONNX stem encodings
differed by no more than one signed 16-bit PCM least-significant bit, and the
user reported no audible difference. Native Windows linking is verified;
native Windows model loading and full separation remain the next checks.
