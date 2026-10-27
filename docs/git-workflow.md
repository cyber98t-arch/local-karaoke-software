# Git workflow

Keep commits small enough to review independently. Preferred sequence for a feature is documentation or test intent, minimal implementation, then isolated refactoring. Use Conventional Commit subjects in the imperative mood, for example:

```text
docs: record read-only media invariant
test: cover Japanese library search
feat: add searchable song library shell
build: add reproducible dev container
```

Before each commit, inspect `git diff`, run the relevant tests, and avoid staging unrelated files. Before stopping, update `HANDOFF.md` and include observed checks in `docs/progress-log.md`. Pushing, releases, tags, history rewrites, and destructive cleanup require explicit user approval.

