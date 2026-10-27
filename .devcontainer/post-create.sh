#!/usr/bin/env bash
set -euo pipefail
sudo chown -R vscode:vscode /home/vscode/.codex /usr/local/cargo/registry
git config --global --add safe.directory "$(pwd)"
npm ci
python3 -m venv .venv
git lfs install --local
printf '\nHibiki development container is ready.\nRun: npm run dev\nRun Codex: codex\nResume Codex: codex resume\n\n'

