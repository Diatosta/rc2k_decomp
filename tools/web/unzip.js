// The web build's importer (a worker): the game folder's zip files unpacked into the origin's
// private storage (OPFS) under `source/`, where the game finds them as `/opfs/source`
// (`src/platform/web.rs`). A folder that is already an installed game (it holds `var/install.cfg`
// or `var/ral.zog`) goes straight to the installed place instead, `home/.local/share/
// rally-championship`: it is played there with its saves and settings, with no installer. Takes {files: [File, ..]} (zips) or {folder: [{file, path}, ..]} (a
// picked folder's files, `path` below the folder), picked by the player. Posts {progress, text}
// as it goes, then {done} or {error}. Zips without ZIP64 (under 4 GB, under 65,536 entries),
// entries stored or deflated; several zips unpack into the one folder.

const enc = new TextDecoder();

async function dirFor(root, parts) {
  let d = root;
  for (const p of parts) d = await d.getDirectoryHandle(p, { create: true });
  return d;
}

// A stream's chunks (Safari's streams cannot be iterated with `for await`).
async function* chunks(stream) {
  const reader = stream.getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) return;
    yield value;
  }
}

// Where the installer puts the game (`tools/web/index.html`, `src/platform/web.rs`).
const INSTALLED = ["home", ".local", "share", "rally-championship"];

// Whether the paths (below the game folder) are those of an installed game.
function isInstalled(paths) {
  return paths.some((p) => /^var\/(install\.cfg|ral\.zog)$/i.test(p.replace(/\\/g, "/").replace(/^\/+/, "")));
}

// What an import leaves out: the replay videos the desktop program renders (gigabytes, and
// the browser's storage is small), and the desktop program's own copies.
function skipped(parts) {
  const top = parts[0].toLowerCase();
  return (parts[1] && top === "savedata" && parts[1].toLowerCase() === "videos") || top === "instances";
}

function u16(v, at) { return v.getUint16(at, true); }
function u32(v, at) { return v.getUint32(at, true); }

// The entries of `file` from its central directory: name, method, sizes, local header place.
async function entries(file) {
  const tail = Math.min(file.size, 65557);
  const end = new DataView(await file.slice(file.size - tail).arrayBuffer());
  let eocd = -1;
  for (let i = tail - 22; i >= 0; i--) {
    if (u32(end, i) === 0x06054b50) { eocd = i; break; }
  }
  if (eocd < 0) throw new Error(file.name + ": not a zip (no end record)");
  const count = u16(end, eocd + 10);
  const size = u32(end, eocd + 12);
  const at = u32(end, eocd + 16);
  if (at === 0xffffffff || count === 0xffff) throw new Error(file.name + ": ZIP64 is not read");
  // A central directory of 64 bytes or more per entry and name is not worth reading whole.
  if (size > 64 * 1024 * 1024 || at + size > file.size) throw new Error(file.name + ": a broken index");
  const dir = new DataView(await file.slice(at, at + size).arrayBuffer());
  const list = [];
  let p = 0;
  for (let n = 0; n < count; n++) {
    if (u32(dir, p) !== 0x02014b50) throw new Error(file.name + ": a broken index");
    const method = u16(dir, p + 10);
    const packed = u32(dir, p + 20);
    const unpacked = u32(dir, p + 24);
    const nameLen = u16(dir, p + 28), extraLen = u16(dir, p + 30), noteLen = u16(dir, p + 32);
    const local = u32(dir, p + 42);
    const name = enc.decode(new Uint8Array(dir.buffer, p + 46, nameLen));
    list.push({ name, method, packed, unpacked, local });
    p += 46 + nameLen + extraLen + noteLen;
  }
  return list;
}

async function unpack(file, root, done, total, report) {
  for (const e of await entries(file)) {
    const parts = e.name.replace(/\\/g, "/").split("/").filter((s) => s && s !== "." && s !== "..");
    if (parts.length === 0 || skipped(parts)) continue;
    if (e.name.endsWith("/")) { await dirFor(root, parts); continue; }
    const head = new DataView(await file.slice(e.local, e.local + 30).arrayBuffer());
    if (u32(head, 0) !== 0x04034b50) throw new Error(e.name + ": a broken entry");
    const start = e.local + 30 + u16(head, 26) + u16(head, 28);
    let stream = file.slice(start, start + e.packed).stream();
    if (e.method === 8) stream = stream.pipeThrough(new DecompressionStream("deflate-raw"));
    else if (e.method !== 0) throw new Error(e.name + ": packed in a way not read (" + e.method + ")");
    const folder = await dirFor(root, parts.slice(0, -1));
    const handle = await folder.getFileHandle(parts[parts.length - 1], { create: true });
    const out = await handle.createSyncAccessHandle();
    try {
      out.truncate(0);
      let at = 0;
      for await (const chunk of chunks(stream)) {
        out.write(chunk, { at });
        at += chunk.byteLength;
        // A packed entry that grows past what the index says is stopped before it fills the storage.
        if (at > e.unpacked) throw new Error(e.name + ": more than the index says (" + e.unpacked + " bytes)");
      }
      if (at !== e.unpacked) throw new Error(e.name + ": " + at + " bytes, the index says " + e.unpacked);
    } finally {
      out.flush();
      out.close();
    }
    done.files++;
    done.bytes += e.packed;
    if (done.files % 20 === 0) report(done.bytes / total, done.files + " files");
  }
}

// One picked folder's file copied into the storage at `path`.
async function copy(file, path, root) {
  const parts = path.split("/").filter((s) => s && s !== "." && s !== "..");
  if (parts.length === 0 || skipped(parts)) return false;
  const folder = await dirFor(root, parts.slice(0, -1));
  const handle = await folder.getFileHandle(parts[parts.length - 1], { create: true });
  const out = await handle.createSyncAccessHandle();
  try {
    out.truncate(0);
    let at = 0;
    for await (const chunk of chunks(file.stream())) {
      out.write(chunk, { at });
      at += chunk.byteLength;
    }
  } finally {
    out.flush();
    out.close();
  }
  return true;
}

onmessage = async (m) => {
  const report = (progress, text) => postMessage({ progress, text });
  try {
    const top = await navigator.storage.getDirectory();
    const done = { files: 0, bytes: 0 };
    let installed;
    if (m.data.folder) {
      installed = isInstalled(m.data.folder.map((e) => e.path));
      const root = await dirFor(top, installed ? INSTALLED : ["source"]);
      const total = m.data.folder.reduce((n, e) => n + e.file.size, 0) || 1;
      for (const e of m.data.folder) {
        const copied = await copy(e.file, e.path, root);
        done.bytes += e.file.size;
        if (copied && ++done.files % 20 === 0) report(done.bytes / total, done.files + " files");
      }
    } else {
      const files = m.data.files || [];
      const names = [];
      for (const f of files) for (const e of await entries(f)) names.push(e.name);
      installed = isInstalled(names);
      const root = await dirFor(top, installed ? INSTALLED : ["source"]);
      const total = files.reduce((n, f) => n + f.size, 0) || 1;
      for (const f of files) await unpack(f, root, done, total, report);
    }
    postMessage({ done: done.files, installed });
  } catch (e) {
    postMessage({ error: String(e && e.message ? e.message : e) });
  }
};
