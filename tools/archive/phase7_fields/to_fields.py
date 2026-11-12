# Rewrites g/p/a/hold_*/negate/ease_zero calls on a typed piece's IDA literals into its fields.
# usage: to_fields.py ROOT MAP.json STATIC USEPATH [--dry]
import re, os, sys, json, collections
ROOT, MAP, ST, USE = sys.argv[1:5]
DRY = '--dry' in sys.argv
fields = {int(k, 16): tuple(v) for k, v in json.load(open(MAP)).items()}
RW_DEF = [r'fn g\(x: (usize|u32)\) -> u32 \{\s*unsafe \{ game_read\(rva\(x( as usize)?\)\) \}\s*\}',
          r'fn g\(x: impl crate::store::At\) -> u32 \{\s*unsafe \{ crate::store::at_read\(x\) \}\s*\}']
W_DEF = [r'fn p\(x: (usize|u32), v: u32\) \{\s*unsafe \{ game_write\(rva\(x( as usize)?\), v\) \}\s*\}',
         r'fn p\(x: impl crate::store::At, v: u32\) \{\s*unsafe \{ crate::store::at_write\(x, v\) \}\s*\}']
RW_MODS = ['physics::flow', 'physics::engine_torque', 'physics::splits', 'physics::drop_hit_parts', 'physics::fill_race_slot']
def imported(s, n, mods):
    for u in re.finditer(r'use crate::([\w:]+)::(\{[^}]*\}|\w+);', s):
        if n in re.findall(r'\w+', u.group(2)) and u.group(1) in mods: return True
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
LIT = re.compile(r'^\s*0x([0-9A-Fa-f]{6})\s*$')
def field_of(a):
    m = LIT.match(a)
    return fields.get(int(m.group(1), 16)) if m else None
total = collections.Counter()
lo, hi = min(fields), max(fields)
for root, _, fs in os.walk(os.path.join(ROOT, 'src')):
    for f in fs:
        p = os.path.join(root, f)
        if not f.endswith('.rs') or p.endswith(USE.split('::')[-2] + '.rs'): continue
        s = open(p).read()
        if not any(lo <= int(x, 16) <= hi for x in re.findall(r'0x([0-9A-Fa-f]{6})\b', s)): continue
        glob = re.search(r'use crate::physics::gearbox::\*;', s) is not None
        g_ok = any(re.search(d, s) for d in RW_DEF) or imported(s, 'g', RW_MODS + ['physics::gearbox']) or glob
        p_ok = any(re.search(d, s) for d in W_DEF) or imported(s, 'p', RW_MODS + ['physics::gearbox']) or glob
        fl = {n: imported(s, n, ['physics::flow']) for n in ('hold_high', 'hold_low', 'a', 'negate', 'ease_zero')}
        clash = re.search(r'\b(const|static)\s+' + ST + r'\b|use [^;]*\b' + ST + r'\b', s) is not None
        name = USE if clash else ST
        out, i, n = [], 0, 0
        for m in re.finditer(r'(?<![\w.:])(g|p|hold_high|hold_low|a|negate|ease_zero)\(', s):
            if m.start() < i: continue
            fn = m.group(1); o = m.end() - 1; e = args_end(s, o)
            if e < 0: continue
            args = split_args(s[o + 1:e]); rep = None
            if fn == 'g' and g_ok and len(args) == 1:
                fk = field_of(args[0])
                if fk: rep = f'{name}.{fk[0]}.' + ('get()' if fk[1] == 'Dword' else 'bits()')
            elif fn == 'p' and p_ok and len(args) == 2:
                fk = field_of(args[0])
                if fk: rep = f'{name}.{fk[0]}.' + ('set(' if fk[1] == 'Dword' else 'set_bits(') + args[1].strip() + ')'
            elif fn == 'a' and fl['a'] and len(args) == 1:
                fk = field_of(args[0])
                if fk: rep = f'({name}.{fk[0]}.addr() as u32)'
            elif fn in ('hold_high', 'hold_low', 'negate', 'ease_zero') and fl[fn]:
                new = [(f'&{name}.{field_of(x)[0]}' if field_of(x) else x.strip()) for x in args]
                if new != [x.strip() for x in args]: rep = f'{fn}(' + ', '.join(new) + ')'
            if rep: out.append(s[i:m.start()]); out.append(rep); i = e + 1; n += 1
        if n == 0: continue
        out.append(s[i:]); s2 = ''.join(out)
        imp = f'use {USE};\n'
        if not clash and imp not in s2:
            ms = list(re.finditer(r'^use [^;]+;\n', s2, re.M))
            if ms: k = ms[0].end(); s2 = s2[:k] + imp + s2[k:]
            else:
                hdr = re.match(r'((?://![^\n]*\n)*)\n?', s2).end()
                s2 = s2[:hdr] + '\n' + imp + s2[hdr:]
        total[p] = n
        if not DRY: open(p, 'w').write(s2)
print(sum(total.values()), len(total))
for k, v in total.most_common(12): print(v, k)
