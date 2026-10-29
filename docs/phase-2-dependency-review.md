# Phase 2 dependency review

Reviewed: 2026-10-27

## Selection

- SQLite binding: `rusqlite` with only the `bundled` feature initially.
- Audio metadata reader: `lofty` with default features disabled unless a required read feature is identified during implementation.

The initial catalog slice added the compatible requirement `rusqlite = 0.40.1`; Cargo resolved it to 0.40.2 with `libsqlite3-sys` 0.38.2. The metadata slice added the compatible requirement `lofty = 0.25.4`; Cargo resolved it to 0.25.5.

## Rationale

### `rusqlite` over SQLx

The catalog is a local, single-process SQLite database behind a synchronous Rust service boundary. `rusqlite` directly matches that scope. Its `bundled` feature compiles and links SQLite with the application, avoiding reliance on a Windows machine's installed SQLite version. The crate is MIT licensed. SQLx is Apache-2.0 licensed and supports SQLite, but its async-first, multi-database toolkit and compile-time query infrastructure add capabilities and dependencies that Phase 2 does not need.

The initial feature set must remain `bundled` only. Features for SQLCipher, extensions, backup, hooks, or additional serialization are excluded until a concrete requirement exists and their licenses have been reviewed.

### `lofty` over Symphonia

Phase 2 needs tags and basic properties for MP3, FLAC, and WAV, not audio decoding or playback. `lofty` provides a unified metadata API for all three formats and is dual licensed under MIT or Apache-2.0. Symphonia also reads these formats and metadata, but it is a broader demuxing and decoding framework under MPL-2.0; that scope belongs closer to the later audio-engine decision gate.

Although `lofty` exposes write operations, application code places it behind the read-only media-library interface and returns only copied title, artist, and duration values. No tag-writing, removal, save, rename, or sidecar API is exposed from that boundary. Default features are disabled; `id3v2_compression_support` is enabled explicitly because it is required for the crate's compressed-ID3 read path and `lofty` 0.25.5 does not compile without it.

## Windows and dependency constraints

- `rusqlite`'s upstream documentation recommends `bundled` for applications that control their own databases because it avoids missing or outdated system SQLite installations. This is appropriate for Windows and VMware qualification.
- Both selected crates are ordinary local Rust libraries and introduce no network service or external AI API.
- Neither selected license is restrictive for this project. SQLite bundled by `libsqlite3-sys` is public domain; optional encryption-related features remain excluded.
- On the current Linux development target, `cargo tree -p rusqlite --prefix none` listed 15 packages including `rusqlite` itself and bundled SQLite build support. Binary-size effect and Windows compilation are not yet measured; do not treat the Linux result as Windows verification.

## Sources

- `rusqlite` repository and bundled-feature guidance: https://github.com/rusqlite/rusqlite
- `rusqlite` manifest and license: https://github.com/rusqlite/rusqlite/blob/master/Cargo.toml
- SQLx repository, scope, and license: https://github.com/transact-rs/sqlx
- `lofty` repository, formats, and license: https://github.com/Serial-ATA/lofty-rs
- `lofty` published manifest: https://docs.rs/crate/lofty/latest/source/Cargo.toml
- Symphonia repository, scope, formats, MSRV, and license: https://github.com/pdeljanov/Symphonia
