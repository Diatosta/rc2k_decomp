# The physics cores handed a list of game addresses (`fn game() -> Vec<u32>`, read through
# raw pointers or `read_or_end`) take a list of cells (`&dyn Cell32`) instead: the list's
# `X.addr()` entries become `&X`, the reads `.bits()`, the writes `.set_bits(..)`.
# Usage: python3 tools/phase13_runs/cell_lists.py
import os, re

def balanced(t, i):
    """The index after the `)` closing the `(` at `i`."""
    d = 0
    while True:
        d += t[i] == '('
        d -= t[i] == ')'
        i += 1
        if d == 0:
            return i

files = [os.path.join(d, f) for d, _, fs in os.walk('src') for f in fs
         if f.endswith('.rs') and 'fn game() -> Vec<u32>' in open(os.path.join(d, f)).read()]
for p in files:
    t = open(p).read()
    # the list
    s = t.index('fn game() -> Vec<u32>')
    e = t.index('\n}\n', s)
    body = t[s:e]
    body = body.replace('fn game() -> Vec<u32>', "fn game() -> Vec<&'static dyn Cell32>")
    body = re.sub(r'\(?((?:[A-Za-z_][\w:]*)(?:\.\w+|\[\d+\])*)\.addr\(\)\)?,', r'&\1,', body)
    body = body.replace('.map(|c| c.addr())', ".map(|c| c as &dyn Cell32)")
    t = t[:s] + body + t[e:]
    t = t.replace('v: &[u32]', 'v: &[&dyn Cell32]')
    # raw reads / writes
    t = re.sub(r'\(crate::pointers::host\((v\[[^\]]+\])\) as \*const u32\)\s*\.read_unaligned\(\)', r'\1.bits()', t)
    for m in list(re.finditer(r'\(crate::pointers::host\((v\[[^\]]+\])\) as \*mut u32\)\s*\.write_unaligned\(', t))[::-1]:
        end = balanced(t, m.end() - 1)
        t = t[:m.start()] + f'{m.group(1)}.set_bits(' + t[m.end():end] + t[end:]
    t = re.sub(r'crate::pointers::read_or_end::<u32>\(\s*([\w\[\]+ ]+?),\s*"[^"]*",?\s*\)', r'\1.bits()', t)
    for m in list(re.finditer(r'crate::pointers::write_or_end::<u32>\(\s*([\w\[\]+ ]+?),', t))[::-1]:
        end = balanced(t, m.start() + len('crate::pointers::write_or_end::<u32>'))
        inner = t[m.end():end - 1]
        val = re.sub(r',\s*"[^"]*",?\s*$', '', inner.strip())
        t = t[:m.start()] + f'{m.group(1)}.set_bits({val})' + t[end:]
    # helpers on one cell
    t = re.sub(r'fn (at|get|put|put_changed)\(a: u32', r'fn \1(a: &dyn Cell32', t)
    if 'use crate::session::Cell32;' not in t:
        t = t.replace('\nuse ', '\nuse crate::session::Cell32;\nuse ', 1)
    open(p, 'w').write(t)
    print(p)
