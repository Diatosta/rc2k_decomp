#!/usr/bin/env python3
"""The frame census's edges by name (`RC2K_FRAME_CENSUS`, `src/write_watch.rs`).

    tools/frame_census.py LOG [STEP_FILTER]

LOG is a watch build's rc2k.log; its last "census:" block is read. Each edge "writer ->
reader" is printed with the store fields it hands over, named from the stores' doc comments
(`/// +0x4 (0x641590): ...` above `pub name: Type`). A filter keeps the edges whose text
contains it (e.g. "mirror0").
"""
import bisect
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'src')
DOC = re.compile(r'^\s*/// \+0x([0-9a-fA-F]+) \(0x([0-9a-fA-F]+)\)')
FIELD = re.compile(r'^\s*pub (\w+):')
STRUCT = re.compile(r'^\s*pub struct (\w+)')


def fields():
    """(ida, 'Struct.field', file) for every documented field, sorted by ida."""
    out = []
    for folder, _, files in os.walk(ROOT):
        for f in files:
            if not f.endswith('.rs'):
                continue
            path = os.path.join(folder, f)
            lines = open(path, errors='replace').read().split('\n')
            struct = '?'
            for i, line in enumerate(lines):
                m = STRUCT.match(line)
                if m:
                    struct = m.group(1)
                m = DOC.match(line)
                if not m:
                    continue
                for nxt in lines[i + 1:i + 8]:
                    fm = FIELD.match(nxt)
                    if fm:
                        rel = os.path.relpath(path, ROOT)
                        out.append((int(m.group(2), 16), f'{struct}.{fm.group(1)}', rel))
                        break
    out.sort()
    return out


CAR_BLOCKS = (0x70F3EC, 0x7194BC, 0xFE0, 0x719FD8)  # `race::blocks_store`: ten work cars' layout


def moved_ranges():
    """(ida, len, home) of `moved::MOVED`'s entries: the names of places no field documents."""
    text = open(os.path.join(ROOT, 'moved.rs'), errors='replace').read()
    out = []
    for m in re.finditer(r'ida: (0x[0-9A-Fa-f]+),[^\n]*\n\s*len: ([^,]+),\s*\n\s*home: ([^\n]+)', text):
        try:
            length = eval(m.group(2).replace('crate::', '').split('::')[-1]) if m.group(2).strip().isdigit() or m.group(2).startswith('0x') else 0
        except Exception:
            length = 0
        home = re.sub(r'Home::\w+\(|&|crate::|\.first\(\)|\)|,$', '', m.group(3)).strip()
        out.append((int(m.group(1), 16), length, home))
    return out


def last_block(log):
    lines = [l.rstrip('\n') for l in open(log, errors='replace') if l.startswith('census: ')]
    starts = [i for i, l in enumerate(lines) if re.match(r'census: \d+ frames', l)]
    return lines[starts[-1]:] if starts else []


def main():
    log = sys.argv[1]
    want = sys.argv[2] if len(sys.argv) > 2 else ''
    table = fields()
    keys = [t[0] for t in table]

    moved = moved_ranges()

    def names(start, length):
        """The fields over [start, start + length)."""
        lo, hi, size, work = CAR_BLOCKS
        if lo <= start < hi:
            n, off = divmod(start - lo, size)
            return [f'car_blocks[{n}].' + names(work + off, min(length, size - off))[0].split('.', 1)[-1]]
        got = []
        k = bisect.bisect_right(keys, start) - 1
        k = max(k, 0)
        while k < len(table) and table[k][0] < start + length:
            ida, name, _ = table[k]
            nxt = table[k + 1][0] if k + 1 < len(table) else ida + 4
            if nxt > start and (ida <= start or ida < start + length) and nxt - ida < 0x40000:
                got.append(name)
            k += 1
        if got:
            return got
        for ida, ln, home in moved:
            if ida <= start < ida + max(ln, 1):
                return [f'{home}+{start - ida:#x}']
        return [f'{start:#x}?']

    block = last_block(log)
    print(block[0])
    for line in block[1:]:
        m = re.match(r'census: (.+?): (\d+) dwords:(.*)', line)
        if not m or want not in m.group(1):
            continue
        counts = {}
        rest = m.group(3)
        for what, n in re.findall(r'\[([^\]+]+)\+0x[0-9a-f]+\+(0x[0-9a-f]+)\]', rest):
            counts[f'[{what}]'] = counts.get(f'[{what}]', 0) + int(n, 16) // 4
        rest = re.sub(r'\[[^\]]*\]', '', rest)
        for item in rest.split():
            if item.startswith('host:'):
                n = int(item.rsplit('+', 1)[1], 16) // 4
                counts['host (not in the image)'] = counts.get('host (not in the image)', 0) + n
                continue
            a, n = item.split('+')
            a, n = int(a, 16), int(n, 16)
            for name in names(a, n):
                counts[name] = counts.get(name, 0) + max(1, n // 4 // len(names(a, n)))
        listed = ', '.join(f'{k}' + (f' x{v}' if v > 1 else '') for k, v in
                           sorted(counts.items(), key=lambda kv: -kv[1]))
        print(f'{m.group(1)} ({m.group(2)}): {listed}')


main()
