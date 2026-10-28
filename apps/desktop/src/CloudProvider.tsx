import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { OPENAI_STREAM_EVENT, isCloudStreamEvent, type CloudStatus } from "@saiki/ipc-contracts";
import { cancelCloudChat, configureCloudProvider, deleteOpenAiKey, getCloudStatus, importOpenAiKeyFromEnvironment, startCloudChat } from "./ipc";
import "./cloud.css";

type RunState = "idle" | "streaming" | "succeeded" | "cancelled" | "failed" | "unavailable";
export function CloudProvider() {
  const [cloud, setCloud] = useState<CloudStatus>(), [model, setModel] = useState(""), [accepted, setAccepted] = useState(false);
  const [prompt, setPrompt] = useState(""), [answer, setAnswer] = useState(""), [requestId, setRequestId] = useState<string>();
  const [runState, setRunState] = useState<RunState>("idle"), [message, setMessage] = useState("Reading owner cloud configuration…");
  useEffect(() => { void getCloudStatus().then((status) => { setCloud(status); setModel(status.model); setMessage(status.enabled ? "OpenAI is owner-enabled." : "OpenAI is disabled."); }).catch((error: unknown) => { setRunState("unavailable"); setMessage(error instanceof Error ? error.message : String(error)); }); }, []);
  useEffect(() => { let disposed = false; const unlisten = listen<unknown>(OPENAI_STREAM_EVENT, ({ payload }) => {
    if (disposed || !isCloudStreamEvent(payload) || payload.request_id !== requestId) return;
    if (payload.kind === "chunk") setAnswer((current) => current + payload.content);
    else if (payload.kind === "finished") { setRunState(payload.outcome); setMessage(payload.outcome === "succeeded" ? "OpenAI response completed." : "OpenAI request cancelled locally."); setRequestId(undefined); }
    else { setRunState(payload.outcome); setMessage(payload.message); setRequestId(undefined); }
  }); return () => { disposed = true; void unlisten.then((stop) => stop()); }; }, [requestId]);
  function update(action: () => Promise<CloudStatus>, success: string): void { void action().then((status) => { setCloud(status); setModel(status.model); setMessage(success); setRunState("idle"); }).catch((error: unknown) => { setRunState("failed"); setMessage(error instanceof Error ? error.message : String(error)); }); }
  function enable(enabled: boolean): void { update(() => configureCloudProvider({ enabled, model, disclosureAccepted: enabled && accepted }), enabled ? "OpenAI is owner-enabled." : "OpenAI is disabled."); }
  function send(): void { if (!cloud?.enabled || !prompt.trim() || runState === "streaming") return; const id = crypto.randomUUID(); setRequestId(id); setAnswer(""); setRunState("streaming"); setMessage("Streaming from OpenAI…"); void startCloudChat({ requestId: id, prompt, timeoutMs: 120_000 }).catch((error: unknown) => { setRequestId(undefined); setRunState("failed"); setMessage(error instanceof Error ? error.message : String(error)); }); }
  return <section className="cloud-provider"><div className="provider-title"><div><p className="eyebrow">OPTIONAL EXTERNAL PROVIDER</p><h2>OpenAI API</h2></div><span className={`model-status ${cloud?.enabled ? "succeeded" : runState}`}>{cloud?.enabled ? "ENABLED" : "DISABLED"}</span></div>
    <p className="provider-message" role="status">{message}</p>
    {cloud && <div className="disclosure"><b>Data disclosure</b><p>When enabled and you press “Send to OpenAI”, SAIKI sends {cloud.disclosure.data.join(" and ")} to <strong>{cloud.disclosure.destination}</strong>. API storage is requested as <code>false</code>.</p><p>{cloud.disclosure.billing}</p><label><input type="checkbox" checked={accepted} onChange={(event) => setAccepted(event.target.checked)} /> I understand this external data transfer and billing disclosure.</label></div>}
    <div className="credential-row"><div><b>Credential</b><small>{cloud?.credentialPresent ? "Stored in Windows Credential Manager" : "Not configured"}</small></div><button onClick={() => update(importOpenAiKeyFromEnvironment, "Credential imported into Windows Credential Manager.")}>Import OPENAI_API_KEY</button><button className="danger-button" disabled={!cloud?.credentialPresent} onClick={() => update(deleteOpenAiKey, "Credential deleted and OpenAI disabled.")}>Delete credential</button></div>
    <p className="scope-note">For setup, launch SAIKI once with <code>OPENAI_API_KEY</code> in its process environment, use Import, then restart without that environment variable. The value never crosses IPC or enters renderer state.</p>
    <label className="model-field"><span>Owner-selected API model identifier</span><input value={model} maxLength={200} onChange={(event) => setModel(event.target.value)} disabled={cloud?.enabled || runState === "streaming"} placeholder="Enter a model available to your API project" /></label>
    <div className="model-actions"><button disabled={!cloud?.credentialPresent || !model.trim() || !accepted || cloud?.enabled} onClick={() => enable(true)}>Enable OpenAI</button><button className="quiet-button" disabled={!cloud?.enabled || runState === "streaming"} onClick={() => enable(false)}>Disable OpenAI</button></div>
    <label className="model-field"><span>Message sent only when you press Send</span><textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} maxLength={32_768} rows={4} disabled={!cloud?.enabled} /></label>
    <div className="model-actions"><button onClick={send} disabled={!cloud?.enabled || !prompt.trim() || runState === "streaming"}>Send to OpenAI</button><button className="quiet-button" onClick={() => requestId && void cancelCloudChat(requestId)} disabled={runState !== "streaming" || !requestId}>Cancel</button></div>
    {(answer || runState === "streaming") && <div className="model-response" aria-live="polite"><h2>Cloud response</h2><pre>{answer || "Waiting for the first event…"}</pre></div>}
  </section>;
}
