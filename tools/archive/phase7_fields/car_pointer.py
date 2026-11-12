# `READ(VAR + 0xOFF)` / `WRITE(VAR + 0xOFF, v)` where VAR holds a car block's address
# (checked by hand per file) -> `Car::at(VAR).field.bits()` / `.set_bits(v)`, when OFF starts
# a field of `Car`. Run: car_pointer.py FILE DIR READ WRITE VAR [VAR ..]
import re, sys, json
P, S, RD, WR, VARS = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5:]
car = {int(k, 16) - 0x719FD8: v for k, v in json.load(open(S + '/car_map.json')).items()}
s = open(P).read(); o = s; n = 0
def close(s, i):
    d = 0
    for j in range(i, len(s)):
        if s[j] in '([': d += 1
        elif s[j] in ')]':
            d -= 1
            if d < 0: return j
    return -1
for V in VARS:
    for fn, w in ((RD, False), (WR, True)):
        pos = 0
        pat = re.compile(r'(?<![\w.])' + re.escape(fn) + r'\(' + re.escape(V) + r' \+ (0x[0-9A-Fa-f]+)' + (r',\s*' if w else r'\)'))
        while True:
            m = pat.search(s, pos)
            if not m: break
            off = int(m.group(1), 16)
            if off not in car: pos = m.end(); continue
            F = 'Car::at(%s).%s' % (V, car[off][0])
            if w:
                e = close(s, m.end())
                rep = F + '.set_bits(' + s[m.end():e].strip() + ')'
                s = s[:m.start()] + rep + s[e + 1:]
            else:
                rep = F + '.bits()'
                s = s[:m.start()] + rep + s[m.end():]
            n += 1; pos = m.start() + len(rep)
if s != o:
    if 'car_store::Car;' not in s and 'car_store::{' not in s:
        i = s.find('\nuse ') + 1; s = s[:i] + 'use crate::physics::car_store::Car;\n' + s[i:]
    open(P, 'w').write(s)
print(P, n)
