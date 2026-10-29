import { invoke } from "@tauri-apps/api/core";

export type LibrarySelectionResult = {
  selected: boolean;
};

export type IndexedSong = {
  id: string;
  title: string | null;
  artist: string | null;
  durationMs: number;
};

export type LibraryIndexResult = {
  songCount: number;
  songs: IndexedSong[];
};

export type LibraryClient = {
  loadLibraryCatalog: () => Promise<LibraryIndexResult>;
  selectLibraryFolder: () => Promise<LibrarySelectionResult>;
  indexSelectedLibrary: () => Promise<LibraryIndexResult>;
};

export const tauriLibraryClient: LibraryClient = {
  loadLibraryCatalog: () => invoke("load_library_catalog"),
  selectLibraryFolder: () => invoke("select_library_folder"),
  indexSelectedLibrary: () => invoke("index_selected_library"),
};
