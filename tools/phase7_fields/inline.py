# Inline game_read / game_write / game_addr on rva(typed literal) -> field accesses.
import re, os, sys, json, collections
ROOT, S = sys.argv[1], sys.argv[2]
maps = [(json.load(open(S + '/car_map.json')), 'CAR', 'crate::physics::car_store::CAR'),
        (json.load(open(S + '/tables_map.json')), 'PHYSICS', 'crate::physics::tables_store::PHYSICS'),
        (json.load(open(S + '/p8_map.json')), 'RACE_WORK', 'crate::race::work_store::RACE_WORK'),
        (json.load(open(S + '/blocks_map.json')), 'CAR_BLOCKS', 'crate::race::blocks_store::CAR_BLOCKS'),
        (json.load(open(S + '/num_map.json')), 'NUMBERS', 'crate::race::numbers_store::NUMBERS'),
        (json.load(open(S + '/draw_map.json')), 'DRAW', 'crate::scene::draw_store::DRAW'),
        (json.load(open(S + '/vars_map.json')), 'RACE_VARS', 'crate::race::vars_store::RACE_VARS'),
        (json.load(open(S + '/view_map.json')), 'RACE_VIEW', 'crate::race::view_store::RACE_VIEW'),
        (json.load(open(S + '/obj_map.json')), 'OBJECT_WORK', 'crate::scene::object_store::OBJECT_WORK'),
        (json.load(open(S + '/course_map.json')), 'COURSE', 'crate::race::course_store::COURSE'),
        (json.load(open(S + '/frame_map.json')), 'FRAME_WORK', 'crate::race::frame_store::FRAME_WORK')]
def typed(v):
    for m, st, use in maps:
        if hex(v) in m: return st, use, m[hex(v)][0], m[hex(v)][1]
FW = r'(?:crate::(?:framework|store)::)?'
L = r'(?:crate::(?:framework|store)::)?rva\(0x([0-9A-Fa-f]{6})\)'
tot = collections.Counter()
for root, _, fs in os.walk(os.path.join(ROOT, 'src')):
    for f in fs:
        p = os.path.join(root, f)
        if not f.endswith('.rs') or f in ('car_store.rs', 'tables_store.rs', 'work_store.rs'): continue
        s = open(p).read(); n = 0; sts = set()
        def R(t):
            clash = re.search(r'\b(const|static)\s+' + t[0] + r'\b', s) is not None
            sts.add((t[0], t[1], clash)); return (t[1] if clash else t[0]) + '.' + t[2]
        def rd(m):
            global n
            t = typed(int(m.group(1), 16))
            if not t: return m.group(0)
            n += 1; return R(t) + ('.get()' if t[3] == 'Dword' else '.bits()')
        def ad(m):
            global n
            t = typed(int(m.group(1), 16))
            if not t: return m.group(0)
            n += 1; return R(t) + '.addr()'
        n = 0
        s2 = re.sub(FW + r'game_read::<u32>\(' + L + r'\)', rd, s)
        s2 = re.sub(r'(let \w+: u32 = )' + FW + r'game_read\(' + L + r'\)', lambda m: m.group(1) + rd(re.match(r'.*?(?:game_read\()' + L, m.group(0))) if typed(int(m.group(2), 16)) else m.group(0), s2)
        s2 = re.sub(FW + r'game_addr\(' + L + r'\)', ad, s2)
        # game_write(rva(L), v) with a balanced v on one line
        def wr(m):
            global n
            t = typed(int(m.group(1), 16))
            if not t or m.group(2).count('(') != m.group(2).count(')'): return m.group(0)
            n += 1; return R(t) + ('.set(' if t[3] == 'Dword' else '.set_bits(') + m.group(2) + ')'
        s2 = re.sub(FW + r'game_write\(' + L + r', ([^;\n]+)\)(?=;)', wr, s2)
        if n == 0: continue
        for st, use, clash in sts:
            if not clash and f'use {use};' not in s2:
                k = re.search(r'^use [^;]+;\n', s2, re.M).end(); s2 = s2[:k] + f'use {use};\n' + s2[k:]
        open(p, 'w').write(s2); tot[p] = n
print(sum(tot.values()), len(tot))
