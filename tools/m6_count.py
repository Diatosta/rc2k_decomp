#!/usr/bin/env python3
"""M6's progress: production uses of the owners reached without a parameter, per module.

Kinds: accessor calls (`race()` ...), the zero-sized statics that deref into an owner,
`owner::STATIC` paths, and `owners::boundary()`. Tests, comments and strings are left out."""
import collections
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from count_unsafe import strip_tests

ACCESSORS = ['race', 'system', 'scene', 'd3d', 'sound', 'audio', 'input', 'net', 'gfx',
             'menus', 'effects', 'install', 'saves', 'service', 'strings', 'screens', 'hud']
ZST = ['CAR_BLOCKS', 'VIEWER_PANEL', 'VIEWER_BUFFERS', 'GEAR', 'REPLAY_WORK',
       'EFFECTS_SETTINGS', 'FACES', 'KIND_TABLES', 'CHECKPOINTS', 'TEXT_BUFFERS',
       'LIGHT_RECORDS', 'ENGINE_TABLES', 'POLYGON_TABLES', 'VERTICES', 'POLYGON_WORK']
STATICS = ['RACE', 'SYSTEM', 'SCENE', 'D3D', 'SOUND', 'AUDIO_OWNER', 'INPUT', 'NET_OWNER',
           'GFX', 'MENUS', 'EFFECTS', 'INSTALL', 'SAVES', 'SERVICE', 'STRINGS',
           'SCREENS_OWNER', 'HUD']

KINDS = {
    'accessor': re.compile(r'(?<![\w.])(?:\w+::)*(?:%s)\(\)' % '|'.join(ACCESSORS)),
    'zst': re.compile(r'(?<![\w.])(?:\w+::)*(?:%s)\b(?!\s*[:(!])' % '|'.join(ZST)),
    'static': re.compile(r'\bowner::(?:%s)\b' % '|'.join(STATICS)),
    'boundary': re.compile(r'\bowners::boundary\(\)'),
}

STRING = re.compile(r'"(?:\\.|[^"\\])*"')


def clean(s):
    s = re.sub(r'/\*.*?\*/', '', s, flags=re.S)
    s = re.sub(r'//[^\n]*', '', s)
    s = strip_tests(s)
    s = STRING.sub('""', s)
    # a definition is not a use
    return re.sub(r'\bfn\s+\w+\(\)', 'fn x()', s)


TEST_MOD = re.compile(r'#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;')


def test_only_files():
    """The files of modules declared only for tests (`#[cfg(test)] mod x;`)."""
    found = set()
    for d, _, fs in os.walk('src'):
        for f in fs:
            if not f.endswith('.rs'):
                continue
            p = os.path.join(d, f)
            here = d if f == 'mod.rs' or p == 'src/lib.rs' else os.path.join(d, f[:-3])
            for name in TEST_MOD.findall(open(p, errors='replace').read()):
                found.add(os.path.join(here, name + '.rs'))
                found.add(os.path.join(here, name, 'mod.rs'))
    return found


def main():
    table = collections.defaultdict(collections.Counter)
    skipped = test_only_files()
    for d, _, fs in os.walk('src'):
        for f in fs:
            p = os.path.join(d, f)
            if not f.endswith('.rs') or p == 'src/owners.rs' or p in skipped or re.search(r'x87_reference|asm_test|asm_reference', p):
                continue
            parts = os.path.relpath(p, 'src').split(os.sep)
            mod = parts[0][:-3] if len(parts) == 1 else parts[0]
            s = clean(open(p, errors='replace').read())
            for k, pat in KINDS.items():
                table[mod][k] += len(pat.findall(s))
    kinds = list(KINDS)
    print('%-18s' % 'module' + ''.join('%10s' % k for k in kinds))
    tot = collections.Counter()
    for mod in sorted(table, key=lambda m: -sum(table[m].values())):
        if not sum(table[mod].values()):
            continue
        print('%-18s' % mod + ''.join('%10d' % table[mod][k] for k in kinds))
        tot.update(table[mod])
    print('%-18s' % 'TOTAL' + ''.join('%10d' % tot[k] for k in kinds))


if __name__ == '__main__':
    main()
