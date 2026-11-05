#!/usr/bin/env python3
"""Run the converted CREPE tiny model on deterministic generated tones."""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort

SAMPLE_RATE = 16_000
WINDOW = 1_024
HOP = 160
CENTS = np.linspace(0, 7180, 360) + 1997.3794084376191


def frames(audio: np.ndarray) -> np.ndarray:
    padded = np.pad(audio.astype(np.float32), WINDOW // 2)
    count = 1 + (len(padded) - WINDOW) // HOP
    framed = np.lib.stride_tricks.as_strided(
        padded,
        shape=(count, WINDOW),
        strides=(HOP * padded.itemsize, padded.itemsize),
    ).copy()
    framed -= framed.mean(axis=1, keepdims=True)
    framed /= np.maximum(framed.std(axis=1, keepdims=True), 1.0e-8)
    return framed


def decode(activation: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    confidence = activation.max(axis=1)
    centers = activation.argmax(axis=1)
    cents = np.empty(len(centers), dtype=np.float64)
    for index, center in enumerate(centers):
        start = max(0, center - 4)
        end = min(360, center + 5)
        weights = activation[index, start:end]
        cents[index] = np.sum(weights * CENTS[start:end]) / np.sum(weights)
    frequency = 10.0 * np.power(2.0, cents / 1200.0)
    return frequency, confidence


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True, type=Path)
    args = parser.parse_args()
    session = ort.InferenceSession(
        str(args.model), providers=["CPUExecutionProvider"]
    )
    input_name = session.get_inputs()[0].name
    results = []
    for expected in (110.0, 220.0, 440.0):
        time_axis = np.arange(2 * SAMPLE_RATE, dtype=np.float32) / SAMPLE_RATE
        audio = 0.2 * np.sin(2.0 * np.pi * expected * time_axis)
        input_frames = frames(audio)
        started = time.perf_counter()
        activation = session.run(None, {input_name: input_frames})[0]
        elapsed = time.perf_counter() - started
        frequency, confidence = decode(activation)
        interior = slice(5, -5)
        measured = float(np.median(frequency[interior]))
        results.append(
            {
                "expectedHz": expected,
                "medianHz": measured,
                "absoluteCentsError": float(
                    abs(1200.0 * np.log2(measured / expected))
                ),
                "medianConfidence": float(np.median(confidence[interior])),
                "frames": len(input_frames),
                "inferenceSeconds": elapsed,
            }
        )
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
