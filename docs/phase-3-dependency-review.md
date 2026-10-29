# Phase 3 audio dependency review

Reviewed: 2026-10-29

## Selection

- Audio I/O: `cpal` 0.18.2 with default features only.
- Windows backend: CPAL's default WASAPI backend in shared mode.
- Excluded features: `asio`, `jack`, `pipewire`, `pulseaudio`, `realtime`, `realtime-dbus`, `audioworklet`, and `custom`.

## Rationale

CPAL exposes device enumeration, supported/default stream configurations, and input/output streams through one low-level boundary. On Windows its default backend is WASAPI. This matches the approved Phase 3 scope while leaving device negotiation and the real-time callback path under the Rust application core.

Rodio is not selected for this phase. It would simplify playback but still uses CPAL underneath, does not replace the microphone boundary, and currently documents an ongoing core-engine rewrite. Direct `windows-rs` WASAPI bindings are also not selected because they would add Windows-specific and potentially unsafe orchestration before the probe demonstrates a need for controls CPAL cannot expose.

## License and version constraints

- The published CPAL 0.18.2 manifest and included `LICENSE` declare Apache-2.0.
- CPAL 0.18.2 requires Rust 1.85; the current Dev Container reports Rust 1.99.0.
- CPAL's default feature set is empty. ASIO and its additional toolchain/driver requirements therefore remain excluded.
- The resolved Linux target added CPAL, `alsa`, `alsa-sys`, and `dasp_sample`; their published licenses are Apache-2.0, Apache-2.0/MIT, MIT, and MIT OR Apache-2.0 respectively. Other target-specific packages remain in the lockfile but are not Linux runtime dependencies.

## Platform and build constraints

- On Windows, CPAL 0.18.2 uses `windows` and `windows-core` 0.62 for WASAPI access.
- On Linux, the default backend requires ALSA. The Dev Container already installs `libasound2-dev`, so adding CPAL does not require another system package.
- The dependency is local and introduces no network service, external AI API, model, or paid component.
- Binary-size and Windows latency effects remain measurements, not assumptions; record them during the native probe.

## Initial integration boundary

The first code slice enumerates devices only through an injected trait and returns application-owned value types. The subsequent playback probe opens a selected output stream for 500 ms using a generated low-amplitude tone. It does not read source audio, perform loopback capture, monitor the microphone, or expose CPAL objects to React.

## Sources

- CPAL repository, platform table, features, and examples: https://github.com/RustAudio/cpal
- CPAL 0.18.2 published manifest and documentation: https://docs.rs/cpal/0.18.2/cpal/
- Rodio repository and engine-rewrite notice: https://github.com/RustAudio/rodio
