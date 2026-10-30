import { useEffect, useState } from "react";
import { inspectRigManifestV2 } from "@saiki/character-rig";
import { CHARACTER_DEBUG_STORAGE_KEY, CHARACTER_MANIFEST_URL, REQUIRED_CHARACTER_FILES } from "./character/asset-foundation";
import "./character-assets.css";

type Audit = { state: "checking" | "ready" | "unavailable"; detail: string; assetCount: number; layerCount: number };
export function CharacterAssetsView() {
  const [audit, setAudit] = useState<Audit>({ state: "checking", detail: "Checking the production asset package…", assetCount: 0, layerCount: 0 });
  const [debug, setDebug] = useState(() => localStorage.getItem(CHARACTER_DEBUG_STORAGE_KEY) === "true");
  useEffect(() => { void auditPackage().then(setAudit); }, []);
  function updateDebug(enabled: boolean) { localStorage.setItem(CHARACTER_DEBUG_STORAGE_KEY, String(enabled)); setDebug(enabled); }
  return <div className="view-stack character-assets">
    <header><div><p className="eyebrow">PHASE 22.5A</p><h1>Character asset foundation</h1><p>Production layered artwork only. The procedural capsule is not loaded.</p></div><strong className={audit.state}>{audit.state.toUpperCase()}</strong></header>
    <section className="asset-status"><h2>Runtime package</h2><p>{audit.detail}</p><dl><dt>Manifest</dt><dd>{CHARACTER_MANIFEST_URL}</dd><dt>Validated assets</dt><dd>{audit.assetCount}</dd><dt>Validated layers</dt><dd>{audit.layerCount}</dd><dt>Runtime</dt><dd>pixi-layered-v2</dd></dl><label><span><b>Layer debug overlay</b><small>Shows sprite bounds, semantic layer names, and pivots in the overlay window.</small></span><input type="checkbox" checked={debug} disabled={audit.state !== "ready"} onChange={(event) => updateDebug(event.target.checked)} /></label></section>
    <section className="reference-candidate"><div><p className="eyebrow">REFERENCE CANDIDATE — NOT A RIG</p><h2>Generated full-body source</h2><p>This admitted image is a real visual reference for mask creation and layer cleanup. It is not loaded by the overlay and does not satisfy Phase 22.5B.</p></div><div className="reference-frame"><img src="/character/saiki-kusuo/reference/reference-clean-v1.webp" alt="AI-generated full-body Saiki character reference candidate" /></div></section>
    <section><h2>Artwork handoff — {REQUIRED_CHARACTER_FILES.length} files</h2><p>Author sources under <code>art/character/saiki-kusuo/</code>; the build command publishes validated runtime exports. Transparent working layers retain identical canvas dimensions and registered pivots.</p><ol className="asset-checklist">{REQUIRED_CHARACTER_FILES.map((file) => <li key={file}><code>{file}</code></li>)}</ol></section>
    <p className="scope-note">Artwork and usage rights are still required. Manifest validation, digest checking, hierarchy construction, full-body/floating visibility profiles, and debug tooling are implemented; Phase 22.5B animation is not.</p>
  </div>;
}
async function auditPackage(): Promise<Audit> {
  try {
    const response = await fetch(CHARACTER_MANIFEST_URL, { cache: "no-store" });
    if (!response.ok) return { state: "unavailable", detail: `Layered artwork is not installed (${response.status}). Supply the documented package; no placeholder character will be substituted.`, assetCount: 0, layerCount: 0 };
    const result = inspectRigManifestV2(await response.json() as unknown);
    if (!result.valid || !result.manifest) return { state: "unavailable", detail: `Manifest rejected: ${result.issues.map((issue) => `${issue.path}: ${issue.message}`).join("; ")}`, assetCount: 0, layerCount: 0 };
    return { state: "ready", detail: "Manifest structure is valid. The overlay performs byte-digest, decoded-dimension, hierarchy, and texture checks before rendering.", assetCount: result.manifest.assets.length, layerCount: result.manifest.layers.length };
  } catch (error) { const reason = error instanceof Error ? error.message : String(error); return { state: "unavailable", detail: `Layered artwork is not installed or could not be read (${reason}). Supply the documented package; no placeholder character will be substituted.`, assetCount: 0, layerCount: 0 }; }
}
