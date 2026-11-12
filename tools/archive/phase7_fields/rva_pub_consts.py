# Public `const NAME: usize = rva(0x..);` of typed scalar places -> field accesses in the
# defining file and in every file that names it through its module (path or `use`).
# The constant goes when nothing names it; `drop_unresolved.py` and `cargo fix` then
# drop the imports. ROOT must be `src` (the module paths are taken from it).
import re, os, sys, json
ROOT, S = sys.argv[1], sys.argv[2]
maps = __import__('maps').load(S)
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0]
SKIP = ('car_store.rs', 'tables_store.rs', 'work_store.rs', 'relocations.rs')
files = {}
for root, _, fs in os.walk(ROOT):
    for f in fs:
        if f.endswith('.rs') and f not in SKIP:
            p = os.path.join(root, f); files[p] = open(p).read()
orig = dict(files); added = {}
def modpath(p):
    m = p[len(ROOT) + 1:-3].replace('/', '::')
    return m[:-5] if m.endswith('::mod') else m
def subst(s, pre, n, F):
    N = pre + n
    s = re.sub(r'(?:crate::(?:framework|store)::)?game_read(?:::<u32>)?\(' + N + r'\)', F + '.bits()', s)
    s = re.sub(r'(?:crate::(?:framework|store)::)?game_read::<f32>\(' + N + r'\)', 'f32::from_bits(' + F + '.bits())', s)
    s = re.sub(r'(?:crate::(?:framework|store)::)?game_read::<i32>\(' + N + r'\)', '(' + F + '.bits() as i32)', s)
    s = re.sub(r'(?:crate::(?:framework|store)::)?game_addr\(' + N + r'\)', F + '.addr()', s)
    s = re.sub(r'(?:crate::(?:framework|store)::)?game_ptr::<(\w+)>\(' + N + r'\)', '(' + F + r'.addr() as *mut \1)', s)
    s = re.sub(r'(?:crate::(?:framework|store)::)?game_write(?:::<u32>)?\(' + N + r',\s*', F + '.set_bits(', s)
    return s
defs = []
for p, s in files.items():
    for m in re.finditer(r'^pub(?:\(crate\))? const (\w+): usize = (?:crate::(?:framework|store)::)?rva\(0x([0-9A-Fa-f]+)\);[^\n]*\n', s, re.M):
        t = typed(int(m.group(2), 16))
        if t and '[' not in t[2] or (t and t[2].endswith('[0]')): defs.append((p, m.group(0), m.group(1), t))
for p, line, n, (st, use, fld) in defs:
    F = st + '.' + fld; mp = modpath(p)
    for q in files:
        s = files[q]
        if q == p:
            s2 = subst(s, '', n, F)
        else:
            s2 = subst(s, r'(?:crate::)?' + re.escape(mp) + '::', n, F)
            imp = re.search(r'use crate::' + re.escape(mp) + r'::(\{[^}]*\b' + n + r'\b[^}]*\}|' + n + r');', s)
            if imp: s2 = subst(s2, r'(?<![\w:])', n, F)
        if s2 != s: added.setdefault(q, set()).add((st, use)); files[q] = s2
    if not any(re.search(r'\b' + n + r'\b', (files[q].replace(line, '') if q == p else files[q])) and (q == p or mp.split('::')[-1] in files[q]) for q in files):
        files[p] = files[p].replace(line, '')
for q, s in files.items():
    if s == orig[q]: continue
    for st, use in added.get(q, ()):
        if not re.search(r'use [\w:]*\b' + st + r'\b', s) and not re.search(r'\bpub (static|struct) ' + st, s):
            s = s.replace(st + '.', use + '.') if False else s
            i = s.find('\nuse ') + 1
            s = s[:i] + 'use ' + use + ';\n' + s[i:]
    open(q, 'w').write(s); print(q)
