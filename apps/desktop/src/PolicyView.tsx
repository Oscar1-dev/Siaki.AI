import { useEffect, useState } from "react";
import type { Decision, PolicyAuditEntry, PolicyState } from "@saiki/ipc-contracts";
import { createGrant, evaluateAction, getPolicyState, listPolicyAudit, resolveApproval, revokeGrant, setKillSwitch } from "./ipc";
import "./policy.css";

export function PolicyView() {
  const [policy, setPolicy] = useState<PolicyState>();
  const [audit, setAudit] = useState<readonly PolicyAuditEntry[]>([]);
  const [message, setMessage] = useState("Reading policy state…");
  const [capability, setCapability] = useState("email.send");
  const [resource, setResource] = useState("mailbox/outreach");
  const [decision, setDecision] = useState<Decision>();
  const [grantResource, setGrantResource] = useState("mailbox/*");
  const [purpose, setPurpose] = useState("");

  useEffect(() => { void refresh("Policy state loaded."); }, []);

  function fail(error: unknown): void { setMessage(error instanceof Error ? error.message : String(error)); }
  async function refresh(note: string): Promise<void> { try { const [state, log] = await Promise.all([getPolicyState(), listPolicyAudit()]); setPolicy(state); setAudit(log); setMessage(note); } catch (error) { fail(error); } }

  function evaluate(): void { void evaluateAction({ actor: "model", capability: capability.trim(), resource: resource.trim() }).then((result) => { setDecision(result); void refresh(`Decision for ${capability.trim()}: ${result.decision.toUpperCase()}.`); }).catch(fail); }
  function grant(): void { if (!capability.trim() || !grantResource.trim() || !purpose.trim()) return; void createGrant({ capability: capability.trim(), resource: grantResource.trim(), purpose: purpose.trim(), maxActions: 5, ttlMs: 60 * 60 * 1000 }).then(() => { setPurpose(""); return refresh("Grant created (owner authority)."); }).catch(fail); }
  function revoke(id: string): void { void revokeGrant(id).then(() => refresh("Grant revoked.")).catch(fail); }
  function decideApproval(id: string, approve: boolean): void { void resolveApproval(id, approve).then(() => refresh(approve ? "Approval granted." : "Approval denied.")).catch(fail); }
  function toggleKill(): void { if (!policy) return; void setKillSwitch(!policy.killSwitchEngaged).then((state) => { setPolicy(state); return refresh(state.killSwitchEngaged ? "Kill switch ENGAGED — all external actions denied." : "Kill switch released."); }).catch(fail); }

  if (!policy) return <div className="view-stack"><p className="provider-message" role="status">{message}</p></div>;

  return <div className="view-stack policy">
    <div className="section-heading"><div><p className="eyebrow">OWNER AUTHORITY · POLICY</p><h1>Policy engine</h1></div><button className={`kill ${policy.killSwitchEngaged ? "engaged" : ""}`} onClick={toggleKill}>{policy.killSwitchEngaged ? "RELEASE KILL SWITCH" : "ENGAGE KILL SWITCH"}</button></div>
    <p className="scope-note">A model may request an action; only this engine authorizes it, deny-by-default. Grants, approvals, and the kill switch are owner-only. Approvals bind an exact payload digest. The audit log is append-only.</p>
    <p className="provider-message" role="status">{message}</p>

    <div className="evaluator">
      <h2>Evaluate a model-proposed action</h2>
      <div className="eval-row"><label className="model-field"><span>Capability</span><input value={capability} onChange={(event) => setCapability(event.target.value)} /></label><label className="model-field"><span>Resource</span><input value={resource} onChange={(event) => setResource(event.target.value)} /></label><button onClick={evaluate}>Evaluate as model</button></div>
      {decision && <div className={`decision decision-${decision.decision}`}>{decision.decision === "allow" ? "ALLOW" : decision.decision === "deny" ? `DENY — ${decision.reason}` : `APPROVAL REQUIRED (${decision.tier}) — ${decision.reason}`}</div>}
    </div>

    <div className="grant-form">
      <h2>Grant capability (owner)</h2>
      <div className="eval-row"><label className="model-field"><span>Resource pattern (trailing * allowed)</span><input value={grantResource} onChange={(event) => setGrantResource(event.target.value)} /></label><label className="model-field"><span>Purpose</span><input value={purpose} onChange={(event) => setPurpose(event.target.value)} placeholder="why this is allowed" /></label><button onClick={grant} disabled={!capability.trim() || !grantResource.trim() || !purpose.trim()}>Grant (5 actions, 1h)</button></div>
    </div>

    <div className="policy-list">
      <h2>Active grants</h2>
      {policy.grants.length === 0 && <p className="empty-note">No active grants — everything external is deny-by-default.</p>}
      {policy.grants.map((g) => <article key={g.id} className="grant"><div><b>{g.capability}</b> <span className={`tier tier-${g.tier}`}>{g.tier}</span><small>{g.resource} · {g.usedActions}/{g.maxActions} used{g.maxSpend !== null ? ` · spend ${g.spent}/${g.maxSpend}` : ""} · {g.purpose}</small></div><button className="danger-button" onClick={() => revoke(g.id)}>Revoke</button></article>)}
    </div>

    {policy.pendingApprovals.length > 0 && <div className="policy-list">
      <h2>Pending approvals</h2>
      {policy.pendingApprovals.map((a) => <article key={a.id} className="grant"><div><b>{a.capability}</b> <span className={`tier tier-${a.tier}`}>{a.tier}</span><small>{a.resource} · from {a.actor} · digest {a.argumentsDigest.slice(0, 12)}…</small></div><div className="approve-actions"><button onClick={() => decideApproval(a.id, true)}>Approve</button><button className="danger-button" onClick={() => decideApproval(a.id, false)}>Deny</button></div></article>)}
    </div>}

    <div className="policy-list">
      <h2>Audit log (append-only)</h2>
      {audit.length === 0 && <p className="empty-note">No audit entries yet.</p>}
      {audit.slice(0, 40).map((entry, index) => <div key={index} className={`audit audit-${entry.decision}`}><span className="a-decision">{entry.decision}</span><span className="a-cap">{entry.capability}</span><span className="a-detail">{entry.reason || entry.detail}</span></div>)}
    </div>
  </div>;
}
