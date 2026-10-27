# Repository instructions

## Mission

Build a Windows-first, local-only AI karaoke application that remains usable on ordinary integrated-GPU laptops. The original music directory is always read-only and must never be renamed, moved, rewritten, tagged, or deleted.

## Required reading before work

Read `docs/requirements.md`, `docs/architecture.md`, `ROADMAP.md`, `DECISIONS.md`, and `HANDOFF.md`. Inspect `git status` before editing. Update `HANDOFF.md` before ending a work session. Use `PROMPTS.md` and `docs/codex-terminal-workflow.md` for terminal session continuity.

## Human approval gates

Stop and request an explicit user decision before:

- changing the UI/backend framework, database, audio engine, model family, model license, cache format, or GPU strategy;
- introducing a network service or external AI API;
- changing the read-only-source invariant or privacy boundary;
- downloading a model larger than 500 MB, accepting a restrictive license, or adding a paid dependency;
- deleting user data, rewriting Git history, pushing, publishing, or creating a release;
- beginning a roadmap phase marked `Decision required`.

Record the question in `DECISIONS.md` with status `Waiting`. Do not silently choose a major alternative.

## Engineering rules

- Keep audio sources behind a read-only library interface. Write derived files only under the application cache.
- All inference must work without external AI APIs. Network use is limited to explicit model/dependency downloads.
- Real-time pitch scoring stays CPU-capable; GPU acceleration is an optional pre-playback optimization.
- Complete expensive separation, melody extraction, and lyric alignment before playback and cache results by content hash plus pipeline version.
- Never show a total score during singing. Show the final total only after the performance ends.
- Prefer small modules and typed boundaries. Avoid hidden global state.
- Add or update tests for observable behavior. Never report a check as passed unless it was run.

## Git and session continuity

- Use small Conventional Commits: `docs:`, `test:`, `feat:`, `fix:`, `refactor:`, `build:`, `chore:`.
- One commit should represent one reviewable purpose. Do not mix formatting or unrelated cleanup with a feature.
- Do not amend, rebase, force-push, tag, or push without explicit approval.
- At each stopping point, update `HANDOFF.md` with the current commit, completed work, test evidence, next safe action, and blockers.
- Put actual verification evidence in `docs/progress-log.md`; keep planned work in `ROADMAP.md`.

## Verification baseline

Run the smallest relevant checks first. Before a phase handoff, run:

```text
npm run lint
npm test
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
```

If a Windows-only audio, microphone, GPU, installer, or VMware check cannot run in the container, mark it as unverified and follow `docs/windows-test-plan.md`.

