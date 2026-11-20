// The web build's save-data worker: the player's saves and settings in the origin's private
// storage (OPFS) as one zip, and back. The folder is the one the installed game uses
// (`home/.local/share/rally-championship`, see `src/platform/web.rs`).
//
// Kept: everything under `savedata/` and the files `var/hst.dat`, `var/ral.dat`, `var/ral.cfg`
// and `var/rc2k.toml`. Nothing else is read from or written into a zip, so a zip from elsewhere
// cannot place a file outside them.
//
// Messages in: {op: "export"} -> {exported: Uint8Array, files}; {op: "import", bytes: Uint8Array}
// -> {imported: files, skipped}; {op: "scan"} -> {scanned: files}; {op: "codeFiles"} ->
// {files: [{path, data}]} (what the save code reads, see below); {op: "writeFiles", files} ->
// {written: files} (what an imported save code writes). Out on failure: {error}.
//
// The save code (`codes_worker.js`) reads the saved games and car set-ups, `var/hst.dat`,
// `var/rc2k.toml` and `var/name.cfg`, and the game's language file (the first `.txt` in
// `files/locale`, else in any folder of `files/`, named ":lexicon": the drivers' names are in it).

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
// The plain ZIP format's limits (no ZIP64): a size or offset in 32 bits, an entry count in 16.
const ZIP_LIMIT = 0xffffffff;
const ZIP_ENTRIES = 0xfffe;

function makeZip(files) {
  if (files.length > ZIP_ENTRIES) throw new Error("Too many files to save in one zip (" + files.length + ").");
  let planned = 0;
  for (const f of files) {
    planned += 30 + 46 + 2 * utf8.encode(f.path).length + f.data.length;
    if (f.data.length > ZIP_LIMIT || planned > ZIP_LIMIT) throw new Error("The saves are too big for one zip (over 4 GB).");
  }
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

// `bytes` inflated, stopped as soon as it gives more than `limit` bytes (the entry's size from
// the zip's index): a packed entry that claims to be small must not fill the memory.
async function inflate(bytes, limit) {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw"));
  const reader = stream.getReader();
  const out = new Uint8Array(limit);
  let at = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    if (at + value.length > limit) {
      await reader.cancel();
      throw new Error("An entry is larger than the zip's index says.");
    }
    out.set(value, at);
    at += value.length;
  }
  return out.subarray(0, at);
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
    else if (method === 8) data = await inflate(raw, unpacked);
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

// `name` of `from` put into `to` as `newName`: moved where the browser can, else copied (folders
// included) and removed.
async function moveEntry(from, name, to, newName) {
  const f = await find(from, name);
  if (!f) return false;
  if (typeof f.handle.move === "function") {
    try {
      await f.handle.move(to, newName);
      return true;
    } catch (e) {
      if (!(e instanceof TypeError) && !(e && e.name === "NotSupportedError")) throw e;
    }
  }
  await copyEntry(f.handle, to, newName);
  await from.removeEntry(f.name, { recursive: true });
  return true;
}

async function copyEntry(handle, to, newName) {
  if (handle.kind === "directory") {
    const dir = await to.getDirectoryHandle(newName, { create: true });
    for await (const [n, h] of handle.entries()) await copyEntry(h, dir, n);
  } else {
    await writeFile(to, [newName], new Uint8Array(await (await handle.getFile()).arrayBuffer()));
  }
}

// The kept places of the installed folder: `savedata` and the `var` files.
async function stagedPlaces(staging) {
  const places = [];
  if (await find(staging, "savedata")) places.push({ dir: [], name: "savedata" });
  const vars = await find(staging, "var");
  if (vars) for (const name of VAR_FILES) if (await find(vars.handle, name)) places.push({ dir: ["var"], name });
  return places;
}

async function folderAt(root, dir, create) {
  let d = root;
  for (const p of dir) {
    const f = await find(d, p);
    if (f) d = f.handle;
    else if (create) d = await d.getDirectoryHandle(p, { create: true });
    else return null;
  }
  return d;
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
  // Everything is read and checked above. The new files are written beside the old ones first
  // (a full storage or a closed tab stops here with the old saves whole), then swapped in by
  // moving: the old ones go aside, the new ones in, and the old ones come back if that fails.
  const root = await installedDir(true);
  await removeIn(root, "import.new");
  await removeIn(root, "import.old");
  const staging = await root.getDirectoryHandle("import.new", { create: true });
  try {
    for (const k of keep) await writeFile(staging, k.parts, k.data);
  } catch (e) {
    await removeIn(root, "import.new");
    throw e;
  }
  const aside = await root.getDirectoryHandle("import.old", { create: true });
  const places = [{ dir: [], name: "savedata" }, ...VAR_FILES.map((name) => ({ dir: ["var"], name }))];
  const moved = [];
  const placed = [];
  try {
    for (const p of places) {
      const from = await folderAt(root, p.dir, false);
      if (!from) continue;
      const to = await folderAt(aside, p.dir, true);
      const found = await find(from, p.name);
      if (found && (await moveEntry(from, p.name, to, found.name))) moved.push({ ...p, name: found.name });
    }
    for (const p of await stagedPlaces(staging)) {
      const from = await folderAt(staging, p.dir, false);
      const to = await folderAt(root, p.dir, true);
      const found = await find(from, p.name);
      if (await moveEntry(from, p.name, to, found.name)) placed.push({ ...p, name: found.name });
    }
  } catch (e) {
    for (const p of placed) await removeIn(await folderAt(root, p.dir, true), p.name).catch(() => {});
    for (const p of moved) {
      const from = await folderAt(aside, p.dir, false);
      const to = await folderAt(root, p.dir, true);
      await moveEntry(from, p.name, to, p.name).catch(() => {});
    }
    throw e;
  }
  await removeIn(root, "import.new");
  await removeIn(root, "import.old");
  return { imported: keep.length, skipped };
}

async function doExport() {
  const files = [];
  for (const f of await gather()) {
    // The desktop program's rendered videos (gigabytes) are not saves.
    if (f.path.startsWith("savedata/videos/")) continue;
    const file = await f.handle.getFile();
    files.push({ path: f.path, data: new Uint8Array(await file.arrayBuffer()) });
  }
  files.sort((a, b) => (a.path < b.path ? -1 : 1));
  return { exported: makeZip(files), files: files.length };
}

// ---- the save code's files ----

const CODE_SLOTS = [["savegame", /^svgame\d\d$/], ["carsetup", /^carset\d\d$/]];

async function readAll(handle) {
  return new Uint8Array(await (await handle.getFile()).arrayBuffer());
}

// The first `.txt` of a folder in name order, or null.
async function firstText(dir) {
  const names = [];
  for await (const [n, h] of dir.entries()) if (h.kind === "file" && /\.txt$/i.test(n)) names.push([n, h]);
  names.sort((a, b) => (a[0].toLowerCase() < b[0].toLowerCase() ? -1 : 1));
  return names.length ? names[0][1] : null;
}

// The game's language file: `files/locale/*.txt`, else the first folder of `files/` that has one.
async function lexicon(root) {
  const files = await find(root, "files");
  if (!files || files.handle.kind !== "directory") return null;
  const locale = await find(files.handle, "locale");
  const folders = [];
  if (locale && locale.handle.kind === "directory") folders.push(locale.handle);
  for await (const [, h] of files.handle.entries()) if (h.kind === "directory") folders.push(h);
  for (const d of folders) {
    const t = await firstText(d);
    if (t) return t;
  }
  return null;
}

async function codeFiles() {
  const root = await installedDir(false);
  const out = [];
  const save = await find(root, "savedata");
  if (save && save.handle.kind === "directory") {
    for (const [folder, pattern] of CODE_SLOTS) {
      const dir = await find(save.handle, folder);
      if (!dir || dir.handle.kind !== "directory") continue;
      for await (const [name, h] of dir.handle.entries()) {
        if (h.kind === "file" && pattern.test(name.toLowerCase())) {
          out.push({ path: "savedata/" + folder + "/" + name.toLowerCase(), data: await readAll(h) });
        }
      }
    }
  }
  const vars = await find(root, "var");
  if (vars && vars.handle.kind === "directory") {
    for (const name of ["hst.dat", "rc2k.toml", "name.cfg"]) {
      const f = await find(vars.handle, name);
      if (f && f.handle.kind === "file") out.push({ path: "var/" + name, data: await readAll(f.handle) });
    }
  }
  const lex = await lexicon(root);
  if (lex) out.push({ path: ":lexicon", data: await readAll(lex) });
  return { files: out };
}

// Writes files an imported save code made; only the kept places (a code cannot name another).
// `keptPath` allows `var/name.cfg` nowhere: the code never writes it.
async function writeFiles(files) {
  const places = files.map((f) => ({ parts: keptPath(f.path), data: f.data }));
  if (places.some((p) => !p.parts)) throw new Error("The code names a file that is not a save.");
  const root = await installedDir(true);
  for (const p of places) await writeFile(root, p.parts, p.data);
  return { written: places.length };
}

onmessage = async (m) => {
  try {
    if (m.data.op === "export") postMessage(await doExport());
    else if (m.data.op === "import") postMessage(await doImport(m.data.bytes));
    else if (m.data.op === "scan") postMessage({ scanned: (await gather()).length });
    else if (m.data.op === "codeFiles") postMessage(await codeFiles());
    else if (m.data.op === "writeFiles") postMessage(await writeFiles(m.data.files));
    else postMessage({ error: "Unknown request: " + m.data.op });
  } catch (e) {
    const gone = e && (e.name === "NotFoundError");
    postMessage({ error: gone ? "No saves found in this browser yet." : String(e && e.message ? e.message : e) });
  }
};
