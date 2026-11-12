# Inline `game_read[::<T>](rva(0xTYPED))` / `game_write(rva(0xTYPED), v)` of typed scalars
# -> field reads / writes (u32 bits; f32 / i32 through the bits). Run: inline_reads.py ROOT DIR
import re, os, sys, json
ROOT, S = sys.argv[1], sys.argv[2]
maps = __import__('maps').load(S)
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0]
for root, _, fs in os.walk(ROOT):
    for f in fs:
        if not f.endswith('.rs') or f.endswith('_store.rs') or f == 'relocations.rs': continue
        p = os.path.join(root, f); s = open(p).read(); o = s; uses = set()
        def rep(m):
            t = typed(int(m.group(2), 16))
            if not t: return m.group(0)
            F = t[0] + '.' + t[2]; uses.add(t)
            ty = m.group(1)
            if ty in (None, 'u32'): return F + '.bits()'
            if ty == 'f32': return 'f32::from_bits(' + F + '.bits())'
            if ty == 'i32': return '(' + F + '.bits() as i32)'
            return m.group(0)
        s = re.sub(r'(?:crate::(?:framework|store)::)?game_read(?:::<(\w+)>)?\((?:crate::(?:framework|store)::)?rva\(0x([0-9A-Fa-f]+)\)\)', rep, s)
        def repw(m):
            t = typed(int(m.group(1), 16))
            if not t: return m.group(0)
            uses.add(t); return t[0] + '.' + t[2] + '.set_bits('
        s = re.sub(r'(?:crate::(?:framework|store)::)?game_write(?:::<u32>)?\((?:crate::(?:framework|store)::)?rva\(0x([0-9A-Fa-f]+)\),\s*', repw, s)
        if s != o:
            for st, use, _ in uses:
                if not re.search(r'use [\w:]*\b' + st + r'\b', s):
                    i = s.find('\nuse ') + 1; s = s[:i] + 'use ' + use + ';\n' + s[i:]
            open(p, 'w').write(s); print(p)
