# Local address / value closures and fns on typed places -> field accesses.
import re, os, sys, json, collections
ROOT, S = sys.argv[1], sys.argv[2]
maps = __import__('maps').load(S)
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0], m[hex(v)][1]
FW = r'(?:crate::(?:framework|store)::)?'
ADDR = [r'let (\w+) = \|(\w+): usize\| ' + FW + r'game_addr\(' + FW + r'rva\(\2\)\) as u32;',
        r'fn (\w+)\((\w+): usize\) -> u32 \{\s*' + FW + r'game_addr\(' + FW + r'rva\(\2\)\) as u32\s*\}',
        r'let (\w+) = \|(\w+): u32\| ' + FW + r'game_addr\(' + FW + r'rva\(\2 as usize\)\) as u32;',
        r'fn (\w+)\((\w+): u32\) -> u32 \{\s*' + FW + r'game_addr\(' + FW + r'rva\(\2 as usize\)\) as u32\s*\}']
ADDRZ = [r'let (\w+) = \|(\w+): usize\| ' + FW + r'game_addr\(' + FW + r'rva\(\2\)\);',
         r'fn (\w+)\((\w+): (?:usize|u32)\) -> usize \{\s*' + FW + r'game_addr\(' + FW + r'rva\(\2( as usize)?\)\)\s*\}']
VAL = [r'let (\w+) = \|(\w+): usize\| (?:unsafe \{ )?game_read::<u32>\(rva\(\2\)\)(?: \})?;',
       r'fn (\w+)\((\w+): usize\) -> u32 \{\s*unsafe \{ game_read\(rva\(\2\)\) \}\s*\}',
       r'fn (\w+)\((\w+): u32\) -> u32 \{\s*unsafe \{ game_read\(rva\(\2 as usize\)\) \}\s*\}']
WR = [r'let (\w+) = \|(\w+): u32, (\w+): u32\| game_write\(rva\(\2 as usize\), \3\);',
      r'fn (\w+)\((\w+): (?:usize|u32), (\w+): u32\) \{\s*unsafe \{ game_write\(rva\(\2( as usize)?\), \3\) \}\s*\}']
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
tot = collections.Counter()
for root, _, fs in os.walk(os.path.join(ROOT, 'src')):
    for f in fs:
        p = os.path.join(root, f)
        if not f.endswith('.rs') or f in ('car_store.rs', 'tables_store.rs', 'work_store.rs'): continue
        s = open(p).read()
        kinds = {}
        for pats, kd in ((ADDR, 'addr'), (ADDRZ, 'addrz'), (VAL, 'val'), (WR, 'wr')):
            for pt in pats:
                for m in re.finditer(pt, s): kinds.setdefault(m.group(1), kd)
        kinds = {k: v for k, v in kinds.items() if k not in ('g', 'p') or v in ('addr', 'addrz') or True}
        if not kinds: continue
        names = '|'.join(map(re.escape, kinds))
        out, i, n, sts = [], 0, 0, set()
        for m in re.finditer(r'(?<![\w.:])(' + names + r')\(', s):
            if m.start() < i: continue
            fn = m.group(1); o = m.end() - 1; e = args_end(s, o)
            if e < 0: continue
            args = [x.strip() for x in split_args(s[o + 1:e])]
            lm = LIT.match(args[0]) if args else None
            t = typed(int(lm.group(1), 16)) if lm else None
            if not t: continue
            clash = re.search(r'\b(const|static)\s+' + t[0] + r'\b', s) is not None
            r = (t[1] if clash else t[0]) + '.' + t[2]
            kd = kinds[fn]; rep = None
            if kd == 'addr' and len(args) == 1: rep = f'({r}.addr() as u32)'
            elif kd == 'addrz' and len(args) == 1: rep = f'{r}.addr()'
            elif kd == 'val' and len(args) == 1: rep = f'{r}.' + ('get()' if t[3] == 'Dword' else 'bits()')
            elif kd == 'wr' and len(args) == 2: rep = f'{r}.' + ('set(' if t[3] == 'Dword' else 'set_bits(') + args[1] + ')'
            if rep:
                sts.add((t[0], t[1], clash)); out.append(s[i:m.start()]); out.append(rep); i = e + 1; n += 1
        if not n: continue
        out.append(s[i:]); s2 = ''.join(out)
        for st, use, clash in sts:
            if not clash and f'use {use};' not in s2:
                k = re.search(r'^use [^;]+;\n', s2, re.M).end(); s2 = s2[:k] + f'use {use};\n' + s2[k:]
        open(p, 'w').write(s2); tot[p] = n
print(sum(tot.values()), len(tot))
for k, v in tot.most_common(10): print(v, k)
