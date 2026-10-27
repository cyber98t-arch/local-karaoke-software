# Progress and verification log

## 2026-10-27 — Repository foundation and initial library UI

- Created a Tauri 2 + React/TypeScript + Rust skeleton.
- Added a Docker-based VS Code Dev Container with Node, Rust, Python, FFmpeg, Git LFS, and Codex CLI.
- Added persistent container volumes for Codex state and build caches.
- Added durable requirements, architecture, roadmap, decision, handoff, and Windows test documents.
- Added a first-pass Japanese song-selection interface and focused UI tests.
- `npm run lint`: passed.
- `npm test`: passed, 2 tests.
- `npm run build`: passed; production assets generated.
- Chrome visual inspection: song list, selection detail, analysis state, privacy note, and post-song score policy rendered correctly.
- Dev Container image: Dockerfile parsed and the base image downloaded; the full first build was stopped by request after scope was limited to preparing the environment. Post-create and Rust checks remain unverified.

## 2026-10-27 — Dev Container setup repair

- Reproduced the VS Code setup failure in the generated container image.
- Root cause: Git rejected the Windows-mounted repository because its container-visible owner differed from the `vscode` user; `git lfs install --local` therefore reported that it was not in a Git repository.
- Repair: register only the active mounted workspace as a Git `safe.directory` before running post-create setup.
- Complete post-create rerun: passed as the non-root `vscode` user. `npm ci`, Python virtual environment creation, local Git LFS initialization, Git repository detection, and `codex --version` all completed successfully.

