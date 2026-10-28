import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { OLLAMA_STREAM_EVENT, isOllamaStreamEvent, type ConversationView as Conversation } from "@saiki/ipc-contracts";
import { appendOwnerMessage, appendSaikiMessage, clearConversation, composeConversationContext, getConversation, recordToolResult, startOllamaChat } from "./ipc";
import "./conversation.css";

type RunState = "idle" | "streaming" | "failed" | "unavailable";

export function ConversationView() {
  const [conversation, setConversation] = useState<Conversation>();
  const [model, setModel] = useState("");
  const [draft, setDraft] = useState("");
  const [toolResult, setToolResult] = useState("");
  const [runState, setRunState] = useState<RunState>("idle");
  const [message, setMessage] = useState("Reading persisted conversation…");
  const [streaming, setStreaming] = useState("");
  const requestIdRef = useRef<string | undefined>(undefined);
  const answerRef = useRef("");

  useEffect(() => { void getConversation().then((view) => { setConversation(view); setMessage(view.messages.length ? `${view.messages.length} message${view.messages.length === 1 ? "" : "s"} restored from disk.` : "No conversation yet. Saiki remembers nothing until you speak."); }).catch(fail); }, []);

  useEffect(() => {
    const unlisten = listen<unknown>(OLLAMA_STREAM_EVENT, ({ payload }) => {
      if (!isOllamaStreamEvent(payload) || payload.request_id !== requestIdRef.current) return;
      if (payload.kind === "chunk") { answerRef.current += payload.content; setStreaming(answerRef.current); }
      else if (payload.kind === "finished" && payload.outcome === "succeeded") finishReply();
      else if (payload.kind === "finished") { reset("idle", "Generation cancelled; no reply was stored."); }
      else if (payload.kind === "failed") { reset(payload.outcome === "unavailable" ? "unavailable" : "failed", payload.message); }
    });
    return () => { void unlisten.then((stop) => stop()); };
  }, []);

  function fail(error: unknown): void { setRunState("unavailable"); setMessage(error instanceof Error ? error.message : String(error)); }
  function reset(state: RunState, note: string): void { requestIdRef.current = undefined; answerRef.current = ""; setStreaming(""); setRunState(state); setMessage(note); }

  function finishReply(): void {
    const text = answerRef.current.trim();
    requestIdRef.current = undefined; answerRef.current = ""; setStreaming("");
    if (!text) { setRunState("failed"); setMessage("The model returned no text; nothing was stored."); return; }
    void appendSaikiMessage(text).then((view) => { setConversation(view); setRunState("idle"); setMessage("Saiki replied and the turn was persisted."); }).catch(fail);
  }

  function send(): void {
    if (!model.trim() || !draft.trim() || runState === "streaming") return;
    const content = draft.trim();
    setDraft("");
    void appendOwnerMessage(content).then(async (view) => {
      setConversation(view);
      const context = await composeConversationContext();
      const id = crypto.randomUUID();
      requestIdRef.current = id; answerRef.current = ""; setStreaming(""); setRunState("streaming");
      setMessage(`Persona-framed context assembled from ${context.includedMessages} message${context.includedMessages === 1 ? "" : "s"}; streaming locally…`);
      await startOllamaChat({ requestId: id, model: model.trim(), prompt: context.prompt });
    }).catch((error: unknown) => { requestIdRef.current = undefined; setRunState("failed"); setMessage(error instanceof Error ? error.message : String(error)); });
  }

  function addToolResult(): void {
    const content = toolResult.trim();
    if (!content) return;
    setToolResult("");
    void recordToolResult(content).then((view) => { setConversation(view); setMessage("Factual tool result recorded verbatim; the persona cannot rewrite it."); }).catch(fail);
  }

  function clear(): void { void clearConversation().then((view) => { setConversation(view); reset("idle", "Conversation cleared from disk."); }).catch(fail); }

  if (!conversation) return <div className="view-stack"><p className="provider-message" role="status">{message}</p></div>;
  const { persona, messages, limits } = conversation;

  return <div className="view-stack conversation">
    <div className="section-heading"><div><p className="eyebrow">PERSONA · CONVERSATION</p><h1>Talk to {persona.name}</h1></div><span className="authority-badge">AUTHORITY: {persona.authority.toUpperCase()}</span></div>
    <p className="scope-note">{persona.tone.join(" · ")}. Persona shapes tone only — it never changes facts, tool results, or outcomes, and it cannot authorize an action. History persists on disk (JSON); the SQLite memory foundation is a later phase.</p>
    <p className="provider-message" role="status">{message}</p>

    <div className="transcript" aria-live="polite">
      {messages.length === 0 && <p className="empty-note">Nothing stored yet.</p>}
      {messages.map((item) => <article key={item.id} className={`turn turn-${item.role}`}>
        <header><b>{item.role === "owner" ? "Owner" : item.role === "saiki" ? persona.name : "Tool result (factual)"}</b>{item.expression && <span className="expression-badge">{item.expression}</span>}</header>
        <pre>{item.content}</pre>
      </article>)}
    </div>

    <label className="model-field"><span>Local model identifier (reuses the Phase 06 Ollama adapter)</span><input value={model} maxLength={200} onChange={(event) => setModel(event.target.value)} placeholder="e.g. qwen3:4b" disabled={runState === "streaming"} /></label>
    <label className="model-field"><span>Message to {persona.name}</span><textarea value={draft} maxLength={limits.maxContentBytes} rows={3} onChange={(event) => setDraft(event.target.value)} disabled={runState === "streaming"} /></label>
    <div className="model-actions"><button onClick={send} disabled={!model.trim() || !draft.trim() || runState === "streaming"}>Send</button><button className="quiet-button" onClick={clear} disabled={runState === "streaming" || messages.length === 0}>Clear conversation</button></div>

    {(streaming || runState === "streaming") && <div className="model-response"><h2>{persona.name} is replying</h2><pre>{streaming || "Waiting for the first token…"}</pre></div>}

    <details className="tool-result-panel"><summary>Record a factual tool result</summary>
      <p className="scope-note">A tool result is a fact, stored verbatim and never styled by the persona. The policy executor that will produce these automatically is a later phase; this field exists to exercise the invariant honestly.</p>
      <label className="model-field"><textarea value={toolResult} maxLength={limits.maxContentBytes} rows={2} onChange={(event) => setToolResult(event.target.value)} placeholder="e.g. build failed: exit code 1" /></label>
      <div className="model-actions"><button className="quiet-button" onClick={addToolResult} disabled={!toolResult.trim()}>Record verbatim</button></div>
    </details>
  </div>;
}
