#!/usr/bin/env python3
"""The memory layer's census (notes/memory_layer_plan.md, step 1): how production code makes
game places and what the stores are made of. Counts leave out items built only for tests.

    tools/memory_census.py [--files]   (--files: the sites by file for each kind)
"""
import collections, os, re, sys

sys.path.insert(0, os.path.dirname(__file__))
from count_unsafe import strip_tests  # noqa: E402

ROOT = os.path.join(os.path.dirname(__file__), '..', 'src')
LAYER = {'moved.rs', 'pointers.rs', 'session.rs', 'store.rs', 'shared_memory.rs', 'address_map.rs'}

# How a place is made.
PLACES = {
    'GamePtr::to (a field of a store)': r'GamePtr::to\(',
    'GamePtr::new (a number)': r'GamePtr::new\(',
    '.addr() (a place to a number)': r'\.addr\(\)',
    'game_address (Rust memory to a number)': r'\bgame_address\(',
    'moved::place / home_of_rva / game_addr (an IDA name)': r'\b(moved::place|home_of_rva|game_addr)\(',
    'host / host_at (a number to a raw pointer)': r'\bhost(_at)?\(',
    'read_or_end / write_or_end / check_or_end (by number)': r'\b(read_or_end|write_or_end|check_or_end)\b',
    '.record::<T> (a number to &T)': r'\.record(::<[^>]*>)?\(',
}
# What the stores are made of.
CELLS = {
    'Dword / Float / Word / Byte (atomic cells, mentions)': r'\b(Dword|Float|Word|Byte)\b',
    'Bytes<N> (raw bytes)': r'\bBytes<',
    'Loose (an unaligned dword)': r'\bLoose\b',
    'Shared<T> (a raw record)': r'\bShared<',
    'Place<T> (raw Rust memory)': r'\bPlace<',
}


def sources():
    for root, _, names in os.walk(ROOT):
        for name in names:
            path = os.path.join(root, name)
            if name.endswith('.rs') and not re.search(r'x87_reference|asm_test', path):
                text = strip_tests(open(path, encoding='utf-8').read())
                yield os.path.relpath(path, ROOT), re.sub(r'//[^\n]*', '', text)


def main():
    by_file = '--files' in sys.argv
    counts = {k: collections.Counter() for k in {**PLACES, **CELLS}}
    shared_types = collections.Counter()
    for path, text in sources():
        layer = os.path.basename(path) in LAYER
        for kind, pattern in {**PLACES, **CELLS}.items():
            n = len(re.findall(pattern, text))
            if n:
                counts[kind]['(layer) ' + path if layer else path] += n
        for t in re.findall(r'\bShared<([^>]+(?:<[^>]*>)?)>', text):
            shared_types[t.strip()] += 1
    for title, kinds in (('How places are made', PLACES), ('What the stores hold', CELLS)):
        print(f'## {title}\n')
        for kind in kinds:
            c = counts[kind]
            layer = sum(v for p, v in c.items() if p.startswith('(layer)'))
            print(f'- {kind}: {sum(c.values())} in {len(c)} files ({layer} in the layer)')
            if by_file:
                for p, v in c.most_common():
                    print(f'    {v:4}  {p}')
        print()
    print('## `Shared<T>` by type\n')
    for t, n in shared_types.most_common():
        print(f'- {n:3}  {t}')


if __name__ == '__main__':
    main()
