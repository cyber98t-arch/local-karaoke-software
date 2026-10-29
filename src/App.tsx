import { useEffect, useMemo, useState } from "react";
import { Disc3, FolderOpen, ListMusic, Mic2, Search, Settings2, SlidersHorizontal } from "lucide-react";
import { songs } from "./data/songs";
import { tauriLibraryClient, type IndexedSong, type LibraryClient } from "./tauri/libraryClient";
import type { Song } from "./types";

const stateLabel: Record<Song["state"], string> = {
  ready: "歌唱できます",
  processing: "解析中",
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
  libraryClient?: Partial<LibraryClient>;
};

function App({ libraryClient = tauriLibraryClient }: AppProps) {
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState("1");
  const [librarySongs, setLibrarySongs] = useState<Song[]>(songs);
  const [importStatus, setImportStatus] = useState<"idle" | "selecting" | "indexing">("idle");
  const [importError, setImportError] = useState<string | null>(null);
  const loadLibraryCatalog = libraryClient
    ? libraryClient.loadLibraryCatalog
    : tauriLibraryClient.loadLibraryCatalog;
  const [catalogLoading, setCatalogLoading] = useState(Boolean(loadLibraryCatalog));
  const visibleSongs = useMemo(
    () => librarySongs.filter((song) => `${song.title} ${song.artist}`.toLowerCase().includes(query.toLowerCase())),
    [librarySongs, query],
  );
  const selected = librarySongs.find((song) => song.id === selectedId) ?? librarySongs[0];
  const importing = catalogLoading || importStatus !== "idle";

  useEffect(() => {
    if (!loadLibraryCatalog) {
      return;
    }

    let active = true;
    loadLibraryCatalog()
      .then((catalog) => {
        if (!active || catalog.songs.length === 0) {
          return;
        }
        const restoredSongs = mapIndexedSongs(catalog.songs);
        setLibrarySongs(restoredSongs);
        setSelectedId(restoredSongs[0].id);
      })
      .catch(() => {
        if (active) {
          setImportError("保存済みライブラリを読み込めませんでした。音源フォルダーを選び直してください。");
        }
      })
      .finally(() => {
        if (active) {
          setCatalogLoading(false);
        }
      });

    return () => {
      active = false;
    };
  }, [loadLibraryCatalog]);

  async function importLibrary() {
    setImportError(null);
    setImportStatus("selecting");
    try {
      const selection = await (libraryClient?.selectLibraryFolder ?? tauriLibraryClient.selectLibraryFolder)();
      if (!selection.selected) {
        return;
      }

      setImportStatus("indexing");
      const indexed = await (libraryClient?.indexSelectedLibrary ?? tauriLibraryClient.indexSelectedLibrary)();
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
        <nav aria-label="メインメニュー">
          <button className="nav-item active"><Disc3 size={19} />楽曲を選ぶ</button>
          <button className="nav-item"><ListMusic size={19} />解析キュー<span className="badge">2</span></button>
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
          <div><h1>曲ライブラリ</h1><p className="page-description">このPCに保存された音源から選択します</p></div>
          <button className="import-button" disabled={importing} onClick={importLibrary}>
            <FolderOpen size={18} />
            {catalogLoading ? "ライブラリを読込中" : importStatus === "selecting" ? "フォルダーを選択中" : importStatus === "indexing" ? "ライブラリを索引中" : "音源フォルダーを追加"}
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
            <table>
              <thead><tr><th>曲名</th><th>アーティスト</th><th>形式</th><th>時間</th><th>状態</th></tr></thead>
              <tbody>
                {visibleSongs.map((song) => (
                  <tr key={song.id} className={selectedId === song.id ? "selected" : ""}>
                    <td><button className="song-select" onClick={() => setSelectedId(song.id)}><Disc3 size={17} /><strong>{song.title}</strong></button></td>
                    <td>{song.artist}</td>
                    <td>{song.format}</td>
                    <td>{song.duration}</td>
                    <td><span className={`state ${song.state}`}><i />{stateLabel[song.state]}{song.progress ? ` ${song.progress}%` : ""}</span></td>
                  </tr>
                ))}
              </tbody>
            </table>
            {visibleSongs.length === 0 && <div className="empty">一致する楽曲がありません。</div>}
          </section>

          {selected ? <aside className="now-card">
            <div className="detail-heading"><span className="detail-icon" style={{ "--accent": selected.accent } as React.CSSProperties}><Disc3 size={24} /></span><span>選択中の曲</span></div>
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

