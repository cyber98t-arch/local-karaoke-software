# Phase 4 analysis decoder review

## Selected boundary

Symphonia `0.6.1` is pinned with default features disabled and only `flac`,
`mp3`, `pcm`, and `wav` enabled. The application opens the source with
`std::fs::File::open`, which is read-only, and gives that handle to Symphonia.
No source path is used as an output location.

Decoded samples remain at the source sample rate. Mono is duplicated to stereo,
stereo is retained, and sources with more than two channels fail with an
actionable format error.

After D-011 approval, Rubato `5.0.1` is pinned with default features disabled
and only `fft_resampler` enabled. Its synchronous FFT resampler converts
decoded stereo to Spleeter's 44.1 kHz rate before inference using a fixed 1024
frame chunk target and the library default Blackman-Harris anti-aliasing
window. Whole-clip `process_all()` trims startup delay. A 44.1 kHz input takes
the identity path without resampling.

## Dependency and license evidence

The resolved decoder packages are Symphonia, `symphonia-core`,
`symphonia-common`, `symphonia-metadata`, `symphonia-bundle-flac`,
`symphonia-bundle-mp3`, `symphonia-codec-pcm`, and
`symphonia-format-riff`, all at `0.6.1`. Cargo.lock records their crates.io
checksums. The selected feature closure also adds the small support crates
`extended`, `lazy_static`, and `regex-lite`; existing shared dependencies are
reused.

Rubato is licensed `MIT OR Apache-2.0`. Its selected dependency closure reuses
`realfft` 3.5.0 and adds the pinned lockfile entries needed by Rubato's audio
adapter boundary. Final installer notices must cover this exact closure.

Symphonia's published crate metadata and included `LICENSE` identify MPL-2.0.
No Symphonia source was modified. A distributable installer must ship the
applicable MPL-2.0 license and source-availability notice; final third-party
notices remain part of the packaging gate rather than being claimed complete
here.

## Verification boundary

Repository fixtures cover MP3, FLAC, and WAV decoding. They are 44.1 kHz mono,
so tests verify native-rate preservation, mono-to-stereo duplication, finite
samples, nonempty output, byte-for-byte source preservation, and no source
directory entry changes. A generated one-second 96 kHz stereo signal verifies
conversion to exactly 44,100 finite stereo frames without collapsing the two
channels. Native Windows decoding/resampling and representative-source timing
remain unverified.
