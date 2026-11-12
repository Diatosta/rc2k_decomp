#!/usr/bin/env python3
"""Writes a store's `new()` as field-by-field literals from the race core's run files.

usage: gen.py <src/race/store_data dir> <store file> <struct name> <START> <END>

Prints the replacement for the `cells_of_bytes(first_values::<{ END - START }>(START))` body
expression: `Self { field: value, .., ..ZeroCells::ZERO }`. Fields that start zero are left to
`ZeroCells::ZERO`. Dwords that look like IDA addresses get a `// an IDA address: wave 3` note.
"""
import re, struct, sys, os

def load_memory(dirname):
    mod = open(os.path.join(dirname, 'mod.rs')).read()
    pieces = re.findall(r'Piece\s*\{\s*start:\s*(0x[0-9A-Fa-f]+),\s*runs:\s*([a-z0-9_]+)::RUNS', mod)
    mem = {}
    for start, name in pieces:
        text = open(os.path.join(dirname, name + '.rs')).read()
        text = re.sub(r'//.*', '', text)
        body = text[text.index('RUNS'):]
        for m in re.finditer(r'\(\s*(0x[0-9a-fA-F]+|\d+)\s*,\s*&\[([^\]]*)\]\s*,?\s*\)', body):
            off = int(m.group(1), 0)
            data = [int(x, 16) for x in re.findall(r'0x([0-9a-fA-F]{2})', m.group(2))]
            for i, b in enumerate(data):
                mem[int(start, 16) + off + i] = b
    return mem

SIZES = {'PlainDword': 4, 'Dword': 4, 'PlainFloat': 4, 'Float': 4, 'PlainFlag': 4, 'Flag': 4,
         'PlainWord': 2, 'Word': 2, 'PlainByte': 1, 'Byte': 1}

def parse_structs(src):
    structs = {}
    for m in re.finditer(r'pub struct (\w+)\s*\{(.*?)\n\}', src, re.S):
        fields = []
        for fm in re.finditer(r'^\s*pub (\w+):\s*([^,\n]+),', m.group(2), re.M):
            fields.append((fm.group(1), fm.group(2).strip()))
        structs[m.group(1)] = fields
    return structs

def num(s):
    return int(s, 0)

def size_of(ty, structs):
    ty = ty.strip()
    m = re.match(r'\[(.+);\s*(\w+)\]$', ty)
    if m:
        return size_of(m.group(1), structs) * num(m.group(2))
    m = re.match(r'(?:Plain)?Bytes<(\w+)>$', ty)
    if m:
        return num(m.group(1))
    if ty in SIZES:
        return SIZES[ty]
    if ty in structs:
        return sum(size_of(t, structs) for _, t in structs[ty])
    raise SystemExit('unknown type ' + ty)

def f32_literal(bits):
    f = struct.unpack('<f', struct.pack('<I', bits))[0]
    if f != f or f in (float('inf'), float('-inf')):
        return None
    if f != 0 and abs(f) < 1.2e-38:
        return None
    if bits == 0x80000000:
        return '-0.0'
    for p in range(1, 10):
        s = '%.*g' % (p, f)
        if struct.unpack('<I', struct.pack('<f', float(s)))[0] == bits:
            break
    else:
        return None
    from decimal import Decimal
    d = Decimal(s)
    if -5 <= d.adjusted() <= 9:
        s = format(d, 'f')
    elif 'e' in s:
        mant, exp = s.split('e')
        if '.' not in mant:
            mant += '.0'
        s = mant + 'e' + str(int(exp))
    if 'e' not in s and '.' not in s:
        s += '.0'
    return s

def is_addr(v):
    return 0x400000 <= v < 0xA8C180

def hexs(v):
    return '0x%X' % v if v > 9 else str(v)

class Gen:
    def __init__(self, mem, structs):
        self.mem, self.structs = mem, structs
        self.addrs = []

    def raw(self, at, n):
        return bytes(self.mem.get(at + i, 0) for i in range(n))

    def scalar(self, ty, at):
        """Rust expression of a one-cell value, or None when all zero."""
        b = self.raw(at, SIZES[ty])
        v = int.from_bytes(b, 'little')
        if v == 0:
            return None
        if ty in ('PlainDword', 'Dword'):
            return '%s::new(%s)' % (ty, hexs(v)) + ('' if not is_addr(v) else ' /*ADDR*/')
        if ty in ('PlainFloat', 'Float'):
            lit = f32_literal(v)
            if lit is None:
                return '%s::new(f32::from_bits(0x%08X))' % (ty, v)
            return '%s::new(%s)' % (ty, lit)
        if ty in ('PlainFlag', 'Flag'):
            if v == 1:
                return '%s::new(true)' % ty
            raise SystemExit('flag with value %d at %x' % (v, at))
        return '%s::new(%s)' % (ty, hexs(v))

    def value(self, ty, at):
        """(expression or None when zero, count of dword addresses)."""
        ty = ty.strip()
        m = re.match(r'\[(.+);\s*(\w+)\]$', ty)
        if m:
            et, n = m.group(1).strip(), num(m.group(2))
            es = size_of(et, self.structs)
            elems = [self.value(et, at + i * es) for i in range(n)]
            nz = [(i, e) for i, e in enumerate(elems) if e is not None]
            if not nz:
                return None
            if len(nz) * 2 >= n:
                items = [e if e is not None else self.zero(et) for e in elems]
                return '[' + ', '.join(items) + ']'
            lines = ['{', 'let mut a: %s = ZeroCells::ZERO;' % ty]
            for i, e in nz:
                lines.append('a[%d] = %s;' % (i, e))
            lines.append('a }')
            return ' '.join(lines)
        m = re.match(r'(Plain)?Bytes<(\w+)>$', ty)
        if m:
            n = num(m.group(2))
            b = self.raw(at, n)
            if not any(b):
                return None
            kind = 'PlainBytes' if m.group(1) else 'Bytes'
            return '%s::new(%s)' % (kind, bytestr(b))
        if ty in SIZES:
            return self.scalar(ty, at)
        if ty in self.structs:
            return self.record(ty, at)
        raise SystemExit('type ' + ty)

    def zero(self, ty):
        return '%s::new(%s)' % (ty, '0.0' if 'Float' in ty else 'false' if 'Flag' in ty else '0') if ty in SIZES else 'ZeroCells::ZERO'

    def record(self, name, at):
        out, off = [], 0
        for fname, ty in self.structs[name]:
            v = self.value(ty, at + off)
            if v is not None:
                if '/*ADDR*/' in v:
                    n = v.count('/*ADDR*/')
                    v = v.replace(' /*ADDR*/', '')
                    note = '// an IDA address: wave 3' if n == 1 and not v.startswith('[') and not v.startswith('{') else '// IDA addresses in it: wave 3'
                    out.append('\n%s\n%s: %s' % (note, fname, v))
                else:
                    out.append('%s: %s' % (fname, v))
            off += size_of(ty, self.structs)
        if not out:
            return None
        return '%s { %s, ..ZeroCells::ZERO }' % (name, ', '.join(out))

def bytestr(b):
    # Rust bytes literal when text-like, else an array
    ok = all(c == 0 or 32 <= c < 127 for c in b)
    if ok and any(c != 0 for c in b):
        s = ''
        for c in b:
            s += '\\0' if c == 0 else '\\\\' if c == 92 else '\\"' if c == 34 else chr(c)
        return '*b"%s"' % s
    return '[' + ', '.join('0x%02X' % c for c in b) + ']'

if __name__ == '__main__':
    d, f, name, start, end = sys.argv[1:6]
    mem = load_memory(d)
    structs = parse_structs(open(f).read())
    g = Gen(mem, structs)
    start, end = int(start, 0), int(end, 0)
    assert size_of(name, structs) == end - start, (size_of(name, structs), end - start)
    r = g.record(name, start)
    print(r.replace(name + ' {', 'Self {', 1) if r else 'ZeroCells::ZERO')
