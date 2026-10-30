import tempfile
from pathlib import Path
import unittest

from onnx_separation_worker import process_request, sha256


class SeparationWorkerTests(unittest.TestCase):
    def test_validates_hashes_and_commits_two_new_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input.wav"
            model = root / "model.onnx"
            output = root / "cache" / "separation" / "request-1"
            source.write_bytes(b"source fixture")
            model.write_bytes(b"model fixture")
            source_before = source.read_bytes()

            def inference(_source: Path, _model: Path, vocals: Path, accompaniment: Path) -> None:
                vocals.write_bytes(b"vocals")
                accompaniment.write_bytes(b"accompaniment")

            response = process_request(
                request(source, model, output), inference=inference
            )

            self.assertTrue(response["success"])
            self.assertEqual((output / "vocals.wav").read_bytes(), b"vocals")
            self.assertEqual(
                (output / "accompaniment.wav").read_bytes(), b"accompaniment"
            )
            self.assertEqual(source.read_bytes(), source_before)
            self.assertFalse((output / "vocals.wav.tmp").exists())

    def test_rejects_hash_mismatch_and_existing_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input.wav"
            model = root / "model.onnx"
            output = root / "output"
            source.write_bytes(b"source fixture")
            model.write_bytes(b"model fixture")
            bad = request(source, model, output)
            bad["operation"]["separateTwoStems"]["expectedSourceSha256"] = "0" * 64

            mismatch = process_request(bad, inference=lambda *_: None)

            self.assertFalse(mismatch["success"])
            self.assertIn("source SHA-256", mismatch["error"])

            output.mkdir()
            (output / "vocals.wav").write_bytes(b"keep")
            existing = process_request(
                request(source, model, output), inference=lambda *_: None
            )
            self.assertFalse(existing["success"])
            self.assertEqual((output / "vocals.wav").read_bytes(), b"keep")


def request(source: Path, model: Path, output: Path) -> dict:
    return {
        "protocolVersion": 1,
        "requestId": "request-1",
        "operation": {
            "separateTwoStems": {
                "inputAudioPath": str(source),
                "modelPath": str(model),
                "expectedSourceSha256": sha256(source),
                "expectedModelSha256": sha256(model),
            }
        },
        "outputDirectory": str(output),
    }


if __name__ == "__main__":
    unittest.main()
