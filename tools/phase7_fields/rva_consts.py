# Private `const NAME: usize = rva(0x..);` of typed scalar places -> field accesses
# (game_read / game_write / game_addr / game_ptr on NAME); the constant goes when unused.
import re, os, sys, json
ROOT, S = sys.argv[1], sys.argv[2]
maps = __import__('maps').load(S)
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0]
SKIP = ('car_store.rs', 'tables_store.rs', 'work_store.rs', 'relocations.rs')
for root, _, fs in os.walk(ROOT):
    for f in fs:
        if not f.endswith('.rs') or f in SKIP: continue
        p = os.path.join(root, f); s = open(p).read(); o = s; uses = set()
        for m in list(re.finditer(r'^\s*const (\w+): usize = (?:crate::(?:framework|store)::)?rva\(0x([0-9A-Fa-f]+)\);[^\n]*\n', s, re.M)):
            n, v = m.group(1), int(m.group(2), 16)
            t = typed(v)
            if not t: continue
            st, use, fld = t
            F = st + '.' + fld
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_read(?:::<u32>)?\(' + n + r'\)', F + '.bits()', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_read::<f32>\(' + n + r'\)', 'f32::from_bits(' + F + '.bits())', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_read::<i32>\(' + n + r'\)', '(' + F + '.bits() as i32)', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_addr\(' + n + r'\)', F + '.addr()', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_ptr::<(\w+)>\(' + n + r'\)', '(' + F + r'.addr() as *mut \1)', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_write(?:::<u32>)?\(' + n + r',\s*', F + '.set_bits(', s)
            if s != o: uses.add((st, use))
            if not re.search(r'\b' + n + r'\b', s.replace(m.group(0), '')):
                s = s.replace(m.group(0), '')
        if s != o:
            for st, use in uses:
                if not re.search(r'use [\w:]*\b' + st + r'\b', s):
                    i = s.find('\nuse ') + 1
                    s = s[:i] + 'use ' + use + ';\n' + s[i:]
            open(p, 'w').write(s); print(p)
