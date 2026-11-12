# Drops the items rustc calls never used: reads clippy's short messages on stdin
# (FILE:LINE:COL: warning: function `x` is never used / constant `X` is never used) and
# removes each item with the doc and plain comments right above it. Run from the root.
import re, sys, collections
hits = collections.defaultdict(list)
for l in sys.stdin:
    m = re.match(r'(src/[^:]+):(\d+):\d+: warning: (function|constant|static) `\w+` is never used', l)
    if m: hits[m.group(1)].append(int(m.group(2)))
for p, lines in hits.items():
    L = open(p).read().split('\n')
    for ln in sorted(set(lines), reverse=True):
        a = ln - 1
        if L[a].split('//')[0].rstrip().endswith(';'):
            b = a
        else:
            d = 0; b = a; seen = False
            while True:
                code = L[b].split('//')[0]
                d += sum(code.count(c) for c in '{[(') - sum(code.count(c) for c in '}])')
                seen = seen or any(c in code for c in '{[(')
                if seen and d == 0: break
                b += 1
        while a > 0 and L[a - 1].lstrip().startswith('//'): a -= 1
        if a > 0 and b + 1 < len(L) and L[a - 1].strip() == '' and L[b + 1].strip() == '': b += 1
        print(p, ln, L[ln - 1].strip()[:60])
        del L[a:b + 1]
    open(p, 'w').write('\n'.join(L))
