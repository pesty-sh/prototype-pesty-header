// The dev server. `npm run dev` runs this file.
//
// Process:
// 1. Build public/engine.wasm in this process.
// 2. Start Vite in its own process group, on port 43123 (`PESTY_PORT` overrides one run).
// 3. On Ctrl+C, stop that group and wait until the port accepts a bind.
//    The terminal then prints "Port 43123 is free."
// 4. A second Ctrl+C force-kills, then checks the port again.
//
// Config is the port in scripts/port.mjs and vite.config.ts. Both read `PESTY_PORT`.

import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  HOST,
  PORT,
  ROOT,
  alive,
  clearPidfile,
  releaseOurs,
  signal,
  stopPid,
  waitUntilFree,
  writePidfile,
} from "./port.mjs";

const preview = process.argv[2] === "preview";
let buildChild = null;
let vite = null;
let stopping = false;
let sigints = 0;

function vitePid() {
  return vite && alive(vite.pid) ? vite.pid : 0;
}

async function shutdown(force, code = 0) {
  if (stopping && !force) return code;
  stopping = true;
  if (buildChild && alive(buildChild.pid)) {
    if (force) stopPid(buildChild.pid, "SIGKILL");
    else signal(buildChild.pid, "SIGINT");
  }
  const pid = vitePid();
  if (pid) {
    if (force) stopPid(pid, "SIGKILL");
    else signal(pid, "SIGTERM");
    const deadline = Date.now() + (force ? 800 : 2000);
    while (alive(pid) && Date.now() < deadline) {
      await new Promise((resolve) => setTimeout(resolve, 40));
    }
    if (alive(pid)) stopPid(pid, "SIGKILL");
  }
  const free = await waitUntilFree(PORT, force ? 1500 : 4000);
  clearPidfile();
  vite = null;
  if (free) {
    console.log(`\nPort ${PORT} is free.`);
    process.exit(code);
  }
  console.error(`\nPort ${PORT} is still in use.`);
  process.exit(1);
}

process.on("SIGINT", () => {
  sigints += 1;
  void shutdown(sigints >= 2);
});
process.on("SIGTERM", () => {
  void shutdown(false);
});
process.on("SIGHUP", () => {
  void shutdown(false);
});

process.on("exit", () => {
  const pid = vite?.pid;
  if (!pid) return;
  try {
    process.kill(-pid, "SIGKILL");
  } catch {
    try {
      process.kill(pid, "SIGKILL");
    } catch {
      // already gone
    }
  }
});

function run(command, args) {
  return new Promise((resolve) => {
    const child = spawn(command, args, { cwd: ROOT, stdio: "inherit" });
    buildChild = child;
    child.on("exit", (code) => {
      if (buildChild === child) buildChild = null;
      resolve(code ?? 1);
    });
  });
}

async function buildWasm() {
  const code = await run("cargo", [
    "build",
    "--release",
    "--manifest-path",
    "engine/Cargo.toml",
    "--target",
    "wasm32-unknown-unknown",
  ]);
  if (stopping || code !== 0) return code || 1;
  const from = path.join(
    ROOT,
    "engine/target/wasm32-unknown-unknown/release/pesty_engine.wasm",
  );
  const to = path.join(ROOT, "public/engine.wasm");
  fs.mkdirSync(path.dirname(to), { recursive: true });
  fs.copyFileSync(from, to);
  return 0;
}

async function main() {
  if (!preview) {
    const code = await buildWasm();
    if (stopping) return;
    if (code !== 0) process.exit(code);
  } else if (!fs.existsSync(path.join(ROOT, "public/engine.wasm"))) {
    console.error("public/engine.wasm is missing. Run npm run build:wasm first.");
    process.exit(1);
  }

  const released = await releaseOurs();
  if (!released.ok) {
    console.error(`Port ${PORT} is in use by another program, so it was left running.`);
    for (const holder of released.foreign) {
      const cmd = holder.cmdline || "(no cmdline)";
      console.error(`  ${holder.pid} ${cmd}`);
    }
    process.exit(1);
  }
  if (released.released) console.log(`Stopped the previous server on port ${PORT}.`);

  const viteBin = path.join(ROOT, "node_modules/vite/bin/vite.js");
  const args = [viteBin];
  if (preview) args.push("preview");
  args.push("--host", HOST, "--port", String(PORT), "--strictPort");

  vite = spawn(process.execPath, args, {
    cwd: ROOT,
    detached: true,
    stdio: "inherit",
    env: { ...process.env, PESTY_PORT: String(PORT) },
  });
  writePidfile({
    port: PORT,
    devPid: process.pid,
    vitePid: vite.pid,
    cwd: ROOT,
    mode: preview ? "preview" : "dev",
  });
  vite.on("error", (error) => {
    console.error(error instanceof Error ? error.message : error);
    void shutdown(true, 1);
  });
  vite.on("exit", (code, signal) => {
    if (stopping) return;
    vite = null;
    void shutdown(signal === "SIGKILL", code ?? 0);
  });
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  void shutdown(true);
});
