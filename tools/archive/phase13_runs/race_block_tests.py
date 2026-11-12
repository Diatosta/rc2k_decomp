# In each `asm_reference` test module: the operand a reference run takes as `rc` (the race
# block, now read by the plain runs through the typed stores) -> `Operand::RaceBlock`;
# `plain: |a|` closures that no longer use `a` -> `|_|`. Run from the root.
import os, re


def items(t, i):
    """The top-level comma-separated items of the bracket opened at t[i]: spans."""
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
                return out
        elif c == ',' and depth == 1:
            out.append((start, j))
            start = j + 1
        j += 1


for d, _, fs in os.walk('src'):
    for f in fs:
        p = os.path.join(d, f)
        if not f.endswith('.rs'):
            continue
        t = open(p).read()
        if 'mod asm_reference' not in t:
            continue
        params = {m.group(1): [s.strip() for s in m.group(2).split(',')]
                  for m in re.finditer(r'reference!\((\w+)\(([^)]*)\)', t)}
        edits = []  # (start, end, text)
        for b in re.finditer(r'Block \{', t):
            end = t.index('\n            },', b.end()) if '\n            },' in t[b.end():] else len(t)
            body = t[b.end():end]
            r = re.search(r'reference: \|a\| unsafe \{ (\w+)\(', body)
            if not r or r.group(1) not in params:
                continue
            ps = params[r.group(1)]
            # the reference's arguments: a[i] in its parameter order
            call = b.end() + r.end() - 1
            args = [t[x:y].strip() for x, y in items(t, call)]
            want = {int(m.group(1)) for a, n in zip(args, ps) if n == 'rc'
                    for m in [re.fullmatch(r'a\[(\d+)\]', a)] if m}
            o = re.search(r'operands: vec!\[', body)
            if o and want:
                ops = items(t, b.end() + o.end() - 1)
                for k in want:
                    x, y = ops[k]
                    if 'Operand::Buffer' in t[x:y]:
                        q = t.index('Operand::Buffer', x)
                        edits.append((q, q + len('Operand::Buffer'), 'Operand::RaceBlock'))
            pl = re.search(r'plain: \|a\| ', body)
            if pl:
                s = b.end() + pl.start()
                line_end = t.index('\n', s)
                rest = t[s:line_end]
                if 'a[' not in rest and not rest.rstrip().endswith('{'):
                    edits.append((s, s + len('plain: |a|'), 'plain: |_|'))
        for x, y, s in sorted(edits, reverse=True):
            t = t[:x] + s + t[y:]
        if edits:
            open(p, 'w').write(t)
            print(p, len(edits))
