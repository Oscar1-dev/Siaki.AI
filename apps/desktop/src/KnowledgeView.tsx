import { useEffect, useState } from "react";
import type { Citation, DocumentSummary, Sensitivity } from "@saiki/ipc-contracts";
import { forgetDocument, ingestDocument, listDocuments, retrieveKnowledge } from "./ipc";
import "./knowledge.css";

const SENSITIVITIES: readonly Sensitivity[] = ["public", "private", "sensitive"];

export function KnowledgeView() {
  const [documents, setDocuments] = useState<readonly DocumentSummary[]>([]);
  const [message, setMessage] = useState("Reading ingested documents…");
  const [path, setPath] = useState("");
  const [sensitivity, setSensitivity] = useState<Sensitivity>("private");
  const [query, setQuery] = useState("");
  const [includeSensitive, setIncludeSensitive] = useState(false);
  const [citations, setCitations] = useState<readonly Citation[]>();

  useEffect(() => { void refresh("Document knowledge restored from the local index."); }, []);

  function fail(error: unknown): void { setMessage(error instanceof Error ? error.message : String(error)); }
  async function refresh(note: string): Promise<void> { try { setDocuments(await listDocuments()); setMessage(note); } catch (error) { fail(error); } }

  function ingest(): void {
    const value = path.trim();
    if (!value) return;
    void ingestDocument({ path: value, sensitivity }).then((doc) => { setPath(""); return refresh(`Ingested “${doc.title}” into ${doc.chunkCount} chunk${doc.chunkCount === 1 ? "" : "s"}.`); }).catch(fail);
  }

  function forget(id: string): void { void forgetDocument(id).then(() => { setCitations(undefined); return refresh("Document and its chunks removed from the index."); }).catch(fail); }

  function retrieve(): void {
    const q = query.trim();
    if (!q) return;
    void retrieveKnowledge({ query: q, includeSensitive }).then((hits) => { setCitations(hits); setMessage(hits.length ? `${hits.length} cited passage${hits.length === 1 ? "" : "s"} for “${q}”.` : `No ingested document matches “${q}”.`); }).catch(fail);
  }

  return <div className="view-stack knowledge">
    <div className="section-heading"><div><p className="eyebrow">KNOWLEDGE · LOCAL DOCUMENTS</p><h1>Document knowledge</h1></div></div>
    <p className="scope-note">Ingest local text files; they are chunked, embedded (deterministic local lexical vectors), and retrieved by cosine similarity. Every result cites the actual source document and chunk. Sensitive documents are excluded from retrieval unless you opt in. Never ingest passwords or secrets.</p>
    <p className="provider-message" role="status">{message}</p>

    <div className="ingest-form">
      <label className="model-field"><span>Local document path (UTF-8 text, ≤ 2 MB)</span><input value={path} maxLength={1024} placeholder="e.g. C:\\Users\\me\\notes\\handbook.txt" onChange={(event) => setPath(event.target.value)} /></label>
      <div className="ingest-row">
        <label className="model-field"><span>Privacy label</span><select value={sensitivity} onChange={(event) => setSensitivity(event.target.value as Sensitivity)}>{SENSITIVITIES.map((value) => <option key={value} value={value}>{value}</option>)}</select></label>
        <button onClick={ingest} disabled={!path.trim()}>Ingest document</button>
      </div>
    </div>

    <div className="doc-list">
      <h2>Ingested documents</h2>
      {documents.length === 0 && <p className="empty-note">No documents ingested yet.</p>}
      {documents.map((doc) => <article key={doc.id} className="doc">
        <div><b>{doc.title}</b><small>{doc.path}</small></div>
        <div className="doc-meta"><span className={`sensitivity sensitivity-${doc.sensitivity}`}>{doc.sensitivity}</span><span>{doc.chunkCount} chunks</span><button className="danger-button" onClick={() => forget(doc.id)}>Forget</button></div>
      </article>)}
    </div>

    <div className="retrieve-form">
      <h2>Ask across documents</h2>
      <div className="memory-search"><input value={query} maxLength={400} placeholder="Retrieve cited passages…" onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => event.key === "Enter" && retrieve()} /><button onClick={retrieve} disabled={!query.trim()}>Retrieve</button></div>
      <label className="switch-inline"><input type="checkbox" checked={includeSensitive} onChange={(event) => setIncludeSensitive(event.target.checked)} /> Include documents labelled sensitive</label>
    </div>

    {citations && <div className="citation-list">
      {citations.length === 0 && <p className="empty-note">No citations — nothing ingested matches, so nothing is fabricated.</p>}
      {citations.map((cite, index) => <article key={`${cite.documentId}-${cite.chunkIndex}-${index}`} className="citation">
        <header><b>{cite.title}</b><span className="score">score {cite.score.toFixed(3)}</span></header>
        <small className="cite-source">{cite.path} · chunk {cite.chunkIndex} · <span className={`sensitivity sensitivity-${cite.sensitivity}`}>{cite.sensitivity}</span></small>
        <p>{cite.snippet}</p>
      </article>)}
    </div>}
  </div>;
}
