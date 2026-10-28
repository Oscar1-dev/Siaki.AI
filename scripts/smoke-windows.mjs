import { spawn } from "node:child_process";
import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { resolve } from "node:path";

if (process.platform !== "win32") throw new Error("The native smoke test is Windows-only.");
const executable = resolve("target/release/saiki.exe");
await access(executable, constants.X_OK);

const startedAt = Date.now();
const child = spawn(executable, ["--smoke-test"], { stdio: "inherit", windowsHide: false });
const timeout = setTimeout(() => child.kill(), 15_000);
const exitCode = await new Promise((resolveExit, reject) => {
  child.once("error", reject);
  child.once("exit", (code, signal) => resolveExit({ code, signal }));
});
clearTimeout(timeout);
if (exitCode.code !== 0) throw new Error(`SAIKI smoke process failed: ${JSON.stringify(exitCode)}`);
const elapsed = Date.now() - startedAt;
if (elapsed < 900 || elapsed > 15_000) throw new Error(`Unexpected shell lifetime: ${elapsed}ms`);
console.log(`SAIKI native smoke passed: clean exit code 0 after ${elapsed}ms.`);
