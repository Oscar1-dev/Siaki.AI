import { useEffect, useState } from "react";
import type { MemoryClaim, Sensitivity } from "@saiki/ipc-contracts";
import { correctMemory, exportMemory, forgetMemory, listMemory, searchMemory, teachMemory } from "./ipc";
import "./memory.css";

const SENSITIVITIES: readonly Sensitivity[] = ["public", "private", "sensitive"];

export function MemoryView() {
  const [claims, setClaims] = useState<readonly MemoryClaim[]>([]);
  const [message, setMessage] = useState("Reading persisted memory…");
  const [query, setQuery] = useState("");
  const [searching, setSearching] = useState(false);
  const [claim, setClaim] = useState("");
  const [source, setSource] = useState("");
  const [confidence, setConfidence] = useState(0.8);
  const [sensitivity, setSensitivity] = useState<Sensitivity>("private");
  const [correcting, setCorrecting] = useState<string>();
  const [exported, setExported] = useState<string>();

  useEffect(() => { void refresh("Memory restored from the local database."); }, []);

  function fail(error: unknown): void { setMessage(error instanceof Error ? error.message : String(error)); }
  async function refresh(note: string): Promise<void> { try { const list = await listMemory(); setClaims(list); setSearching(false); setMessage(note); } catch (error) { fail(error); } }

  function submit(): void {
    if (!claim.trim() || !source.trim()) return;
    const request = { claim: claim.trim(), source: source.trim(), confidence, sensitivity };
    const action = correcting ? correctMemory({ ...request, id: correcting }) : teachMemory(request);
    void action.then(() => { setClaim(""); setCorrecting(undefined); setExported(undefined); return refresh(correcting ? "Claim corrected; the prior version is kept for audit." : "Claim taught and indexed."); }).catch(fail);
  }

  function runSearch(): void {
    const q = query.trim();
    if (!q) { void refresh("Showing all active memory."); return; }
    void searchMemory(q).then((hits) => { setClaims(hits); setSearching(true); setMessage(`${hits.length} active claim${hits.length === 1 ? "" : "s"} match “${q}”.`); }).catch(fail);
  }

  function beginCorrect(target: MemoryClaim): void { setCorrecting(target.id); setClaim(target.claim); setSource(target.source); setConfidence(target.confidence); setSensitivity(target.sensitivity); setExported(undefined); }
  function forget(id: string): void { void forgetMemory(id).then(() => { setExported(undefined); return refresh("Claim forgotten; its content was deleted and the deletion logged."); }).catch(fail); }
  function doExport(): void { void exportMemory().then((data) => { setExported(JSON.stringify(data, null, 2)); setMessage(`Exported ${data.claims.length} claim${data.claims.length === 1 ? "" : "s"} and ${data.events.length} audit event${data.events.length === 1 ? "" : "s"}.`); }).catch(fail); }

  return <div className="view-stack memory">
    <div className="section-heading"><div><p className="eyebrow">MEMORY · SQLITE + FTS5</p><h1>Memory</h1></div><button className="quiet-button" onClick={doExport}>Export</button></div>
    <p className="scope-note">Owner-teachable claims with provenance (source, confidence, sensitivity) and a correction chain. Corrections supersede and keep history; forget deletes content and logs the deletion. Keyword retrieval via FTS5; embeddings are a later phase. Never store passwords or secrets here.</p>
    <p className="provider-message" role="status">{message}</p>

    <div className="memory-search"><input value={query} maxLength={200} placeholder="Search memory…" onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => event.key === "Enter" && runSearch()} /><button onClick={runSearch}>Search</button>{searching && <button className="quiet-button" onClick={() => { setQuery(""); void refresh("Showing all active memory."); }}>Show all</button>}</div>

    <div className="memory-list">
      {claims.length === 0 && <p className="empty-note">No active claims.</p>}
      {claims.map((item) => <article key={item.id} className="claim">
        <p className="claim-text">{item.claim}</p>
        <div className="claim-meta"><span>source: {item.source}</span><span>confidence: {item.confidence.toFixed(2)}</span><span className={`sensitivity sensitivity-${item.sensitivity}`}>{item.sensitivity}</span>{item.supersedes && <span className="supersedes">corrects earlier</span>}</div>
        <div className="claim-actions"><button className="quiet-button" onClick={() => beginCorrect(item)}>Correct</button><button className="danger-button" onClick={() => forget(item.id)}>Forget</button></div>
      </article>)}
    </div>

    <div className="teach-form">
      <h2>{correcting ? "Correct a claim" : "Teach a claim"}</h2>
      <label className="model-field"><span>Claim</span><textarea value={claim} maxLength={4096} rows={2} onChange={(event) => setClaim(event.target.value)} /></label>
      <div className="teach-row">
        <label className="model-field"><span>Source</span><input value={source} maxLength={200} onChange={(event) => setSource(event.target.value)} placeholder="e.g. owner" /></label>
        <label className="model-field"><span>Confidence {confidence.toFixed(2)}</span><input type="range" min={0} max={1} step={0.05} value={confidence} onChange={(event) => setConfidence(Number(event.target.value))} /></label>
        <label className="model-field"><span>Sensitivity</span><select value={sensitivity} onChange={(event) => setSensitivity(event.target.value as Sensitivity)}>{SENSITIVITIES.map((value) => <option key={value} value={value}>{value}</option>)}</select></label>
      </div>
      <div className="model-actions"><button onClick={submit} disabled={!claim.trim() || !source.trim()}>{correcting ? "Save correction" : "Teach"}</button>{correcting && <button className="quiet-button" onClick={() => { setCorrecting(undefined); setClaim(""); }}>Cancel</button>}</div>
    </div>

    {exported && <div className="model-response"><h2>Export (local data)</h2><pre>{exported}</pre></div>}
  </div>;
}
