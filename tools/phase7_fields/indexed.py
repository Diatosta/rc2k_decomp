# `READ(BASE + 4 * i)` -> typed element reads, per file, for bases whose index is bounded
# there (checked by hand) or read through an unbounded helper (`RACE_WORK.car_block`).
# READ: g / k / game_read::<u32>(rva(..)). Run: indexed.py ROOT
import re, sys
ROOT = sys.argv[1]
U = {'CAR': 'crate::physics::car_store::CAR', 'PHYSICS': 'crate::physics::tables_store::PHYSICS',
     'RACE_WORK': 'crate::race::work_store::RACE_WORK'}
BLOCKS = {0x71B3B8: ('RACE_WORK', 'RACE_WORK.car_block({})')}
PER = {
 'physics/crossed_edge.rs': {0x71AAC8: 'CAR.probe_point_x[{}].bits()', 0x71ABC8: 'CAR.probe_point_y[{}].bits()', 0x71ACC8: 'CAR.probe_point_z[{}].bits()'},
 'physics/ground_contact.rs': {0x71AAC8: 'CAR.probe_point_x[{}].bits()', 0x71ABC8: 'CAR.probe_point_y[{}].bits()', 0x71ACC8: 'CAR.probe_point_z[{}].bits()'},
 'physics/wall_contacts.rs': {0x71AAC8: 'CAR.probe_point_x[{}].bits()', 0x71ABC8: 'CAR.probe_point_y[{}].bits()', 0x71ACC8: 'CAR.probe_point_z[{}].bits()', 0x71ADC8: 'CAR.probe_point_face[{}].get()'},
 'physics/crash_contact.rs': {0x71ADC8: 'CAR.probe_point_face[{}].get()'},
 'physics/drop_broken_parts.rs': {0x71AA14: 'CAR.part_damage_limits[{}].get()', 0x71BA2C: 'RACE_WORK.part_break_limits[{}].get()'},
 'physics/fill_race_slot.rs': {0x71A10C: 'CAR.position[{} as usize].get()', 0x71A08C: 'CAR.crash_matrix[{} as usize].bits()', 0x71A15C: 'CAR.spin[{} as usize].bits()'},
 'physics/car_sounds.rs': {0x71A10C: 'CAR.position[{}].get()', 0x71A67C: 'CAR.camera_position[{}].get()'},
 'physics/point_hits_car.rs': {0x71A10C: 'CAR.position[{} as usize].get()'},
}
ADDR = {'physics/limit_motion.rs': {0x71A15C: '(CAR.spin[{}].addr() as u32)', 0x71A170: '(CAR.velocity[{}].addr() as u32)'}}
def strip_cast(e):
    e = e.strip()
    m = re.fullmatch(r'(.+?) as usize', e)
    return m.group(1) if m else e
def close(s, i):
    d = 0
    for j in range(i, len(s)):
        if s[j] == '(': d += 1
        elif s[j] == ')':
            d -= 1
            if d < 0: return j
    return -1
import os
for root, _, fs in os.walk(ROOT):
    for f in fs:
        if not f.endswith('.rs') or f.endswith('_store.rs'): continue
        p = os.path.join(root, f); rel = p[len(ROOT) + 1:]; s = open(p).read(); o = s
        table = dict((a, t[1]) for a, t in BLOCKS.items()); table.update(PER.get(rel, {}))
        for kind, tb in (('read', table), ('addr', ADDR.get(rel, {}))):
            pats = [r'\bg\(', r'\bk\('] if kind == 'read' else [r'\ba\(']
            if kind == 'read': pats.append(r'game_read::<u32>\(rva\(')
            for base, tmpl in tb.items():
                for pat in pats:
                    while True:
                        m = re.search(pat + r'0x%X \+ 4 \* ' % base, s)
                        if not m: break
                        e = close(s, m.end())
                        idx = s[m.end():e]
                        end = e + 1
                        if 'rva' in pat:
                            if s[end] != ')': break
                            end += 1
                        if base in BLOCKS: rep = tmpl.format('(' + strip_cast(idx) + ') as usize' if ' ' in strip_cast(idx) else strip_cast(idx) + ' as usize')
                        else: rep = tmpl.format(idx)
                        s = s[:m.start()] + rep + s[end:]
        if s != o:
            for st, use in U.items():
                if st + '.' in s and not re.search(r'use [\w:]*\b' + st + r'\b', s):
                    i = s.find('\nuse ') + 1; s = s[:i] + 'use ' + use + ';\n' + s[i:]
            open(p, 'w').write(s); print(p)
