# Drops the `unsafe { .. }` blocks clippy calls unnecessary: reads clippy's short messages
# on stdin (FILE:LINE:COL: warning: unnecessary `unsafe` block), and at each place turns
# `unsafe { X }` into `X` (one expression) or `unsafe {` .. `}` into `{` .. `}` (a block
# of statements, left for a hand tidy). Run from the repository root.
import re, sys, collections
hits = collections.defaultdict(list)
for l in sys.stdin:
    m = re.match(r'(src/[^:]+):(\d+):(\d+): warning: unnecessary `unsafe` block', l)
    if m: hits[m.group(1)].append((int(m.group(2)), int(m.group(3))))
for p, places in hits.items():
    lines = open(p).read().split('\n')
    starts = [0]
    for l in lines: starts.append(starts[-1] + len(l) + 1)
    s = '\n'.join(lines)
    for ln, col in sorted(places, reverse=True):
        i = starts[ln - 1] + col - 1
        assert s.startswith('unsafe', i), (p, ln, s[i:i + 20])
        j = s.index('{', i); d = 0
        for k in range(j, len(s)):
            if s[k] == '{': d += 1
            elif s[k] == '}':
                d -= 1
                if d == 0: break
        inner = s[j + 1:k]
        if ';' not in inner.strip().rstrip(';') and '\n' not in inner.strip():
            s = s[:i] + inner.strip() + s[k + 1:]
        else:
            s = s[:i] + s[j:]
        print(p, ln)
    open(p, 'w').write(s)
