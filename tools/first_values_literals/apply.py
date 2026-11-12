#!/usr/bin/env python3
"""apply.py <repo root> <store file under src/race> <struct>: replaces the store's `new()`
body (the `cells_of_bytes(first_values::<{ END - START }>(START))` call) with the literals
from gen.py, derives `ZeroCells` on the struct (and the records it holds), imports the trait."""
import re, subprocess, sys, os
root, rel, name = sys.argv[1:4]
path = os.path.join(root, rel)
src = open(path).read()
start = int(re.search(r'pub const START: usize = (0x[0-9A-Fa-f]+);', src).group(1), 16)
end = int(re.search(r'pub const END: usize = (0x[0-9A-Fa-f]+);', src).group(1), 16)
gen = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'gen.py')
out = subprocess.run([sys.executable, gen, os.path.join(root, 'src/race/store_data'), path, name, hex(start), hex(end)], capture_output=True, text=True)
if out.returncode:
    sys.exit(out.stderr)
lit = out.stdout.strip()
pat = re.compile(r'crate::cells::cells_of_bytes\(\s*crate::race::store::first_values::<\{ END - START \}>\(START\),?\s*\)')
assert pat.search(src), 'no first_values call'
src = pat.sub(lambda m: lit, src, count=1)
src = re.sub(r'#\[derive\(rc2k_derive::Cells\)\]', '#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]', src)
if 'use crate::cells::ZeroCells;' not in src:
    src = re.sub(r'\nuse ', '\nuse crate::cells::ZeroCells;\nuse ', src, count=1)
open(path, 'w').write(src)
