import type { Song } from "../types";

export const songs: Song[] = [
  { id: "1", title: "夜明けの航路", artist: "Aster Lane", duration: "4:12", key: "+1", format: "FLAC", state: "ready", accent: "#f0b65a" },
  { id: "2", title: "透明な街", artist: "Mizuki", duration: "3:48", key: "±0", format: "MP3", state: "ready", accent: "#5dd3c8" },
  { id: "3", title: "群青レコード", artist: "Northbound", duration: "5:03", key: "-2", format: "WAV", state: "processing", progress: 68, accent: "#7ba7ff" },
  { id: "4", title: "雨音とコーヒー", artist: "白昼夢", duration: "4:36", key: "±0", format: "FLAC", state: "waiting", accent: "#bb8af7" },
  { id: "5", title: "Starry Signal", artist: "Lumen", duration: "3:29", key: "+2", format: "MP3", state: "ready", accent: "#ff819f" },
];

