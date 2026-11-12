#!/usr/bin/env python3
"""Writes a store's `new()` from its run file, field by field.

usage: gen_literals.py <rust file with the store's struct> <struct name> <run file>

The struct's fields are read from the `pub struct` (scalar cells, arrays of them, `Bytes<N>`),
laid out in order with no padding, and the run file's bytes (src/race/store_data/*.rs) are
cut among them. Every field that starts non-zero gets a literal typed as the field is (a float
as a float literal that rounds to the same bits); the rest is `..ZeroCells::ZERO`. A dword that
looks like an IDA address (0x400000 .. 0xA80000) is listed on stderr: the caller keeps it
with a comment. The golden file (data/first_values.golden) is the check of the result."""
import re, struct, sys

SIZES = {'PlainDword': 4, 'Dword': 4, 'PlainFloat': 4, 'Float': 4, 'PlainWord': 2, 'Word': 2,
         'PlainByte': 1, 'Byte': 1, 'PlainFlag': 4, 'Flag': 4}
FMT = {4: '<I', 2: '<H', 1: '<B'}


def parse_type(t):
    t = t.strip()
    m = re.fullmatch(r'\[(.*);\s*([^;\]]+)\]', t)
    if m:
        return ('arr', parse_type(m.group(1)), eval(m.group(2)), t)
    m = re.fullmatch(r'((?:Plain)?Bytes)<(.*)>', t)
    if m:
        return ('bytes', m.group(1), eval(m.group(2)), t)
    if t in SIZES:
        return ('scalar', t, t)
    raise SystemExit('unknown type ' + t)


def size(ty):
    if ty[0] == 'scalar':
        return SIZES[ty[1]]
    if ty[0] == 'arr':
        return size(ty[1]) * ty[2]
    return ty[2]


def parse_struct(path, name):
    src = open(path).read()
    m = re.search(r'pub struct %s \{(.*?)\n\}' % name, src, re.S)
    fields = []
    for line in m.group(1).split('\n'):
        line = line.strip()
        if line.startswith('//'):
            continue
        mm = re.match(r'pub (\w+): (.*),$', line)
        if mm:
            fields.append((mm.group(1), parse_type(mm.group(2))))
    return fields


def runs_bytes(runfile, total):
    src = open(runfile).read()
    buf = bytearray(total)
    for m in re.finditer(r'\(\s*(0x[0-9a-f]+),\s*&\[(.*?)\]', src, re.S):
        off = int(m.group(1), 16)
        bs = [int(x, 16) for x in re.findall(r'0x[0-9a-f]{2}', m.group(2))]
        buf[off:off + len(bs)] = bytes(bs)
    return buf


def float_text(bits):
    x = struct.unpack('<f', struct.pack('<I', bits))[0]
    if x != x or x in (float('inf'), float('-inf')):
        return 'f32::from_bits(0x%08X)' % bits
    if x != 0 and abs(x) < 1e-30:
        return 'f32::from_bits(0x%08X)' % bits
    for p in range(1, 10):
        s = '%.*g' % (p, x)
        if struct.pack('<f', float(s)) == struct.pack('<f', x):
            if bits == 0x80000000:
                return '-0.0'
            if 'e' in s or 'inf' in s:
                return repr(float(s)) if 'e' not in s else s.replace('e', 'e')
            return s if '.' in s else s + '.0'
    raise SystemExit('float')


def scalar(ty, data, addresses, where):
    kind = ty[1]
    n = SIZES[kind]
    v = struct.unpack(FMT[n], bytes(data))[0] if n in FMT else 0
    if 'Float' in kind:
        return '%s::new(%s)' % (kind, float_text(v))
    if 'Flag' in kind:
        return '%s::new(%s)' % (kind, 'true' if v else 'false')
    if n == 4 and 0x400000 <= v < 0xA80000:
        addresses.append((where, v))
    lit = str(v) if v < 10 else ('0x%X' % v if n == 4 and v > 0xFFFF else '0x%X' % v)
    return '%s::new(%s)' % (kind, lit)


def leaves(ty, data, path, out, addresses):
    """Non-zero leaves of a value as (index path, text)."""
    if ty[0] == 'scalar':
        if any(data):
            out.append((path, scalar(ty, data, addresses, path)))
    elif ty[0] == 'arr':
        s = size(ty[1])
        for i in range(ty[2]):
            leaves(ty[1], data[i * s:(i + 1) * s], path + [i], out, addresses)
    else:
        raise SystemExit('Bytes fields by hand')


def value(ty, data, addresses, where):
    out = []
    leaves(ty, data, [], out, addresses)
    # a short array whole, a long one with its non-zero elements assigned
    if ty[0] == 'scalar':
        return out[0][1]
    flat = 1
    t = ty
    while t[0] == 'arr':
        flat *= t[2]
        t = t[1]
    if ty[0] == 'arr' and ty[1][0] == 'scalar' and ty[2] <= 16:
        elems = {p[0]: text for p, text in out}
        zero = '%s::new(0)' % ty[1][1]
        return '[%s]' % ', '.join(elems.get(i, zero) for i in range(ty[2]))
    lines = ['{', 'let mut a: %s = ZeroCells::ZERO;' % ty[3]]
    for p, text in out:
        lines.append('a%s = %s;' % (''.join('[%d]' % i for i in p), text))
    lines += ['a', '}']
    return '\n'.join(lines)


def main(path, name, runfile):
    fields = parse_struct(path, name)
    total = sum(size(t) for _, t in fields)
    buf = runs_bytes(runfile, total)
    addresses = []
    body = []
    at = 0
    for n, t in fields:
        s = size(t)
        if any(buf[at:at + s]):
            body.append('%s: %s,' % (n, value(t, buf[at:at + s], addresses, n)))
        at += s
    print('Self {')
    print('\n'.join(body))
    print('..ZeroCells::ZERO')
    print('}')
    for where, v in addresses:
        print('IDA address: %s %s 0x%X' % (name, where, v), file=sys.stderr)


if __name__ == '__main__':
    main(*sys.argv[1:4])
