import { useMemo, useState } from "react";
import { Disc3, FolderOpen, Mic2, Search, Settings2, SlidersHorizontal, Sparkles } from "lucide-react";
import { songs } from "./data/songs";
import type { Song } from "./types";

const stateLabel: Record<Song["state"], string> = {
  ready: "歌唱できます",
  processing: "AI解析中",
  waiting: "解析待ち",
};

function App() {
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState("1");
  const visibleSongs = useMemo(
    () => songs.filter((song) => `${song.title} ${song.artist}`.toLowerCase().includes(query.toLowerCase())),
    [query],
  );
  const selected = songs.find((song) => song.id === selectedId) ?? songs[0];

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
          <button className="import-button"><FolderOpen size={18} />音源フォルダーを追加</button>
        </header>

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

          <aside className="now-card">
            <div className="hero-art" style={{ "--accent": selected.accent } as React.CSSProperties}><span>選択中</span><Disc3 size={74} strokeWidth={1.2} /></div>
            <span className="eyebrow">SELECTED SONG</span>
            <h2>{selected.title}</h2>
            <p>{selected.artist}</p>
            <div className="facts"><span>キー <strong>{selected.key}</strong></span><span>長さ <strong>{selected.duration}</strong></span></div>
            <div className="cache-status"><i className={selected.state} /><div><strong>{stateLabel[selected.state]}</strong><small>伴奏・メロディ・歌詞をローカル保存</small></div></div>
            <button className="sing-button" disabled={selected.state !== "ready"}><Mic2 size={19} />この曲を歌う</button>
            <small className="score-note">歌唱中は総合点を表示しません。結果は終了後に確認できます。</small>
          </aside>
        </div>
      </section>
    </main>
  );
}

export default App;

