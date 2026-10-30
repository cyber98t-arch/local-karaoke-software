# Spleeter ONNX feasibility plan

Prepared 2026-10-30 under D-007. This is a research plan, not an engine-adoption record. No package, model, or fixture was downloaded while preparing it.

## Known conversion path and risks

- The pinned Spleeter 2-stem v1.4.0 artifact is a TensorFlow checkpoint archive, not a SavedModel or ONNX file.
- `tf2onnx` officially accepts TensorFlow checkpoints when explicit input and output tensor names are supplied. It can compare TensorFlow and ONNX outputs and capture performance for configured pretrained models.
- Spleeter does not publish an official ONNX artifact or documented tensor-name recipe. Discovering and freezing the inference graph is therefore part of feasibility, not a presumed mechanical conversion.
- Current `tf2onnx` 1.17.0 primarily tests TensorFlow 2.13–2.15. Spleeter 2.4.2 pins TensorFlow 2.12.1. A compatible converter/ONNX/protobuf set must be resolved and locked rather than relying on current unconstrained dependencies.
- Failed conversion, unsupported operators, shape changes, or a need for custom operators counts as negative feasibility evidence and must not be hidden behind runtime-specific code.

## Environment separation

### Conversion/reference environment (temporary research only)

- Python 3.11, Spleeter 2.4.2, TensorFlow 2.12.1, the pinned 2-stem model, tf2onnx, ONNX, and a CPU ONNX Runtime used for conversion checks.
- Pin every direct and transitive distribution with filename, version, byte size, SHA-256, source URL, and license before calling the environment reproducible.
- Keep the entire environment outside the repository and application cache. Preserve only scripts, manifests, measurements, and small textual reports in Git.
- Never ship this TensorFlow environment as a consequence of successful conversion.

### Candidate runtime environment

- CPU ONNX Runtime only for the first feasibility result. Record its exact version, wheel/native-library size, SHA-256, MIT license, and third-party notices.
- Load the converted graph with CPU provider explicitly selected; provider fallback must be visible in logs rather than silently accepted.
- No ONNX Runtime crate/package is added to the application until conversion and comparison pass and the user separately approves adoption.

## Comparison protocol

Use identical decoded float PCM tensors and identical Spleeter framing/post-processing for both paths.

1. Verify tensor names, shapes, sample rate, channel order, output stem order, and finite values.
2. Save TensorFlow and ONNX vocals/accompaniment as cache-owned WAV files with identical encoding.
3. Report maximum absolute difference, mean absolute error, RMSE, cosine similarity, and SI-SDR between corresponding stems. Report mixture-reconstruction error separately.
4. Measure cold start, warm inference, total processing time, real-time factor, and peak resident memory. Repeat enough times to report median and range.
5. Conduct level-matched, randomized A/B listening on a permissively licensed vocal-music fixture. Record audible vocal leakage, missing vocal content, transient smearing, high-frequency loss, and musical-noise artifacts. Do not use the user's library as the committed test fixture.
6. Record failures and unsupported operations verbatim but do not commit environment paths, user names, or large logs.

Numerical closeness alone does not prove acceptable audio quality, and a listening preference does not replace reproducible numerical/performance evidence.

## Future GPU feasibility (documentation only in Phase 4)

- Intel Arc: OpenVINO Plugin Execution Provider can target Intel integrated/discrete GPUs. WinML/DirectML is a cross-vendor Windows alternative.
- AMD Radeon: WinML/DirectML is the primary Windows candidate; do not assume the deprecated ROCm provider is a Windows distribution strategy.
- NVIDIA: CUDA Execution Provider is the vendor-specific candidate; WinML/DirectML remains a cross-vendor fallback.
- DirectML remains supported but is in sustained engineering; official ONNX Runtime guidance recommends WinML for new Windows projects. Provider choice, binary packaging, drivers, CUDA/cuDNN, and fallback policy remain Phase 5 decisions.

No provider is approved or benchmarked by this plan.

## License preflight

| Component | Upstream license | Research implication |
|---|---|---|
| Spleeter code and documented pretrained models | MIT | Preserve provenance and MIT notice for the converted derivative; reconfirm before distribution. |
| TensorFlow | Apache-2.0 | Conversion/reference environment only; transitive notices still require inventory. |
| tf2onnx | Apache-2.0 | Version and conversion script revision must be pinned. |
| ONNX | Apache-2.0 | Pin schema/opset tool version and notices. |
| ONNX Runtime | MIT plus third-party notices | Audit the exact CPU package and any later provider-specific package separately. |

This table does not approve redistribution. The complete resolved dependency inventory and converted-artifact provenance are required before an adoption decision.

## Proposed large-download approval envelope

The earlier probe observed a 585.9 MB TensorFlow wheel before dependencies. A conversion/reference environment, pinned Spleeter model, converter, CPU runtime, and comparison tools are conservatively expected to require:

- up to **1.0 GB network transfer**;
- up to **3.0 GB temporary installed/extracted disk space**;
- no permanent application installation and no committed binaries.

The environment will be placed under a fresh `/tmp` directory and retained only long enough to capture hashes, conversion output, and measurements. Download/execution must not begin until the user explicitly approves this capacity and purpose.

## Official sources reviewed

- tf2onnx repository and conversion CLI: <https://github.com/onnx/tensorflow-onnx>
- tf2onnx 1.17.0 release metadata: <https://pypi.org/project/tf2onnx/1.17.0/>
- ONNX Runtime providers and installation: <https://onnxruntime.ai/docs/execution-providers/> and <https://onnxruntime.ai/docs/install/>
- DirectML provider: <https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html>
- CUDA provider: <https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html>
- OpenVINO provider: <https://onnxruntime.ai/docs/reference/execution-providers/OpenVINO-ExecutionProvider.html>
- ONNX Runtime CPU package metadata: <https://pypi.org/project/onnxruntime/>
