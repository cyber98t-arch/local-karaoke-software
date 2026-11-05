"""Development-only protocol adapter for an explicitly supplied whisper.cpp CLI."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

PROTOCOL_VERSION = 2


def response(request_id: str, *, success: bool, transcription=None, error=None) -> dict:
    return {"protocolVersion": PROTOCOL_VERSION, "requestId": request_id, "success": success,
            "metrics": None, "separation": None, "transcription": transcription, "error": error}


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def parse_whisper_json(path: Path) -> dict:
    document = json.loads(path.read_text(encoding="utf-8"))
    entries = document.get("transcription")
    if not isinstance(entries, list) or not entries:
        raise ValueError("whisper.cpp JSON contains no transcription segments")
    segments = []
    for entry in entries:
        offsets = entry.get("offsets", {})
        start = offsets.get("from")
        end = offsets.get("to")
        text = entry.get("text")
        if not isinstance(start, int) or not isinstance(end, int) or not isinstance(text, str):
            raise ValueError("whisper.cpp segment has an unsupported shape")
        segments.append({"startMs": start, "endMs": end, "text": text.strip()})
    duration = max(segment["endMs"] for segment in segments)
    return {"language": "ja", "durationMs": duration, "segments": segments}


def run(request: dict, cli: Path) -> dict:
    request_id = request.get("requestId")
    if not isinstance(request_id, str) or not request_id:
        raise ValueError("requestId must be a nonempty string")
    if request.get("protocolVersion") != PROTOCOL_VERSION:
        return response(request_id, success=False, error="protocol version mismatch")
    operation = request.get("operation", {}).get("transcribeJapanese")
    if not isinstance(operation, dict) or operation.get("language") != "ja":
        return response(request_id, success=False, error="unsupported operation")
    audio_value = operation.get("inputAudioPath")
    model_value = operation.get("modelPath")
    output_value = request.get("outputDirectory")
    if not all(isinstance(value, str) and value for value in (audio_value, model_value, output_value)):
        return response(request_id, success=False, error="input, model, and output paths are required")
    audio = Path(audio_value)
    model = Path(model_value)
    output = Path(output_value)
    if not cli.is_file() or not audio.is_file() or not model.is_file():
        return response(request_id, success=False, error="CLI, input, model, and output must exist")
    if file_sha256(audio) != operation.get("expectedSourceSha256"):
        return response(request_id, success=False, error="source SHA-256 mismatch")
    if file_sha256(model) != operation.get("expectedModelSha256"):
        return response(request_id, success=False, error="model SHA-256 mismatch")
    output.mkdir(parents=True, exist_ok=True)
    prefix = output / "whisper-result"
    completed = subprocess.run(
        [str(cli), "-m", str(model), "-f", str(audio), "-l", "ja", "-oj", "-of", str(prefix), "--no-prints"],
        stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        check=False, timeout=3600, text=True, encoding="utf-8", errors="replace")
    if completed.returncode != 0:
        detail = completed.stderr[-2000:].strip()
        return response(request_id, success=False, error=f"whisper.cpp failed: {detail}")
    transcription = parse_whisper_json(prefix.with_suffix(".json"))
    return response(request_id, success=True, transcription=transcription)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--whisper-cli", required=True, type=Path)
    arguments = parser.parse_args()
    try:
        result = run(json.load(sys.stdin), arguments.whisper_cli)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.TimeoutExpired) as error:
        result = response("invalid-request", success=False, error=str(error))
    json.dump(result, sys.stdout, ensure_ascii=False, separators=(",", ":"))
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
