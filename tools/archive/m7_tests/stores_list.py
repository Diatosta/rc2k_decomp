#!/usr/bin/env python3
"""Writes `stores.rs`'s list of the stores (`LISTED`) from `moved::MOVED` (run once, while
`moved.rs` exists): each entry's registry name (`NAMES`, same order), its home's store
expression, and the game range's length where it is shorter than the store.

Usage: stores_list.py <root> <lengths>, where <lengths> holds one `<name> <len> <size>` line
(hex) per entry of `MOVED`, in its order (written by a one-off test). The window offsets
given as IDA differences are evaluated to plain offsets. The list replaces `NAMES` in
`src/stores.rs` in place; rustfmt formats it after.
"""
import re
import sys

root, lengths = sys.argv[1], sys.argv[2]
moved = open(f'{root}/src/moved.rs').read()
stores_path = f'{root}/src/stores.rs'
stores = open(stores_path).read()

net = open(f'{root}/src/net/store.rs').read()
net_start = int(re.search(r'impl NetVars \{\s*pub const START: usize = (0x[0-9A-Fa-f]+);', net).group(1), 16)

start = moved.index('pub(crate) const MOVED')
end = moved.index('\n];', start)
homes = []
i = start
while True:
    j = moved.find('    Moved {', i)
    if j < 0 or j > end:
        break
    h = moved.index('home: Home::Store(', j) + len('home: Home::Store(')
    depth, k = 1, h
    while depth:
        c = moved[k]
        depth += {'(': 1, ')': -1}.get(c, 0)
        k += 1
    home = re.sub(r'\s+', ' ', moved[h:k - 1]).strip().rstrip(',').strip()
    home = re.sub(r'\( ', '(', home)
    home = re.sub(r',\s*\)', ')', home)
    home = re.sub(r'\s*\)', ')', home)
    home = re.sub(r'(0x[0-9A-Fa-f]+) - crate::net::store::NetVars::START',
                  lambda m: hex(int(m.group(1), 16) - net_start), home)
    homes.append(home)
    i = k

lens = [line.split() for line in open(lengths).read().splitlines()]
assert len(lens) == len(homes), (len(lens), len(homes))

out = []
for (name, ln, size), home in zip(lens, homes):
    ln, size = int(ln, 16), int(size, 16)
    assert ln <= size, name
    if ln == size:
        out.append(f'    whole("{name}", {home}),')
    else:
        out.append(f'    first("{name}", {home}, {ln:#x}),')

names = re.search(r'/// The name of each entry.*?\nconst NAMES: \[&str; \d+\] = \[\n.*?\n\];\n', stores, re.S)
assert names, 'NAMES in stores.rs'
listed = (
    '/// Every store, by the path of its owner (the static the store is a field of); a name\n'
    '/// repeats where one record is listed in parts.\n'
    f'const LISTED: [Listed; {len(out)}] = [\n' + '\n'.join(out) + '\n];\n'
)
stores = stores[:names.start()] + listed + stores[names.end():]
open(stores_path, 'w').write(stores)
print(len(out), 'stores,', sum('first(' in o for o in out), 'shorter than their store')
