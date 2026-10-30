"""Validated, cache-only boundary for the local two-stem ONNX worker."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import sys
from typing import Callable, Optional


PROTOCOL_VERSION = 1
Inference = Callable[[Path, Path, Path, Path], Optional[tuple[int, int, int]]]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def response(request_id: str, *, success: bool, separation=None, error=None) -> dict:
    return {
        "protocolVersion": PROTOCOL_VERSION,
        "requestId": request_id,
        "success": success,
        "metrics": None,
        "separation": separation,
        "error": error,
    }


def process_request(request: dict, *, inference: Inference) -> dict:
    request_id = request.get("requestId")
    try:
        if not isinstance(request_id, str) or not safe_token(request_id):
            raise ValueError("requestId must be a nonempty safe token")
        if request.get("protocolVersion") != PROTOCOL_VERSION:
            raise ValueError("protocol version mismatch")
        operation = request.get("operation")
        if not isinstance(operation, dict) or set(operation) != {"separateTwoStems"}:
            raise ValueError("unsupported operation")
        arguments = operation["separateTwoStems"]
        if not isinstance(arguments, dict):
            raise ValueError("separation arguments are required")
        source = required_file(arguments, "inputAudioPath")
        model = required_file(arguments, "modelPath")
        verify_hash(source, arguments.get("expectedSourceSha256"), "source")
        verify_hash(model, arguments.get("expectedModelSha256"), "model")
        output = required_path(request, "outputDirectory")
        output.mkdir(parents=True, exist_ok=True)
        vocals = output / "vocals.wav"
        accompaniment = output / "accompaniment.wav"
        if vocals.exists() or accompaniment.exists():
            raise ValueError("separation artifacts already exist; refusing to overwrite")
        vocals_temporary = output / "vocals.wav.tmp"
        accompaniment_temporary = output / "accompaniment.wav.tmp"
        if vocals_temporary.exists() or accompaniment_temporary.exists():
            raise ValueError("stale separation temporary artifacts require recovery")

        try:
            measured = inference(
                source, model, vocals_temporary, accompaniment_temporary
            )
            for temporary in (vocals_temporary, accompaniment_temporary):
                if not temporary.is_file() or temporary.stat().st_size == 0:
                    raise ValueError("inference did not produce both nonempty artifacts")
            sample_rate, channels, frames = measured or (44_100, 2, 1)
            if sample_rate <= 0 or channels != 2 or frames <= 0:
                raise ValueError("inference returned invalid audio metrics")
            os.replace(vocals_temporary, vocals)
            os.replace(accompaniment_temporary, accompaniment)
        except Exception:
            vocals_temporary.unlink(missing_ok=True)
            accompaniment_temporary.unlink(missing_ok=True)
            raise

        return response(
            request_id,
            success=True,
            separation={
                "sampleRate": sample_rate,
                "channels": channels,
                "frames": frames,
                "vocals": artifact(vocals),
                "accompaniment": artifact(accompaniment),
            },
        )
    except (OSError, ValueError) as error:
        return response(
            request_id if isinstance(request_id, str) else "invalid-request",
            success=False,
            error=str(error),
        )


def artifact(path: Path) -> dict:
    return {"fileName": path.name, "bytes": path.stat().st_size, "sha256": sha256(path)}


def required_path(container: dict, key: str) -> Path:
    value = container.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{key} is required")
    return Path(value)


def required_file(container: dict, key: str) -> Path:
    path = required_path(container, key)
    if not path.is_file():
        raise ValueError(f"{key} must identify an existing file")
    return path


def verify_hash(path: Path, expected: object, label: str) -> None:
    if not isinstance(expected, str) or len(expected) != 64:
        raise ValueError(f"expected {label} SHA-256 is invalid")
    if sha256(path) != expected:
        raise ValueError(f"{label} SHA-256 mismatch")


def safe_token(value: str) -> bool:
    return value not in {"", ".", ".."} and all(
        character.isascii() and (character.isalnum() or character in ".-_")
        for character in value
    )


def unavailable_inference(_source: Path, _model: Path, _vocals: Path, _accompaniment: Path):
    raise ValueError("ONNX inference runtime is not connected to the worker yet")


def main() -> int:
    try:
        request = json.load(sys.stdin)
        result = process_request(request, inference=unavailable_inference)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        result = response("invalid-request", success=False, error=str(error))
    json.dump(result, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
