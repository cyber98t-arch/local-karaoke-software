export type AnalysisState = "ready" | "processing" | "waiting";

export type Song = {
  id: string;
  title: string;
  artist: string;
  duration: string;
  key: string;
  format: "MP3" | "FLAC" | "WAV" | "LOCAL";
  state: AnalysisState;
  progress?: number;
  accent: string;
};

