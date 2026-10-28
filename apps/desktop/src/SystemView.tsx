import { useEffect, useState } from "react";
import type { DirEntryInfo, SysToolsStatus } from "@saiki/ipc-contracts";
import { addAllowedApp, configureWorkspace, getSysToolsStatus, launchApp, listWorkspaceDir, readWorkspaceFile, removeAllowedApp, rollbackWrite, writeWorkspaceFile } from "./ipc";
import "./system.css";

export function SystemView() {
  const [tools, setTools] = useState<SysToolsStatus>();
  const [message, setMessage] = useState("Reading system tools…");
  const [root, setRoot] = useState("");
  const [entries, setEntries] = useState<readonly DirEntryInfo[]>([]);
  const [cwd, setCwd] = useState(".");
  const [openPath, setOpenPath] = useState<string>();
  const [content, setContent] = useState("");
  const [lastToken, setLastToken] = useState<string>();
  const [appName, setAppName] = useState("");
  const [appPath, setAppPath] = useState("");

  useEffect(() => { void getSysToolsStatus().then((status) => { setTools(status); setRoot(status.workspaceRoot ?? ""); setMessage(status.workspaceConfigured ? "Workspace ready." : "Configure a workspace root to enable file tools."); if (status.workspaceConfigured && status.workspacePresent) void browse("."); }).catch(fail); }, []);

  function fail(error: unknown): void { setMessage(error instanceof Error ? error.message : String(error)); }
  async function browse(path: string): Promise<void> { try { setEntries(await listWorkspaceDir(path)); setCwd(path); } catch (error) { fail(error); } }

  function saveRoot(): void { if (!root.trim()) return; void configureWorkspace(root.trim()).then((status) => { setTools(status); setMessage("Workspace root configured."); return browse("."); }).catch(fail); }
  function open(entry: DirEntryInfo): void { const next = cwd === "." ? entry.name : `${cwd}/${entry.name}`; if (entry.isDir) { void browse(next); return; } void readWorkspaceFile(next).then((file) => { setOpenPath(next); setContent(file.content); setLastToken(undefined); setMessage(`Opened ${next} (${file.bytes} bytes).`); }).catch(fail); }
  function up(): void { if (cwd === ".") return; const parts = cwd.split("/"); parts.pop(); void browse(parts.length ? parts.join("/") : "."); }
  function save(): void { if (!openPath) return; void writeWorkspaceFile({ path: openPath, content }).then((result) => { setLastToken(result.rollbackToken); setMessage(`Saved ${openPath} (${result.bytes} bytes). ${result.previousExisted ? "Previous content captured" : "New file created"} — rollback available.`); return browse(cwd); }).catch(fail); }
  function undo(): void { if (!lastToken) return; void rollbackWrite(lastToken).then((ok) => { setMessage(ok ? "Rolled back the last write." : "Nothing to roll back."); setLastToken(undefined); if (openPath) void readWorkspaceFile(openPath).then((file) => setContent(file.content)).catch(() => setContent("")); return browse(cwd); }).catch(fail); }
  function addApp(): void { if (!appName.trim() || !appPath.trim()) return; void addAllowedApp({ name: appName.trim(), path: appPath.trim() }).then((status) => { setTools(status); setAppName(""); setAppPath(""); setMessage("Application added to the allowlist."); }).catch(fail); }
  function removeApp(id: string): void { void removeAllowedApp(id).then((status) => { setTools(status); setMessage("Application removed from the allowlist."); }).catch(fail); }
  function launch(id: string): void { void launchApp(id).then((ok) => setMessage(ok ? "Application launched." : "Launch refused.")).catch(fail); }

  if (!tools) return <div className="view-stack"><p className="provider-message" role="status">{message}</p></div>;

  return <div className="view-stack system">
    <div className="section-heading"><div><p className="eyebrow">SYSTEM · WORKSPACE FILES</p><h1>System &amp; files</h1></div><span className="metrics">{tools.metrics.cpuCount} CPU · {tools.metrics.os}/{tools.metrics.arch} · {tools.metrics.pointerWidthBits}-bit</span></div>
    <p className="scope-note">Every file operation is confined to the configured workspace root — absolute paths and “..” are refused, and symlinks are resolved and checked. Writes are reversible via rollback. App launching is allowlist-only.</p>
    <p className="provider-message" role="status">{message}</p>

    <div className="ws-config">
      <label className="model-field"><span>Workspace root (an existing directory)</span><input value={root} maxLength={1024} placeholder="C:\\Users\\me\\saiki-workspace" onChange={(event) => setRoot(event.target.value)} /></label>
      <button onClick={saveRoot} disabled={!root.trim()}>Set workspace</button>
    </div>

    {tools.workspaceConfigured && !tools.workspacePresent && <p className="scope-note warn">The configured workspace root does not currently exist on disk.</p>}

    {tools.workspaceConfigured && tools.workspacePresent && <div className="files">
      <div className="file-browser">
        <div className="browser-head"><b>/{cwd === "." ? "" : cwd}</b>{cwd !== "." && <button className="quiet-button" onClick={up}>Up</button>}</div>
        {entries.length === 0 && <p className="empty-note">Empty.</p>}
        <ul>{entries.map((entry) => <li key={entry.name}><button onClick={() => open(entry)}>{entry.isDir ? "📁" : "📄"} {entry.name}</button></li>)}</ul>
      </div>
      <div className="file-editor">
        {openPath ? <>
          <div className="editor-head"><b>{openPath}</b><div><button onClick={save}>Save</button><button className="quiet-button" onClick={undo} disabled={!lastToken}>Rollback</button></div></div>
          <textarea value={content} onChange={(event) => setContent(event.target.value)} spellCheck={false} />
        </> : <p className="empty-note">Select a file to view and edit, confined to the workspace.</p>}
      </div>
    </div>}

    <div className="app-allowlist">
      <h2>Application allowlist</h2>
      {tools.allowedApps.length === 0 && <p className="empty-note">No applications allowed — launching anything is refused by default.</p>}
      {tools.allowedApps.map((app) => <article key={app.id} className="app"><div><b>{app.name}</b><small>{app.path}</small></div><div className="app-actions"><button onClick={() => launch(app.id)}>Launch</button><button className="danger-button" onClick={() => removeApp(app.id)}>Remove</button></div></article>)}
      <div className="add-app"><input value={appName} maxLength={100} placeholder="Name" onChange={(event) => setAppName(event.target.value)} /><input value={appPath} maxLength={1024} placeholder="Absolute path to .exe" onChange={(event) => setAppPath(event.target.value)} /><button onClick={addApp} disabled={!appName.trim() || !appPath.trim()}>Allow app</button></div>
    </div>
  </div>;
}
