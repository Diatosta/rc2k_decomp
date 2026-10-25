# g/p(BASE + 4 * i) inside `for i in 0..N` (N known, fits the array) -> FIELD[k + i].
import re, os, sys, json, collections
ROOT, S = sys.argv[1], sys.argv[2]
maps = __import__('maps').load(S)
lens = {}
for m, st, use in maps:
    for k, (n, t) in m.items():
        mm = re.match(r'^(.*)\[(\d+)\]$', n)
        if mm: lens[(st, mm.group(1))] = max(lens.get((st, mm.group(1)), 0), int(mm.group(2)) + 1)
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0], m[hex(v)][1]
RW = [r'fn g\(x: (usize|u32)\) -> u32 \{\s*unsafe \{ game_read\(rva\(x( as usize)?\)\) \}\s*\}', r'fn g\(x: impl crate::store::At\) -> u32 \{\s*unsafe \{ crate::store::at_read\(x\) \}\s*\}']
W = [r'fn p\(x: (usize|u32), v: u32\) \{\s*unsafe \{ game_write\(rva\(x( as usize)?\), v\) \}\s*\}', r'fn p\(x: impl crate::store::At, v: u32\) \{\s*unsafe \{ crate::store::at_write\(x, v\) \}\s*\}']
MODS = ['physics::flow', 'physics::engine_torque', 'physics::splits', 'physics::drop_hit_parts', 'physics::fill_race_slot']
def imported(s, n):
    for u in re.finditer(r'use crate::([\w:]+)::(\{[^}]*\}|\w+);', s):
        if n in re.findall(r'\w+', u.group(2)) and u.group(1) in MODS: return True
    return False
def args_end(s, i):
    d = 0
    for j in range(i, len(s)):
        if s[j] == '(': d += 1
        elif s[j] == ')':
            d -= 1
            if d == 0: return j
    return -1
def split_args(t):
    out, d, cur = [], 0, ''
    for c in t:
        if c in '([{': d += 1
        if c in ')]}': d -= 1
        if c == ',' and d == 0: out.append(cur); cur = ''
        else: cur += c
    out.append(cur); return out
ARG = re.compile(r'^\s*0x([0-9A-Fa-f]{6}) \+ 4 \* (\w+)\s*$')
tot = collections.Counter()
for root, _, fs in os.walk(os.path.join(ROOT, 'src')):
    for f in fs:
        p = os.path.join(root, f)
        if not f.endswith('.rs') or f in ('car_store.rs', 'tables_store.rs', 'work_store.rs'): continue
        s = open(p).read()
        g_ok = any(re.search(d, s) for d in RW) or imported(s, 'g')
        p_ok = any(re.search(d, s) for d in W) or imported(s, 'p')
        if not (g_ok or p_ok): continue
        out, i, n, sts = [], 0, 0, set()
        for m in re.finditer(r'(?<![\w.:])(g|p)\(', s):
            if m.start() < i: continue
            fn = m.group(1); o = m.end() - 1; e = args_end(s, o)
            if e < 0: continue
            args = split_args(s[o + 1:e]); am = ARG.match(args[0])
            if not am or (fn == 'g' and (not g_ok or len(args) != 1)) or (fn == 'p' and (not p_ok or len(args) != 2)): continue
            t = typed(int(am.group(1), 16)); var = am.group(2)
            if not t: continue
            fm = re.match(r'^(.*)\[(\d+)\]$', t[2])
            if not fm: continue
            # the nearest enclosing `for var in 0..N`
            loops = list(re.finditer(r'for (?:\(\s*)?' + var + r'\b[^{]*? in 0\.\.(\d+)(u32|usize)?', s[:m.start()]))
            if not loops: continue
            lp = loops[-1]; N = int(lp.group(1)); base = int(fm.group(2))
            if base + N > lens[(t[0], fm.group(1))]: continue
            idx = (var + ' as usize') if lp.group(2) == 'u32' else var
            if base: idx = f'{base} + {idx}'
            clash = re.search(r'\b(const|static)\s+' + t[0] + r'\b', s) is not None
            r = f'{t[1] if clash else t[0]}.{fm.group(1)}[{idx}]'
            if fn == 'g': rep = r + ('.get()' if t[3] == 'Dword' else '.bits()')
            else: rep = r + ('.set(' if t[3] == 'Dword' else '.set_bits(') + args[1].strip() + ')'
            sts.add((t[0], t[1], clash)); out.append(s[i:m.start()]); out.append(rep); i = e + 1; n += 1
        if not n: continue
        out.append(s[i:]); s2 = ''.join(out)
        for st, use, clash in sts:
            if not clash and f'use {use};' not in s2:
                k = re.search(r'^use [^;]+;\n', s2, re.M).end(); s2 = s2[:k] + f'use {use};\n' + s2[k:]
        open(p, 'w').write(s2); tot[p] = n
print(sum(tot.values()), len(tot))
for k, v in tot.most_common(10): print(v, k)
