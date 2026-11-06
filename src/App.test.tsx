import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";
import App from "./App";

describe("song library", () => {
  it("shows enumerated input and output devices without starting a stream", async () => {
    const listAudioDevices = vi.fn().mockResolvedValue([
      { id: "mic-1", name: "Built-in Microphone", direction: "input", isDefault: true },
      { id: "out-1", name: "Laptop Speakers", direction: "output", isDefault: true },
    ]);

    render(<App libraryClient={{}} audioClient={{ listAudioDevices }} />);

    expect(await screen.findByRole("combobox", { name: "入力デバイス" })).toHaveValue("mic-1");
    expect(screen.getByRole("combobox", { name: "出力デバイス" })).toHaveValue("out-1");
    expect(screen.getByRole("option", { name: "Built-in Microphone（既定）" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Laptop Speakers（既定）" })).toBeInTheDocument();
    expect(listAudioDevices).toHaveBeenCalledOnce();
  });

  it("runs a bounded output probe only for the selected output device", async () => {
    const listAudioDevices = vi.fn().mockResolvedValue([
      { id: "out-1", name: "Laptop Speakers", direction: "output", isDefault: true },
    ]);
    const playOutputProbe = vi.fn().mockResolvedValue({
      deviceName: "Laptop Speakers", sampleRate: 48_000, channels: 2, durationMs: 500,
    });

    render(<App libraryClient={{}} audioClient={{ listAudioDevices, playOutputProbe }} />);
    fireEvent.click(await screen.findByRole("button", { name: "テスト音を再生" }));

    expect(await screen.findByRole("status")).toHaveTextContent("テスト音の再生が完了しました");
    expect(playOutputProbe).toHaveBeenCalledWith("out-1");
    expect(playOutputProbe).toHaveBeenCalledOnce();
  });

  it("measures microphone level without starting audible monitoring", async () => {
    const listAudioDevices = vi.fn().mockResolvedValue([
      { id: "mic-1", name: "Built-in Microphone", direction: "input", isDefault: true },
    ]);
    const probeInputLevel = vi.fn().mockResolvedValue({
      deviceName: "Built-in Microphone", sampleRate: 48_000, channels: 1, durationMs: 1_500,
      callbackCount: 75, averageCallbackIntervalMs: 20, peakLevel: 0.42,
    });

    render(<App libraryClient={{}} audioClient={{ listAudioDevices, probeInputLevel }} />);
    fireEvent.click(await screen.findByRole("button", { name: "マイクを確認" }));

    expect(await screen.findByRole("status")).toHaveTextContent("マイク入力を検出しました（ピーク 42% · 75回 · 平均20.0ms）");
    expect(probeInputLevel).toHaveBeenCalledWith("mic-1");
    expect(probeInputLevel).toHaveBeenCalledOnce();
  });

  it("does not claim that microphone audio was detected when callbacks contain silence", async () => {
    const listAudioDevices = vi.fn().mockResolvedValue([
      { id: "mic-1", name: "Muted Microphone", direction: "input", isDefault: true },
    ]);
    const probeInputLevel = vi.fn().mockResolvedValue({
      deviceName: "Muted Microphone", sampleRate: 48_000, channels: 1, durationMs: 1_500,
      callbackCount: 147, averageCallbackIntervalMs: 10.2, peakLevel: 0,
    });

    render(<App libraryClient={{}} audioClient={{ listAudioDevices, probeInputLevel }} />);
    fireEvent.click(await screen.findByRole("button", { name: "マイクを確認" }));

    expect(await screen.findByRole("status")).toHaveTextContent("音声レベルは検出されませんでした（ピーク 0% · 147回 · 平均10.2ms）");
    expect(screen.queryByText(/マイク入力を検出しました/)).not.toBeInTheDocument();
  });

  it("uses a compact library table with plain analysis wording", () => {
    render(<App libraryClient={{}} audioClient={{}} />);

    expect(screen.getByRole("columnheader", { name: "曲名" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "アーティスト" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "時間" })).toBeInTheDocument();
    expect(screen.getByText("解析中 68%")).toBeInTheDocument();
    expect(screen.queryByText(/AI解析中/)).not.toBeInTheDocument();
  });

  it("filters songs by Japanese title", () => {
    render(<App libraryClient={{}} audioClient={{}} />);
    fireEvent.change(screen.getByLabelText("曲名またはアーティストを検索"), { target: { value: "透明" } });
    expect(screen.getByText("透明な街")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /夜明けの航路/ })).not.toBeInTheDocument();
  });

  it("keeps the total score hidden during singing", () => {
    render(<App libraryClient={{}} audioClient={{}} />);
    expect(screen.getByText(/歌唱中は総合点を表示しません/)).toBeInTheDocument();
    expect(screen.queryByText(/総合点: \d/)).not.toBeInTheDocument();
  });

  it("indexes and renders native library metadata after folder selection", async () => {
    const selectLibraryFolder = vi.fn().mockResolvedValue({ selected: true });
    const indexSelectedLibrary = vi.fn().mockResolvedValue({
      songCount: 1,
      songs: [
        {
          id: "a".repeat(64),
          title: "ローカル曲",
          artist: "端末内アーティスト",
          durationMs: 125_000,
        },
      ],
    });

    render(<App libraryClient={{ selectLibraryFolder, indexSelectedLibrary }} audioClient={{}} />);
    fireEvent.click(screen.getByRole("button", { name: "音源フォルダーを追加" }));

    expect((await screen.findAllByText("ローカル曲")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("端末内アーティスト").length).toBeGreaterThan(0);
    expect(screen.getAllByText("2:05").length).toBeGreaterThan(0);
    expect(selectLibraryFolder).toHaveBeenCalledOnce();
    expect(indexSelectedLibrary).toHaveBeenCalledOnce();
    expect(selectLibraryFolder.mock.invocationCallOrder[0]).toBeLessThan(
      indexSelectedLibrary.mock.invocationCallOrder[0],
    );
  });

  it("does not index or replace the library when folder selection is cancelled", async () => {
    const selectLibraryFolder = vi.fn().mockResolvedValue({ selected: false });
    const indexSelectedLibrary = vi.fn();

    render(<App libraryClient={{ selectLibraryFolder, indexSelectedLibrary }} audioClient={{}} />);
    fireEvent.click(screen.getByRole("button", { name: "音源フォルダーを追加" }));

    await waitFor(() => expect(selectLibraryFolder).toHaveBeenCalledOnce());
    expect(indexSelectedLibrary).not.toHaveBeenCalled();
    expect(screen.getAllByText("夜明けの航路").length).toBeGreaterThan(0);
  });

  it("restores path-free song metadata from the existing catalog on startup", async () => {
    const loadLibraryCatalog = vi.fn().mockResolvedValue({
      songCount: 1,
      songs: [
        {
          id: "b".repeat(64),
          title: "保存済みの曲",
          artist: "ローカル歌手",
          durationMs: 181_000,
        },
      ],
    });

    render(<App libraryClient={{ loadLibraryCatalog }} audioClient={{}} />);

    expect((await screen.findAllByText("保存済みの曲")).length).toBeGreaterThan(0);
    expect(screen.queryByText("夜明けの航路")).not.toBeInTheDocument();
    expect(loadLibraryCatalog).toHaveBeenCalledOnce();
  });

  it("submits pasted lyrics by song hash without returning a path", async () => {
    const songId = "c".repeat(64);
    const loadLibraryCatalog = vi.fn().mockResolvedValue({
      songCount: 1,
      songs: [{ id: songId, title: "歌詞テスト", artist: "ローカル歌手", durationMs: 60_000 }],
    });
    const setPastedLyrics = vi.fn().mockResolvedValue({ sha256: "d".repeat(64), byteSize: 9 });
    render(<App libraryClient={{ loadLibraryCatalog }} audioClient={{}} lyricsClient={{ setPastedLyrics }} />);

    fireEvent.change(await screen.findByLabelText("正しい歌詞"), { target: { value: "今日は" } });
    fireEvent.click(screen.getByRole("button", { name: "貼り付けた歌詞を使用" }));

    expect(await screen.findByRole("status")).toHaveTextContent("歌詞をローカルで受け付けました");
    expect(setPastedLyrics).toHaveBeenCalledWith(songId, "今日は");
  });

  it("keeps lyrics unchanged when native file selection is cancelled", async () => {
    const selectLyricsFile = vi.fn().mockResolvedValue({ selected: false, lyrics: null });
    render(<App libraryClient={{}} audioClient={{}} lyricsClient={{ selectLyricsFile }} />);

    fireEvent.click(screen.getByRole("button", { name: ".txtを選択" }));

    await waitFor(() => expect(selectLyricsFile).toHaveBeenCalledOnce());
    expect(screen.queryByText("歌詞をローカルで受け付けました")).not.toBeInTheDocument();
  });

  it("starts four-stage analysis only after authoritative lyrics are accepted", async () => {
    const songId = "e".repeat(64);
    const loadLibraryCatalog = vi.fn().mockResolvedValue({
      songCount: 1,
      songs: [{ id: songId, title: "解析する曲", artist: "ローカル歌手", durationMs: 60_000 }],
    });
    const setPastedLyrics = vi.fn().mockResolvedValue({ sha256: "f".repeat(64), byteSize: 9 });
    const startAnalysis = vi.fn().mockResolvedValue({ songId, state: { status: "queued" } });
    const getAnalysisStatus = vi.fn().mockResolvedValue({
      songId,
      state: { status: "running", stage: "separation" },
    });
    render(<App
      libraryClient={{ loadLibraryCatalog }}
      audioClient={{}}
      lyricsClient={{ setPastedLyrics }}
      analysisClient={{ startAnalysis, getAnalysisStatus }}
    />);

    const startButton = await screen.findByRole("button", { name: "この曲を解析" });
    expect(startButton).toBeDisabled();
    fireEvent.change(screen.getByLabelText("正しい歌詞"), { target: { value: "今日は" } });
    fireEvent.click(screen.getByRole("button", { name: "貼り付けた歌詞を使用" }));
    await waitFor(() => expect(startButton).toBeEnabled());
    fireEvent.click(startButton);

    await waitFor(() => expect(startAnalysis).toHaveBeenCalledWith(songId));
    expect(await screen.findByText("音源をボーカルと伴奏に分離中")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("value", "15");
  });

  it("allows an active analysis to be cancelled", async () => {
    const songId = "a".repeat(64);
    const loadLibraryCatalog = vi.fn().mockResolvedValue({
      songCount: 1,
      songs: [{ id: songId, title: "中止する曲", artist: "ローカル歌手", durationMs: 60_000 }],
    });
    const setPastedLyrics = vi.fn().mockResolvedValue({ sha256: "b".repeat(64), byteSize: 9 });
    const startAnalysis = vi.fn().mockResolvedValue({
      songId,
      state: { status: "running", stage: "melody" },
    });
    const getAnalysisStatus = vi.fn().mockResolvedValue({
      songId,
      state: { status: "running", stage: "melody" },
    });
    const cancelAnalysis = vi.fn().mockResolvedValue({
      songId,
      state: { status: "running", stage: "melody" },
    });
    render(<App
      libraryClient={{ loadLibraryCatalog }}
      audioClient={{}}
      lyricsClient={{ setPastedLyrics }}
      analysisClient={{ startAnalysis, getAnalysisStatus, cancelAnalysis }}
    />);

    fireEvent.change(await screen.findByLabelText("正しい歌詞"), { target: { value: "今日は" } });
    fireEvent.click(screen.getByRole("button", { name: "貼り付けた歌詞を使用" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "この曲を解析" })).toBeEnabled());
    fireEvent.click(screen.getByRole("button", { name: "この曲を解析" }));
    fireEvent.click(await screen.findByRole("button", { name: "解析を中止" }));

    expect(cancelAnalysis).toHaveBeenCalledWith(songId);
  });

  it("offers retry only when a failed stage is retryable", async () => {
    const songId = "c".repeat(64);
    const loadLibraryCatalog = vi.fn().mockResolvedValue({
      songCount: 1,
      songs: [{ id: songId, title: "再試行する曲", artist: "ローカル歌手", durationMs: 60_000 }],
    });
    const setPastedLyrics = vi.fn().mockResolvedValue({ sha256: "d".repeat(64), byteSize: 9 });
    const startAnalysis = vi.fn().mockResolvedValue({ songId, state: { status: "queued" } });
    const retryAnalysis = vi.fn().mockResolvedValue({ songId, state: { status: "queued" } });
    const getAnalysisStatus = vi.fn().mockResolvedValue({
      songId,
      state: { status: "failed", stage: "transcription", message: "whisper-cli timed out", retryable: true },
    });
    render(<App
      libraryClient={{ loadLibraryCatalog }}
      audioClient={{}}
      lyricsClient={{ setPastedLyrics }}
      analysisClient={{ startAnalysis, retryAnalysis, getAnalysisStatus }}
    />);

    fireEvent.change(await screen.findByLabelText("正しい歌詞"), { target: { value: "今日は" } });
    fireEvent.click(screen.getByRole("button", { name: "貼り付けた歌詞を使用" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "この曲を解析" })).toBeEnabled());
    fireEvent.click(screen.getByRole("button", { name: "この曲を解析" }));

    const retry = await screen.findByRole("button", { name: "解析を再試行" });
    expect(screen.getByRole("alert")).toHaveTextContent("whisper-cli timed out");
    fireEvent.click(retry);
    await waitFor(() => expect(retryAnalysis).toHaveBeenCalledWith(songId));
    expect(startAnalysis).toHaveBeenCalledOnce();
  });
});

