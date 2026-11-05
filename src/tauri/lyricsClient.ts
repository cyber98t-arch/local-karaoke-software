import { invoke } from "@tauri-apps/api/core";

export type LyricsAcceptance = { sha256: string; byteSize: number };
export type LyricsSelectionResult = { selected: boolean; lyrics: LyricsAcceptance | null };
export type LyricsClient = {
  setPastedLyrics: (songId: string, text: string) => Promise<LyricsAcceptance>;
  selectLyricsFile: (songId: string) => Promise<LyricsSelectionResult>;
};

export const tauriLyricsClient: LyricsClient = {
  setPastedLyrics: (songId, text) => invoke("set_pasted_lyrics", { songId, text }),
  selectLyricsFile: (songId) => invoke("select_lyrics_file", { songId }),
};
