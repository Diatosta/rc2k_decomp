#!/usr/bin/env python3
"""Counts what the memory safety work tracks, leaving out items only built for tests."""
import collections
import os
import re
import sys


def strip_tests(s):
    out = ''
    i = 0
    pat = re.compile(r'#\[cfg\((?:test|all\([^\]]*\btest\b[^\]]*\)|any\([^\]]*\btest\b[^\]]*\))\)\]')
    while True:
        m = pat.search(s, i)
        if not m:
            return out + s[i:]
        out += s[i:m.start()]
        k = s.find('{', m.end())
        e = s.find(';', m.end())
        if k < 0 or 0 <= e < k:
            i = e + 1
            continue
        d = 0
        p = k
        while p < len(s):
            if s[p] == '{':
                d += 1
            elif s[p] == '}':
                d -= 1
                if d == 0:
                    break
            p += 1
        i = p + 1


PATTERNS = {
    'unsafe {': r'\bunsafe\s*\{',
    'unsafe fn': r'\bunsafe\s+(?:extern\s+"[^"]*"\s+)?fn\b',
    'GamePtr::new': r'GamePtr::new',
    '.addr()': r'\.addr\(\)',
    'pointers::host': r'pointers::host',
}


def main():
    totals = collections.Counter()
    files = {k: collections.Counter() for k in PATTERNS}
    for root in ('src', 'rc2k_core/src'):
        for d, _, fs in os.walk(root):
            for f in fs:
                if not f.endswith('.rs'):
                    continue
                p = os.path.join(d, f)
                s = open(p, errors='replace').read()
                if re.search(r'x87_reference|asm_test', p):
                    continue
                s = strip_tests(re.sub(r'//[^\n]*', '', s))
                for k, pat in PATTERNS.items():
                    n = len(re.findall(pat, s))
                    totals[k] += n
                    if n:
                        files[k][p] = n
    print(dict(totals))
    if len(sys.argv) > 1:
        for p, n in files[sys.argv[1]].most_common():
            print(n, p)


if __name__ == '__main__':
    main()
