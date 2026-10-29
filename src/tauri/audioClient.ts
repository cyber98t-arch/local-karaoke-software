import { invoke } from "@tauri-apps/api/core";

export type AudioDevice = {
  id: string;
  name: string;
  direction: "input" | "output";
  isDefault: boolean;
};

export type AudioClient = {
  listAudioDevices: () => Promise<AudioDevice[]>;
  playOutputProbe: (outputDeviceId: string) => Promise<PlaybackProbeResult>;
};

export type PlaybackProbeResult = {
  deviceName: string;
  sampleRate: number;
  channels: number;
  durationMs: number;
};

export const tauriAudioClient: AudioClient = {
  listAudioDevices: () => invoke<AudioDevice[]>("list_audio_devices"),
  playOutputProbe: (outputDeviceId) => invoke<PlaybackProbeResult>("play_output_probe", { outputDeviceId }),
};
