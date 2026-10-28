/** Phase 00 boundary vocabulary. This module deliberately implements no side effects. */
type Brand<T, N extends string> = T & { readonly __brand: N };
export type ActionId = Brand<string, "ActionId">;
export type ActorId = Brand<string, "ActorId">;
export type ApprovalId = Brand<string, "ApprovalId">;
export type CapabilityId = Brand<string, "CapabilityId">;
export type CorrelationId = Brand<string, "CorrelationId">;
export type IdentityId = Brand<string, "IdentityId">;
export type MemoryId = Brand<string, "MemoryId">;
export type PayloadDigest = Brand<string, "PayloadDigest">;
export type PermitToken = Brand<string, "PermitToken">;
export type SecretReference = Brand<string, "SecretReference">;
export type ToolId = Brand<string, "ToolId">;
export type IsoTimestamp = Brand<string, "IsoTimestamp">;
export type RiskTier = "low" | "moderate" | "high" | "critical";
export type Sensitivity = "public" | "internal" | "confidential" | "restricted";

export interface Provenance {
  readonly sourceKind: "owner" | "model" | "tool" | "document" | "web" | "system";
  readonly sourceRef: string;
  readonly observedAt: IsoTimestamp;
  readonly trust: "trusted_owner" | "verified_system" | "untrusted_external" | "model_generated";
}

/** Untrusted planner output: intent, never authority. */
export interface ProposedAction {
  readonly kind: "proposed_action";
  readonly goal: string;
  readonly tool: ToolId;
  readonly target: string;
  readonly arguments: Readonly<Record<string, unknown>>;
  readonly rationale: string;
  readonly provenance: readonly Provenance[];
}

export interface ExecutionBudget {
  readonly maxDurationMs: number;
  readonly maxAttempts: number;
  readonly maxCostMinorUnits?: number;
  readonly currency?: string;
  readonly maxOutputBytes: number;
}

/** Trusted normalization submitted to deterministic policy. */
export interface ActionRequest {
  readonly kind: "action_request";
  readonly actionId: ActionId;
  readonly correlationId: CorrelationId;
  readonly actorId: ActorId;
  readonly identityId?: IdentityId;
  readonly requestedCapability: CapabilityId;
  readonly tool: ToolId;
  readonly target: string;
  readonly payloadDigest: PayloadDigest;
  readonly riskTier: RiskTier;
  readonly requestedAt: IsoTimestamp;
  readonly deadline: IsoTimestamp;
  readonly idempotencyKey?: string;
  readonly budget: ExecutionBudget;
  readonly provenance: readonly Provenance[];
}

export interface ExecutionConstraints {
  readonly allowedTarget: string;
  readonly requiredPayloadDigest: PayloadDigest;
  readonly expiresAt: IsoTimestamp;
  readonly maxAttempts: number;
  readonly maxCostMinorUnits?: number;
  readonly currency?: string;
}

export type PolicyDecision =
  | { readonly outcome: "deny"; readonly reasonCode: string; readonly explanation: string }
  | { readonly outcome: "require_owner_approval"; readonly reasonCode: string; readonly requestDigest: PayloadDigest; readonly expiresAt: IsoTimestamp }
  | { readonly outcome: "permit"; readonly constraints: ExecutionConstraints };

export interface ApprovalGrant {
  readonly kind: "owner_approval";
  readonly approvalId: ApprovalId;
  readonly ownerActorId: ActorId;
  readonly actionId: ActionId;
  readonly requestDigest: PayloadDigest;
  readonly grantedAt: IsoTimestamp;
  readonly expiresAt: IsoTimestamp;
  readonly constraints: ExecutionConstraints;
}

/** Opaque one-use authorization created only after policy evaluation. */
export interface ExecutionPermit {
  readonly kind: "execution_permit";
  readonly token: PermitToken;
  readonly actionId: ActionId;
  readonly constraints: ExecutionConstraints;
  readonly issuedAt: IsoTimestamp;
}

export type ActionStatus = "succeeded" | "partially_succeeded" | "failed" | "denied" | "cancelled" | "unknown" | "unavailable";
export interface VerificationEvidence {
  readonly method: "read_after_write" | "provider_receipt" | "local_observation";
  readonly verifiedAt: IsoTimestamp;
  readonly evidenceDigest: PayloadDigest;
  readonly summary: string;
}
export interface ActionResult {
  readonly actionId: ActionId;
  readonly correlationId: CorrelationId;
  readonly status: ActionStatus;
  readonly startedAt?: IsoTimestamp;
  readonly finishedAt: IsoTimestamp;
  readonly summary: string;
  readonly providerReference?: string;
  readonly verification?: VerificationEvidence;
  readonly retryable: boolean;
  readonly redactionsApplied: readonly string[];
}

export interface MemoryRecord {
  readonly memoryId: MemoryId;
  readonly kind: "episodic" | "semantic" | "procedural" | "preference" | "goal";
  readonly content: string;
  readonly provenance: readonly Provenance[];
  readonly sensitivity: Sensitivity;
  readonly confidence: number;
  readonly createdAt: IsoTimestamp;
  readonly updatedAt: IsoTimestamp;
  readonly retention: "session" | "until_date" | "owner_managed";
  readonly retainUntil?: IsoTimestamp;
  readonly lifecycle: "active" | "superseded" | "quarantined" | "deleted";
}

export interface IdentityBinding {
  readonly identityId: IdentityId;
  readonly provider: string;
  readonly ownerActorId: ActorId;
  readonly secretRef?: SecretReference;
  readonly capabilities: readonly CapabilityId[];
  readonly status: "disabled" | "pending_owner_setup" | "active" | "revoked" | "unavailable";
}
