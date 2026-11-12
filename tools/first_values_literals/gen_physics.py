#!/usr/bin/env python3
"""Turns the dump of `dump.rs` (store, path, type, offset, hex) into a store's `new()`:
the non-zero cells named field by field, `..ZeroCells::ZERO` for the rest.

usage: gen.py <dump.tsv> <store key> <rust file> <struct name>
Replaces the body of `pub const fn new() -> Self { ... }` of `impl <struct>` in the file and
prints the IDA-address-looking dwords it kept (marked `an IDA address: wave 3`)."""
import re, struct, sys

dump, key, path, name = sys.argv[1:5]
src = open(path).read()

# top-level field types of the struct
m = re.search(r'pub struct %s\b[^{]*\{(.*?)\n\}' % name, src, re.S)
types = dict(re.findall(r'^\s*pub (\w+): (.+?),\s*$', m.group(1), re.M))

entries = []
for line in open(dump):
    k, p, ty, off, hx = line.rstrip('\n').split('\t')
    if k != key:
        continue
    entries.append((p, ty.split('::')[-1], int(off, 16), bytes.fromhex(hx)))

def tokens(p):
    return re.findall(r'\.(\w+)|\[(\d+)\]', p)

def f32(bits):
    exp = (bits >> 23) & 0xFF
    if bits == 0x80000000:
        return '-0.0'
    if exp == 0 or exp == 255:
        return 'f32::from_bits(0x%X)' % bits
    v = struct.unpack('<f', struct.pack('<I', bits))[0]
    for prec in range(1, 10):
        s = '%.*g' % (prec, v)
        if struct.pack('<f', float(s)) == struct.pack('<f', v):
            break
    if 'e' in s or 'E' in s:
        s = repr(float(s))
    elif '.' not in s:
        s += '.0'
    return s

def num(v):
    return str(v) if v < 0x100 else '0x%X' % v

addresses = []

def leaf(ty, data, where):
    plain = ty.startswith('Plain')
    base = ty[5:] if plain else ty
    if base.startswith('Bytes'):
        n = int(re.search(r'<(\d+)>', base).group(1))
        runs = []
        i = 0
        while i < n:
            if data[i] == 0:
                i += 1
                continue
            j = i
            zeros = 0
            end = i
            while j < n and zeros < 8:
                if data[j] == 0:
                    zeros += 1
                else:
                    zeros = 0
                    end = j
                j += 1
            runs.append((i, data[i:end + 1]))
            i = end + 1
        body = ', '.join('(%d, b"%s")' % (a, ''.join(esc(c) for c in b)) for a, b in runs)
        return '%s%s::new(with_runs::<%d>(&[%s]))' % ('Plain' if plain else '', 'Bytes', n, body), None
    v = int.from_bytes(data, 'little')
    cell = ('Plain' if plain else '') + base
    if base == 'Float':
        return '%s::new(%s)' % (cell, f32(v)), None
    if base == 'Flag':
        assert v in (0, 1), (where, v)
        return '%s::new(true)' % cell, None
    note = None
    if base == 'Dword' and 0x400000 <= v < 0x800000:
        note = 'an IDA address: wave 3'
        addresses.append((where, '0x%X' % v))
    return '%s::new(%s)' % (cell, num(v)), note

def esc(c):
    if c == 0x22: return '\\"'
    if c == 0x5C: return '\\\\'
    if 0x20 <= c < 0x7F: return chr(c)
    return '\\x%02x' % c

groups = {}
order = []
for p, ty, off, data in entries:
    t = tokens(p)
    top = t[0][0]
    if top not in groups:
        groups[top] = []
        order.append(top)
    groups[top].append((t[1:], ty, data, p))

fields = []
for top in order:
    items = groups[top]
    if len(items) == 1 and not items[0][0]:
        _, ty, data, p = items[0]
        e, note = leaf(ty, data, p)
        fields.append((top, e, note))
        continue
    ft = types[top]
    lines = ['{', 'let mut v: %s = ZeroCells::ZERO;' % ft]
    for rest, ty, data, p in items:
        acc = ''.join('.' + a if a else '[%s]' % b for a, b in rest)
        e, note = leaf(ty, data, p)
        lines.append('v%s = %s;%s' % (acc, e, ' // ' + note if note else ''))
    lines += ['v', '}']
    fields.append((top, '\n'.join(lines), None))

body = ['pub const fn new() -> Self {']
used = set()
text = ''.join(e for _, e, _ in fields)
uses = sorted(set(re.findall(r'\b(Plain(?:Bytes|Dword|Float|Flag|Word|Byte|Loose)|Bytes|Dword|Float|Flag|Word|Byte|Loose)::new', text)))
if 'Bytes' in text or 'PlainBytes' in text:
    uses.append('with_runs')
body.append('use crate::cells::ZeroCells;')
body.append('use crate::session::{%s};' % ', '.join(uses))
body.append('Self {')
for top, e, note in fields:
    body.append('%s: %s,%s' % (top, e, ' // ' + note if note else ''))
body.append('..ZeroCells::ZERO')
body.append('}')
body.append('}')
new_fn = '\n'.join(body)

# replace the function
pat = re.compile(r'(impl %s \{\n(?:\s*///.*\n)*)(\s*pub const fn new\(\) -> Self \{.*?\n    \}\n)(\}\n)' % name, re.S)
mm = pat.search(src)
assert mm, 'no new() found'
src = src[:mm.start(2)] + new_fn + '\n' + src[mm.end(2):]
open(path, 'w').write(src)
for w, a in addresses:
    print('ADDR', key, w, a)
