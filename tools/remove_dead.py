#!/usr/bin/env python3
"""Conversion phase 4: removes the items rustc reports as never used (functions, constants,
statics, structs, enums, type aliases), with the doc comments and attributes above them.
Reads `cargo clippy --message-format=short` output on stdin. Run again until it removes
nothing: a removal can leave more unused.

    cargo clippy --target i686-pc-windows-msvc --message-format=short 2>&1 \
        | tools/remove_dead.py
"""
import importlib.util
import os
import re
import sys

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location('strip_hook', os.path.join(here, 'strip_hook.py'))
strip_hook = importlib.util.module_from_spec(spec)
sys.argv = sys.argv[:1]
spec.loader.exec_module(strip_hook)

warning = re.compile(
    r'^(src/[^:]+):(\d+):\d+: warning: (?:function|constant|static|struct|enum|type alias|'
    r'associated function|method)s? `[^`]+`(?:, `[^`]+`)*(?: and `[^`]+`)? (?:is|are) never used')
never_built = re.compile(r'^(src/[^:]+):(\d+):\d+: warning: struct `([^`]+)` is never constructed')
wanted = {}
structs = {}
for line in sys.stdin:
    m = warning.match(line)
    if m:
        wanted.setdefault(m.group(1), set()).add(int(m.group(2)))
    m = never_built.match(line)
    if m:
        wanted.setdefault(m.group(1), set()).add(int(m.group(2)))
        structs.setdefault(m.group(1), []).append(m.group(3))

removed = 0
for path, lines in wanted.items():
    s = open(path).read()
    # from the last line up, so earlier offsets stay right
    for number in sorted(lines, reverse=True):
        start = 0
        for _ in range(number - 1):
            start = s.index('\n', start) + 1
        head = s[start:s.index('\n', start)]
        item = start
        if not re.match(r'\s*(pub(\([^)]*\))? )?(unsafe )?(const |static |fn |struct |enum |type |extern )', head):
            print(f'{path}:{number}: not an item start, left: {head.strip()}')
            continue
        while start > 0:
            prev = s.rfind('\n', 0, start - 1) + 1
            above = s[prev:start].lstrip()
            if above.startswith('///') or above.startswith('#['):
                start = prev
            elif above.startswith('//') and not above.startswith('// ----'):
                # a plain comment goes with the item only if doc comments are above it
                k = prev
                while k > 0:
                    k2 = s.rfind('\n', 0, k - 1) + 1
                    t = s[k2:k].lstrip()
                    if t.startswith('///'):
                        break
                    if not (t.startswith('//') and not t.startswith('// ----')):
                        k = -1
                        break
                    k = k2
                if k <= 0:
                    break
                start = prev
            else:
                break
        brace = s.find('{', item)
        semi = s.find(';', item)
        if re.match(r'\s*(pub(\([^)]*\))? )?(const|static|type) ', head) and not re.match(
            r'\s*(pub(\([^)]*\))? )?const (unsafe )?fn ', head
        ):
            end = strip_hook.cut_item(s, s.index('=', item) + 1)
        elif semi != -1 and (brace == -1 or semi < brace) and 'fn ' not in head:
            # `struct X;`, or a tuple struct `struct X(..);` whose fields may hold `;`
            # (`[u8; 4]`): its end is the `;` after the closing parenthesis
            paren = s.find('(', item)
            if paren != -1 and paren < semi:
                depth, k = 0, paren
                while True:
                    if s[k] == '(':
                        depth += 1
                    elif s[k] == ')':
                        depth -= 1
                        if depth == 0:
                            break
                    k += 1
                semi = s.index(';', k)
            end = semi + 1
        else:
            end = strip_hook.cut_item(s, start)
        s = s[:start] + s[end:].lstrip('\n')
        removed += 1
    # the `impl` blocks of a struct never built go with it
    for name in structs.get(path, []):
        while True:
            m = re.search(r'^impl(<[^>]*>)? ([\w:<>, ]+ for )?' + name + r'\b[^{;]*\{', s, re.M)
            if not m:
                break
            start = m.start()
            while start > 0:
                prev = s.rfind('\n', 0, start - 1) + 1
                above = s[prev:start].lstrip()
                if above.startswith('///') or above.startswith('#['):
                    start = prev
                else:
                    break
            s = s[:start] + s[strip_hook.cut_item(s, m.start()):].lstrip('\n')
            removed += 1
    open(path, 'w').write(re.sub(r'\n{3,}', '\n\n', s))
print(f'removed {removed}')
