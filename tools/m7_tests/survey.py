#!/usr/bin/env python3
"""M7 step 5's survey: the old-address idioms the tests still use, per module and per kind.

Run from the repository root: `python3 tools/m7_tests/survey.py [--files] [--production]`.
Test text is a file's text minus `count_unsafe.strip_tests` (plus test-only files whole);
comments are left out. The old-address layer is deleted (round 30: moved.rs, address_map.rs,
pointers.rs, ral_image.bin): the kinds named after it stay as tripwires and must read 0. The files that
remain of the memory plumbing (framework.rs, stores.rs, write_watch.rs, shared_memory.rs,
cells, pool.rs) are listed apart as LAYER."""
import collections
import os
import re
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from count_unsafe import strip_tests  # noqa: E402
from m6_count import test_only_files  # noqa: E402

KINDS = collections.OrderedDict([
    ('image_bytes', r'\bimage_bytes\('),
    ('moved::place', r'\bmoved::place\('),
    ('moved::named', r'\bmoved::named(?:_in)?\('),
    ('moved::snapshot/restore', r'\bmoved::(?:snapshot|restore)\('),
    ('host_at', r'\bhost_at\('),
    ('game_addr', r'\bgame_addr\('),
    ('rva', r'(?<![\w.])rva\('),
    ('game_ptr', r'(?<![\w.])game_ptr\('),
    ('game_read/write', r'(?<![\w.])game_(?:read|write)(?:_bytes)?\('),
    ('fn home()', r'\bfn home\(\)'),
    ('pointers::host', r'\bpointers::host\('),
    ('GamePtr::new', r'\bGamePtr::new\('),
    ('asm_base/OldPlaces', r'\b(?:asm_base\(|OldPlaces\b)'),
    ('IMAGE_BASE', r'\bIMAGE_BASE\b'),
    ('home_of_rva', r'\bhome_of_rva\('),
    ('ida literal', r'\b0x0*(?:4[0-9A-Fa-f]|5[0-9A-Fa-f]|6[0-9A-Fa-f]|7[0-4])[0-9A-Fa-f_]{4}\b'),
])
PATS = {k: re.compile(v) for k, v in KINDS.items()}
LAYER = ('src/framework.rs', 'src/stores.rs', 'src/write_watch.rs', 'src/shared_memory.rs',
         'src/cells.rs', 'src/cells/', 'src/pool.rs')


def uncomment(s):
    s = re.sub(r'/\*.*?\*/', '', s, flags=re.S)
    return re.sub(r'//[^\n]*', '', s)


def counts(s):
    return collections.Counter({k: len(p.findall(s)) for k, p in PATS.items()})


def module_of(path):
    parts = path.split('/')
    if len(parts) == 2:
        return parts[1][:-3]
    return parts[1]


def main():
    production = '--production' in sys.argv
    tonly = test_only_files()
    per_module = collections.defaultdict(collections.Counter)
    per_file = {}
    for d, _, fs in os.walk('src'):
        for f in sorted(fs):
            if not f.endswith('.rs'):
                continue
            p = os.path.join(d, f)
            s = uncomment(open(p, errors='replace').read())
            stripped = '' if p in tonly else strip_tests(s)
            full, prod = counts(s), counts(stripped)
            c = prod if production else full - prod
            if not +c:
                continue
            key = 'LAYER' if p.startswith(LAYER) else module_of(p)
            per_module[key] += c
            per_file[p] = c
    kinds = list(KINDS)
    idiom_kinds = [k for k in kinds if k != 'ida literal']
    print('| module | files | idioms | ' + ' | '.join(kinds) + ' |')
    print('|' + ' --- |' * (len(kinds) + 3))
    total = collections.Counter()
    nfiles = collections.Counter(module_of(p) if not p.startswith(LAYER) else 'LAYER'
                                 for p in per_file)
    for m in sorted(per_module, key=lambda m: -sum(per_module[m][k] for k in idiom_kinds)):
        c = per_module[m]
        total += c
        print(f'| {m} | {nfiles[m]} | {sum(c[k] for k in idiom_kinds)} | '
              + ' | '.join(str(c[k]) for k in kinds) + ' |')
    print(f'| total | {len(per_file)} | {sum(total[k] for k in idiom_kinds)} | '
          + ' | '.join(str(total[k]) for k in kinds) + ' |')
    if '--files' in sys.argv:
        for p in sorted(per_file, key=lambda p: -sum(per_file[p].values())):
            c = per_file[p]
            print(p, sum(c[k] for k in idiom_kinds), dict(+c))


if __name__ == '__main__':
    main()
