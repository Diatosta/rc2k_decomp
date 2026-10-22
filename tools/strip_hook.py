#!/usr/bin/env python3
"""Conversion phase 4: removes a hook's machinery from its Rust files, keeping the Rust
version. Cuts `static ORIGINAL`, `pub static HOOK`, `hook_asm!(..)`, `fn dispatch` and
`fn self_test` (with the doc comments and attributes above them) and the `// ---- hook ----`
style banners. Unused imports and helpers are left for `cargo clippy --fix` and by hand.

    tools/strip_hook.py src/<module>/<function>.rs ...
"""
import re
import sys


def cut_item(s, start):
    """From `start`, the end of the item: a `{..}` block, or a `(..);` / `..;` statement;
    string, char and comment text skipped."""
    i = start
    depth = 0
    opened = False
    n = len(s)
    while i < n:
        c = s[i]
        if c == '/' and s.startswith('//', i):
            i = s.index('\n', i)
            continue
        if c == '"' or (c in 'rb' and s.startswith(('r"', 'r#"', 'b"', 'br"'), i)):
            j = i
            while s[j] != '"':
                j += 1
            hashes = s[i:j].count('#')
            raw = 'r' in s[i:j]
            j += 1
            while True:
                if not raw and s[j] == '\\':
                    j += 2
                    continue
                if s[j] == '"' and s[j + 1:j + 1 + hashes] == '#' * hashes:
                    j += 1 + hashes
                    break
                j += 1
            i = j
            continue
        if c == "'":
            # a char literal ('x', '\n', '\u{..}'), not a lifetime
            if s[i + 1] == '\\':
                i = s.index("'", i + 2) + 1
                continue
            if i + 2 < n and s[i + 2] == "'":
                i += 3
                continue
        if c in '{([':
            depth += 1
            opened = True
        elif c in '})]':
            depth -= 1
            if opened and depth == 0:
                j = i + 1
                if j < n and s[j] == ';':
                    return j + 1
                if c == '}':
                    return j
                # a `(..)` or `[..]` not ended by `;`: a function's parameters (its body
                # follows) or part of an expression
                opened = False
        elif c == ';' and depth == 0:
            return i + 1
        i += 1
    raise SystemExit('unbalanced')


def strip(s):
    pats = [r'^static ORIGINAL: AtomicUsize[^\n]*\n',
            r'^pub static HOOK: RawHook\s*=',
            r'^hook_asm!\(',
            r'^(?:pub )?extern "C" fn dispatch\(',
            r'^(?:pub )?fn self_test\(\)']
    for p in pats:
        while True:
            m = re.search(p, s, re.M)
            if not m:
                break
            start = m.start()
            while start > 0:
                prev = s.rfind('\n', 0, start - 1) + 1
                line = s[prev:start].lstrip()
                if line.startswith('///') or line.startswith('#['):
                    start = prev
                else:
                    break
            if p.endswith(r'\n'):
                end = m.end()
            else:
                end = cut_item(s, m.end() - 1 if s[m.end() - 1] in '({' else m.end())
            s = s[:start] + s[end:].lstrip('\n')
    s = re.sub(r'^// ---- (hook|self-test|self test|test)[^\n]*----\n', '', s, flags=re.M)
    return re.sub(r'\n{3,}', '\n\n', s)


for path in sys.argv[1:]:
    text = strip(open(path).read())
    open(path, 'w').write(text)
    print('stripped', path)
