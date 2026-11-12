# `HELPER(NAME, v)` / `HELPER(NAME[i], v)` with HELPER a file's own one-dword writer
# (`fn set(r: usize, v: u32) { game_write(r, v) }`, checked by hand) and NAME a constant
# `rva(0x..)` (or an array of them) on a typed scalar -> `STATIC.field.set_bits(v)`.
# With `read` last: `HELPER(NAME)` / `HELPER(NAME[i])` -> `STATIC.field.bits()` (HELPER a
# one-dword reader).
# usage: local_set.py FILE HELPER MAP STATIC USE [read]
import re, sys, json
F, H, MAP, ST, USE = sys.argv[1:6]
amap = json.load(open(MAP)); s = open(F).read()
consts = {}
for m in re.finditer(r'const (\w+): usize = rva\((0x[0-9A-Fa-f]+)\);', s):
    consts[m.group(1)] = [hex(int(m.group(2), 16))]
for m in re.finditer(r'const (\w+): \[usize; \d+\] = \[([^\]]+)\];', s):
    consts[m.group(1)] = [hex(int(a, 16)) for a in re.findall(r'rva\((0x[0-9A-Fa-f]+)\)', m.group(2))]
n = 0
def rep(m):
    global n
    name, idx, v = m.group(1), m.group(3), m.group(4)
    if name not in consts: return m.group(0)
    a = consts[name][int(idx) if idx else 0]
    if a not in amap: return m.group(0)
    n += 1
    return f'{ST}.{amap[a][0]}.set_bits({v});'
READ = sys.argv[6:7] == ['read']
def rep_read(m):
    global n
    name, idx = m.group(1), m.group(3)
    if name not in consts: return m.group(0)
    a = consts[name][int(idx) if idx else 0]
    if a not in amap: return m.group(0)
    n += 1
    return f'{ST}.{amap[a][0]}.bits()'
if READ:
    s = re.sub(rf'\b{H}\((\w+)(\[(\d)\])?\)', rep_read, s)
else:
    s = re.sub(rf'\b{H}\((\w+)(\[(\d)\])?, ([^;\n]+)\);', rep, s)
if n and USE not in s:
    s = s.replace('\nuse ', f'\nuse {USE};\nuse ', 1)
open(F, 'w').write(s); print(F, n)
