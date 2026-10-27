# Reusable Codex prompts

## Start or continue a phase

```text
AGENTS.md、HANDOFF.md、DECISIONS.md、ROADMAP.md、docs/requirements.md、docs/architecture.md を読んでください。git status と直近のコミットを確認し、現在承認済みのフェーズだけを特定してください。まず現状、未検証事項、次の小さな作業、ユーザー判断が必要な項目を日本語で報告してください。重大な選択が必要なら DECISIONS.md に Waiting として記録し、実装せず私の回答を待ってください。
```

## Execute one approved step

```text
現在承認済みフェーズの次の最小ステップだけを、テスト先行で実装してください。元音源は必ず読み取り専用にし、外部AI APIを使わないでください。関連チェックを実行し、観測した結果だけを docs/progress-log.md と HANDOFF.md に記録してください。作業を小さな Conventional Commit に分け、次フェーズには進まないでください。
```

## Stop and hand off

```text
ここで安全に中断できる状態にしてください。git diff とテスト状況を確認し、HANDOFF.md に現在のコミット、完了内容、実際に通った検証、未検証事項、次の安全な一手、判断待ちを記録してください。DECISIONS.md と ROADMAP.md は事実に変化がある場合だけ更新してください。秘密情報、個人音声、著作権音源、大きなモデルをコミットしないでください。最後に再開コマンドを示してください。
```

