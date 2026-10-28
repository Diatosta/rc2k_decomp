# After typed_runs.py: `crate::math::x87_float::{ld, int, st, to_int}(` and the stores'
# `crate::..::NAME` paths shortened with a `use` when the file binds no other item of that
# name (rustc checks the rest). Run from the root on the files given.
import re, sys

for p in sys.argv[1:]:
    t = open(p).read()
    paths = set(re.findall(r'crate::math::x87_float::(ld|int|st|to_int)\(', t))
    full = {n: 'crate::math::x87_float::' + n for n in paths}
    for m in re.finditer(r'\b(crate::(?:[a-z_0-9]+::)+([A-Z][A-Z_0-9]+))\b(?!::)', t):
        full.setdefault(m.group(2), m.group(1))
    uses = []
    for name, path in sorted(full.items()):
        if t.count(path) < 1:
            continue
        # the name bound already: by this very path, or by something else
        bound_same = re.search(r'use ' + re.escape(path) + r';', t) or re.search(
            r'use ' + re.escape(path.rsplit('::', 1)[0]) + r'::\{[^}]*\b' + name + r'\b', t)
        other = re.search(r'\b(fn|static|const|let|mod|struct|enum|type) ' + name + r'\b', t) or \
            re.search(r'use [^;]*\b' + name + r'\b[^;]*;', t)
        if not bound_same and other:
            continue
        rx = r'\b' + re.escape(path) + r'\b' + (r'(?=\()' if name.islower() else '')
        # not inside `use` lines (an import of this very path stays as it is)
        t = ''.join(l if l.startswith('use ') else re.sub(rx, name, l)
                    for l in t.splitlines(keepends=True))
        if not bound_same:
            uses.append(f'use {path};')
    if uses:
        # after the last top-level `use` (or the header comments)
        ms = list(re.finditer(r'^use [^;]*;\n', t, re.M))
        at = ms[-1].end() if ms else re.search(r'^(?!//)', t, re.M).start()
        t = t[:at] + ''.join(u + '\n' for u in uses) + t[at:]
    open(p, 'w').write(t)
