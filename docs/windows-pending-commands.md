# Pending Windows commands

Updated: 2026-11-05

This is the single queue for native Windows checks that cannot run in the Dev
Container. Do not copy individual commands from chat. Pull the latest `main`,
then run the checked-in script once from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\run-pending-phase4-checks.ps1
```

The script stops on the first failure and currently performs these checks:

1. Download the D-016-approved CREPE development model only when it is absent
   (1,956,735 bytes, approximately 1.96 MB).
2. Verify its exact byte size and SHA-256 before loading it.
3. Run the generated-vocal native CREPE integration probe.
4. Locate the newest existing `cache\native-separation\**\vocals.flac`.
5. If a vocal stem exists, open it read-only, run native CREPE inference, verify
   its content hash is unchanged, and write `melody.json` only beneath
   `cache\native-melody`.
6. If no vocal stem exists, report that only the representative check was
   skipped; do not fail or modify a music folder.

## Expected model identity

- File: `models\crepe-tiny-research.onnx`
- Bytes: `1,956,735`
- SHA-256: `03FB909A5618FA9F19942E20DE749BC8E658A1AC0DF4A778B354C15B6F7A59D4`

## Maintenance rule

Future native Windows-only checks should be appended to the script and this
document. Prefer one final user-run batch over repeatedly asking the user to
paste terminal commands. Never place source music, model binaries, generated
caches, microphone recordings, or the user's reference screenshots in Git.
