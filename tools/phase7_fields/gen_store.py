# Generates an overlay store over the race block from a spec.
# usage: gen_store.py SPEC OUT START END STRUCT STATIC MAPOUT DOCFILE
import sys, json
SP, OUT, START, END, ST, STAT, MAPOUT, DOC = sys.argv[1:9]
START, END = int(START, 16), int(END, 16)
RECORDS = {
    'L': ('RaceSlot', 0x78, [
        (0x0, 'part', 'Dword', 1, 'the part (0 the slot empty).'),
        (0x4, 'place', 'Dword', 3, 'its place, whole numbers.'),
        (0x10, 'matrix', 'Float', 16, 'its matrix.'),
        (0x50, 'motion', 'Float', 3, 'its motion.'),
        (0x5C, 'unknown_5c', 'Dword', 1, 'not named.'),
        (0x60, 'car_word', 'Dword', 1, 'the car\'s +0xE00 when it broke off.'),
        (0x64, 'age', 'Dword', 1, 'the car\'s step clock when it broke off (the lowest is reused).'),
        (0x68, 'spin', 'Float', 3, 'its spin.'),
        (0x74, 'spin_length', 'Float', 1, 'the spin\'s length.'),
    ], 'A loose part flying off a car (0x78 bytes, `physics::fill_race_slot`; readings).'),
}
rows = []
for l in open(SP):
    if l.startswith('#') or not l.strip(): continue
    a, n, k, c, doc = l.rstrip('\n').split('\t')
    rows.append((int(a, 16), n, k, int(c), doc))
TY = {'D': 'Dword', 'F': 'Float'}
SIZE = {'D': 4, 'F': 4}
for k, (t, sz, _, _) in RECORDS.items(): TY[k] = t; SIZE[k] = sz
used = sorted({k for _, _, k, _, _ in rows if k in RECORDS})
out = []; w = out.append
for l in open(DOC).read().rstrip('\n').split('\n'): w('//! ' + l if l else '//!')
w('')
w('use crate::session::{Dword, Float};')
w('')
for k in used:
    t, sz, fs, doc = RECORDS[k]
    w(f'/// {doc}'); w('#[repr(C)]'); w(f'pub struct {t} {{')
    for off, n, ty, c, d in fs:
        w(f'    /// +{off:#x}: {d}')
        w(f'    pub {n}: [{ty}; {c}],' if c > 1 else f'    pub {n}: {ty},')
    w('}'); w('')
    w(f'const _: () = assert!(core::mem::size_of::<{t}>() == {sz:#x});'); w('')
w('#[repr(C)]'); w(f'pub struct {ST} {{')
at = START; fields = []
def gap(a, b):
    if b > a:
        n = (b - a) // 4
        w(f'    /// +{a-START:#x} (0x{a:X}): not named.')
        w(f'    pub unknown_{a-START:x}: [Dword; {n}],' if n > 1 else f'    pub unknown_{a-START:x}: Dword,')
for a, n, k, c, doc in rows:
    assert a >= at, hex(a)
    gap(at, a)
    w(f'    /// +{a-START:#x} (0x{a:X}): {doc}')
    w(f'    pub {n}: [{TY[k]}; {c}],' if c > 1 else f'    pub {n}: {TY[k]},')
    fields.append((a, n, k, c)); at = a + SIZE[k] * c
gap(at, END)
w('}'); w('')
w(f'/// The piece: the race block\'s bytes at IDA 0x{START:X} seen as `{ST}`. They stay there')
w('/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address')
w('/// (`moved::place`): one copy for all three.')
w('pub struct InRaceBlock;'); w('')
w('impl core::ops::Deref for InRaceBlock {')
w(f'    type Target = {ST};')
w(f'    fn deref(&self) -> &{ST} {{')
w('        // SAFETY: the race block lives as long as the program and is aligned to 4 (the')
w('        // test checks); every field is an atomic or a cell, as the block\'s bytes were shared.')
w(f'        unsafe {{ &*(piece_addr() as *const {ST}) }}')
w('    }'); w('}'); w('')
w(f'pub static {STAT}: InRaceBlock = InRaceBlock;'); w('')
w(f'/// IDA 0x{START:X}, the first byte of the piece.'); w(f'pub const START: usize = 0x{START:X};')
w(f'/// IDA 0x{END:X}, the end of the piece.'); w(f'pub const END: usize = 0x{END:X};'); w('')
w(f'const _: () = assert!(END - START == core::mem::size_of::<{ST}>());'); w('')
w(f'/// Where IDA 0x{START:X} is now (in the race block).')
w('pub fn piece_addr() -> usize {'); w('    crate::race::store::home(START)'); w('}'); w('')
w('#[cfg(test)]'); w('mod tests {'); w('    use super::*;'); w('    use core::mem::offset_of;'); w('')
w('    #[test]'); w('    fn piece_sits_on_the_race_block() {')
w('        assert_eq!(piece_addr() % 4, 0);')
w('        assert_eq!(piece_addr(), crate::framework::game_addr(crate::framework::rva(START)));')
for a, n, k, c in fields[::6]:
    w(f'        assert_eq!(offset_of!({ST}, {n}), 0x{a:X} - START);')
w('    }'); w('}')
open(OUT, 'w').write('\n'.join(out) + '\n')
amap = {}
for a, n, k, c in fields:
    for i in range(c):
        nm = f'{n}[{i}]' if c > 1 else n
        if k in 'DF': amap[hex(a + 4 * i)] = (nm, TY[k])
        else:
            t, sz, fs, _ = RECORDS[k]
            for off, fn, ty, fc, _ in fs:
                for j in range(fc):
                    amap[hex(a + sz * i + off + 4 * j)] = (f'{nm}.{fn}' + (f'[{j}]' if fc > 1 else ''), ty)
json.dump(amap, open(MAPOUT, 'w')); print(len(fields), len(amap))
