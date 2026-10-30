"""Minimal CPU ONNX Runtime path for the Spleeter 2-stem spectrogram model.

This research module deliberately performs STFT and inverse STFT in NumPy. The
full TensorFlow graph cannot be converted by tf2onnx 1.16.1 because its RFFT
layout and IRFFT operations are unsupported.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import time
import wave

import numpy as np
import onnxruntime as ort


FRAME_LENGTH = 4096
FRAME_STEP = 1024
SEGMENT_FRAMES = 512
MODEL_FREQUENCIES = 1024
EPSILON = 1e-10
WINDOW_COMPENSATION_FACTOR = 2.0 / 3.0


def periodic_hann(length: int = FRAME_LENGTH) -> np.ndarray:
    positions = np.arange(length, dtype=np.float32)
    return (0.5 - 0.5 * np.cos(2.0 * np.pi * positions / length)).astype(np.float32)


def stft(waveform: np.ndarray) -> np.ndarray:
    """Match Spleeter's padded TensorFlow STFT, returning frames/frequency/channel."""
    if waveform.ndim != 2 or waveform.shape[1] != 2:
        raise ValueError("waveform must have shape (samples, 2)")
    padded_input = waveform.astype(np.float32)
    frame_count = (len(padded_input) + FRAME_STEP - 1) // FRAME_STEP
    padded_length = (frame_count - 1) * FRAME_STEP + FRAME_LENGTH
    padded_input = np.pad(padded_input, ((0, padded_length - len(padded_input)), (0, 0)))
    frames = np.lib.stride_tricks.sliding_window_view(
        padded_input, FRAME_LENGTH, axis=0
    )[::FRAME_STEP]
    frames = np.transpose(frames, (0, 2, 1))
    return np.fft.rfft(frames * periodic_hann()[None, :, None], axis=1).astype(
        np.complex64
    )


def inverse_stft(spectrum: np.ndarray, output_samples: int) -> np.ndarray:
    """Match Spleeter's Hann inverse STFT, compensation, and leading crop."""
    frames = np.fft.irfft(spectrum, n=FRAME_LENGTH, axis=1).astype(np.float32)
    frames *= periodic_hann()[None, :, None]
    output_length = (len(frames) - 1) * FRAME_STEP + FRAME_LENGTH
    waveform = np.zeros((output_length, spectrum.shape[2]), dtype=np.float32)
    for index, frame in enumerate(frames):
        start = index * FRAME_STEP
        waveform[start : start + FRAME_LENGTH] += frame
    waveform *= WINDOW_COMPENSATION_FACTOR
    cropped = waveform[:output_samples]
    if len(cropped) < output_samples:
        cropped = np.pad(cropped, ((0, output_samples - len(cropped)), (0, 0)))
    return cropped


def partition_magnitude(spectrum: np.ndarray) -> np.ndarray:
    magnitude = np.abs(spectrum).astype(np.float32)
    pad_frames = (-len(magnitude)) % SEGMENT_FRAMES
    if pad_frames:
        magnitude = np.pad(magnitude, ((0, pad_frames), (0, 0), (0, 0)))
    return magnitude.reshape(-1, SEGMENT_FRAMES, magnitude.shape[1], magnitude.shape[2])


def masks_from_estimates(estimates: list[np.ndarray], frame_count: int) -> list[np.ndarray]:
    powers = [estimate.astype(np.float32) ** 2 for estimate in estimates]
    denominator = np.sum(powers, axis=0) + EPSILON
    masks = []
    for power in powers:
        mask = (power + EPSILON / len(powers)) / denominator
        mask = np.pad(
            mask,
            ((0, 0), (0, 0), (0, FRAME_LENGTH // 2 + 1 - MODEL_FREQUENCIES), (0, 0)),
        )
        masks.append(mask.reshape(-1, mask.shape[2], mask.shape[3])[:frame_count])
    return masks


def separate(session: ort.InferenceSession, waveform: np.ndarray) -> tuple[dict[str, np.ndarray], float]:
    spectrum = stft(waveform)
    model_input = partition_magnitude(spectrum)[:, :, :MODEL_FREQUENCIES, :]
    started = time.perf_counter()
    estimates = session.run(None, {"Abs:0": model_input})
    inference_seconds = time.perf_counter() - started
    masks = masks_from_estimates(estimates, len(spectrum))
    outputs = {
        name: inverse_stft(mask * spectrum, len(waveform))
        for name, mask in zip(("vocals", "accompaniment"), masks)
    }
    return outputs, inference_seconds


def read_pcm16_stereo(path: Path) -> tuple[np.ndarray, int]:
    with wave.open(str(path), "rb") as source:
        if source.getnchannels() != 2 or source.getsampwidth() != 2:
            raise ValueError("input must be 16-bit stereo PCM WAV")
        sample_rate = source.getframerate()
        data = np.frombuffer(source.readframes(source.getnframes()), dtype="<i2")
    return data.reshape(-1, 2).astype(np.float32) / 32768.0, sample_rate


def write_pcm16_stereo(path: Path, waveform_data: np.ndarray, sample_rate: int) -> None:
    samples = np.clip(waveform_data, -1.0, 1.0)
    encoded = np.round(samples * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as destination:
        destination.setnchannels(2)
        destination.setsampwidth(2)
        destination.setframerate(sample_rate)
        destination.writeframes(encoded.tobytes())


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("model", type=Path)
    parser.add_argument("input_wav", type=Path)
    parser.add_argument("output_directory", type=Path)
    arguments = parser.parse_args()
    waveform_data, sample_rate = read_pcm16_stereo(arguments.input_wav)
    if sample_rate != 44_100:
        parser.error("input sample rate must be 44100 Hz")
    session = ort.InferenceSession(str(arguments.model), providers=["CPUExecutionProvider"])
    outputs, inference_seconds = separate(session, waveform_data)
    arguments.output_directory.mkdir(parents=True, exist_ok=True)
    for name, output in outputs.items():
        write_pcm16_stereo(arguments.output_directory / f"{name}.wav", output, sample_rate)
    print(
        json.dumps(
            {
                "modelSha256": sha256(arguments.model),
                "inputSha256": sha256(arguments.input_wav),
                "samples": len(waveform_data),
                "inferenceSeconds": inference_seconds,
                "providers": session.get_providers(),
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
