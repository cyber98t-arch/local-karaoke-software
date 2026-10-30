"""Emit a deterministic inventory of installed Python distributions."""

from __future__ import annotations

import argparse
import hashlib
from importlib import metadata
import json
from pathlib import Path


def distribution_digest(distribution: metadata.Distribution) -> tuple[str, int]:
    digest = hashlib.sha256()
    count = 0
    for relative in sorted(distribution.files or [], key=str):
        path = Path(distribution.locate_file(relative))
        if not path.is_file() or path.suffix == ".pyc":
            continue
        digest.update(str(relative).encode("utf-8"))
        digest.update(b"\0")
        with path.open("rb") as source:
            for block in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(block)
        digest.update(b"\0")
        count += 1
    return digest.hexdigest(), count


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    records = []
    for distribution in sorted(
        metadata.distributions(), key=lambda item: item.metadata["Name"].lower()
    ):
        digest, file_count = distribution_digest(distribution)
        classifiers = distribution.metadata.get_all("Classifier") or []
        license_classifiers = [
            value for value in classifiers if value.startswith("License ::")
        ]
        records.append(
            {
                "name": distribution.metadata["Name"],
                "version": distribution.version,
                "licenseExpression": distribution.metadata.get("License-Expression"),
                "licenseField": distribution.metadata.get("License"),
                "licenseClassifiers": license_classifiers,
                "installedFiles": file_count,
                "installedFilesSha256": digest,
            }
        )
    document = {
        "schemaVersion": 1,
        "hashScope": "relative path and bytes of every installed RECORD file except .pyc",
        "distributions": records,
    }
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(
        json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps({"distributions": len(records), "output": str(arguments.output)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
