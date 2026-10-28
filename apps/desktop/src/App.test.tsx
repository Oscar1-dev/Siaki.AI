import { render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import { App } from "./App";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));

beforeEach(() => invoke.mockReset());

test("renders verified native status", async () => {
  invoke.mockResolvedValueOnce({ appName: "SAIKI", version: "0.1.0", platform: "windows", lifecycle: "ready", sessionId: "123456789", startedAtUnixMs: 1 }).mockResolvedValue(undefined);
  render(<App />);
  expect(await screen.findByText("Present.")).toBeInTheDocument();
  expect(screen.getByText(/session 12345678/)).toBeInTheDocument();
});

test("reports invalid native contracts honestly", async () => {
  invoke.mockResolvedValueOnce({ appName: "SAIKI" });
  render(<App />);
  expect(await screen.findByText("Shell unavailable")).toBeInTheDocument();
  expect(screen.getByText(/invalid status contract/i)).toBeInTheDocument();
});
