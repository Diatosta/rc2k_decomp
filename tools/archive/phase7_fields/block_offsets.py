# `g(B + [0xFE0 * n | NEXT * n | n * NEXT | 0xFE0 | NEXT +] off)` / `p(.., v)` where B names
# the cars' blocks (0x70F3EC) -> `CAR_BLOCKS.cars[n].field` (`.bits()` / `.set_bits(v)`),
# when off starts a field of `Car`. Prints each index for review. Run: block_offsets.py ROOT DIR
import re, os, sys, json
ROOT, S = sys.argv[1], sys.argv[2]
car = {int(k, 16) - 0x719FD8: v for k, v in json.load(open(S + '/car_map.json')).items()}
USE = 'use crate::race::blocks_store::CAR_BLOCKS;\n'
def close(s, i):
    d = 0
    for j in range(i, len(s)):
        if s[j] in '([': d += 1
        elif s[j] in ')]':
            d -= 1
            if d < 0: return j
    return -1
def split_top(t):
    d = 0
    for i, c in enumerate(t):
        if c in '([': d += 1
        elif c in ')]': d -= 1
        elif c == ',' and d == 0: return t[:i], t[i + 1:]
    return t, None
for root, _, fs in os.walk(ROOT):
    for f in fs:
        if not f.endswith('.rs') or f.endswith('_store.rs'): continue
        p = os.path.join(root, f); s = open(p).read(); o = s
        bases = [m.group(1) for m in re.finditer(r'const (\w+): usize = (?:rva\()?0x70F3EC\)?;', s)]
        if not bases: continue
        steps = ['0xFE0'] + [m.group(1) for m in re.finditer(r'const (\w+): usize = 0xFE0;', s)]
        st = '|'.join(re.escape(x) for x in steps)
        for B in bases:
            for fn in ('g', 'p'):
                pos = 0
                while True:
                    m = re.compile(r'\b' + fn + r'\(' + B + r'\b').search(s, pos)
                    if not m: break
                    e = close(s, m.end() - len(B))
                    inner = s[m.end() - len(B):e]
                    addr, val = split_top(inner) if fn == 'p' else (inner, None)
                    if (fn == 'p') != (val is not None): pos = m.end(); continue
                    a = re.sub(r'\s+', ' ', addr.strip())
                    mm = re.fullmatch(B + r'(?: \+ (?:(?:' + st + r') \* ([^+]+?)|([^+]+?) \* (?:' + st + r')|(' + st + r')))?(?: \+ (0x[0-9A-Fa-f]+|\d+))?', a)
                    if not mm: pos = m.end(); continue
                    n = mm.group(1) or mm.group(2)
                    idx = '1' if mm.group(3) else ('0' if n is None else n)
                    off = int(mm.group(4), 0) if mm.group(4) else 0
                    if off not in car: pos = m.end(); continue
                    if not re.fullmatch(r'\d+', idx): idx = (idx if re.fullmatch(r'\w+', idx) else '(' + idx + ')') + ' as usize'
                    F = 'CAR_BLOCKS.cars[%s].%s' % (idx, car[off][0])
                    rep = F + '.bits()' if fn == 'g' else F + '.set_bits(' + val.strip() + ')'
                    print(p, ':', a, '->', rep)
                    s = s[:m.start()] + rep + s[e + 1:]
                    pos = m.start() + len(rep)
        if s != o:
            if 'blocks_store::CAR_BLOCKS;' not in s:
                i = s.find('\nuse ') + 1; s = s[:i] + USE + s[i:]
            open(p, 'w').write(s)
