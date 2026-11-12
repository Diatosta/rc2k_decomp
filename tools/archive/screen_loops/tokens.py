#!/usr/bin/env python3
"""String ids from the game's language file: tokens.py <LOCALE/PORTUGES.TXT> <out.json>.
A string's id is the number of its line among the `<token> <text>` lines, from 0. The output
is game data: keep it out of the repository."""
import json, re, sys
toks = []
for line in open(sys.argv[1], encoding='latin1'):
    line = line.rstrip('\r\n')
    if not line.strip() or line.lstrip().startswith(';'):
        continue
    m = re.match(r'^(\S+)\s*(.*)$', line)
    toks.append((m.group(1), m.group(2)))
json.dump(toks, open(sys.argv[2], 'w'))
print(len(toks), 'tokens')
