#!/usr/bin/env python3
"""Convert the pinned CREPE tiny Keras weights and compare CPU outputs."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import onnxruntime as ort
import tensorflow as tf
import tf2onnx


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_model() -> tf.keras.Model:
    inputs = tf.keras.layers.Input(shape=(1024,), name="input", dtype="float32")
    output = tf.keras.layers.Reshape((1024, 1, 1), name="input-reshape")(inputs)
    filters = [128, 16, 16, 16, 32, 64]
    widths = [512, 64, 64, 64, 64, 64]
    strides = [(4, 1), (1, 1), (1, 1), (1, 1), (1, 1), (1, 1)]
    for layer, count, width, stride in zip(
        range(1, 7), filters, widths, strides, strict=True
    ):
        output = tf.keras.layers.Conv2D(
            count,
            (width, 1),
            strides=stride,
            padding="same",
            activation="relu",
            name=f"conv{layer}",
        )(output)
        output = tf.keras.layers.BatchNormalization(name=f"conv{layer}-BN")(output)
        output = tf.keras.layers.MaxPool2D(
            pool_size=(2, 1), padding="valid", name=f"conv{layer}-maxpool"
        )(output)
        output = tf.keras.layers.Dropout(0.25, name=f"conv{layer}-dropout")(output)
    output = tf.keras.layers.Permute((2, 1, 3), name="transpose")(output)
    output = tf.keras.layers.Flatten(name="flatten")(output)
    output = tf.keras.layers.Dense(360, activation="sigmoid", name="classifier")(
        output
    )
    return tf.keras.Model(inputs=inputs, outputs=output, name="crepe-tiny")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--weights", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    model = build_model()
    model.load_weights(args.weights)
    signature = (tf.TensorSpec((None, 1024), tf.float32, name="input"),)
    tf2onnx.convert.from_keras(
        model,
        input_signature=signature,
        opset=13,
        output_path=str(args.output),
    )

    rng = np.random.default_rng(20261105)
    frames = rng.normal(size=(8, 1024)).astype(np.float32)
    frames -= frames.mean(axis=1, keepdims=True)
    frames /= np.maximum(frames.std(axis=1, keepdims=True), 1.0e-8)
    tensorflow_output = model(frames, training=False).numpy()
    session = ort.InferenceSession(
        str(args.output), providers=["CPUExecutionProvider"]
    )
    onnx_output = session.run(None, {session.get_inputs()[0].name: frames})[0]
    difference = tensorflow_output - onnx_output
    result = {
        "weightsSha256": sha256(args.weights),
        "onnxBytes": args.output.stat().st_size,
        "onnxSha256": sha256(args.output),
        "inputName": session.get_inputs()[0].name,
        "inputShape": session.get_inputs()[0].shape,
        "outputName": session.get_outputs()[0].name,
        "outputShape": session.get_outputs()[0].shape,
        "parameterCount": model.count_params(),
        "rmse": float(np.sqrt(np.mean(difference * difference))),
        "maxAbsoluteDifference": float(np.max(np.abs(difference))),
    }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
