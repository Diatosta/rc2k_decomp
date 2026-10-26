//! More of `race::loadlevel_init`'s steps (IDA, 2026-10-16): "load prefs" (the stage's
//! `localdat`, 0x206C bytes at 0x608CB4, which hold `g_textureKinds` at +0x100 and the
//! lists below), "skidmarks init", "render3d loadlightsdat" and "getmatcrash".

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

pub const STAGE_RVA: usize = rva(0x623850); // dword_623850: the stage's number
pub const PREFS_RVA: usize = rva(0x608CB4); // the stage's localdat
pub const PREFS_LEN: u32 = 0x206C;
/// `files\levels\levelxx\localdat`, `srclvls\levelxx\localdat` (the xx the course's).
const PREFS_PATHS: [usize; 2] = [rva(0x60B05C), rva(0x60B043)];
const KINDS_RVA: usize = rva(0x608DB4); // g_textureKinds
const SWAPPED_RVA: usize = rva(0x60A67C); // dword_60A67C: 0x40 kinds (>> 4) swapped in
const TWEAKED_RVA: usize = rva(0x60A564); // dword_60A564 .. dword_60A56C
const FLAG_RVA: usize = rva(0x71C800); // dword_71C800
const CLEARED_RVA: usize = rva(0x609344); // qword_609344
const SKID_OBJECTS: [usize; 2] = [0x70F3EC, 0x7103CC]; // byte_70F3EC, unk_7103CC
const SKID_MARKS_RVA: usize = rva(0x643AE8); // word_643AE8: 0x400 of 8 bytes
const SKID_COUNT_RVA: usize = rva(0x645B10); // dword_645B10
pub const LIGHTS_RVA: usize = rva(0x6077E8); // the lights' data, 0x1320 bytes
const LIGHTS_PATH_RVA: usize = rva(0x608B08); // "files\s\data\ldata"
pub const LIGHTS_LEN: u32 = 0x1320;
pub const LIGHT_RECORDS_RVA: usize = rva(0x5C906C); // 0x22 records of 0x134
pub const LIGHT_RECORDS: u32 = rc2k_core::race::stage_objects::LIGHT_RECORDS as u32;
const CRASH_ANGLES_RVA: usize = rva(0x71A998); // three dwords, angles << 16 (y, x, z)
pub const CRASH_TURNS_RVA: usize = rva(0x63503C); // dword_63503C .. dword_635044
pub const CRASH_MATRIX_RVA: usize = rva(0x71A08C); // dword_71A08C

unsafe fn kind(n: u32) -> u32 {
    unsafe { game_read(KINDS_RVA + 4 * n as usize) }
}

unsafe fn swapped(n: u32) -> u32 {
    unsafe { game_read(SWAPPED_RVA + 4 * n as usize) }
}

/// `race::swap_stage_texture` (0x439B03): for the stages of the core's `TEXTURE_SWAPS`,
/// the first of the first 0xFF texture kinds that is the stage's old kind becomes its new
/// one; then, unless the new kind is in the swapped list already, it goes >> 4 into the
/// list's first free place of 0x40 (the test is made on the kind before the shift:
/// section 16). Leaves eax and ebx the pair last tried (ebx shifted once it was), ecx
/// where each search stopped.
pub unsafe fn swap_stage_texture(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::stage::swap_texture`.
    use rc2k_core::race::stage::{SWAP_KINDS, SWAPPED_PLACES, Swap, swap_texture};
    let mut g = *entry;
    unsafe {
        let stage: u32 = game_read(STAGE_RVA);
        let ((old, new), swap) = swap_texture(stage, |n| kind(n), |n| swapped(n));
        g.eax = old;
        g.ebx = new;
        let swapped_kind = match swap {
            Swap::NotThisStage => return g,
            Swap::NotFound => {
                g.ecx = SWAP_KINDS;
                return g;
            }
            Swap::Listed { kind, at } => {
                g.ecx = at;
                kind
            }
            Swap::Added { kind, at } => {
                g.ebx = new >> 4;
                g.ecx = at;
                game_write(SWAPPED_RVA + 4 * at as usize, g.ebx);
                kind
            }
            Swap::Full { kind } => {
                g.ebx = new >> 4;
                g.ecx = SWAPPED_PLACES;
                kind
            }
        };
        game_write(KINDS_RVA + 4 * swapped_kind as usize, new);
    }
    g
}

/// `race::tweak_stage` (0x439AAB): for the stages 0x15, 0x3E and 0x36, `dword_60A564` ..
/// `dword_60A56C` 0xFF.
pub unsafe fn tweak_stage() {
    unsafe {
        // Converted: `rc2k_core::race::stage::stage_tweaked`.
        if rc2k_core::race::stage::stage_tweaked(game_read(STAGE_RVA)) {
            game_write(TWEAKED_RVA, [0xFFu32; 3]);
        }
    }
}

/// `race::clear_prefs_qword` (0x465381): with `dword_71C800` set, `qword_609344` 0.
pub unsafe fn clear_prefs_qword() {
    unsafe {
        if game_read::<u32>(FLAG_RVA) != 0 {
            game_write(CLEARED_RVA, [0u32; 2]);
        }
    }
}

/// A call of `files::read_whole_file` (eax the path, ebx the buffer, ecx the count; inside
/// `pusha`), as part 1 of `level_init::Part`.
fn read(
    g: &Regs,
    path: usize,
    buffer: usize,
    count: u32,
    part: crate::race::level_init::Part,
) -> Result<(), Refused> {
    let mut r = Regs {
        eax: game_addr(path) as u32,
        ebx: game_addr(buffer) as u32,
        ecx: count,
        ..*g
    };
    part(1, &mut r)
}

/// `race::load_prefs` (0x445698): `files\levels\levelxx\localdat`, then
/// `srclvls\levelxx\localdat`, each 0x206C bytes at most into 0x608CB4; then
/// `swap_stage_texture`, `tweak_stage`, `clear_prefs_qword`. Leaves the registers as
/// `swap_stage_texture` does.
pub unsafe fn load_prefs(
    entry: &Regs,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    for path in PREFS_PATHS {
        read(entry, path, PREFS_RVA, PREFS_LEN, part)?;
    }
    unsafe {
        let g = swap_stage_texture(entry);
        tweak_stage();
        clear_prefs_qword();
        Ok(g)
    }
}

/// `race::reset_skid_slots` (0x4585BA): the object's (esi) four dwords at +0xF48 -1.
pub unsafe fn reset_skid_slots(object: u32) {
    unsafe { std::ptr::write_bytes((object + 0xF48) as *mut u8, 0xFF, 16) };
}

/// `race::init_skidmarks` (0x45857D): the two skid objects' slots, the 0x400 marks' first
/// two words -1, `dword_645B10` 0. Leaves esi the second object, edi past the marks, ecx 0.
pub unsafe fn init_skidmarks(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        for object in SKID_OBJECTS {
            g.esi = game_addr(rva(object)) as u32;
            reset_skid_slots(g.esi);
        }
        g.edi = game_addr(SKID_MARKS_RVA) as u32;
        for _ in 0..0x400 {
            (g.edi as *mut u32).write_unaligned(u32::MAX);
            g.edi += 8;
        }
        g.ecx = 0;
        game_write(SKID_COUNT_RVA, 0u32);
    }
    g
}

/// `race::copy_lights_data` (0x438B03): 0x22 records of 0x90 bytes from the lights' data
/// to +0xA4 of the 0x134-byte records at 0x5C906C. Leaves esi and edi past them, edx 0x22.
pub unsafe fn copy_lights_data(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::stage_objects::lay_lights`.
    use rc2k_core::race::stage_objects::{LIGHT_DATA, LIGHT_RECORD, lay_lights};
    let mut g = *entry;
    let (data, records) = (game_addr(LIGHTS_RVA), game_addr(LIGHT_RECORDS_RVA));
    unsafe {
        lay_lights(
            std::slice::from_raw_parts(data as *const u8, LIGHTS_LEN as usize),
            std::slice::from_raw_parts_mut(
                records as *mut u8,
                LIGHT_RECORDS as usize * LIGHT_RECORD,
            ),
        );
    }
    g.esi = (data + LIGHT_RECORDS as usize * LIGHT_DATA) as u32;
    g.edi = (records + LIGHT_RECORDS as usize * LIGHT_RECORD) as u32;
    g.edx = LIGHT_RECORDS;
    g
}

/// `race::load_lights_data` (0x438AE5): `files\s\data\ldata` (0x1320 bytes at most) into
/// the lights' data, then `copy_lights_data`.
pub unsafe fn load_lights_data(
    entry: &Regs,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    read(entry, LIGHTS_PATH_RVA, LIGHTS_RVA, LIGHTS_LEN, part)?;
    Ok(unsafe { copy_lights_data(entry) })
}

/// `race::get_mat_crash` (0x43D43E): the three angles (`dword_71A99C`, `dword_71A998`,
/// `dword_71A9A0`, each >> 16) kept at `dword_63503C` .., the x, y and z turns made of
/// them, Z x X into the lit matrix, that x Y into `dword_71A08C`. Leaves eax and ebx the z
/// turn's sine and cosine, ecx and edx 0, esi the lit matrix, edi the y turn's, ebp the
/// crash matrix.
pub unsafe fn get_mat_crash(entry: &Regs) -> Regs {
    use crate::scene::matrix::{self, LIT_RVA, TURN_X_RVA, TURN_Y_RVA};
    use crate::scene::view_matrix::TURN_Z_RVA;
    let mut g = *entry;
    unsafe {
        // Converted: `rc2k_core::race::stage_objects::crash_turns` (the matrices the
        // scene's).
        let turns = rc2k_core::race::stage_objects::crash_turns(game_read(CRASH_ANGLES_RVA));
        game_write(CRASH_TURNS_RVA, turns);
        matrix::set_turn_x(turns[0]);
        matrix::set_turn_y(turns[1]);
        matrix::set_turn_z_16(turns[2]).apply(&mut g);
        g.edx = turns[2];
        let a = |r: usize| game_addr(r);
        matrix::multiply_matrices(a(TURN_Z_RVA), a(TURN_X_RVA), a(LIT_RVA)).apply(&mut g);
        matrix::multiply_matrices(a(LIT_RVA), a(TURN_Y_RVA), a(CRASH_MATRIX_RVA)).apply(&mut g);
        g.esi = a(LIT_RVA) as u32;
        g.edi = a(TURN_Y_RVA) as u32;
        g.ebp = a(CRASH_MATRIX_RVA) as u32;
    }
    g
}

/// The skid objects' slots and marks, what `init_skidmarks` writes.
pub fn skid_regions() -> Vec<crate::service::harness::Region> {
    let mut r: Vec<_> = SKID_OBJECTS
        .iter()
        .map(|&o| (game_addr(rva(o)) as u32 + 0xF48, 16))
        .collect();
    r.extend([
        (game_addr(SKID_MARKS_RVA) as u32, 0x2000),
        crate::service::harness::at(SKID_COUNT_RVA),
    ]);
    r
}

/// What a step left in the registers, as a `Left` (every general register but esp).
pub fn left_of(g: &Regs) -> crate::screens::harness::Left {
    use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI};
    crate::screens::harness::Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
        .with(EDI, g.edi)
        .with(EBP, g.ebp)
}
