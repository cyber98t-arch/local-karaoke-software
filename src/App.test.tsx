import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";
import App from "./App";

describe("song library", () => {
  it("uses a compact library table with plain analysis wording", () => {
    render(<App libraryClient={{}} />);

    expect(screen.getByRole("columnheader", { name: "曲名" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "アーティスト" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "時間" })).toBeInTheDocument();
    expect(screen.getByText("解析中 68%")).toBeInTheDocument();
    expect(screen.queryByText(/AI解析中/)).not.toBeInTheDocument();
  });

  it("filters songs by Japanese title", () => {
    render(<App libraryClient={{}} />);
    fireEvent.change(screen.getByLabelText("曲名またはアーティストを検索"), { target: { value: "透明" } });
    expect(screen.getByText("透明な街")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /夜明けの航路/ })).not.toBeInTheDocument();
  });

  it("keeps the total score hidden during singing", () => {
    render(<App libraryClient={{}} />);
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

    render(<App libraryClient={{ selectLibraryFolder, indexSelectedLibrary }} />);
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

    render(<App libraryClient={{ selectLibraryFolder, indexSelectedLibrary }} />);
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

    render(<App libraryClient={{ loadLibraryCatalog }} />);

    expect((await screen.findAllByText("保存済みの曲")).length).toBeGreaterThan(0);
    expect(screen.queryByText("夜明けの航路")).not.toBeInTheDocument();
    expect(loadLibraryCatalog).toHaveBeenCalledOnce();
  });
});

