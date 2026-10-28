import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { test } from "node:test";
const read = (path) => readFile(new URL(`../${path}`, import.meta.url), "utf8");

test("required Phase 00 artifacts are substantive", async () => {
  for (const path of ["AGENTS.md", "SECURITY.md", "docs/architecture/README.md", "docs/architecture/diagrams.md", "docs/phase-00/acceptance-matrix.md", "contracts/README.md", "contracts/src/index.ts"])
    assert.ok((await read(path)).length > 500, `${path} missing or too small`);
});

test("seven frozen ADRs are accepted and complete", async () => {
  const dir = new URL("../docs/architecture/adr/", import.meta.url);
  const files = (await readdir(dir)).filter((name) => name.endsWith(".md"));
  assert.equal(files.length, 7);
  for (const file of files) {
    const body = await read(`docs/architecture/adr/${file}`);
    for (const heading of ["- Status: Accepted", "## Context", "## Decision", "## Consequences"]) assert.match(body, new RegExp(heading));
  }
});

test("contracts separate cognition from authority and preserve honest outcomes", async () => {
  const body = await read("contracts/src/index.ts");
  const proposed = body.slice(body.indexOf("export interface ProposedAction"), body.indexOf("export interface ExecutionBudget"));
  assert.doesNotMatch(proposed, /approval|permit|secret/i);
  for (const token of ["deny", "require_owner_approval", "permit", "succeeded", "partially_succeeded", "failed", "denied", "cancelled", "unknown", "unavailable"])
    assert.match(body, new RegExp(`"${token}"`));
});

test("security invariants prohibit secret exposure and fabricated success", async () => {
  const security = await read("SECURITY.md");
  const agents = await read("AGENTS.md");
  assert.match(security, /Only the Rust policy\/execution boundary can authorize side effects/);
  assert.match(security, /must not enter prompts, logs, SQLite, browser storage/);
  assert.match(agents, /never fabricate success/);
  assert.match(agents, /arbitrary shell execution/);
});

test("diagrams and acceptance gates cover Phase 00", async () => {
  const diagrams = await read("docs/architecture/diagrams.md");
  assert.ok((diagrams.match(/```mermaid/g) ?? []).length >= 3);
  assert.match(diagrams, /read-after-write verification/i);
  const matrix = await read("docs/phase-00/acceptance-matrix.md");
  for (let id = 1; id <= 14; id++) assert.match(matrix, new RegExp(`P00-${String(id).padStart(2, "0")}`));
  assert.match(matrix, /Managed artifact receipt/);
  assert.doesNotMatch(matrix, /Pending artifact/);
});
