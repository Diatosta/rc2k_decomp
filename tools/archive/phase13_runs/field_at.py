# The field of a typed struct at byte offsets: python3 tools/phase13_runs/field_at.py Car 0x654 0x8C ..
import sys
src = open('tools/phase13_runs/typed_runs.py').read()
argv, sys.argv = sys.argv, ['x']
g = {}
exec(compile(src[:src.index('READS = {')], 'typed_runs', 'exec'), g)
ty = argv[1]
for o in argv[2:]:
    off = int(o, 0)
    r = g['walk'](ty, off, '')
    if r:
        print(o, r[0].lstrip('.'), r[1])
    else:
        lay = g['layout'](ty)
        hit = [(fn, ft, at) for fn, ft, at in lay[0] if at <= off < at + g['size'](ft)]
        print(o, 'not a whole cell:', hit[0][0] if hit else None, hit[0][1] if hit else '', hex(off - hit[0][2]) if hit else '')
