//! `scene::set_object_material` (0x45170E): what an object's drawing needs from its
//! kind's record, and for a kind that moves between two shapes, how far it has moved.
//!
//! In:  nothing
//! Out: the original leaves eax, ecx, esi (and for a moving kind ebx, edx) changed; its
//!      caller goes on without them, and the Rust version keeps them
//!
//! `dword_623DD0` = 0 and the polygon's colour white; the object's kind
//! (`g_objectKinds[object]`) picks a record of 0x80 bytes at `unk_97DF30`, whose byte
//! +0x20 goes to `dword_6214DC`; then the object's first face and number of faces
//! (`dword_632B70`, `dword_632F70`) to `g_objectFirstFace` and `g_objectFaceCount`.
//! With that byte at 4 the object has two shapes, the second right after the first:
//! `dword_623DD0` = the bytes between them (the next object's first point less this
//! one's, times 12), and `sub_45079A` works out the fraction `flt_623DD4` from a
//! state of four dwords (+0x30 .. +0x3C of the record `dword_5CCCD0` of 0x40 bytes
//! at `dword_5CCCBC`: the clock when last here, the time into the phase, the phase's
//! length, the phase): the time since last here (the clock `dword_613830`, at most
//! 100) is added; while the time is not under the length, it loses the length, the
//! phase goes on by one of four, and the new length is a base (200, 400, 200, 400)
//! plus a random part of 400 (`math::next_random`, its top half times 400). The
//! fraction: 0 in phase 0, time over length in phase 1, 1 in phase 2, 1 less that
//! in phase 3. The original has a second way for when the four random parts are
//! all 0, which they never are (it sets them itself just before): left out.
//! One caller, `sub_45045D`.
//! Refuses an object's number past the tables and, for a moving kind, no state.
//! Shadow mode: the Rust version, its writes and the random numbers' state put back,
//! the original, compared.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_object_material_plain`, through
//! `rc2k_core::scene::materials` (the fraction a plain float; the x87 version stays for the
//! Rust callers in unconverted code). Shadow mode and the self-test hold the fraction to
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::next_random::next_random;
use crate::math::x87;

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const OBJECTS: u32 = 0xFF;
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
pub const MATERIALS_RVA: usize = rva(0x97DF30); // unk_97DF30
pub const MATERIAL_LEN: usize = 0x80;
const MOVE_BY_RVA: usize = rva(0x623DD0); // dword_623DD0
const MOVE_FRACTION_RVA: usize = rva(0x623DD4); // flt_623DD4
const COLOUR_RVA: usize = rva(0x6240CC); // g_polygonColour
const WAY_RVA: usize = rva(0x6214DC); // dword_6214DC
const FIRST_POINT_RVA: usize = rva(0x631F70); // g_objectFirstPoint
const FIRST_FACES_RVA: usize = rva(0x632B70); // dword_632B70
const FACE_COUNTS_RVA: usize = rva(0x632F70); // dword_632F70
const FIRST_FACE_RVA: usize = rva(0x6350BC); // g_objectFirstFace
const FACE_COUNT_RVA: usize = rva(0x6350C0); // g_objectFaceCount
const STATES_RVA: usize = rva(0x5CCCBC); // dword_5CCCBC
const STATE_NUMBER_RVA: usize = rva(0x5CCCD0); // dword_5CCCD0
const CLOCK_RVA: usize = rva(0x613830); // dword_613830
/// The state as the function works on it: the clock when last here (`dword_62DB54`),
/// then `dword_631B5C` the phase's length, `dword_631B60` the time into it,
/// `dword_631B64` the phase.
const LAST_RVA: usize = rva(0x62DB54);
const PHASE_WORK_RVA: usize = rva(0x631B5C);
/// `dword_62DB30` ..: four bases, four random parts, and a 0.
const LENGTHS_RVA: usize = rva(0x62DB30);
const LENGTHS: [u32; 9] = [200, 400, 200, 400, 400, 400, 400, 400, 0];

// ---- Rust version ----

/// The state's address, or None without one.
unsafe fn state() -> Option<usize> {
    unsafe {
        let states: u32 = game_read(STATES_RVA);
        (states != 0)
            .then(|| states.wrapping_add(game_read::<u32>(STATE_NUMBER_RVA) << 6) as usize + 0x30)
    }
}

/// `sub_45079A`.
unsafe fn step_shape(state: usize) {
    unsafe {
        let [last, mut time, mut length, mut phase]: [u32; 4] =
            (state as *const [u32; 4]).read_unaligned();
        game_write(LENGTHS_RVA, LENGTHS);
        let clock: u32 = game_read(CLOCK_RVA);
        time = time.wrapping_add(clock.wrapping_sub(last).min(100));
        while time >= length {
            time -= length;
            phase = (phase + 1) & 3;
            let part = ((next_random() as u64 * LENGTHS[4 + phase as usize] as u64) >> 32) as u32;
            length = part.wrapping_add(LENGTHS[phase as usize]);
        }
        let fraction = match phase {
            1 => x87::int_ratio(time as i32, length as i32),
            2 => 1f32.to_bits(),
            3 => x87::one_less_int_ratio(time as i32, length as i32),
            _ => 0f32.to_bits(),
        };
        game_write(MOVE_FRACTION_RVA, fraction);
        game_write(LAST_RVA, clock);
        game_write(PHASE_WORK_RVA, [length, time, phase]);
        (state as *mut [u32; 4]).write_unaligned([clock, time, length, phase]);
    }
}

pub unsafe fn set_object_material() -> Result<(), BadInput> {
    unsafe {
        let object: u32 = game_read(OBJECT_RVA);
        if object >= OBJECTS {
            return Err(BadInput("an object's number past the tables"));
        }
        let slot = object as usize * 4;
        let kind: u32 = game_read(KINDS_RVA + slot);
        let material =
            game_addr(MATERIALS_RVA).wrapping_add((kind as usize).wrapping_mul(MATERIAL_LEN));
        let way = ((material + 0x20) as *const u8).read() as u32;
        let state = state();
        if way == 4 && state.is_none() {
            return Err(BadInput("a moving object without its state"));
        }
        game_write(MOVE_BY_RVA, 0u32);
        game_write(COLOUR_RVA, 0x00FF_FFFFu32);
        game_write(WAY_RVA, way);
        if let (4, Some(state)) = (way, state) {
            let points: [u32; 2] = game_read(FIRST_POINT_RVA + slot);
            game_write(
                MOVE_BY_RVA,
                points[1].wrapping_sub(points[0]).wrapping_mul(12),
            );
            step_shape(state);
        }
        game_write(FIRST_FACE_RVA, game_read::<u32>(FIRST_FACES_RVA + slot));
        game_write(FACE_COUNT_RVA, game_read::<u32>(FACE_COUNTS_RVA + slot));
        Ok(())
    }
}

/// The converted `set_object_material` (plain floats): the hook's. The x87 version above
/// stays for the Rust callers in unconverted code.
// Converted: `rc2k_core::scene::materials`
pub unsafe fn set_object_material_plain() -> Result<(), BadInput> {
    use rc2k_core::scene::materials::{LENGTHS, MOVING, ShapeClock, move_by};
    unsafe {
        let object: u32 = game_read(OBJECT_RVA);
        if object >= OBJECTS {
            return Err(BadInput("an object's number past the tables"));
        }
        let slot = object as usize * 4;
        let kind: u32 = game_read(KINDS_RVA + slot);
        let material =
            game_addr(MATERIALS_RVA).wrapping_add((kind as usize).wrapping_mul(MATERIAL_LEN));
        let way = ((material + 0x20) as *const u8).read() as u32;
        let state = state();
        if way == MOVING && state.is_none() {
            return Err(BadInput("a moving object without its state"));
        }
        game_write(MOVE_BY_RVA, 0u32);
        game_write(COLOUR_RVA, 0x00FF_FFFFu32);
        game_write(WAY_RVA, way);
        if let (MOVING, Some(state)) = (way, state) {
            game_write(MOVE_BY_RVA, move_by(game_read(FIRST_POINT_RVA + slot)));
            let [last, time, length, phase]: [u32; 4] = (state as *const [u32; 4]).read_unaligned();
            let mut c = ShapeClock {
                last,
                time,
                length,
                phase,
            };
            game_write(LENGTHS_RVA, LENGTHS);
            let fraction = c.step(game_read(CLOCK_RVA), || next_random());
            game_write(MOVE_FRACTION_RVA, fraction.to_bits());
            game_write(LAST_RVA, c.last);
            game_write(PHASE_WORK_RVA, [c.length, c.time, c.phase]);
            (state as *mut [u32; 4]).write_unaligned([c.last, c.time, c.length, c.phase]);
        }
        game_write(FIRST_FACE_RVA, game_read::<u32>(FIRST_FACES_RVA + slot));
        game_write(FACE_COUNT_RVA, game_read::<u32>(FACE_COUNTS_RVA + slot));
        Ok(())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    if let Err(e) = unsafe { set_object_material_plain() } {
        return Err(e.into());
    }
    Ok(())
}
