// Who may be stopped when the dev server quits.
//
// Process:
// 1. `PORT` is 43123, or `PESTY_PORT` for one command.
// 2. On quit, look at the listener. Signal it only when its working directory
//    is this repo and its command is Vite or scripts/dev.mjs.
// 3. A listener that fails that test is printed and left running.
//
// Config: change the fallback `43123` here and in vite.config.ts together.
// `.pesty/dev.json` is the pid file. It is gitignored.

import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));

export const ROOT = path.resolve(here, "..");
export const PORT = Number(process.env.PESTY_PORT || 43123);
export const HOST = "0.0.0.0";
export const PIDFILE = path.join(ROOT, ".pesty", "dev.json");

export function portBusy(port = PORT, host = HOST) {
  return new Promise((resolve) => {
    const server = net.createServer();
    const finish = (busy) => {
      server.removeAllListeners();
      if (server.listening) server.close(() => resolve(busy));
      else resolve(busy);
    };
    server.once("error", () => finish(true));
    server.once("listening", () => finish(false));
    try {
      server.listen({ port, host, exclusive: true });
    } catch {
      finish(true);
    }
  });
}

export function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function waitUntilFree(port = PORT, timeoutMs = 4000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (!(await portBusy(port))) return true;
    await sleep(40);
  }
  return !(await portBusy(port));
}

export function readPidfile() {
  try {
    const data = JSON.parse(fs.readFileSync(PIDFILE, "utf8"));
    if (!data || typeof data !== "object") return null;
    return data;
  } catch {
    return null;
  }
}

export function writePidfile(data) {
  fs.mkdirSync(path.dirname(PIDFILE), { recursive: true });
  const tmp = `${PIDFILE}.${process.pid}.tmp`;
  fs.writeFileSync(tmp, `${JSON.stringify(data, null, 2)}\n`);
  fs.renameSync(tmp, PIDFILE);
}

export function clearPidfile() {
  try {
    fs.unlinkSync(PIDFILE);
  } catch {
    // already gone
  }
}

export function alive(pid) {
  if (!pid || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function procStat(pid) {
  const raw = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
  const end = raw.lastIndexOf(")");
  const rest = raw.slice(end + 2).trim().split(/\s+/);
  return {
    ppid: Number(rest[1]),
    pgrp: Number(rest[2]),
  };
}

function cmdlineOf(pid) {
  try {
    return fs
      .readFileSync(`/proc/${pid}/cmdline`)
      .toString("utf8")
      .replaceAll("\0", " ")
      .replace(/\s+/g, " ")
      .trim();
  } catch {
    return "";
  }
}

function cwdOf(pid) {
  try {
    return fs.readlinkSync(`/proc/${pid}/cwd`);
  } catch {
    return "";
  }
}

export function describe(pid) {
  return { pid, cmdline: cmdlineOf(pid), cwd: cwdOf(pid) };
}

function inRepo(cwd) {
  return cwd === ROOT || cwd.startsWith(`${ROOT}${path.sep}`);
}

// A process is ours when it is this repo's vite, or the dev supervisor.
// The pidfile alone is not enough: the pid may have been reused.
export function isOurs(info) {
  if (!info || info.pid === process.pid) return false;
  const cmd = info.cmdline || "";
  const cwdOk = inRepo(info.cwd);
  if (!cwdOk || !cmd) return false;
  if (cmd.includes(`${path.sep}scripts${path.sep}dev.mjs`) || cmd.includes("scripts/dev.mjs")) {
    return true;
  }
  return cmd.includes("vite") && (cmd.includes(String(PORT)) || cmd.includes("pesty"));
}

function listenInodes(port) {
  const hex = port.toString(16).toUpperCase().padStart(4, "0");
  const inodes = new Set();
  for (const file of ["/proc/net/tcp", "/proc/net/tcp6"]) {
    let text = "";
    try {
      text = fs.readFileSync(file, "utf8");
    } catch {
      continue;
    }
    for (const line of text.split("\n").slice(1)) {
      const parts = line.trim().split(/\s+/);
      if (parts.length < 10 || parts[3] !== "0A") continue;
      const local = parts[1];
      const portHex = local.slice(local.lastIndexOf(":") + 1).toUpperCase();
      if (portHex === hex) inodes.add(parts[9]);
    }
  }
  return inodes;
}

export function listeners(port = PORT) {
  const inodes = listenInodes(port);
  if (inodes.size === 0) return [];
  let names = [];
  try {
    names = fs.readdirSync("/proc");
  } catch {
    return [];
  }
  const found = [];
  for (const name of names) {
    if (!/^\d+$/.test(name)) continue;
    const pid = Number(name);
    let fds = [];
    try {
      fds = fs.readdirSync(`/proc/${pid}/fd`);
    } catch {
      continue;
    }
    for (const fd of fds) {
      let link = "";
      try {
        link = fs.readlinkSync(`/proc/${pid}/fd/${fd}`);
      } catch {
        continue;
      }
      const match = /^socket:\[(\d+)\]$/.exec(link);
      if (match && inodes.has(match[1])) {
        found.push(describe(pid));
        break;
      }
    }
  }
  return found;
}

function childrenOf(pid) {
  try {
    return fs
      .readFileSync(`/proc/${pid}/task/${pid}/children`, "utf8")
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map(Number);
  } catch {
    return [];
  }
}

export function signal(pid, sig) {
  if (!pid || pid <= 0 || pid === process.pid) return;
  try {
    process.kill(pid, sig);
  } catch {
    // already gone
  }
}

// Signal a process. A group is signaled only when this pid leads it,
// so a leftover vite cannot take the shell down with it. Used for the
// force-kill pass. The first pass signals the pid alone so Vite can
// stop its own children.
export function stopPid(pid, signal, seen = new Set()) {
  if (!pid || pid <= 0 || pid === process.pid || seen.has(pid)) return;
  seen.add(pid);
  for (const child of childrenOf(pid)) stopPid(child, signal, seen);
  let leader = false;
  try {
    leader = procStat(pid).pgrp === pid;
  } catch {
    leader = false;
  }
  if (leader) {
    try {
      process.kill(-pid, signal);
      return;
    } catch {
      // fall through to the single pid
    }
  }
  try {
    process.kill(pid, signal);
  } catch {
    // already gone
  }
}

function targets() {
  const ids = new Set();
  const record = readPidfile();
  for (const pid of [record?.devPid, record?.vitePid]) {
    if (!alive(pid)) continue;
    if (isOurs(describe(pid))) ids.add(pid);
  }
  for (const info of listeners(PORT)) {
    if (isOurs(info)) ids.add(info.pid);
  }
  return [...ids];
}

export async function releaseOurs() {
  const before = targets();
  const foreign = listeners(PORT).filter((info) => !isOurs(info));
  if (before.length === 0) {
    clearPidfile();
    if (foreign.length > 0 || (await portBusy(PORT))) {
      return { ok: false, released: false, foreign: foreign.length ? foreign : listeners(PORT) };
    }
    return { ok: true, released: false, foreign: [] };
  }

  for (const pid of before) signal(pid, "SIGTERM");
  let free = await waitUntilFree(PORT, 3000);
  if (!free) {
    for (const pid of targets()) stopPid(pid, "SIGKILL");
    free = await waitUntilFree(PORT, 2000);
  }
  if (!free) {
    return {
      ok: false,
      released: true,
      foreign: listeners(PORT).filter((info) => !isOurs(info)),
    };
  }
  clearPidfile();
  return { ok: true, released: true, foreign: [] };
}

export function formatHolder(info) {
  const cmd = info.cmdline || "(no cmdline)";
  const cwd = info.cwd ? ` cwd ${info.cwd}` : "";
  return `${info.pid} ${cmd}${cwd}`;
}
