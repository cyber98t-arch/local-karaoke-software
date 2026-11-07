# Pending Windows commands

Updated: 2026-11-06

## Probe Phase 5 hardware without changing the system

Before deciding the GPU-provider strategy, run this read-only probe once from
the repository root and return its JSON output:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\probe-phase5-hardware.ps1
```

The probe reads Windows, CPU, RAM, display-adapter, and driver metadata through
CIM. It does not install or download anything and does not write a file.

## Run the approved DirectML generated-fixture probes

After pulling the D-027 implementation, run this once from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\run-phase5-directml-probes.ps1
```

The first DirectML build downloads the pinned 32,457,673-byte (30.95 MiB)
compressed ONNX Runtime development archive through Cargo. The script uses the
already verified local Spleeter and CREPE models, runs generated/permissive
fixtures, requires DirectML to be the selected provider, prints elapsed times
and output hashes, and leaves CPU fallback intact. It does not open source music
or publish/bundle provider DLLs.

To launch the development app later with DirectML preferred and CPU fallback:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\start-phase4-native-dev.ps1 -DirectML
```

## Run the Phase 4 native development app

The selected-song UI is now connected to the real four-stage local analysis
runner. From the repository root, this single command verifies or downloads the
four pinned development artifacts and starts Tauri with `native-onnx` enabled:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\start-phase4-native-dev.ps1
```

When all four files are absent, the script downloads approximately 220 MiB in
total: Spleeter ONNX (75.0 MiB), CREPE ONNX (1.9 MiB), multilingual Whisper base
(141.1 MiB), and the CPU-only Windows `whisper-cli` (2.0 MiB). Each file is
downloaded to a temporary name, checked against its pinned byte size and
SHA-256, and only then moved under the Git-ignored `models` directory. Existing
files are never replaced silently. Use `-SkipDownload` to prohibit downloads,
or `-VerifyOnly` to validate everything without starting the app.

In the app, choose a music folder, select a song, provide its correct lyrics,
and click **この曲を解析**. The UI reports separation, melody extraction,
transcription, and alignment without exposing a source path. Derived files are
written only to the application analysis cache.

For the current native verification, start an analysis with intentionally
incorrect lyrics, press **解析を中止**, and confirm it reaches the cancelled
state promptly. Save corrected lyrics for the same song, confirm
**修正した歌詞で再解析** appears, and run it. The corrected revision should
finish much faster by validating and reusing the existing stems, melody, and
transcript; only lyric alignment and revision-local job snapshots are new.

This script is for the development environment only. It does not bundle models,
publish an application, or enable a GPU provider.

The native development server uses localhost port 1421. VS Code automatic port
forwarding is disabled; an older forwarding listener remaining on port 1420 can
therefore coexist without closing VS Code.

## Run the older focused CREPE checks

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
