# Hibiki Local Karaoke

A Windows-first, local-only AI karaoke application. This repository currently contains the development foundation and a working first-pass song-selection interface.

## Start in VS Code

1. Install Docker Desktop and VS Code with the Dev Containers extension.
2. Open this repository in VS Code.
3. Choose **Dev Containers: Reopen in Container**.
4. After setup completes, run `npm run dev` and open the forwarded port 1420.

The Linux container provides repeatable editing, web UI builds, tests, Python tooling, Rust checks, and Codex CLI. Windows application launch, microphone latency, GPU providers, and installer verification run on the Windows host; see `docs/windows-test-plan.md`.

## Codex from the terminal

```text
codex
codex resume
```

The container keeps Codex state in the Docker volume `hibiki-codex-home`, so rebuilding the container does not erase its local sessions. Sign in interactively the first time. Repository decisions and progress also live in version-controlled files, so development can continue even when a chat session is unavailable.

For the recommended start/stop/resume routine and a way to avoid very long conversation context, read `docs/codex-terminal-workflow.md` and use the ready-to-paste prompts in `PROMPTS.md`.

## Common commands

```text
npm run dev
npm test
npm run lint
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

`npm run tauri dev` should normally be run on Windows because Linux containers cannot validate Windows audio devices, GPU providers, or the final WebView runtime.

## Privacy and source safety

Original MP3, FLAC, and WAV files will be opened read-only. Derived stems, pitch tracks, synchronized lyrics, and analysis metadata belong in a separate application cache. No external AI API is permitted.

