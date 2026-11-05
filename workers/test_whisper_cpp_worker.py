import json
from pathlib import Path
import tempfile
import unittest

import whisper_cpp_worker as worker


class WhisperWorkerTests(unittest.TestCase):
    def test_parses_timestamped_japanese_segments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "result.json"
            path.write_text(json.dumps({"transcription": [
                {"offsets": {"from": 0, "to": 720}, "text": " 今日は "},
                {"offsets": {"from": 800, "to": 1500}, "text": "晴れです"},
            ]}), encoding="utf-8")
            result = worker.parse_whisper_json(path)
            self.assertEqual(result["language"], "ja")
            self.assertEqual(result["durationMs"], 1500)
            self.assertEqual(result["segments"][0]["text"], "今日は")

    def test_rejects_missing_segments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "result.json"
            path.write_text('{"transcription":[]}', encoding="utf-8")
            with self.assertRaises(ValueError):
                worker.parse_whisper_json(path)


if __name__ == "__main__":
    unittest.main()
