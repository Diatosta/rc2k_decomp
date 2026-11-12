# Reads clippy's short output on stdin; for each `unused variable: `rc`` drops the
# parameter `rc: usize` with each call's argument in that position (the runs are private
# `fn`s, called in their own file), or the unused `let rc = ..;` line. Run from the root.
import re, sys

hits = {}
for l in sys.stdin:
    m = re.match(r'(src/[^:]+):(\d+):(\d+): warning: unused variable: `rc`', l)
    if m:
        hits.setdefault(m.group(1), set()).add((int(m.group(2)), int(m.group(3))))


def args_span(t, i):
    """From the `(` at i: the top-level argument spans and the closing index."""
    depth, spans, start = 0, [], i + 1
    j = i
    while True:
        c = t[j]
        if c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
            if depth == 0:
                spans.append((start, j))
                return spans, j
        elif c == ',' and depth == 1:
            spans.append((start, j))
            start = j + 1
        j += 1


for p, spots in hits.items():
    t = open(p).read()
    lines = t.split('\n')
    drops = []  # (fn name, index)
    for ln, col in sorted(spots, reverse=True):
        line = lines[ln - 1]
        if re.match(r'\s*let rc\b', line):
            lines[ln - 1] = None
            continue
        # the fn whose signature holds this parameter
        k = ln - 1
        while k >= 0 and not re.search(r'\bfn (\w+)\(', lines[k]):
            k -= 1
        name = re.search(r'\bfn (\w+)\(', lines[k]).group(1)
        if re.search(r'\bpub(\([^)]*\))? fn ' + name, lines[k]):
            print('PUBLIC', p, name)
            continue
        drops.append(name)
    t = '\n'.join(l for l in lines if l is not None)
    for name in drops:
        # the index of `rc` in its signature
        m = re.search(r'\bfn ' + name + r'\(', t)
        spans, end = args_span(t, m.end() - 1)
        params = [t[a:b].strip() for a, b in spans]
        idx = next(n for n, s in enumerate(params) if re.match(r'rc\s*:', s))
        keep = [s for n, s in enumerate(params) if n != idx and s]
        t = t[:m.end()] + ', '.join(keep) + t[end:]
        # each call
        out, i = [], 0
        for c in re.finditer(r'(?<!fn )\b' + name + r'\(', t):
            if c.start() < i:
                continue
            spans, end = args_span(t, c.end() - 1)
            a = [t[x:y].strip() for x, y in spans]
            a = [s for n, s in enumerate(a) if n != idx and s]
            out.append(t[i:c.end()] + ', '.join(a))
            i = end
        out.append(t[i:])
        t = ''.join(out)
    open(p, 'w').write(t)
    print(p, len(drops))
