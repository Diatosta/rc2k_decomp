#!/usr/bin/env python3
"""Wave-1 clean-up of a store file: deletes the gap fields (and their docs and `new()` lines),
the `START` / `END` constants with their size assertion, the offset tests and `piece_addr`,
and the `+0x..` offsets in the docs. usage: strip_layout.py [--keep-offsets] files..."""
import re, sys

keep = '--keep-offsets' in sys.argv
GAP = r'_(?:gap|lead|pad|tail)[a-z0-9_]*'

def remove_fn(s, header_re):
    m = re.search(header_re, s)
    while m:
        i = s.index('{', m.end() - 1) if s[m.end() - 1] != '{' else m.end() - 1
        depth = 0
        j = i
        while True:
            if s[j] == '{': depth += 1
            elif s[j] == '}':
                depth -= 1
                if depth == 0: break
            j += 1
        # also drop the attribute lines before and the blank line after
        start = m.start()
        s = s[:start] + s[j + 1:].lstrip('\n')
        m = re.search(header_re, s)
    return s

for path in [a for a in sys.argv[1:] if not a.startswith('--')]:
    s = open(path).read()
    # gap fields with their doc lines
    s = re.sub(r'((?:[ \t]*///[^\n]*\n)+)?[ \t]*pub %s: [^\n]*,\n' % GAP, '', s)
    # the gaps' initialisers in new() (single-line)
    s = re.sub(r'\n[ \t]*%s: [^\n]*,(?=\n)' % GAP, '', s)
    # START / END / size assertion
    s = re.sub(r'(?:/// IDA 0x[0-9A-Fa-f]+, [^\n]*\n)?pub const START: usize = [^\n]*;\n', '', s)
    s = re.sub(r'(?:/// IDA 0x[0-9A-Fa-f]+, [^\n]*\n)?pub const END: usize = [^\n]*;\n', '', s)
    s = re.sub(r'\nconst _: \(\) = assert!\(END - START == [^\n]*\);\n', '\n', s)
    # offset tests
    s = remove_fn(s, r'[ \t]*#\[test\]\n[ \t]*fn fields_are_at_the_games_offsets\(\)')
    if '\npub fn piece_addr()' in s and s.count('piece_addr()') == 1:
        s = remove_fn(s, r'(?:/// The piece\'s host address[^\n]*\n)?#\[cfg\(test\)\]\npub fn piece_addr\(\)')
    s = s.replace('    use core::mem::offset_of;\n', '')
    if not keep:
        s = re.sub(r'(///\s*)\+0[xX][0-9a-fA-F]+ (\(0[xX][0-9A-Fa-f]+\))', r'\1\2', s)
        s = re.sub(r'(///\s*)\+0[xX][0-9a-fA-F]+: ', r'\1', s)
    s = re.sub(r'\n{3,}', '\n\n', s)
    open(path, 'w').write(s)
