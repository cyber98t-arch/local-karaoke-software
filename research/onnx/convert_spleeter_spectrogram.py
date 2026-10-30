"""Convert the pinned Spleeter 2-stem neural subgraph to ONNX."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import onnx


INPUT_TENSOR = "Abs:0"
OUTPUT_TENSORS = (
    "vocals_spectrogram/mul:0",
    "accompaniment_spectrogram/mul:0",
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("checkpoint_directory", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--opset", type=int, default=13)
    arguments = parser.parse_args()
    meta = arguments.checkpoint_directory / "model.meta"
    if not meta.is_file():
        parser.error("checkpoint directory does not contain model.meta")
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            sys.executable,
            "-m",
            "tf2onnx.convert",
            "--checkpoint",
            str(meta),
            "--inputs",
            INPUT_TENSOR,
            "--outputs",
            ",".join(OUTPUT_TENSORS),
            "--output",
            str(arguments.output),
            "--opset",
            str(arguments.opset),
        ],
        check=True,
    )
    model = onnx.load(str(arguments.output))
    onnx.checker.check_model(model)
    actual_inputs = [value.name for value in model.graph.input]
    actual_outputs = [value.name for value in model.graph.output]
    if actual_inputs != [INPUT_TENSOR] or actual_outputs != list(OUTPUT_TENSORS):
        raise RuntimeError(
            f"unexpected ONNX interface: inputs={actual_inputs}, outputs={actual_outputs}"
        )
    checkpoint_files = []
    for path in sorted(arguments.checkpoint_directory.glob("model.*")):
        checkpoint_files.append(
            {"name": path.name, "bytes": path.stat().st_size, "sha256": sha256(path)}
        )
    print(
        json.dumps(
            {
                "schemaVersion": 1,
                "checkpointFiles": checkpoint_files,
                "opset": arguments.opset,
                "output": {
                    "bytes": arguments.output.stat().st_size,
                    "sha256": sha256(arguments.output),
                },
                "inputTensor": INPUT_TENSOR,
                "outputTensors": list(OUTPUT_TENSORS),
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
