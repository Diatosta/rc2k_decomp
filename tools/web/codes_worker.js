// The saves page's save code worker: the game's own save code (`tools/web/codes/`, built into
// `rc2k_web_codes.wasm`) on the module's memory, so the page makes and reads the QR codes the
// game's save code page shows and takes in.
//
// Messages in: {id, op, ...}; out: {id, ...} or {id, error}.
//   encode  {files, now, large}          -> {parts: [text]}    the QR codes' texts of the saves
//   modules {text}                       -> {side, dark}       one QR code's modules (1 for dark)
//   frame   {rgba, width, height}        -> {read, held, count}  a camera frame taken into the scan
//   text    {text}                       -> {read, held, count}  a code's text taken into the scan
//   reset   {}                           -> {}                 forgets the parts collected
//   whole   {}                           -> {state, lines|error}   0 not yet, 1 complete, -1 broken
//   import  {files, packed}              -> {files, report}    what the complete scan writes
// `files` is [{path, data: Uint8Array}] (the paths of `saves.js`'s `codeFiles`).
// `read`: 0 nothing, 1 a new part, 2 one already held, 3 a part of another code, 4 not a save code.

const utf8 = new TextEncoder();
const decoder = new TextDecoder();
let wasm;

const ready = (async () => {
  const bytes = await (await fetch("rc2k_web_codes.wasm")).arrayBuffer();
  wasm = (await WebAssembly.instantiate(bytes, {})).instance.exports;
})();

function memory() { return new Uint8Array(wasm.memory.buffer); }

// Runs `fn(ptr, len)` on a copy of `bytes` in the module's memory.
function withBytes(bytes, fn) {
  const ptr = wasm.alloc(bytes.length);
  memory().set(bytes, ptr);
  try { return fn(ptr, bytes.length); } finally { wasm.dealloc(ptr, bytes.length); }
}

function result() {
  return memory().slice(wasm.result_ptr(), wasm.result_ptr() + wasm.result_len());
}

function resultText() { return decoder.decode(result()); }

// The files as the module reads them: a count, then each file's path and data with their lengths.
function packFiles(files) {
  const parts = [];
  let size = 4;
  const names = files.map((f) => utf8.encode(f.path));
  files.forEach((f, i) => { size += 8 + names[i].length + f.data.length; });
  const out = new Uint8Array(size);
  const view = new DataView(out.buffer);
  view.setUint32(0, files.length, true);
  let at = 4;
  files.forEach((f, i) => {
    view.setUint32(at, names[i].length, true); at += 4;
    out.set(names[i], at); at += names[i].length;
    view.setUint32(at, f.data.length, true); at += 4;
    out.set(f.data, at); at += f.data.length;
  });
  return out;
}

function unpackFiles(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const files = [];
  let at = 4;
  for (let n = view.getUint32(0, true); n > 0; n--) {
    const nameLen = view.getUint32(at, true); at += 4;
    const path = decoder.decode(bytes.subarray(at, at + nameLen)); at += nameLen;
    const dataLen = view.getUint32(at, true); at += 4;
    files.push({ path, data: bytes.slice(at, at + dataLen) }); at += dataLen;
  }
  return files;
}

function check(status) {
  if (status !== 0) throw new Error(resultText());
}

function progress(read) {
  const [held, count] = resultText().split(",").map(Number);
  return { read, held, count };
}

const ops = {
  encode(m) {
    check(withBytes(packFiles(m.files), (p, n) => wasm.encode(p, n, m.now, m.large ? 1 : 0)));
    return { parts: resultText().split("\n") };
  },
  modules(m) {
    check(withBytes(utf8.encode(m.text), (p, n) => wasm.qr_modules(p, n)));
    const out = result();
    const side = new DataView(out.buffer).getUint32(0, true);
    return { side, dark: out.subarray(4) };
  },
  frame(m) {
    const read = withBytes(new Uint8Array(m.rgba), (p) => wasm.scan_frame(p, m.width, m.height));
    return read === 0 ? { read, held: 0, count: 0 } : progress(read);
  },
  text(m) {
    return progress(withBytes(utf8.encode(m.text), (p, n) => wasm.scan_text(p, n)));
  },
  reset() { wasm.scan_reset(); return {}; },
  whole() {
    const state = wasm.scan_whole();
    if (state === 0) return { state };
    const text = resultText();
    return state === 1 ? { state, lines: text.split("\n").filter(Boolean) } : { state, error: text };
  },
  import(m) {
    check(withBytes(packFiles(m.files), (p, n) => wasm.scan_import(p, n, m.packed)));
    const files = unpackFiles(result());
    const report = files.find((f) => f.path === ":report");
    return {
      files: files.filter((f) => f.path !== ":report"),
      report: decoder.decode(report.data).split("\n").filter(Boolean),
    };
  },
};

onmessage = async (e) => {
  const { id, op } = e.data;
  try {
    await ready;
    const out = ops[op](e.data);
    postMessage({ id, ...out }, out.dark ? [out.dark.buffer] : []);
  } catch (err) {
    postMessage({ id, error: String(err && err.message ? err.message : err) });
  }
};
