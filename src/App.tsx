import { useMemo, useState } from "react";
import { Disc3, FolderOpen, Mic2, Search, Settings2, SlidersHorizontal, Sparkles } from "lucide-react";
import { songs } from "./data/songs";
import { tauriLibraryClient, type IndexedSong, type LibraryClient } from "./tauri/libraryClient";
import type { Song } from "./types";

const stateLabel: Record<Song["state"], string> = {
  ready: "歌唱できます",
  processing: "AI解析中",
  waiting: "解析待ち",
};

const importedAccents = ["#f0b65a", "#5dd3c8", "#7ba7ff", "#bb8af7", "#ff819f"];

function formatDuration(durationMs: number) {
  const totalSeconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  return `${minutes}:${String(totalSeconds % 60).padStart(2, "0")}`;
}

function mapIndexedSongs(indexedSongs: IndexedSong[]): Song[] {
  return indexedSongs.map((song, index) => ({
    id: `${song.id}:${index}`,
    title: song.title ?? "タイトル不明",
    artist: song.artist ?? "アーティスト不明",
    duration: formatDuration(song.durationMs),
    key: "未解析",
    format: "LOCAL",
    state: "waiting",
    accent: importedAccents[index % importedAccents.length],
  }));
}

type AppProps = {
  libraryClient?: LibraryClient;
};

function App({ libraryClient = tauriLibraryClient }: AppProps) {
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState("1");
  const [librarySongs, setLibrarySongs] = useState<Song[]>(songs);
  const [importStatus, setImportStatus] = useState<"idle" | "selecting" | "indexing">("idle");
  const [importError, setImportError] = useState<string | null>(null);
  const visibleSongs = useMemo(
    () => librarySongs.filter((song) => `${song.title} ${song.artist}`.toLowerCase().includes(query.toLowerCase())),
    [librarySongs, query],
  );
  const selected = librarySongs.find((song) => song.id === selectedId) ?? librarySongs[0];
  const importing = importStatus !== "idle";

  async function importLibrary() {
    setImportError(null);
    setImportStatus("selecting");
    try {
      const selection = await libraryClient.selectLibraryFolder();
      if (!selection.selected) {
        return;
      }

      setImportStatus("indexing");
      const indexed = await libraryClient.indexSelectedLibrary();
      const importedSongs = mapIndexedSongs(indexed.songs);
      setLibrarySongs(importedSongs);
      setSelectedId(importedSongs[0]?.id ?? "");
      setQuery("");
    } catch {
      setImportError("ライブラリの読み込みに失敗しました。音源フォルダーを選び直してください。");
    } finally {
      setImportStatus("idle");
    }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark"><Mic2 size={20} /></span><span>HIBIKI</span></div>
        <nav aria-label="メインメニュー">
          <button className="nav-item active"><Disc3 size={19} />楽曲を選ぶ</button>
          <button className="nav-item"><Sparkles size={19} />解析キュー<span className="badge">2</span></button>
          <button className="nav-item"><Settings2 size={19} />設定</button>
        </nav>
        <div className="device-card">
          <span className="eyebrow">入力デバイス</span>
          <strong>既定のマイク</strong>
          <span className="level"><i /><i /><i /><i /><i /></span>
          <small>待機中 · CPU採点</small>
        </div>
        <p className="privacy-note">音源と歌声は、このPCの外へ送信されません。</p>
      </aside>

      <section className="content">
        <header>
          <div><span className="eyebrow">LOCAL KARAOKE LIBRARY</span><h1>今日は、何を歌いますか。</h1></div>
          <button className="import-button" disabled={importing} onClick={importLibrary}>
            <FolderOpen size={18} />
            {importStatus === "selecting" ? "フォルダーを選択中" : importStatus === "indexing" ? "ライブラリを索引中" : "音源フォルダーを追加"}
          </button>
        </header>

        {importError && <p className="import-error" role="alert">{importError}</p>}

        <div className="toolbar">
          <label className="search"><Search size={18} /><input aria-label="曲名またはアーティストを検索" placeholder="曲名、アーティストを検索" value={query} onChange={(event) => setQuery(event.target.value)} /></label>
          <button className="filter"><SlidersHorizontal size={17} />すべての曲</button>
          <span className="song-count">{visibleSongs.length}曲</span>
        </div>

        <div className="library-layout">
          <section className="song-list" aria-label="楽曲一覧">
            {visibleSongs.map((song) => (
              <button key={song.id} className={`song-row ${selectedId === song.id ? "selected" : ""}`} onClick={() => setSelectedId(song.id)}>
                <span className="cover" style={{ "--accent": song.accent } as React.CSSProperties}><Disc3 size={24} /></span>
                <span className="song-copy"><strong>{song.title}</strong><small>{song.artist}</small></span>
                <span className="song-meta">{song.format}<small>{song.duration}</small></span>
                <span className={`state ${song.state}`}><i />{stateLabel[song.state]}{song.progress ? ` ${song.progress}%` : ""}</span>
              </button>
            ))}
            {visibleSongs.length === 0 && <div className="empty">一致する楽曲がありません。</div>}
          </section>

          {selected ? <aside className="now-card">
            <div className="hero-art" style={{ "--accent": selected.accent } as React.CSSProperties}><span>選択中</span><Disc3 size={74} strokeWidth={1.2} /></div>
            <span className="eyebrow">SELECTED SONG</span>
            <h2>{selected.title}</h2>
            <p>{selected.artist}</p>
            <div className="facts"><span>キー <strong>{selected.key}</strong></span><span>長さ <strong>{selected.duration}</strong></span></div>
            <div className="cache-status"><i className={selected.state} /><div><strong>{stateLabel[selected.state]}</strong><small>伴奏・メロディ・歌詞をローカル保存</small></div></div>
            <button className="sing-button" disabled={selected.state !== "ready"}><Mic2 size={19} />この曲を歌う</button>
            <small className="score-note">歌唱中は総合点を表示しません。結果は終了後に確認できます。</small>
          </aside> : <aside className="now-card empty-selection"><Disc3 size={48} /><strong>楽曲がありません</strong><small>MP3、FLAC、WAVを含むフォルダーを選択してください。</small></aside>}
        </div>
      </section>
    </main>
  );
}

export default App;

