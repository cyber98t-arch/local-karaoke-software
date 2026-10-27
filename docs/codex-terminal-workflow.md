# Codex terminal workflow

## First start

Open this repository in VS Code and choose **Dev Containers: Reopen in Container** yourself. In the container terminal, run:

```text
codex
```

Sign in when prompted, then paste the **Start or continue a phase** prompt from `PROMPTS.md`.

## Pause and resume the same conversation

Use `Ctrl+C` to stop an active Codex turn. The CLI session is stored under `CODEX_HOME`; this project mounts that directory on the persistent Docker volume `hibiki-codex-home`.

To resume later from the repository terminal:

```text
codex resume
```

Select the relevant saved session. Rebuilding the Dev Container keeps the volume, but deleting the Docker volume removes its local session data.

## Prevent long-context drift

Do not depend on an ever-growing chat as the only memory. At each stopping point, ask Codex to update:

- `HANDOFF.md` for the exact restart position;
- `DECISIONS.md` for approved or waiting decisions;
- `docs/progress-log.md` for observed test evidence;
- the current roadmap status, only when its acceptance conditions are met.

After a phase ends, it is usually better to start a fresh `codex` session for the next phase instead of indefinitely resuming one conversation. The new session reads the repository files and receives only the focused phase prompt. This keeps context smaller while preserving important facts in Git.

## Recommended stopping sequence

1. Tell Codex: `PROMPTS.md の「Stop and hand off」を実行してください。`
2. Review the resulting Git diff and test evidence.
3. Commit the handoff with the phase work.
4. Exit with `Ctrl+C`.

## Recovery if the session list is lost

Run a new `codex` session and use the orientation prompt in `PROMPTS.md`. The version-controlled handoff and decision files are the authoritative recovery path; CLI chat history is a convenience, not the only project memory.

