# Private `const NAME: usize = rva(0x..);` of typed scalar places -> field accesses
# (game_read / game_write / game_addr / game_ptr on NAME); the constant goes when unused.
import re, os, sys, json
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
        if hex(v) in m: return st, use, m[hex(v)][0]
SKIP = ('car_store.rs', 'tables_store.rs', 'work_store.rs', 'relocations.rs')
for root, _, fs in os.walk(ROOT):
    for f in fs:
        if not f.endswith('.rs') or f in SKIP: continue
        p = os.path.join(root, f); s = open(p).read(); o = s; uses = set()
        for m in list(re.finditer(r'^\s*const (\w+): usize = (?:crate::(?:framework|store)::)?rva\(0x([0-9A-Fa-f]+)\);[^\n]*\n', s, re.M)):
            n, v = m.group(1), int(m.group(2), 16)
            t = typed(v)
            if not t: continue
            st, use, fld = t
            F = st + '.' + fld
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_read(?:::<u32>)?\(' + n + r'\)', F + '.bits()', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_read::<f32>\(' + n + r'\)', 'f32::from_bits(' + F + '.bits())', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_read::<i32>\(' + n + r'\)', '(' + F + '.bits() as i32)', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_addr\(' + n + r'\)', F + '.addr()', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_ptr::<(\w+)>\(' + n + r'\)', '(' + F + r'.addr() as *mut \1)', s)
            s = re.sub(r'(?:crate::(?:framework|store)::)?game_write(?:::<u32>)?\(' + n + r',\s*', F + '.set_bits(', s)
            if s != o: uses.add((st, use))
            if not re.search(r'\b' + n + r'\b', s.replace(m.group(0), '')):
                s = s.replace(m.group(0), '')
        if s != o:
            for st, use in uses:
                if not re.search(r'use [\w:]*\b' + st + r'\b', s):
                    i = s.find('\nuse ') + 1
                    s = s[:i] + 'use ' + use + ';\n' + s[i:]
            open(p, 'w').write(s); print(p)
