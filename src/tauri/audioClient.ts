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
  probeInputLevel: (inputDeviceId: string) => Promise<InputProbeResult>;
};

export type PlaybackProbeResult = {
  deviceName: string;
  sampleRate: number;
  channels: number;
  durationMs: number;
};

export type InputProbeResult = {
  deviceName: string;
  sampleRate: number;
  channels: number;
  durationMs: number;
  callbackCount: number;
  averageCallbackIntervalMs: number;
  peakLevel: number;
};

export const tauriAudioClient: AudioClient = {
  listAudioDevices: () => invoke<AudioDevice[]>("list_audio_devices"),
  playOutputProbe: (outputDeviceId) => invoke<PlaybackProbeResult>("play_output_probe", { outputDeviceId }),
  probeInputLevel: (inputDeviceId) => invoke<InputProbeResult>("probe_input_level", { inputDeviceId }),
};
