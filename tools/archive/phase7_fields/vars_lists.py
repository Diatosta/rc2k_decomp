# Rewrites `const VARIABLES: [usize; N] = [...]` + its one `game_addr(rva(x))` map into typed addresses.
import re, os, sys, json, collections
ROOT = sys.argv[1]; S = sys.argv[2]
maps = __import__('maps').load(S)
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0]
    return None
VEC = re.compile(r'VARIABLES\s*\.iter\(\)\s*\.map\(\|&x\| game_addr\(rva\(x\)\) as u32\)\s*\.collect\(\)')
ARR = re.compile(r'VARIABLES\.map\(\|x\| game_addr\(rva\(x\)\) as u32\)')
done = collections.Counter(); skipped = []
for root, _, fs in os.walk(os.path.join(ROOT, 'src')):
    for f in fs:
        p = os.path.join(root, f); s = open(p).read()
        m = re.search(r'(?:(?:///[^\n]*\n)*)(pub )?const VARIABLES: \[usize; (\d+)\] = \[([^\]]*)\];\n', s)
        if not m: continue
        vals = [int(x, 16) for x in re.findall(r'0x([0-9A-Fa-f]+)', m.group(3))]
        if len(vals) != int(m.group(2)) or not any(typed(v) for v in vals): continue
        code = '\n'.join(l.split('//')[0] for l in s.split('\n'))
        uses = len(re.findall(r'\bVARIABLES\b', code)) - 1
        mv, ma = VEC.search(s), ARR.search(s)
        if uses != 1 or not (mv or ma):
            skipped.append((p, uses)); continue
        sts = set(); exprs = []
        for v in vals:
            t = typed(v)
            if t: sts.add((t[0], t[1])); exprs.append(f'{t[0]}.{t[2]}.addr() as u32')
            else: exprs.append(f'game_addr(rva({v:#X})) as u32'.replace('0X', '0x'))
        body = ('vec![' if mv else '[') + ', '.join(exprs) + ']'
        mm = mv or ma
        s2 = s[:mm.start()] + body + s[mm.end():]
        # keep the const's doc as a comment on nothing: drop the const and its doc
        s2 = s2.replace(m.group(0), '', 1)
        if m.group(1) and re.search(r'::VARIABLES', open(os.devnull).read()): pass
        for st, use in sorted(sts):
            if re.search(r'\b(const|static)\s+' + st + r'\b', s2): s2 = s2.replace(f'{st}.', use + '.')
            elif f'use {use};' not in s2:
                k = re.search(r'^use [^;]+;\n', s2, re.M).end(); s2 = s2[:k] + f'use {use};\n' + s2[k:]
        open(p, 'w').write(s2); done[p] = len(vals)
print(len(done), sum(done.values())); print('skipped', skipped)
