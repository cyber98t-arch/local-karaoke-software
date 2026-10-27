# Current handoff

- Updated: 2026-10-27
- Current phase: Phase 1 complete; Phase 2 not started.
- Repository state: Initial Tauri/React skeleton, Dev Container, Codex CLI setup, durable project documents, terminal start/stop/resume guidance, and mock song-selection UI are present.
- Verified: `npm run lint`, `npm test` (2 tests), and `npm run build` passed on the Windows host. The UI was opened in Chrome and visually inspected. Dockerfile parsing and base-image download started successfully; the full first container build was intentionally stopped when the user limited this step to environment setup.
- Unverified: Rust/Tauri check inside the Dev Container, Windows Tauri launch, microphone, GPU providers, installer, VMware guest, and all future AI pipelines.
- Next safe action: Review the song-selection UI and decide whether to proceed to Phase 2. Do not choose a catalog/database implementation without updating `DECISIONS.md` and obtaining user approval.
- Blockers: None for reviewing the UI. Git ownership setup was repaired. A second Windows launch failure caused by VS Code's stale WSLg Wayland mount was repaired with a workspace-level setting. Phase 2 has a human decision gate.

## Resume checklist

1. Run `git status --short --branch` and inspect recent commits.
2. Read `AGENTS.md`, `DECISIONS.md`, `ROADMAP.md`, and this file.
3. Run the verification baseline from `AGENTS.md`.
4. Continue only the currently approved phase.

