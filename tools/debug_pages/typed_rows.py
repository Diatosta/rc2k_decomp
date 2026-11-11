#!/usr/bin/env python3
"""Rewrites the rows of src/debug/display_rows.rs and watch_rows.rs (as watch.py and groups.py
write them: `Row::new(.., 0xADDR, ..)`) into rows read through their fields (`Row::typed`), so
that no generator leaves a row on its IDA address. Run from the repository's root; watch.py and
groups.py run it at their end.

1. Rows in the cars' blocks (the `Car` struct's layout, read from src/physics/car_store.rs):
   `car_cell!(block, field[i], 0xADDR)`.
2. The others: `cargo test dump_row_paths` (src/debug/row_paths.rs, built on the `Cells`
   derive's `field_path`) writes, for each address, the moved range holding it, the record's
   type and the field's path; `store_cell!(ida, home, Type, 0xADDR, field.path)`, or `indexed_cell!(home, field[i].path)`
   when the path has an index (`offset_of!` has none; the test `typed_rows_are_where_their_
   addresses_say` checks those). The moved
   range's start and home come from src/moved.rs's `MOVED`, entry by entry.
A row whose address is in no field of a record (a range the debug test cannot name, a byte
run) stays `Row::new`; the compiler checks every rewritten row against its address.
"""
import os, re, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
FILES = [os.path.join(ROOT, "src/debug", f) for f in ("display_rows.rs", "watch_rows.rs")]
START, BL = 0x70F3EC, 0xFE0
S = r'("(?:[^"\\]|\\.)*")'
PAT = re.compile(r"Row::new\(\s*" + S + r",\s*" + S + r",\s*(Shown::\w+),\s*(0x[0-9A-Fa-f]+),\s*(true|false),\s*\((\d+),\s*(\d+)\),?\s*\)")


def car_fields():
    t = open(os.path.join(ROOT, "src/physics/car_store.rs")).read()
    body = t[t.index("pub struct Car {"):]
    body = body[:body.index("\n}\n")]
    fs = re.findall(r"/// \+0x([0-9a-f]+)[^\n]*\n(?:\s*///[^\n]*\n)*\s*pub (\w+): ([^,]+),", body)

    def size(ty):
        if ty in ("Dword", "Float"):
            return 4
        m = re.fullmatch(r"\[(Dword|Float); (\w+|0x[0-9a-fA-F]+)\]", ty)
        if m:
            return 4 * int(m.group(2), 0)
        m = re.fullmatch(r"Bytes<(\w+)>", ty)
        if m:
            return int(m.group(1), 0)
        raise Exception(ty)

    off, out = 0, []
    for o, n, ty in fs:
        assert int(o, 16) == off, (n, o, hex(off))
        out.append((off, n, ty, size(ty)))
        off += size(ty)
    return out


def car_cell(fields, at):
    if not START <= at < START + 10 * BL:
        return None
    n, o = divmod(at - START, BL)
    for off, name, ty, sz in fields:
        if off <= o < off + sz:
            if ty.startswith("Bytes") or (o - off) % 4:
                return None
            if ty in ("Dword", "Float"):
                return f"car_cell!({n}, {name}, {at:#x})"
            return f"car_cell!({n}, {name}[{(o - off) // 4}], {at:#x})"
    return None


def moved_entries():
    src = open(os.path.join(ROOT, "src/moved.rs")).read()
    src = src[src.index("pub(crate) const MOVED"):]
    return re.findall(r"Moved \{\s*ida:\s*([^,]+),[^\n]*\n\s*len:\s*(?:.+?),\n\s*home:\s*Home::Store\(&(.*?)\),\n", src, re.S)


def row_paths():
    out = os.path.join(tempfile.mkdtemp(), "paths.tsv")
    env = dict(os.environ, RC2K_ROW_PATHS=out)
    cmd = ["cargo", "test", "--target", "x86_64-unknown-linux-gnu", "-p", "rc2k_decomp", "--lib", "dump_row_paths"]
    subprocess.run(cmd, cwd=ROOT, env=env, check=True)
    paths = {}
    for line in open(out):
        at, entry, ty, path, leaf = line.rstrip("\n").split("\t")
        if entry != "-" and leaf.split("::")[-1] in ("Dword", "Float"):
            paths[int(at, 16)] = (int(entry), ty.replace("rc2k_decomp::", "crate::"), path.lstrip("."))
    return paths


def main():
    fields = car_fields()
    entries = moved_entries()
    texts = {f: open(f).read() for f in FILES}
    # The first pass: the cars' blocks; the second needs the tree to build with its result.
    for f in FILES:
        texts[f] = PAT.sub(lambda m: rewrite(m, lambda at: car_cell(fields, at)), texts[f])
        open(f, "w").write(add_imports(texts[f]))
    paths = row_paths()

    def store_cell(at):
        if at not in paths:
            return None
        entry, ty, path = paths[at]
        ida, home = entries[entry][0].strip(), entries[entry][1].strip()
        if not path:
            return f"store_cell!({ida}, {home}, {at:#x})"
        if "[" in path:
            return f"indexed_cell!({home}, {path})"
        return f"store_cell!({ida}, {home}, {ty}, {at:#x}, {path})"

    count = 0
    for f in FILES:
        texts[f] = PAT.sub(lambda m: rewrite(m, store_cell), texts[f])
        open(f, "w").write(add_imports(texts[f]))
    left = sum(len(PAT.findall(t)) for t in texts.values())
    print("rows left on their IDA address:", left)


def rewrite(m, cell):
    name, label, shown, at, bar, x, y = m.groups()
    c = cell(int(at, 16))
    if c is None:
        return m.group(0)
    return f"Row::typed({name}, {label}, {shown}, {at}, {c}, {bar}, ({x}, {y}))"


def add_imports(text):
    text = re.sub(r"use super::row_cells::\{[^}]*\};\n|use super::row_cells::\w+;\n", "", text)
    return text.replace("use super::ui::", "use super::row_cells::{car_cell, indexed_cell, store_cell};\nuse super::ui::", 1)


main()
