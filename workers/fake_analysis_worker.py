"""Dependency-free worker used to verify the local analysis protocol."""

from __future__ import annotations

import json
import os
from pathlib import Path
import sys


PROTOCOL_VERSION = 1
OPERATION = "syntheticSeparationProbe"


def response(request_id: str, *, success: bool, metrics=None, error=None) -> dict:
    return {
        "protocolVersion": PROTOCOL_VERSION,
        "requestId": request_id,
        "success": success,
        "metrics": metrics,
        "error": error,
    }


def run(request: dict) -> dict:
    request_id = request.get("requestId")
    if not isinstance(request_id, str) or not request_id:
        raise ValueError("requestId must be a nonempty string")
    if request.get("protocolVersion") != PROTOCOL_VERSION:
        return response(request_id, success=False, error="protocol version mismatch")
    if request.get("operation") != OPERATION:
        return response(request_id, success=False, error="unsupported operation")

    output_directory = request.get("outputDirectory")
    if not isinstance(output_directory, str) or not output_directory:
        return response(request_id, success=False, error="outputDirectory is required")

    output_path = Path(output_directory)
    output_path.mkdir(parents=True, exist_ok=True)
    metrics = {"sampleRate": 16000, "channels": 2, "frames": 16000}
    temporary = output_path / "probe.json.tmp"
    final = output_path / "probe.json"
    temporary.write_text(json.dumps(metrics, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, final)
    return response(request_id, success=True, metrics=metrics)


def main() -> int:
    try:
        request = json.load(sys.stdin)
        result = run(request)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        result = response("invalid-request", success=False, error=str(error))
    json.dump(result, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
