# The converters type a whole-run `game_read::<[u32; N]>` / `game_write(at, [..])` as
# its first cell (`X[0].bits()`, `X[0].set_bits([..])`). Reads clippy's short output on
# stdin and, on each line rustc flags (`expected `[u32; N]`, found `u32`` and back), turns
# them into `crate::session::read_bits(&X)` / `write_bits(&X, ..)`. Run from the root.
import re, sys
fixes = []
for l in sys.stdin:
    m = re.match(r'(src/[^:]+):(\d+):\d+: error\[E0308\]: mismatched types: expected `([^`]+)`, found `([^`]+)`', l)
    if m and ('[' in m.group(3)) != ('[' in m.group(4)): fixes.append((m.group(1), int(m.group(2)), '[' in m.group(3)))
done = set()
for p, ln, read in fixes:
    if (p, ln) in done: continue
    done.add((p, ln))
    L = open(p).read().split('\n'); l = L[ln - 1]
    if read:
        l2 = re.sub(r'([A-Z_]+(?:\.\w+)+)\[0\]\.(?:bits|get)\(\)', r'crate::session::read_bits(&\1)', l)
    else:
        l2 = re.sub(r'([A-Z_]+(?:\.\w+)+)\[0\]\.(?:set_bits|set)\(', r'crate::session::write_bits(&\1, ', l)
    print(p, ln, 'ok' if l2 != l else 'UNCHANGED')
    L[ln - 1] = l2; open(p, 'w').write('\n'.join(L))
