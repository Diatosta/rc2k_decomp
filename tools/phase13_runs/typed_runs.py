# The x87 runs' operands at the race block's RVAs (`fld(rc, 0x224044)`, `rc` =
# `race::store::asm_base()`) -> the typed stores' fields (`ld(RACE_VARS.x.bits())`).
# Reads every `#[repr(C)]` struct in src/ and each overlay static's `START` / `Target`,
# walks a constant operand's IDA address (RVA + 0x400000) to its field, and rewrites it
# when the field is a 4-byte cell at that very address. Indexed operands are left.
# Usage: python3 tools/phase13_runs/typed_runs.py [--write]   (from the repository root)
import os, re, sys

SRC = 'src'
PRIM = {'Dword': 4, 'Float': 4, 'Loose': 4, 'Word': 2, 'Byte': 1, 'u32': 4, 'i32': 4,
        'f32': 4, 'u16': 2, 'u8': 1}
CELLS = {'Dword', 'Float'}

files = {}
for d, _, fs in os.walk(SRC):
    for f in fs:
        if f.endswith('.rs'):
            p = os.path.join(d, f)
            files[p] = open(p).read()

consts = {}
for t in files.values():
    for m in re.finditer(r'\bconst ([A-Z_0-9]+): usize = (0x[0-9A-Fa-f_]+|\d+);', t):
        consts.setdefault(m.group(1), int(m.group(2).replace('_', ''), 0))

structs = {}  # name -> {file: fields}
CUR = ['']  # the store's file: a name defined twice is taken from it


def struct(name):
    s = structs.get(name)
    if not s:
        return None
    if len(s) == 1:
        return next(iter(s.values()))
    return s.get(CUR[0])


def bytes_len(ty):
    b = re.fullmatch(r'(?:crate::session::)?Bytes<(.+)>', ty)
    if not b:
        return None
    n = b.group(1).strip()
    return int(n, 0) if re.fullmatch(r'0x[0-9A-Fa-f]+|\d+', n) else consts.get(n.split('::')[-1])
for p, t in files.items():
    for m in re.finditer(r'#\[repr\(C\)\]\s*(?:#\[[^\]]*\]\s*)*pub struct (\w+) \{(.*?)\n\}', t, re.S):
        body = re.sub(r'//[^\n]*', '', m.group(2))
        fields = re.findall(r'pub (\w+): ([^,]+),', body)
        structs.setdefault(m.group(1), {})[p] = fields


ALIGN = {'Loose': 1}


def align(ty):
    ty = ty.strip()
    a = re.fullmatch(r'\[(.+); (.+)\]', ty)
    if a:
        return align(a.group(1))
    ty = ty.split('::')[-1]
    if ty in PRIM:
        return ALIGN.get(ty, PRIM[ty])
    if bytes_len(ty) is not None:
        return 1
    s = struct(ty)
    return max([align(ft) for _, ft in s] or [1]) if s else 1


def layout(ty):
    """Each field's (name, type, offset) and the struct's size (repr(C))."""
    s = struct(ty)
    if not s:
        return None
    at, out = 0, []
    for fn, ft in s:
        z = size(ft)
        if z is None:
            return None
        at = (at + align(ft) - 1) // align(ft) * align(ft)
        out.append((fn, ft, at))
        at += z
    a = align(ty)
    return out, (at + a - 1) // a * a


def size(ty):
    ty = ty.strip()
    a = re.fullmatch(r'\[(.+); (.+)\]', ty)
    if a:
        n = a.group(2).strip()
        n = int(n, 0) if re.fullmatch(r'0x[0-9A-Fa-f]+|\d+', n) else consts.get(n.split('::')[-1])
        e = size(a.group(1))
        return None if n is None or e is None else n * e
    if bytes_len(ty) is not None:
        return bytes_len(ty)
    ty = ty.split('::')[-1]
    if ty in PRIM:
        return PRIM[ty]
    l = layout(ty)
    return l[1] if l else None


def walk(ty, off, path):
    """The leaf cell at `off` in `ty`: (path, cell type), or None."""
    ty = ty.strip()
    a = re.fullmatch(r'\[(.+); (.+)\]', ty)
    if a:
        e = size(a.group(1))
        if not e:
            return None
        return walk(a.group(1), off % e, f'{path}[{off // e}]')
    if bytes_len(ty) is not None:
        return None
    base = ty.split('::')[-1]
    if base in PRIM:
        return (path, base) if off == 0 else None
    l = layout(base)
    if not l:
        return None
    for fn, ft, at in l[0]:
        if at <= off < at + size(ft):
            return walk(ft, off - at, f'{path}.{fn}')
    return None


bad = 0
for p_, t_ in files.items():
    CUR[0] = p_
    for m in re.finditer(r'size_of::<(\w+)>\(\) == (0x[0-9A-Fa-f_]+|\d+)', t_):
        z = size(m.group(1))
        if z is not None and z != int(m.group(2).replace('_', ''), 0):
            print('SIZE MISMATCH', m.group(1), hex(z), m.group(2), file=sys.stderr)
            bad += 1
if bad:
    sys.exit(1)

stores = []  # (start, end, static, target, module path)
for p, t in files.items():
    st = re.search(r'pub const START: usize = (0x[0-9A-Fa-f]+);', t)
    tg = re.search(r'type Target = (\w+);', t)
    sv = re.findall(r'pub static ([A-Z_0-9]+): (\w+) = \2;', t)
    if not (st and tg and sv):
        continue
    CUR[0] = p
    z = size(tg.group(1))
    if z is None:
        print('NO SIZE', p, tg.group(1), file=sys.stderr)
        continue
    start = int(st.group(1), 0)
    mod = 'crate::' + p[len(SRC) + 1:-3].replace('/', '::')
    stores.append((start, start + z, sv[0][0], tg.group(1), mod, p))


def field(ida):
    for s, e, name, tgt, mod, p in stores:
        if s <= ida < e:
            CUR[0] = p
            r = walk(tgt, ida - s, f'{mod}::{name}')
            return r
    return None


def close(t, i):
    """The index of the `)` closing the `(` before `i`."""
    depth = 1
    while depth:
        c = t[i]
        depth += c == '('
        depth -= c == ')'
        i += 1
    return i - 1


READS = {'fld': 'crate::math::x87_float::ld({}.bits())',
         'fild': 'crate::math::x87_float::int({}.bits() as i32)'}
WRITES = {'fstp': '{}.set_bits(crate::math::x87_float::st({}))',
          'fistp': '{}.set_bits(crate::math::x87_float::to_int({}) as u32)',
          'mov': '{}.set_bits({})',
          'st16': '{}.set_bits({})'}
# the marks module's names (`marks::plain`)
ALIAS = {'ld': 'fld', 'ldi': 'fild', 'st': 'fstp', 'sti': 'fistp'}
write = '--write' in sys.argv
done = left = 0
missed = {}
for p, t in files.items():
    out, i, changed = [], 0, False
    for m in re.finditer(r'\b(fld|fild|fstp|fistp|mov|ld|ldi|st|sti|st16)\(\s*rc,\s*(?:(\w+) \* 4 \+ )?(0x[0-9A-F]+)\s*([,)])', t):
        if m.start() < i:
            continue
        op, n, k = ALIAS.get(m.group(1), m.group(1)), m.group(2), int(m.group(3), 0)
        f = field(k + 0x400000)
        if not f or f[1] not in (('Word',) if op == 'st16' else CELLS):
            left += 1
            missed[hex(k + 0x400000)] = f
            continue
        if n:
            # an element of an array of cells: its index plus `n`
            e = re.fullmatch(r'(.*)\[(\d+)\]', f[0])
            if not e:
                left += 1
                continue
            first = int(e.group(2))
            f = (f'{e.group(1)}[{n}]' if first == 0 else f'{e.group(1)}[{first} + {n}]', f[1])
        if op in READS:
            if m.group(4) != ')':
                continue
            out.append(t[i:m.start()] + READS[op].format(f[0]))
            i = m.end()
        else:
            if m.group(4) != ',':
                continue
            end = close(t, m.end())
            arg = t[m.end():end].strip().rstrip(',').strip()
            out.append(t[i:m.start()] + WRITES[op].format(f[0], arg))
            i = end + 1
        done += 1
        changed = True
    if changed:
        out.append(t[i:])
        if write:
            open(p, 'w').write(''.join(out))
print(f'stores {len(stores)}; converted {done}; left {left}', file=sys.stderr)
for a, f in sorted(missed.items()):
    print('left', a, f, file=sys.stderr)
