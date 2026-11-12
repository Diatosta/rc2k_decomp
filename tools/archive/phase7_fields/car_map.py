# The work car's address map (IDA address -> [field, kind]) from src/physics/car_store.rs'
# `/// +0xOFF (0xADDR): ..` / `pub name: Type` pairs. Run: car_map.py OUT
import re, sys, json
s = open('src/physics/car_store.rs').read()
out = {}
for m in re.finditer(r'/// \+0x[0-9a-fA-F]+ \((0x[0-9A-Fa-f]+)\):[^\n]*\n(?:\s*///[^\n]*\n)*\s*pub (\w+): (\[(\w+); (\d+|0x[0-9A-Fa-f]+)\]|\w+),', s):
    a = int(m.group(1), 16); name = m.group(2)
    if m.group(4):
        for k in range(int(m.group(5), 0)): out[hex(a + 4 * k)] = ['%s[%d]' % (name, k), m.group(4)]
    else: out[hex(a)] = [name, m.group(3)]
json.dump(out, open(sys.argv[1], 'w')); print(len(out))
