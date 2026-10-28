import { render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import { App } from "./App";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));
const shell = { appName: "SAIKI", version: "0.1.0", platform: "windows", lifecycle: "ready", sessionId: "123456789", startedAtUnixMs: 1 };
const overlay = { paused: false, activeMonitorIndex: 0, pauseHotkey: "Ctrl+Shift+F12", monitors: [{ index: 0, name: "DISPLAY1", position: { x: 0, y: 0 }, size: { width: 2560, height: 1440 }, scaleFactor: 1 }] };

beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((command: string) => command === "get_shell_status" ? Promise.resolve(shell) : command === "get_overlay_status" ? Promise.resolve(overlay) : Promise.resolve(undefined));
});

test("renders monitor topology and click-through state", async () => {
  render(<App />);
  expect(await screen.findByText("CLICK-THROUGH")).toBeInTheDocument();
  expect(screen.getByText("DISPLAY1")).toBeInTheDocument();
  expect(screen.getByText(/2560×1440/)).toBeInTheDocument();
});

test("reports invalid overlay contracts honestly", async () => {
  invoke.mockImplementation((command: string) => command === "get_shell_status" ? Promise.resolve(shell) : Promise.resolve({ paused: false }));
  render(<App />);
  expect(await screen.findByText(/Unavailable:/)).toBeInTheDocument();
  expect(screen.getByText(/invalid overlay status contract/i)).toBeInTheDocument();
});
