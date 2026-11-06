import { invoke } from "@tauri-apps/api/core";

export type AnalysisStage = "separation" | "melody" | "transcription" | "alignment";

export type AnalysisJobState =
  | { status: "queued" }
  | { status: "running"; stage: AnalysisStage }
  | { status: "completed" }
  | { status: "failed"; stage: AnalysisStage; message: string; retryable: boolean }
  | { status: "cancelled" };

export type AnalysisStatus = {
  songId: string;
  state: AnalysisJobState;
};

export type AnalysisClient = {
  startAnalysis(songId: string): Promise<AnalysisStatus>;
  retryAnalysis(songId: string): Promise<AnalysisStatus>;
  getAnalysisStatus(songId: string): Promise<AnalysisStatus | null>;
  cancelAnalysis(songId: string): Promise<AnalysisStatus>;
};

export const tauriAnalysisClient: AnalysisClient = {
  startAnalysis: (songId) => invoke("start_analysis", { songId }),
  retryAnalysis: (songId) => invoke("retry_analysis", { songId }),
  getAnalysisStatus: (songId) => invoke("get_analysis_status", { songId }),
  cancelAnalysis: (songId) => invoke("cancel_analysis", { songId }),
};
