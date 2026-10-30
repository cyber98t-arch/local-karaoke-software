import unittest

import numpy as np

from spleeter_onnx_runtime import inverse_stft, stft


class SignalTransformTests(unittest.TestCase):
    def test_stft_inverse_round_trip_matches_unpadded_waveform(self) -> None:
        sample_rate = 44_100
        time = np.arange(sample_rate, dtype=np.float32) / sample_rate
        waveform = np.stack(
            (
                0.1 * np.sin(2 * np.pi * 220 * time),
                0.1 * np.sin(2 * np.pi * 330 * time),
            ),
            axis=1,
        ).astype(np.float32)

        spectrum = stft(waveform)
        reconstructed = inverse_stft(spectrum, len(waveform))

        np.testing.assert_allclose(
            reconstructed[4096:-4096], waveform[4096:-4096], atol=2e-5, rtol=2e-5
        )


if __name__ == "__main__":
    unittest.main()
