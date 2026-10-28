import { useEffect, useState } from "react";
import type { CommandResult, DevToolsStatus, GitStatus } from "@saiki/ipc-contracts";
import { addAllowedCommand, cancelCommand, configureDevTools, gitDiff, gitStatus, getDevToolsStatus, removeAllowedCommand, runCommand } from "./ipc";
import "./devtools.css";

export function DevToolsView() {
  const [tools, setTools] = useState<DevToolsStatus>();
  const [message, setMessage] = useState("Reading developer tools…");
  const [repoRoot, setRepoRoot] = useState("");
  const [git, setGit] = useState<GitStatus>();
  const [diff, setDiff] = useState<string>();
  const [newCommand, setNewCommand] = useState("");
  const [cmd, setCmd] = useState("git");
  const [cmdArgs, setCmdArgs] = useState("status");
  const [result, setResult] = useState<CommandResult>();
  const [running, setRunning] = useState<string>();

  useEffect(() => { void getDevToolsStatus().then((status) => { setTools(status); setRepoRoot(status.repoRoot ?? ""); setMessage(status.repoConfigured ? "Developer tools ready." : "Configure a repository root to begin."); }).catch(fail); }, []);

  function fail(error: unknown): void { setMessage(error instanceof Error ? error.message : String(error)); }
  function saveRepo(): void { if (!repoRoot.trim()) return; void configureDevTools({ repoRoot: repoRoot.trim() }).then((status) => { setTools(status); setMessage(status.isGitRepo ? "Repository configured (git detected)." : "Repository root set (no .git found)."); }).catch(fail); }
  function refreshGit(): void { void gitStatus().then((status) => { setGit(status); setDiff(undefined); setMessage(`On ${status.branch}: ${status.clean ? "clean" : `${status.entries.length} change(s)`}.`); }).catch(fail); }
  function showDiff(): void { void gitDiff().then((d) => { setDiff(d.diff || "(no unstaged changes)"); setMessage(d.truncated ? "Diff shown (truncated)." : "Diff shown."); }).catch(fail); }
  function addCommand(): void { if (!newCommand.trim()) return; void addAllowedCommand(newCommand.trim()).then((status) => { setTools(status); setNewCommand(""); setMessage("Command allowlisted."); }).catch(fail); }
  function removeCommand(name: string): void { void removeAllowedCommand(name).then((status) => { setTools(status); setMessage("Command removed from allowlist."); }).catch(fail); }
  function run(): void {
    if (!cmd.trim()) return;
    const id = crypto.randomUUID();
    const args = cmdArgs.trim() ? cmdArgs.trim().split(/\s+/) : [];
    setRunning(id); setResult(undefined); setMessage(`Running ${cmd.trim()}…`);
    void runCommand({ requestId: id, command: cmd.trim(), args }).then((r) => { setResult(r); setRunning(undefined); setMessage(`Command ${r.outcome.toUpperCase()}.`); }).catch((error: unknown) => { setRunning(undefined); fail(error); });
  }
  function cancel(): void { if (running) void cancelCommand(running).then(() => setMessage("Cancellation requested.")).catch(fail); }

  if (!tools) return <div className="view-stack"><p className="provider-message" role="status">{message}</p></div>;

  return <div className="view-stack devtools">
    <div className="section-heading"><div><p className="eyebrow">DEVELOPER TOOLS</p><h1>Terminal &amp; dev tools</h1></div></div>
    <p className="scope-note">Git analysis is read-only. The bounded shell runs only allowlisted commands, and every run is checked against the policy engine before it executes — without an owner grant for <code>shell.exec</code>, commands are refused. VS Code and Codex integrations are owner-configured or unavailable.</p>
    <p className="provider-message" role="status">{message}</p>

    <div className="repo-config"><label className="model-field"><span>Repository root</span><input value={repoRoot} maxLength={1024} placeholder="C:\\Users\\me\\project" onChange={(event) => setRepoRoot(event.target.value)} /></label><button onClick={saveRepo} disabled={!repoRoot.trim()}>Set repo</button></div>

    <div className="capability-row">
      <span className={`cap ${tools.isGitRepo ? "ok" : "off"}`}>Git {tools.isGitRepo ? "detected" : "not found"}</span>
      <span className={`cap ${tools.vscodePresent ? "ok" : "off"}`}>VS Code {tools.vscodeConfigured ? (tools.vscodePresent ? "ready" : "missing") : "not configured"}</span>
      <span className="cap off">Codex unavailable — {tools.codex.reason}</span>
    </div>

    {tools.isGitRepo && <div className="git-panel">
      <div className="git-actions"><button onClick={refreshGit}>Git status</button><button className="quiet-button" onClick={showDiff}>Git diff</button></div>
      {git && <div className="git-status">{git.clean ? <p className="empty-note">Working tree clean on {git.branch}.</p> : <ul>{git.entries.map((entry) => <li key={entry.path}><code>{entry.status || "·"}</code> {entry.path}</li>)}</ul>}</div>}
      {diff !== undefined && <pre className="git-diff">{diff}</pre>}
    </div>}

    <div className="allowlist">
      <h2>Command allowlist</h2>
      {tools.allowedCommands.length === 0 && <p className="empty-note">No commands allowed — the bounded shell refuses everything by default.</p>}
      <div className="chips">{tools.allowedCommands.map((name) => <span key={name} className="chip">{name}<button onClick={() => removeCommand(name)} aria-label={`Remove ${name}`}>×</button></span>)}</div>
      <div className="add-chip"><input value={newCommand} maxLength={64} placeholder="e.g. npm" onChange={(event) => setNewCommand(event.target.value)} /><button onClick={addCommand} disabled={!newCommand.trim()}>Allow command</button></div>
    </div>

    <div className="runner">
      <h2>Bounded shell (policy-gated)</h2>
      <div className="run-row"><input value={cmd} placeholder="command" onChange={(event) => setCmd(event.target.value)} /><input value={cmdArgs} placeholder="arguments" onChange={(event) => setCmdArgs(event.target.value)} /><button onClick={run} disabled={!!running || !cmd.trim()}>Run</button><button className="quiet-button" onClick={cancel} disabled={!running}>Cancel</button></div>
      {result && <div className={`run-result outcome-${result.outcome}`}><div className="run-head"><b>{result.outcome.toUpperCase()}</b>{result.exitCode !== null && <span>exit {result.exitCode}</span>}</div>{result.stdout && <pre>{result.stdout}</pre>}{result.stderr && <pre className="run-stderr">{result.stderr}</pre>}</div>}
    </div>
  </div>;
}
