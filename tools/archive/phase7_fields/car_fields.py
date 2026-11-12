# Rewrites g/p/hold_* calls on g_car's IDA literals into CAR fields.
import re, os, sys, collections
ROOT = sys.argv[1]
DRY = '--dry' in sys.argv
cs = open(os.path.join(ROOT, 'src/physics/car_store.rs')).read()
fields = {}  # addr -> (expr, kind)
for m in re.finditer(r'/// \+0x([0-9a-fA-F]+) \(0x[0-9A-Fa-f]+\):[^\n]*\n(?:\s*///[^\n]*\n)*\s*pub (\w+): ([^,\n]+),', cs):
    off, name, ty = int(m.group(1), 16), m.group(2), m.group(3).strip()
    mm = re.match(r'\[(Dword|Float); (\d+)\]', ty)
    if mm:
        for i in range(int(mm.group(2))):
            fields[0x719FD8 + off + 4 * i] = (f'{name}[{i}]', mm.group(1))
    else:
        fields[0x719FD8 + off] = (name, ty)
RW_DEF = [
    r'fn g\(x: (usize|u32)\) -> u32 \{\s*unsafe \{ game_read\(rva\(x( as usize)?\)\) \}\s*\}',
    r'fn g\(x: impl crate::store::At\) -> u32 \{\s*unsafe \{ crate::store::at_read\(x\) \}\s*\}',
]
W_DEF = [r'fn p\(x: (usize|u32), v: u32\) \{\s*unsafe \{ game_write\(rva\(x( as usize)?\), v\) \}\s*\}',
         r'fn p\(x: impl crate::store::At, v: u32\) \{\s*unsafe \{ crate::store::at_write\(x, v\) \}\s*\}']
RW_MODS = ['physics::flow', 'physics::engine_torque', 'physics::splits', 'physics::drop_hit_parts', 'physics::fill_race_slot']

def imported(s, n, mods):
    for u in re.finditer(r'use crate::([\w:]+)::(\{[^}]*\}|\w+);', s):
        names = re.findall(r'\w+', u.group(2))
        if n in names and u.group(1) in mods:
            return True
    return False

def args_end(s, i):  # i at '(' ; returns index of matching ')'
    d = 0
    for j in range(i, len(s)):
        c = s[j]
        if c == '(': d += 1
        elif c == ')':
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
    out.append(cur)
    return out

LIT = re.compile(r'^\s*0x([0-9A-Fa-f]{6})\s*$')
def field_of(a):
    m = LIT.match(a)
    if not m: return None
    return fields.get(int(m.group(1), 16))

total = collections.Counter()
for root, _, fs in os.walk(os.path.join(ROOT, 'src')):
    for f in fs:
        p = os.path.join(root, f)
        if not f.endswith('.rs') or f == 'car_store.rs': continue
        s = open(p).read()
        if not re.search(r'0x71[9A][0-9A-Fa-f]{3}', s): continue
        g_ok = any(re.search(d, s) for d in RW_DEF) or imported(s, 'g', RW_MODS)
        p_ok = any(re.search(d, s) for d in W_DEF) or imported(s, 'p', RW_MODS)
        hold_ok = {n: imported(s, n, ['physics::flow']) for n in ('hold_high', 'hold_low', 'a')}
        clash = re.search(r'\b(const|static|use [^;]*)\s*\bCAR\b', s) is not None
        car = 'crate::physics::car_store::CAR' if clash else 'CAR'
        out, i, n = [], 0, 0
        for m in re.finditer(r'(?<![\w.:])(g|p|hold_high|hold_low|a)\(', s):
            if m.start() < i: continue
            name = m.group(1)
            o = m.end() - 1
            e = args_end(s, o)
            if e < 0: continue
            args = split_args(s[o + 1:e])
            rep = None
            if name == 'g' and g_ok and len(args) == 1:
                fk = field_of(args[0])
                if fk: rep = f'{car}.{fk[0]}.' + ('get()' if fk[1] == 'Dword' else 'bits()')
            elif name == 'p' and p_ok and len(args) == 2:
                fk = field_of(args[0])
                if fk: rep = f'{car}.{fk[0]}.' + ('set(' if fk[1] == 'Dword' else 'set_bits(') + args[1].strip() + ')'
            elif name == 'a' and hold_ok['a'] and len(args) == 1:
                fk = field_of(args[0])
                if fk: rep = f'{car}.{fk[0]}.addr() as u32'
            elif name in ('hold_high', 'hold_low') and hold_ok[name] and len(args) == 2:
                new = []
                for a in args:
                    fk = field_of(a)
                    new.append(f'&{car}.{fk[0]}' if fk else a.strip())
                if new != [a.strip() for a in args]: rep = f'{name}(' + ', '.join(new) + ')'
            if rep:
                out.append(s[i:m.start()]); out.append(rep); i = e + 1; n += 1
        if n == 0: continue
        out.append(s[i:]); s2 = ''.join(out)
        if not clash and 'use crate::physics::car_store::CAR;' not in s2:
            # after the first block of top-level use lines
            ms = list(re.finditer(r'^use [^;]+;\n', s2, re.M))
            if ms:
                k = ms[0].end()
                s2 = s2[:k] + 'use crate::physics::car_store::CAR;\n' + s2[k:]
            else:
                hdr = re.match(r'((?://![^\n]*\n)*)\n?', s2).end()
                s2 = s2[:hdr] + '\nuse crate::physics::car_store::CAR;\n' + s2[hdr:]
        total[p] = n
        if not DRY: open(p, 'w').write(s2)
print(sum(total.values()), len(total))
for k, v in total.most_common(): print(v, k)
