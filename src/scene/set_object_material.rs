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

use crate::errors::BadInput;
use crate::framework::{
    GameHook, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::next_random::{STATE_RVA as RANDOM_RVA, next_random};
use crate::math::x87;
use crate::scene::polygon::{dword, put};
use crate::service::floats::{Agreement, Floats, at_test_precisions, compare_memory, note_drift};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::set_object_material",
    rva(0x45170E),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

/// The byte +0x20 of the record of the object's kind, or None for an object's number
/// past the tables.
unsafe fn way() -> Option<u32> {
    unsafe {
        let object: u32 = game_read(OBJECT_RVA);
        if object >= OBJECTS {
            return None;
        }
        let kind: u32 = game_read(KINDS_RVA + object as usize * 4);
        let material =
            game_addr(MATERIALS_RVA).wrapping_add((kind as usize).wrapping_mul(MATERIAL_LEN));
        Some(((material + 0x20) as *const u8).read() as u32)
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

/// The fraction (`flt_623DD4`) of the regions, the one float place.
fn floats() -> [(Region, crate::service::floats::Tolerance); 1] {
    [(at(MOVE_FRACTION_RVA), crate::service::floats::FLOAT)]
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = [
        MOVE_BY_RVA,
        MOVE_FRACTION_RVA,
        COLOUR_RVA,
        WAY_RVA,
        FIRST_FACE_RVA,
        FACE_COUNT_RVA,
        LAST_RVA,
    ]
    .map(at)
    .to_vec();
    regions.push((game_addr(PHASE_WORK_RVA) as u32, 12));
    regions.push((game_addr(LENGTHS_RVA) as u32, 36));
    // The state only for an object that has one: `dword_5CCCBC` goes on naming a
    // stage's memory after the stage has given it back (2026-10-13: reading it for
    // a menu's object ended the game).
    // The random state too only then: no other object draws, and other threads do (a
    // capture of it would compare, and its restore undo, their draws).
    if let (Some(4), Some(state)) = (unsafe { way() }, unsafe { state() }) {
        regions.push((state as u32, 16));
        regions.push(at(RANDOM_RVA));
    }
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = unsafe { set_object_material_plain() } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { set_object_material_plain() }.map(|()| capture(&regions));
    restore(&regions, &before);
    unsafe { call_original(r) };
    let Ok(mine) = mine else {
        return;
    };
    let theirs = capture(&regions);
    let places = floats();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    match compare_memory(&regions, &theirs, &mine, &floats) {
        Agreement::Same => {}
        Agreement::Drift(n) => note_drift(HOOK.name(), || format!("{n} places")),
        Agreement::Differ => HOOK.mismatch(|| {
            format!(
                "object {:#x}: before {before:x?}; original {theirs:x?}; rust {mine:x?}",
                unsafe { game_read::<u32>(OBJECT_RVA) }
            )
        }),
    }
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const OBJECT: u32 = 0xF0;
    let slot = OBJECT as usize * 4;
    // The test's object, an unused one, is given the kind of the first real object for
    // the time of the test: that kind's record is there to be read and changed.
    let kinds: [u32; 0x100] = unsafe { game_read(KINDS_RVA) };
    // The records run from `unk_97DF30` up to `g_faces`: room for 0x4B0 kinds.
    let Some(&kind) = kinds[1..].iter().find(|&&kind| kind < 0x4B0) else {
        return t.skip(&format!(
            "no object is loaded whose kind's record the test could use (the first kinds: {:x?})",
            &kinds[..4]
        ));
    };
    let way_at =
        game_addr(MATERIALS_RVA).wrapping_add((kind as usize).wrapping_mul(MATERIAL_LEN)) + 0x20;
    let mut inputs: Vec<Region> = [
        OBJECT_RVA,
        FIRST_FACES_RVA + slot,
        FACE_COUNTS_RVA + slot,
        STATES_RVA,
        STATE_NUMBER_RVA,
        CLOCK_RVA,
        KINDS_RVA + slot,
    ]
    .map(at)
    .to_vec();
    inputs.push((game_addr(FIRST_POINT_RVA + slot) as u32, 8));
    inputs.push((way_at as u32, 1));
    let saved_inputs = capture(&inputs);
    unsafe { game_write(KINDS_RVA + slot, kind) };
    // The test's own table of states: the one in use is number 2.
    let mut states = [0u32; 0x40];
    unsafe {
        game_write(STATES_RVA, states.as_mut_ptr() as u32);
        game_write(STATE_NUMBER_RVA, 2u32);
        // As a moving object, so that its state is among the regions.
        game_write(OBJECT_RVA, OBJECT);
        (way_at as *mut u8).write(4);
    }
    let regions = regions();
    let saved = capture(&regions);
    let state_at = states.as_mut_ptr() as usize + 0x80 + 0x30;
    let places = floats();
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        let mut cases = 0u32;
        for way in [0u8, 4, 5] {
            for (last, time, length, phase) in [
                (1000u32, 0u32, 300u32, 0u32),
                (1000, 250, 300, 1),
                (1000, 299, 300, 2),
                (1000, 100, 300, 3),
                (1000, 0, 0, 3),
                (0, 5000, 7, 1),
                (990, 0x7FFF_FFF0, 0x7FFF_FFFF, 3),
            ] {
                for clock in [1000u32, 1030, 1200, 999] {
                    cases += 1;
                    let seed = 0x1357_9BDFu32.wrapping_mul(cases);
                    let run = |rust: bool| unsafe {
                        restore(&regions, &vec![0xAA; saved.len()]);
                        game_write(RANDOM_RVA, seed);
                        game_write(OBJECT_RVA, OBJECT);
                        (way_at as *mut u8).write(way);
                        game_write(FIRST_POINT_RVA + slot, [0x100u32, 0x100 + cases]);
                        game_write(FIRST_FACES_RVA + slot, 0x4000 + cases);
                        game_write(FACE_COUNTS_RVA + slot, cases);
                        game_write(CLOCK_RVA, clock);
                        put(state_at, last);
                        put(state_at + 4, time);
                        put(state_at + 8, length);
                        put(state_at + 12, phase);
                        let done = if rust {
                            set_object_material_plain().is_ok()
                        } else {
                            call_original(&mut Regs::default());
                            true
                        };
                        (done, capture(&regions), dword(state_at))
                    };
                    let (theirs, mine) = (run(false), run(true));
                    let same = theirs.0 == mine.0
                        && theirs.2 == mine.2
                        && compare_memory(&regions, &theirs.1, &mine.1, held) != Agreement::Differ;
                    t.check(same, || {
                    format!(
                        "control word {word:#x}, way {way}, state {last} {time} {length} {phase}, clock {clock}, seed {seed:#x}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
                }
            }
        }
    });
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
