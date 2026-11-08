# A run's offset parameter (`o3: usize`, used only as `fld(rc, o3)` / `fild` / `fstp` /
# `fistp`) -> `o3: &impl Cell32`, each call's constant RVA -> the store field it names
# (typed_runs.py's walk). A parameter any call passes a non-constant is left.
# Usage: python3 tools/phase13_runs/offset_params.py FILE..   (from the repository root)
import re, sys

src = open('tools/phase13_runs/typed_runs.py').read()
g = {'__name__': 'walker'}
sys_argv, sys.argv = sys.argv, ['x']
exec(compile(src[:src.index('READS = {')], 'typed_runs', 'exec'), g)
sys.argv = sys_argv
field, CELLS = g['field'], g['CELLS']


def items(t, i):
    depth, out, start, j = 0, [], i + 1, i
    while True:
        c = t[j]
        if c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
            if depth == 0:
                if t[start:j].strip():
                    out.append((start, j))
                return out, j
        elif c == ',' and depth == 1:
            out.append((start, j))
            start = j + 1
        j += 1


def body_span(t, sig_end):
    i = t.index('{', sig_end)
    depth = 0
    for j in range(i, len(t)):
        depth += t[j] == '{'
        depth -= t[j] == '}'
        if depth == 0:
            return i, j + 1


for p in sys.argv[1:]:
    t = open(p).read()
    changed = 0
    for fm in list(re.finditer(r'\bfn (\w+)\(', t)):
        name = fm.group(1)
        m = re.search(r'\bfn ' + name + r'\(', t)
        ps, end = items(t, m.end() - 1)
        params = [t[a:b].strip() for a, b in ps]
        b0, b1 = body_span(t, end)
        body = t[b0:b1]
        for idx, prm in enumerate(params):
            pm = re.fullmatch(r'(\w+): usize', prm)
            if not pm or pm.group(1) in ('rc', 'c', 'k'):
                continue
            v = pm.group(1)
            uses = len(re.findall(r'\b' + v + r'\b', body))
            ops = len(re.findall(r'\b(?:fld|fild|fstp|fistp)\(rc, ' + v + r'[,)]', body))
            if uses == 0 or uses != ops:
                continue
            # every call's argument at idx: a constant naming a cell
            calls, ok = [], True
            for c in re.finditer(r'(?<!fn )\b' + name + r'\(', t):
                a, _ = items(t, c.end() - 1)
                if idx >= len(a):
                    ok = False
                    break
                arg = t[a[idx][0]:a[idx][1]].strip()
                f = field(int(arg, 0) + 0x400000) if re.fullmatch(r'0x[0-9A-Fa-f]+', arg) else None
                if not f or f[1] not in CELLS:
                    ok = False
                    break
                calls.append((a[idx], f[0]))
            if not ok or not calls:
                print('LEFT', p, name, v)
                continue
            nb = re.sub(r'\bfld\(rc, ' + v + r'\)', f'crate::math::x87_float::ld({v}.bits())', body)
            nb = re.sub(r'\bfild\(rc, ' + v + r'\)', f'crate::math::x87_float::int({v}.bits() as i32)', nb)
            for op, fmt in (('fstp', '{v}.set_bits(crate::math::x87_float::st({x}))'),
                            ('fistp', '{v}.set_bits(crate::math::x87_float::to_int({x}) as u32)')):
                while True:
                    w = re.search(r'\b' + op + r'\(rc, ' + v + r',', nb)
                    if not w:
                        break
                    a, e = items(nb, w.end() - len(op + '(rc, ' + v + ',') + len(op))
                    x = nb[a[2][0]:a[2][1]].strip()
                    nb = nb[:w.start()] + fmt.format(v=v, x=x) + nb[e + 1:]
            # write back from the end: calls after the body, the body, the signature, calls before
            edits = [((s, e), ' &' + path) for (s, e), path in calls]
            edits.append(((b0, b1), nb))
            edits.append(((ps[idx][0], ps[idx][1]), (' ' if idx else '') + f'{v}: &impl crate::session::Cell32'))
            for (s, e), txt in sorted(edits, key=lambda x: x[0][0], reverse=True):
                t = t[:s] + txt + t[e:]
            changed += 1
            break  # positions moved: the fn again from the top
    # a fn may have more: rerun until nothing changes
    open(p, 'w').write(t)
    print(p, changed)
