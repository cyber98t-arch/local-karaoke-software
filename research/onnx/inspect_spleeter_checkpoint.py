"""Inspect a Spleeter TensorFlow checkpoint without running inference."""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

import tensorflow as tf


IGNORED_TERMINAL_OPS = {
    "Assign",
    "AssignAdd",
    "AssignSub",
    "Const",
    "NoOp",
    "Placeholder",
    "RestoreV2",
    "SaveV2",
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def tensor_record(tensor: tf.Tensor) -> dict:
    return {
        "name": tensor.name,
        "dtype": tensor.dtype.name,
        "shape": tensor.shape.as_list() if tensor.shape.rank is not None else None,
    }


def inspect(model_directory: Path, candidate_limit: int) -> dict:
    meta_path = model_directory / "model.meta"
    checkpoint_prefix = model_directory / "model"
    if not meta_path.is_file():
        raise ValueError("model.meta is missing")
    if not checkpoint_prefix.with_suffix(".index").is_file():
        raise ValueError("model.index is missing")

    tf.compat.v1.disable_eager_execution()
    graph = tf.Graph()
    with graph.as_default():
        tf.compat.v1.train.import_meta_graph(str(meta_path), clear_devices=True)

    placeholders = [
        tensor_record(operation.outputs[0])
        for operation in graph.get_operations()
        if operation.type == "Placeholder" and operation.outputs
    ]
    terminals = []
    for operation in graph.get_operations():
        if operation.type in IGNORED_TERMINAL_OPS:
            continue
        for tensor in operation.outputs:
            if not tensor.consumers():
                terminals.append(tensor_record(tensor))

    files = []
    for path in sorted(model_directory.glob("model.*")):
        files.append({"name": path.name, "bytes": path.stat().st_size, "sha256": sha256(path)})

    operation_counts = Counter(operation.type for operation in graph.get_operations())
    return {
        "schemaVersion": 1,
        "checkpointFiles": files,
        "operationCount": len(graph.get_operations()),
        "operationTypes": dict(sorted(operation_counts.items())),
        "placeholderCount": len(placeholders),
        "placeholders": placeholders[:candidate_limit],
        "terminalCandidateCount": len(terminals),
        "terminalCandidates": terminals[:candidate_limit],
        "collectionKeys": sorted(graph.get_all_collection_keys()),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("model_directory", type=Path)
    parser.add_argument(
        "--candidate-limit",
        type=int,
        default=50,
        help="maximum placeholder and terminal candidate records to emit",
    )
    arguments = parser.parse_args()
    if arguments.candidate_limit < 0:
        parser.error("--candidate-limit must be non-negative")
    print(
        json.dumps(
            inspect(arguments.model_directory, arguments.candidate_limit),
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
