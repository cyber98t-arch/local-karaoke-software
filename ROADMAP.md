# Roadmap

| Phase | Status | Outcome | Gate |
|---|---|---|---|
| 0. Repository foundation | Complete | Git, Dev Container, Codex continuity, requirements and verification baseline | Approved: Tauri 2 + React + Rust + Python |
| 1. Song-selection UI skeleton | Complete | Searchable/selectable mock library with analysis states | No new gate |
| 2. Read-only library and cache catalog | Not started | Scan MP3/FLAC/WAV without changing sources; content-addressed catalog | Decision required: metadata/database choice |
| 3. Playback and microphone probe | Not started | Windows audio device selection, playback, loopback-free mic monitoring | Decision required: Windows audio engine/library |
| 4. Analysis pipeline prototype | Not started | Local separation, melody extraction, Japanese lyric alignment, progress reporting | Decision required: model families and licenses |
| 5. GPU provider matrix | Not started | DirectML/CUDA/other supported providers with measured CPU fallback | Decision required after hardware probes |
| 6. Singing and scoring UI | Not started | Low-latency live pitch/timing feedback; no live total score | Scoring weights require approval |
| 7. Result and lyric correction | Not started | Post-song total/breakdown and editable synced Japanese lyrics | UX acceptance review |
| 8. Packaging and VMware qualification | Not started | Windows installer and CPU-only Windows 11 Home VM evidence | Release approval |

Every phase begins with a read-only preflight, explicit acceptance criteria, and a small test-first implementation plan. Do not start a gated phase until its decision is recorded as approved in `DECISIONS.md`.

