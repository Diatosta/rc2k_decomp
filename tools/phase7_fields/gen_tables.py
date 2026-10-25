import sys
SP = sys.argv[1]; OUT = sys.argv[2]
START, END = 0x613828, 0x614D38
rows = []
for l in open(SP):
    if l.startswith('#') or not l.strip(): continue
    a, n, k, c, doc = l.rstrip('\n').split('\t')
    rows.append((int(a, 16), n, k, int(c), doc))
SIZE = {'D': 4, 'F': 4, 'S': 0x30, 'R': 0x20}
TY = {'D': 'Dword', 'F': 'Float', 'S': 'Surface', 'R': 'ForceRecord'}
out = []
w = out.append
w("//! The physics' tables and work variables (IDA 0x613828 .. 0x614D38; phase 7, typed): the")
w("//! race clock, the inputs, the ground kinds' records, the handling constants, the car's")
w("//! working figures, the force and contact records, the gearbox and the spec's figures.")
w("//! Named from the code's uses (2026-10-25); most names are readings, `unknown_*` runs are")
w("//! places no ported code names. `#[repr(C)]` with the image's layout, over the race block's")
w("//! bytes (`TABLES`, as `physics::car_store::CAR`) until the code reaching them by RVA from")
w("//! `race::store::asm_base()` is typed.")
w("")
w("use crate::session::{Dword, Float};")
w("")
w("/// A kind of ground (0x30 bytes, `off_613978`): what `physics::wheel_surfaces` blends")
w("/// between for each point, and `physics::apply_patches` patches (readings).")
w("#[repr(C)]")
w("pub struct Surface {")
w("    /// +0x0: its table in `surface_tables` (an address).")
w("    pub table: Dword,")
w("    /// +0x4: slide grip (`patchslidegrip`).")
w("    pub slide_grip: Float,")
w("    /// +0x8: acceleration grip (`patchaccelgrip`).")
w("    pub accel_grip: Float,")
w("    /// +0xc: brake grip (`patchbrakegrip`).")
w("    pub brake_grip: Float,")
w("    /// +0x10, +0x14: blended into the points' figures.")
w("    pub blend_10: [Float; 2],")
w("    /// +0x18: not named.")
w("    pub unknown_18: [Dword; 2],")
w("    /// +0x20: the drag (10.0 on two grasses, `patchstickygrass`).")
w("    pub drag: Float,")
w("    /// +0x24: blended into the points' figures.")
w("    pub blend_24: Float,")
w("    /// +0x28: the grip blended into `point_grips`.")
w("    pub grip: Float,")
w("    /// +0x2c: blended into the car's +0x948.")
w("    pub blend_2c: Float,")
w("}")
w("")
w("/// A force on the car (0x20 bytes): `physics::apply_force` takes it, the step clears its")
w("/// first six dwords (`physics::clear_records`).")
w("#[repr(C)]")
w("pub struct ForceRecord {")
w("    /// +0x0: the force.")
w("    pub force: [Float; 3],")
w("    /// +0xc: where it acts.")
w("    pub point: [Float; 3],")
w("    /// +0x18: not cleared, not named.")
w("    pub unknown_18: [Dword; 2],")
w("}")
w("")
w("#[repr(C)]")
w("pub struct PhysicsTables {")
at = START
def gap(a, b):
    if b > a:
        w(f"    /// +{a-START:#x} (0x{a:X}): not named.")
        n = (b - a) // 4
        w(f"    pub unknown_{a-START:x}: [Dword; {n}]," if n > 1 else f"    pub unknown_{a-START:x}: Dword,")
fields = []
for a, n, k, c, doc in rows:
    assert a >= at, hex(a)
    gap(at, a)
    w(f"    /// +{a-START:#x} (0x{a:X}): {doc}")
    t = TY[k]
    w(f"    pub {n}: [{t}; {c}]," if c > 1 else f"    pub {n}: {t},")
    fields.append((a, n, k, c))
    at = a + SIZE[k] * c
gap(at, END)
w("}")
w("")
w(f"const _: () = assert!(core::mem::size_of::<PhysicsTables>() == 0x{END-START:X});")
w("const _: () = assert!(core::mem::size_of::<Surface>() == 0x30);")
w("const _: () = assert!(core::mem::size_of::<ForceRecord>() == 0x20);")
w("")
w("/// The tables: the race block's bytes at IDA 0x613828 seen as `PhysicsTables`")
w("/// (`TABLES.gravity.get()`). They stay there while code reaches them by RVA from")
w("/// `race::store::asm_base()` and by IDA address (`moved::place`): one copy for all three.")
w("pub struct InRaceBlock;")
w("")
w("impl core::ops::Deref for InRaceBlock {")
w("    type Target = PhysicsTables;")
w("    fn deref(&self) -> &PhysicsTables {")
w("        // SAFETY: the race block lives as long as the program and is aligned to 4 (the")
w("        // test checks); every field is an atomic or a cell, as the block's bytes were shared.")
w("        unsafe { &*(tables_addr() as *const PhysicsTables) }")
w("    }")
w("}")
w("")
w("pub static TABLES: InRaceBlock = InRaceBlock;")
w("")
w(f"/// IDA 0x{START:X}, the first byte of the piece.")
w(f"pub const START: usize = 0x{START:X};")
w(f"/// IDA 0x{END:X}, the end of the piece.")
w(f"pub const END: usize = 0x{END:X};")
w("")
w("const _: () = assert!(END - START == core::mem::size_of::<PhysicsTables>());")
w("")
w(f"/// Where IDA 0x{START:X} is now (in the race block).")
w("pub fn tables_addr() -> usize {")
w("    crate::race::store::home(START)")
w("}")
w("")
w("#[cfg(test)]")
w("mod tests {")
w("    use super::*;")
w("    use core::mem::offset_of;")
w("")
w("    #[test]")
w("    fn tables_sit_on_the_race_block() {")
w("        assert_eq!(tables_addr() % 4, 0);")
w("        assert_eq!(tables_addr(), crate::framework::game_addr(crate::framework::rva(START)));")
for a, n, k, c in fields:
    if n in ('gravity', 'gear_ratios', 'forces', 'contacts', 'pace_slot', 'surfaces', 'scene_clock', 'gear_count'):
        w(f"        assert_eq!(offset_of!(PhysicsTables, {n}), 0x{a:X} - START);")
w("        assert_eq!(TABLES.gravity.get(), 9.8);")
w("        assert_eq!(TABLES.surfaces[1].slide_grip.get(), 1.0);")
w("        assert_eq!(TABLES.contacts[0].force[0].addr(), crate::framework::game_addr(crate::framework::rva(0x614A50)));")
w("    }")
w("}")
open(OUT, 'w').write('\n'.join(out) + '\n')
# address map for the converter
import json
amap = {}
for a, n, k, c in fields:
    for i in range(c):
        if k in 'DF':
            amap[a + 4 * i] = (f'{n}[{i}]' if c > 1 else n, 'Dword' if k == 'D' else 'Float')
        elif k == 'S':
            base = a + 0x30 * i
            for off, f, t in [(0, 'table', 'Dword'), (4, 'slide_grip', 'Float'), (8, 'accel_grip', 'Float'), (0xC, 'brake_grip', 'Float'), (0x10, 'blend_10[0]', 'Float'), (0x14, 'blend_10[1]', 'Float'), (0x18, 'unknown_18[0]', 'Dword'), (0x1C, 'unknown_18[1]', 'Dword'), (0x20, 'drag', 'Float'), (0x24, 'blend_24', 'Float'), (0x28, 'grip', 'Float'), (0x2C, 'blend_2c', 'Float')]:
                amap[base + off] = (f'{n}[{i}].{f}', t)
        else:
            base = a + 0x20 * i
            for j in range(3):
                amap[base + 4 * j] = (f'{n}[{i}].force[{j}]', 'Float')
                amap[base + 0xC + 4 * j] = (f'{n}[{i}].point[{j}]', 'Float')
            amap[base + 0x18] = (f'{n}[{i}].unknown_18[0]', 'Dword')
            amap[base + 0x1C] = (f'{n}[{i}].unknown_18[1]', 'Dword')
json.dump({hex(k): v for k, v in amap.items()}, open(OUT + '.map.json', 'w'))
print(len(fields), len(amap))
