import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { OLLAMA_STREAM_EVENT, isOllamaStreamEvent, type OllamaModel, type OllamaStreamEvent } from "@saiki/ipc-contracts";
import { cancelOllamaChat, discoverOllamaModels, startOllamaChat } from "./ipc";
import "./models.css";
import { CloudProvider } from "./CloudProvider";

type Status = "idle" | "discovering" | "streaming" | "succeeded" | "cancelled" | "unavailable" | "failed";
export function ModelView() {
  const [models, setModels] = useState<readonly OllamaModel[]>([]), [model, setModel] = useState(""), [prompt, setPrompt] = useState(""), [answer, setAnswer] = useState("");
  const [requestId, setRequestId] = useState<string>(), [status, setStatus] = useState<Status>("idle"), [message, setMessage] = useState("Discovery has not run.");
  const [proposals, setProposals] = useState<Extract<OllamaStreamEvent, { kind: "tool_proposal" }>["proposal"][]>([]);
  useEffect(() => { let disposed = false; const unlisten = listen<unknown>(OLLAMA_STREAM_EVENT, ({ payload }) => {
    if (disposed || !isOllamaStreamEvent(payload) || payload.request_id !== requestId) return;
    if (payload.kind === "chunk") setAnswer((current) => current + payload.content);
    else if (payload.kind === "tool_proposal") setProposals((current) => [...current, payload.proposal]);
    else if (payload.kind === "finished") { setStatus(payload.outcome); setMessage(payload.outcome === "succeeded" ? "Local response completed." : "Request cancelled locally."); setRequestId(undefined); }
    else { setStatus(payload.outcome); setMessage(payload.message); setRequestId(undefined); }
  }); return () => { disposed = true; void unlisten.then((stop) => stop()); }; }, [requestId]);
  function discover(): void { setStatus("discovering"); setMessage("Querying local Ollama…"); void discoverOllamaModels().then((found) => { setModels(found); setModel((current) => current || found[0]?.name || ""); setStatus(found.length ? "idle" : "unavailable"); setMessage(found.length ? `${found.length} local model${found.length === 1 ? "" : "s"} available.` : "Ollama is running but no local models are installed."); }).catch((error: unknown) => { setStatus("unavailable"); setMessage(error instanceof Error ? error.message : String(error)); }); }
  function send(): void { if (!model || !prompt.trim() || status === "streaming") return; const id = crypto.randomUUID(); setRequestId(id); setAnswer(""); setProposals([]); setStatus("streaming"); setMessage("Streaming from local Ollama…"); void startOllamaChat({ requestId: id, model, prompt, tools: [], timeoutMs: 120_000 }).catch((error: unknown) => { setRequestId(undefined); setStatus("failed"); setMessage(error instanceof Error ? error.message : String(error)); }); }
  function cancel(): void { if (requestId) void cancelOllamaChat(requestId); }
  return <div className="view-stack model-view"><div className="section-heading"><div><p className="eyebrow">LOCAL INFERENCE</p><h1>Models</h1></div><span className={`model-status ${status}`}>{status.toUpperCase()}</span></div>
    <section className="model-discovery"><div><h2>Ollama</h2><p>{message}</p></div><button onClick={discover} disabled={status === "discovering" || status === "streaming"}>Discover local models</button></section>
    <label className="model-field"><span>Model</span><select value={model} onChange={(event) => setModel(event.target.value)} disabled={!models.length || status === "streaming"}><option value="">No model selected</option>{models.map((item) => <option key={item.digest} value={item.name}>{item.name} · {(item.size / 1_000_000_000).toFixed(1)} GB</option>)}</select></label>
    <label className="model-field"><span>Message</span><textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} maxLength={32_768} rows={5} placeholder="Send a message to a model running on this PC." /></label>
    <div className="model-actions"><button onClick={send} disabled={!model || !prompt.trim() || status === "streaming"}>Send locally</button><button className="quiet-button" onClick={cancel} disabled={status !== "streaming" || !requestId}>Cancel</button></div>
    {(answer || status === "streaming") && <section className="model-response" aria-live="polite"><h2>Response</h2><pre>{answer || "Waiting for the first token…"}</pre></section>}
    {proposals.length > 0 && <section className="proposal-list"><h2>Untrusted tool proposals</h2>{proposals.map((proposal, index) => <article key={`${proposal.name}-${index}`}><b>{proposal.name}</b><code>{JSON.stringify(proposal.arguments)}</code><span>NOT EXECUTED</span></article>)}</section>}
    <p className="scope-note">Requests stay on 127.0.0.1. Model tool calls are displayed as untrusted proposals and cannot authorize or execute an action.</p>
    <CloudProvider />
  </div>;
}
