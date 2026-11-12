#!/usr/bin/env python3
"""M7 step 5's codemod: the tests off IDA addresses. Run from the repository root.

  codemod.py collect <module|file>...        IDA-looking literals in test text, `hex file:line`
  codemod.py report <names> <module|file>... each test line with an idiom, its literals named
  codemod.py rewrite <names> <module|file>... the exact rewrites, in place; prints what is left

<names> is what `tools/m7_tests/freeze.sh names <collected> <names>` wrote (an IDA address ->
`store:<name> <offset> <field path>`, or `image`). The exact rewrites:

  R1 `offset_of!(T, f), 0xADDR - START)` (START a hex constant of the file, or `T::START` of
     an `impl T` in it) -> the offset as a hex number.
  R2 `crate::gfx::first_values_are_the_games(&BLOCK, A, B)` ->
     `crate::test_support::first_values::assert_block_starts_as_the_games(&BLOCK)`, where A is
     a constant of the file whose address is a registry store's first byte.
  R3 `image_bytes(0xADDR, LEN)` (LEN a hex or decimal literal) -> `first_values::bytes(KEY)`,
     KEY the store's name, field path and `/LEN` for a store's address, or for an image constant the
     `super::NAME` the statement compares with; the line `KEY 0xADDR LEN` is appended to
     `tools/m7_tests/first_values.manifest` (run `freeze.sh` after). Others are reported.
"""
import os
import re
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from count_unsafe import strip_tests  # noqa: E402
from m6_count import test_only_files  # noqa: E402

LIT = re.compile(r'\b0x0*((?:4[0-9A-Fa-f]|5[0-9A-Fa-f]|6[0-9A-Fa-f]|7[0-4])[0-9A-Fa-f_]{4})\b')
IDIOM = re.compile(r'\b(?:image_bytes|moved::place|moved::named(?:_in)?|moved::snapshot|'
                   r'moved::restore|host_at|game_addr|game_ptr|game_read|game_write|'
                   r'pointers::host|GamePtr::new|asm_base|OldPlaces|IMAGE_BASE|home_of_rva)\b'
                   r'|(?<![\w.])rva\(|\bfn home\(\)')
MANIFEST = 'tools/m7_tests/first_values.manifest'


def files_of(args):
    for a in args:
        if a.endswith('.rs'):
            yield a if os.path.exists(a) else os.path.join('src', a)
            continue
        root = a if a.startswith('src') else os.path.join('src', a)
        if os.path.isfile(root + '.rs'):
            yield root + '.rs'
        for d, _, fs in os.walk(root):
            for f in sorted(fs):
                if f.endswith('.rs'):
                    yield os.path.join(d, f)


def test_spans(path, text):
    """(start, end) character spans of the file's test text."""
    if path in TEST_ONLY:
        return [(0, len(text))]
    spans = []
    pat = re.compile(r'#\[cfg\((?:test|all\([^\]]*\btest\b[^\]]*\)|any\([^\]]*\btest\b[^\]]*\))\)\]')
    for m in pat.finditer(text):
        k, e = text.find('{', m.end()), text.find(';', m.end())
        if k < 0 or 0 <= e < k:
            spans.append((m.start(), e + 1))
            continue
        d, p = 0, k
        while p < len(text):
            d += {'{': 1, '}': -1}.get(text[p], 0)
            if d == 0:
                break
            p += 1
        spans.append((m.start(), p + 1))
    return spans


def in_spans(spans, at):
    return any(a <= at < b for a, b in spans)


def test_lines(path):
    text = open(path, errors='replace').read()
    spans = test_spans(path, text)
    at = 0
    for n, line in enumerate(text.split('\n'), 1):
        code = line.split('//')[0]
        if in_spans(spans, at) and code.strip():
            yield n, code
        at += len(line) + 1


def load_names(path):
    names = {}
    for line in open(path):
        w = line.split()
        if len(w) >= 2:
            names[int(w[0], 16)] = w[1:]
    return names


def describe(names, addr):
    w = names.get(addr)
    if not w:
        return '?'
    if w[0] == 'image':
        return 'image (not a store: a constant; name it by the Rust item)'
    key, off, field = w[0], w[1], w[2] if len(w) > 2 else '-'
    return f'{key} +{off} {field if field != "-" else ""}'.strip()


def collect(args):
    seen = {}
    for p in files_of(args):
        for n, code in test_lines(p):
            for m in LIT.finditer(code):
                seen.setdefault(int(m.group(1).replace('_', ''), 16), f'{p}:{n}')
        # The file's address constants too (R2 asks where `START` is).
        for v in consts_of(open(p, errors='replace').read()).values():
            if LIT.fullmatch(f'{v:#x}'):
                seen.setdefault(v, f'{p}: const')
    for a in sorted(seen):
        print(f'{a:#x} {seen[a]}')


def report(names, args):
    for p in files_of(args):
        out = []
        for n, code in test_lines(p):
            lits = [int(m.group(1).replace('_', ''), 16) for m in LIT.finditer(code)]
            if IDIOM.search(code) or lits:
                out.append(f'  {n}: {code.strip()}')
                for a in lits:
                    out.append(f'      {a:#x} = {describe(names, a)}')
        if out:
            print(p)
            print('\n'.join(out))


def module_path(path):
    p = path[len('src/'):-len('.rs')]
    if p.endswith('/mod'):
        p = p[:-len('/mod')]
    return p.replace('/', '.')


def block_end(text, k):
    """The index past the `}` closing the `{` at `k`."""
    d = 0
    for p in range(k, len(text)):
        d += {'{': 1, '}': -1}.get(text[p], 0)
        if d == 0:
            return p + 1
    return len(text)


CONST = re.compile(r'\bconst\s+(\w+)\s*:\s*usize\s*=\s*0x([0-9A-Fa-f_]+)\s*;')


def consts_of(text):
    """The file's hex constants that name one value: `START` (defined once outside any
    `impl`) and `T::START` (defined once in `impl T`'s braces) -> value."""
    found, impls = {}, []
    for m in re.finditer(r'\bimpl\s+(\w+)\s*\{', text):
        k = m.end() - 1
        end = block_end(text, k)
        impls.append((k, end))
        for c in CONST.finditer(text, k, end):
            found.setdefault(f'{m.group(1)}::{c.group(1)}', []).append(
                int(c.group(2).replace('_', ''), 16))
    # A name also brought in by a `use` may mean another module's constant.
    imported = set(re.findall(r'\b(\w+)\b', ' '.join(re.findall(r'\buse\s[^;]*;', text))))
    for c in CONST.finditer(text):
        if c.group(1) in imported:
            continue
        if not any(a <= c.start() < b for a, b in impls):
            found.setdefault(c.group(1), []).append(int(c.group(2).replace('_', ''), 16))
    return {k: v[0] for k, v in found.items() if len(set(v)) == 1 and len(v) == 1}


def rewrite(names, args):
    added = []
    keys = set()
    if os.path.exists(MANIFEST):
        keys = {line.split()[0] for line in open(MANIFEST) if line.strip() and line[0] != '#'}
    for p in files_of(args):
        text = open(p, errors='replace').read()
        spans = test_spans(p, text)
        consts = consts_of(text)
        edits = []

        def r1(m):
            base = consts.get(m.group(3))
            if base is None or not in_spans(spans, m.start()):
                return None
            return m.group(1) + f'{int(m.group(2).replace("_", ""), 16) - base:#x}'
        for m in re.finditer(r'(offset_of!\([^()]*\),\s*)0x([0-9A-Fa-f_]+)\s*-\s*((?:\w+::)?START)\b',
                             text):
            new = r1(m)
            if new is not None:
                edits.append((m.start(), m.end(), new))
        for m in re.finditer(r'crate::gfx::first_values_are_the_games\(\s*(&[^,]+?),\s*([\w:]+),\s*[^,()]+,?\s*\)',
                             text):
            # Only where the block's first byte is a registry store's first byte.
            start = consts.get(m.group(2))
            w = names.get(start) if start is not None else None
            if in_spans(spans, m.start()) and w and w[0] != 'image' and w[1] == '0x0':
                edits.append((m.start(), m.end(),
                              'crate::test_support::first_values::assert_block_starts_as_the_games('
                              + m.group(1) + ')'))
        for m in re.finditer(r'(?:crate::moved::)?image_bytes\(\s*0x([0-9A-Fa-f_]+)\s*,\s*'
                             r'(0x[0-9A-Fa-f_]+|\d+)\s*\)', text):
            if not in_spans(spans, m.start()):
                continue
            addr = int(m.group(1).replace('_', ''), 16)
            length = int(m.group(2).replace('_', ''), 0)
            w = names.get(addr)
            key = None
            if w and w[0] != 'image':
                # A read from a store's first byte is named by the store alone.
                field = w[2] if len(w) > 2 and w[2] != '-' and w[1] != '0x0' else ''
                key = w[0][len('store:'):] + field + f'/{length}'
            else:
                tail = text[m.end():text.find(';', m.end())]
                c = re.search(r'super::(\w+)', tail)
                if c:
                    key = f'{module_path(p)}.{c.group(1)}'
            if key is None:
                continue
            edits.append((m.start(), m.end(),
                          f'crate::test_support::first_values::bytes("{key}")'))
            if key not in keys:
                keys.add(key)
                added.append(f'{key} {addr:#x} {length:#x}')
        if not edits:
            continue
        for a, b, new in sorted(edits, reverse=True):
            text = text[:a] + new + text[b:]
        text = re.sub(r'\n\s*use crate::moved::image_bytes;', '', text) \
            if not re.search(r'(?<![:\w])image_bytes\(', text) else text
        open(p, 'w').write(text)
        print(f'{p}: {len(edits)} rewritten')
    if added:
        with open(MANIFEST, 'a') as f:
            f.write('\n' + '\n'.join(added) + '\n')
        print(f'{len(added)} manifest lines added: run tools/m7_tests/freeze.sh')
    report(names, args)


TEST_ONLY = set()


def main():
    global TEST_ONLY
    TEST_ONLY = test_only_files()
    cmd, rest = sys.argv[1], sys.argv[2:]
    if cmd == 'collect':
        collect(rest)
    elif cmd == 'report':
        report(load_names(rest[0]), rest[1:])
    elif cmd == 'rewrite':
        rewrite(load_names(rest[0]), rest[1:])
    else:
        sys.exit(__doc__)


if __name__ == '__main__':
    main()
