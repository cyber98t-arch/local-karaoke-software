import { invoke } from "@tauri-apps/api/core";

export type AudioDevice = {
  id: string;
  name: string;
  direction: "input" | "output";
  isDefault: boolean;
};

export type AudioClient = {
  listAudioDevices: () => Promise<AudioDevice[]>;
};

export const tauriAudioClient: AudioClient = {
  listAudioDevices: () => invoke<AudioDevice[]>("list_audio_devices"),
};
