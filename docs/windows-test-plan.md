# Windows and VMware test plan

## Host test

1. Install current Node.js LTS, Rust MSVC toolchain, Visual Studio C++ Build Tools, and WebView2 if absent.
2. Run `npm ci`, `npm test`, `npm run build`, and `npm run tauri dev`.
3. Confirm the application opens without network access after dependencies are installed.
4. Confirm the source music directory remains byte-for-byte unchanged after import and analysis tests.
5. Measure microphone round-trip behavior and pitch update cadence using at least one built-in laptop microphone.
6. Probe available ONNX Runtime providers and verify automatic CPU fallback.

## VMware Windows 11 Home guest

Use a disposable snapshot without GPU passthrough as the CPU-only baseline. Install the packaged application as a normal user, import copied test fixtures from a read-only shared folder, complete one cached analysis, sing a short fixture, and uninstall. Record CPU model, assigned cores/RAM, analysis duration, peak memory, startup time, errors, and whether any source file changed.

## Evidence policy

Store sanitized measurements and pass/fail results in `docs/progress-log.md`. Do not commit personal audio, copyrighted songs, credentials, machine identifiers, or large model files.

