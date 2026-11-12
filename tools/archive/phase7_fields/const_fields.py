# Named constants of typed places -> field accesses.
import re, os, sys, json, collections
ROOT, S = sys.argv[1], sys.argv[2]
maps = __import__('maps').load(S)
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0], m[hex(v)][1]
RW = [r'fn g\(x: (usize|u32)\) -> u32 \{\s*unsafe \{ game_read\(rva\(x( as usize)?\)\) \}\s*\}', r'fn g\(x: impl crate::store::At\) -> u32 \{\s*unsafe \{ crate::store::at_read\(x\) \}\s*\}']
W = [r'fn p\(x: (usize|u32), v: u32\) \{\s*unsafe \{ game_write\(rva\(x( as usize)?\), v\) \}\s*\}', r'fn p\(x: impl crate::store::At, v: u32\) \{\s*unsafe \{ crate::store::at_write\(x, v\) \}\s*\}']
MODS = ['physics::flow', 'physics::engine_torque', 'physics::splits', 'physics::drop_hit_parts', 'physics::fill_race_slot']
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
tot = collections.Counter(); removed = 0
for root, _, fs in os.walk(os.path.join(ROOT, 'src')):
    for f in fs:
        p = os.path.join(root, f)
        if not f.endswith('.rs') or f in ('car_store.rs', 'tables_store.rs', 'work_store.rs'): continue
        s = open(p).read()
        consts = {}
        for m in re.finditer(r'^(?:\s*///[^\n]*\n)*\s*const (\w+): usize = (rva\()?0x([0-9A-Fa-f]{6})\)?;[^\n]*\n', s, re.M):
            t = typed(int(m.group(3), 16))
            if t: consts[m.group(1)] = (bool(m.group(2)), t, m.group(0))
        if not consts: continue
        g_ok = any(re.search(d, s) for d in RW) or imported(s, 'g', MODS)
        p_ok = any(re.search(d, s) for d in W) or imported(s, 'p', MODS)
        fl = {n: imported(s, n, ['physics::flow']) for n in ('hold_high', 'hold_low', 'a', 'negate', 'ease_zero')}
        sts = set(); n = 0
        def ref(t, clash):
            st, use, fld, ty = t
            return (use if clash else st) + '.' + fld
        out, i = [], 0
        pat = r'(?<![\w.:])(g|p|a|hold_high|hold_low|negate|ease_zero|game_read|game_read::<u32>|game_write)\('
        for m in re.finditer(pat, s):
            if m.start() < i: continue
            fn = m.group(1); o = m.end() - 1; e = args_end(s, o)
            if e < 0: continue
            args = [x.strip() for x in split_args(s[o + 1:e])]
            c0 = consts.get(args[0]) if args else None
            rep = None
            if c0:
                is_rva, t, _ = c0
                clash = re.search(r'\b(const|static)\s+' + t[0] + r'\b', s) is not None
                r = ref(t, clash); getter = 'get()' if t[3] == 'Dword' else 'bits()'; setter = 'set' if t[3] == 'Dword' else 'set_bits'
                if not is_rva:
                    if fn == 'g' and g_ok and len(args) == 1: rep = f'{r}.{getter}'
                    elif fn == 'p' and p_ok and len(args) == 2: rep = f'{r}.{setter}({args[1]})'
                    elif fn == 'a' and fl['a'] and len(args) == 1: rep = f'({r}.addr() as u32)'
                    elif fn in ('hold_high', 'hold_low', 'negate', 'ease_zero') and fl[fn]:
                        rep = f'{fn}(' + ', '.join([f'&{ref(consts[x][1], clash)}' if x in consts and not consts[x][0] else x for x in args]) + ')'
                else:
                    if fn == 'game_read::<u32>' and len(args) == 1: rep = f'{r}.{getter}'
                    elif fn == 'game_write' and len(args) == 2 and re.search(r'game_write\b', s): rep = f'{r}.{setter}({args[1]})'
                if rep: sts.add((t[0], t[1], clash))
            if rep: out.append(s[i:m.start()]); out.append(rep); i = e + 1; n += 1
        if n == 0: continue
        out.append(s[i:]); s2 = ''.join(out)
        for name, (_, _, line) in consts.items():
            if not re.search(r'\b' + name + r'\b', s2.replace(line, '', 1)):
                s2 = s2.replace(line, '', 1); removed += 1
        for st, use, clash in sts:
            if not clash and f'use {use};' not in s2:
                k = re.search(r'^use [^;]+;\n', s2, re.M).end(); s2 = s2[:k] + f'use {use};\n' + s2[k:]
        open(p, 'w').write(s2); tot[p] = n
print(sum(tot.values()), len(tot), 'consts removed', removed)
