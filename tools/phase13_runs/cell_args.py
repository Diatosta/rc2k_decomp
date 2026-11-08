# Helpers that took addresses now take cells (`math::cells::Cells`): each call's address
# arguments -> `&FIELD` / `&FIELD[k..]` when FIELD (a typed store's array, its length read
# from the `#[repr(C)]` stores as typed_runs.py does) holds the cells the helper reaches,
# else `At(addr as u32)` (and printed). Usage: python3 tools/phase13_runs/cell_args.py [--write]
import os, re, sys

src = open('tools/phase13_runs/typed_runs.py').read()
argv, sys.argv = sys.argv, ['x']
g = {}
exec(compile(src[:src.index('READS = {')], 'typed_runs', 'exec'), g)
sys.argv = argv
NEED = {  # cells each argument is reached at (0 .. need)
    'multiply_matrices': [15, 15, 15],
    'copy_matrix_with_place': [16, 15, 16],
    'invert_turn': [11, 16],
    'square_matrix': [16],
    'unit_vector': [3, 3],
    'cross_product': [3, 3, 3],
    'rotate_point': [15, None],
    'vector_length': [3],
    'dot_product': [3, 3],
    'scale_vector': [3, None],
}
statics = {name: (tgt, p) for s, e, name, tgt, mod, p in g['stores']}


def length(path):
    """The cells from `PATH[k]` on: (static.field.field)[k] -> len - k, or None."""
    m = re.fullmatch(r'([A-Z][A-Z_0-9]*)((?:\.\w+|\[\d+\])*)\[(\d+)\]', path)
    if not m or m.group(1) not in statics:
        return None
    ty, p = statics[m.group(1)]
    g['CUR'][0] = p
    for part in re.findall(r'\.(\w+)|\[(\d+)\]', m.group(2)):
        if part[0]:
            lay = g['layout'](ty.split('::')[-1])
            if not lay:
                return None
            hit = [ft for fn, ft, at in lay[0] if fn == part[0]]
            if not hit:
                return None
            ty = hit[0]
        else:
            a = re.fullmatch(r'\[(.+); (.+)\]', ty.strip())
            if not a:
                return None
            ty = a.group(1)
    a = re.fullmatch(r'\[(.+); (.+)\]', ty.strip())
    if not a or a.group(1).split('::')[-1] not in ('Float', 'Dword'):
        return None
    n = a.group(2).strip()
    n = int(n, 0) if re.fullmatch(r'0x[0-9A-Fa-f]+|\d+', n) else g['consts'].get(n)
    return None if n is None else n - int(m.group(3))


def items(t, i):
    d, out, s, j = 0, [], i + 1, i
    while True:
        c = t[j]
        if c in '([{':
            d += 1
        elif c in ')]}':
            d -= 1
            if d == 0:
                if t[s:j].strip():
                    out.append((s, j))
                return out
        elif c == ',' and d == 1:
            out.append((s, j))
            s = j + 1
        j += 1


write = '--write' in argv
for d, _, fs in os.walk('src'):
    for f in fs:
        p = os.path.join(d, f)
        if not p.endswith('.rs'):
            continue
        t = open(p).read()
        edits = []
        for name, need in NEED.items():
            for m in re.finditer(r'(?<!fn )(?<![\w.])' + name + r'\(', t):
                args = items(t, m.end() - 1)
                if len(args) != len(need):
                    continue
                for (s, e), n in zip(args, need):
                    if n is None:
                        continue
                    a = re.sub(r'\s+', ' ', t[s:e].strip())
                    a = re.sub(r'^\((.*)\) as usize$', r'\1', a)
                    a = re.sub(r' as usize$', '', a)
                    a = re.sub(r'^\((.*)\)$', r'\1', a)
                    fm = re.fullmatch(r'((?:crate::(?:\w+::)+)?([A-Z][A-Z_0-9]*(?:\.\w+|\[\d+\])*)\[(\d+)\])\.addr\(\)', a)
                    if fm:
                        full = fm.group(1)
                        k = int(fm.group(3))
                        base = full[:full.rindex('[')]
                        have = length(fm.group(2) + f'[{k}]')
                        if have is not None and have >= n:
                            new = f'&{base}' if k == 0 else f'&{base}[{k}..]'
                        else:
                            new = f'At({full}.addr())'
                            print('SHORT', p, name, full, have, n)
                    else:
                        lm = re.fullmatch(r'([a-z_]\w*(?:\.\w+)*)\[(\d+)\]\.addr\(\)', a)
                        if lm:
                            k = int(lm.group(2))
                            new = f'&{lm.group(1)}' if k == 0 else f'&{lm.group(1)}[{k}..]'
                            print('LOCAL', p, name, a, '->', new, 'need', n)
                        else:
                            new = f'At(({a}) as u32)' if ' ' in a else f'At({a} as u32)'
                            print('AT', p, name, a)
                    edits.append((s, e, ' ' + new if t[s] == ' ' else new))
        if edits and write:
            for s, e, new in sorted(edits, reverse=True):
                t = t[:s] + new + t[e:]
            if 'At(' in ''.join(x[2] for x in edits) and 'math::cells::At' not in t:
                t = t.replace('\nuse ', '\nuse crate::math::cells::At;\nuse ', 1)
            open(p, 'w').write(t)
