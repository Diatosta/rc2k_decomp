// The web build's save-data worker: the player's saves and settings in the origin's private
// storage (OPFS) as one zip, and back. The folder is the one the installed game uses
// (`home/.local/share/rally-championship`, see `src/platform/web.rs`).
//
// Kept: everything under `savedata/` and the files `var/hst.dat`, `var/ral.dat`, `var/ral.cfg`
// and `var/rc2k.toml`. Nothing else is read from or written into a zip, so a zip from elsewhere
// cannot place a file outside them.
//
// Messages in: {op: "export"} -> {exported: Uint8Array, files}; {op: "import", bytes: Uint8Array}
// -> {imported: files, skipped}; {op: "scan"} -> {scanned: files}. Out on failure: {error}.

const INSTALLED = ["home", ".local", "share", "rally-championship"];
const VAR_FILES = ["hst.dat", "ral.dat", "ral.cfg", "rc2k.toml"];
const MAX_TOTAL = 512 * 1024 * 1024;
const text = new TextDecoder();
const utf8 = new TextEncoder();

// The entry of `dir` named `name` in any letter case (the game finds its files that way).
async function find(dir, name) {
  const want = name.toLowerCase();
  for await (const [n, handle] of dir.entries()) {
    if (n.toLowerCase() === want) return { name: n, handle };
  }
  return null;
}

async function installedDir(create) {
  let d = await navigator.storage.getDirectory();
  for (const p of INSTALLED) d = await d.getDirectoryHandle(p, { create });
  return d;
}

// ---- reading the folder ----

async function walk(dir, prefix, out) {
  for await (const [name, handle] of dir.entries()) {
    if (handle.kind === "directory") await walk(handle, prefix + name.toLowerCase() + "/", out);
    else out.push({ path: prefix + name.toLowerCase(), handle });
  }
}

// The files to keep: [{path, handle}], paths in lower case with `/`.
async function gather() {
  const root = await installedDir(false);
  const list = [];
  const save = await find(root, "savedata");
  if (save && save.handle.kind === "directory") await walk(save.handle, "savedata/", list);
  const vars = await find(root, "var");
  if (vars && vars.handle.kind === "directory") {
    for (const name of VAR_FILES) {
      const f = await find(vars.handle, name);
      if (f && f.handle.kind === "file") list.push({ path: "var/" + name, handle: f.handle });
    }
  }
  return list;
}

// ---- zip ----

const crcTable = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function crc32(bytes) {
  let c = 0xffffffff;
  for (let i = 0; i < bytes.length; i++) c = crcTable[(c ^ bytes[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

// A zip of `files` ([{path, data}]), entries stored (the saves are small and mostly packed).
function makeZip(files) {
  const d = new Date();
  const time = (d.getHours() << 11) | (d.getMinutes() << 5) | (d.getSeconds() >> 1);
  const day = (Math.max(d.getFullYear() - 1980, 0) << 9) | ((d.getMonth() + 1) << 5) | d.getDate();
  const parts = [];
  const central = [];
  let at = 0;
  for (const f of files) {
    const name = utf8.encode(f.path);
    const crc = crc32(f.data);
    const head = new DataView(new ArrayBuffer(30));
    head.setUint32(0, 0x04034b50, true);
    head.setUint16(4, 20, true);
    head.setUint16(6, 0x0800, true); // names in UTF-8
    head.setUint16(10, time, true);
    head.setUint16(12, day, true);
    head.setUint32(14, crc, true);
    head.setUint32(18, f.data.length, true);
    head.setUint32(22, f.data.length, true);
    head.setUint16(26, name.length, true);
    parts.push(new Uint8Array(head.buffer), name, f.data);
    const entry = new DataView(new ArrayBuffer(46));
    entry.setUint32(0, 0x02014b50, true);
    entry.setUint16(4, 20, true);
    entry.setUint16(6, 20, true);
    entry.setUint16(8, 0x0800, true);
    entry.setUint16(12, time, true);
    entry.setUint16(14, day, true);
    entry.setUint32(16, crc, true);
    entry.setUint32(20, f.data.length, true);
    entry.setUint32(24, f.data.length, true);
    entry.setUint16(28, name.length, true);
    entry.setUint32(42, at, true);
    central.push(new Uint8Array(entry.buffer), name);
    at += 30 + name.length + f.data.length;
  }
  const size = central.reduce((n, p) => n + p.length, 0);
  const end = new DataView(new ArrayBuffer(22));
  end.setUint32(0, 0x06054b50, true);
  end.setUint16(8, files.length, true);
  end.setUint16(10, files.length, true);
  end.setUint32(12, size, true);
  end.setUint32(16, at, true);
  const all = [...parts, ...central, new Uint8Array(end.buffer)];
  const out = new Uint8Array(all.reduce((n, p) => n + p.length, 0));
  let p = 0;
  for (const a of all) { out.set(a, p); p += a.length; }
  return out;
}

const u16 = (v, at) => v.getUint16(at, true);
const u32 = (v, at) => v.getUint32(at, true);

async function inflate(bytes) {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

// The entries of a zip as [{name, data}], every one decoded and checked against its CRC.
async function readZip(bytes) {
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let eocd = -1;
  for (let i = bytes.length - 22; i >= Math.max(0, bytes.length - 65557); i--) {
    if (u32(v, i) === 0x06054b50) { eocd = i; break; }
  }
  if (eocd < 0) throw new Error("This is not a zip.");
  const count = u16(v, eocd + 10);
  const at = u32(v, eocd + 16);
  if (at === 0xffffffff || count === 0xffff) throw new Error("ZIP64 is not read.");
  const list = [];
  let total = 0;
  let p = at;
  for (let n = 0; n < count; n++) {
    if (p + 46 > bytes.length || u32(v, p) !== 0x02014b50) throw new Error("The zip's index is broken.");
    const method = u16(v, p + 10);
    const crc = u32(v, p + 16);
    const packed = u32(v, p + 20);
    const unpacked = u32(v, p + 24);
    const nameLen = u16(v, p + 28), extraLen = u16(v, p + 30), noteLen = u16(v, p + 32);
    const local = u32(v, p + 42);
    const name = text.decode(bytes.subarray(p + 46, p + 46 + nameLen));
    p += 46 + nameLen + extraLen + noteLen;
    if (name.endsWith("/")) continue;
    total += unpacked;
    if (total > MAX_TOTAL) throw new Error("The zip holds too much.");
    if (local + 30 > bytes.length || u32(v, local) !== 0x04034b50) throw new Error(name + ": a broken entry.");
    const start = local + 30 + u16(v, local + 26) + u16(v, local + 28);
    if (start + packed > bytes.length) throw new Error(name + ": cut short.");
    const raw = bytes.subarray(start, start + packed);
    let data;
    if (method === 0) data = raw;
    else if (method === 8) data = await inflate(raw);
    else throw new Error(name + ": packed in a way not read (" + method + ").");
    if (data.length !== unpacked || crc32(data) !== crc) throw new Error(name + ": damaged.");
    list.push({ name, data });
  }
  return list;
}

// ---- writing the folder ----

// The path as the kept place it names (lower case, `/`), or null when it is not one of them.
function keptPath(name) {
  const parts = name.replace(/\\/g, "/").split("/").filter((s) => s && s !== ".");
  if (parts.some((s) => s === "..")) return null;
  const low = parts.map((s) => s.toLowerCase());
  if (low[0] === "savedata" && low.length >= 2) return low;
  if (low[0] === "var" && low.length === 2 && VAR_FILES.includes(low[1])) return low;
  return null;
}

async function removeIn(dir, name) {
  const f = await find(dir, name);
  if (f) await dir.removeEntry(f.name, { recursive: true });
}

async function writeFile(root, parts, data) {
  let d = root;
  for (const p of parts.slice(0, -1)) {
    const f = await find(d, p);
    d = f ? f.handle : await d.getDirectoryHandle(p, { create: true });
  }
  const last = parts[parts.length - 1];
  const handle = await d.getFileHandle(last, { create: true });
  const out = await handle.createSyncAccessHandle();
  try {
    out.truncate(0);
    out.write(data, { at: 0 });
  } finally {
    out.flush();
    out.close();
  }
}

async function doImport(bytes) {
  const entries = await readZip(bytes);
  const keep = [];
  let skipped = 0;
  for (const e of entries) {
    const parts = keptPath(e.name);
    if (parts) keep.push({ parts, data: e.data });
    else skipped++;
  }
  if (keep.length === 0) throw new Error("The zip holds no saves or settings of the game.");
  // Everything is read and checked above; only now is what is there replaced.
  const root = await installedDir(true);
  await removeIn(root, "savedata");
  const vars = await find(root, "var");
  if (vars) for (const name of VAR_FILES) await removeIn(vars.handle, name);
  for (const k of keep) await writeFile(root, k.parts, k.data);
  return { imported: keep.length, skipped };
}

async function doExport() {
  const files = [];
  for (const f of await gather()) {
    const file = await f.handle.getFile();
    files.push({ path: f.path, data: new Uint8Array(await file.arrayBuffer()) });
  }
  files.sort((a, b) => (a.path < b.path ? -1 : 1));
  return { exported: makeZip(files), files: files.length };
}

onmessage = async (m) => {
  try {
    if (m.data.op === "export") postMessage(await doExport());
    else if (m.data.op === "import") postMessage(await doImport(m.data.bytes));
    else if (m.data.op === "scan") postMessage({ scanned: (await gather()).length });
  } catch (e) {
    const gone = e && (e.name === "NotFoundError");
    postMessage({ error: gone ? "No saves found in this browser yet." : String(e && e.message ? e.message : e) });
  }
};
