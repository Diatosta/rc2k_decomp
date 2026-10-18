//! `race::draw_view_scene` (0x4557AB): a view's 3D scene. With `unk_71AF80` set (a reading:
//! the view covered) only `race::draw_view_tint`. Else `dword_635014`, `dword_635018` 0 and
//! in turn, with the race's sound tick (`sound::race_sound_tick`, eax 0x14 .. 0x1A) between
//! the groups: the car lights' modes, the point records cleared, the fog, the camera's view,
//! the sky and the draw batch flushed, the view's light; the lamps reset, `dword_63DB6C` 0,
//! the course's view prepared, its visible faces, the EAX environment, the lamps gathered and
//! wanted; the sort entries sorted and drawn, the race's slots; the cars' shadows, the
//! skidmarks, the car lights; the depth entries sorted and drawn, the batch flushed, the
//! view's value stored.
//!
//! In:  nothing
//! Out: the registers as the last call leaves them; no flags read
//!
//! Callers: the views of `sub_45429B` (five) and `race::draw_mirror_view`. Its calls through
//! `part` by place (`CALLEES`); in the game the sound tick's and the flush's Rust versions,
//! the others (all hooked) through the game. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose 31 calls go to stand-ins, against the Rust version, covered and
//! not; the calls and their registers compared.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::mud::{copy, run_both, test_part};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_view_scene", rva(0x4557AB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 24] = [
    0x421BA9, // 0 sound::race_sound_tick
    0x455B70, // 1 race::car_light_modes
    0x4500FF, // 2 race::clear_point_records
    0x44FBD9, // 3 race::set_up_fog
    0x438325, // 4 race::camera_view
    0x455540, // 5 race::draw_sky
    0x45FD1D, // 6 race::flush_draw_batch
    0x455896, // 7 race::view_light
    0x45487A, // 8 race::reset_lamps
    0x4548A8, // 9 race::prepare_course_view
    0x454B71, // 10 race::gather_visible_faces
    0x45C7EE, // 11 race::eax_environment
    0x454E08, // 12 race::gather_lamps
    0x4536D7, // 13 race::lamps_wanted
    0x454F62, // 14 race::sort_sort_entries
    0x454EAB, // 15 race::draw_sorted_entries
    0x4621E5, // 16 race::draw_race_slots
    0x45754C, // 17 race::cast_car_shadows
    0x4585E3, // 18 race::draw_skidmarks_if_on
    0x455CFF, // 19 race::car_lights
    0x454ED4, // 20 race::sort_depth_entries
    0x454E84, // 21 race::draw_depth_entries
    0x455528, // 22 race::store_view_value
    0x456092, // 23 race::draw_view_tint
];

const COVERED_RVA: usize = rva(0x71AF80); // unk_71AF80
const CLEARED: [usize; 2] = [rva(0x635014), rva(0x635018)]; // dword_635014, dword_635018
const LAMPS_RVA: usize = rva(0x63DB6C); // dword_63DB6C

/// A call by place (`CALLEES`); place 0 with eax its tick number.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The steps after the tick numbered by the first: (tick, places).
const GROUPS: [(u32, &[u32]); 7] = [
    (0x14, &[1, 2, 3, 4, 5, 6, 7]),
    (0x15, &[8, 9, 10, 11, 12, 13]),
    (0x16, &[14, 15, 16]),
    (0x17, &[17, 18, 19]),
    (0x18, &[20]),
    (0x19, &[21]),
    (0x1A, &[6, 22]),
];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_view_scene(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        if game_read::<u32>(COVERED_RVA) != 0 {
            part(23, &mut r)?;
            return Ok(r);
        }
        for at in CLEARED {
            game_write(at, 0u32);
        }
        for (tick, places) in GROUPS {
            r.eax = tick;
            part(0, &mut r)?;
            for &place in places {
                part(place, &mut r)?;
                if place == 8 {
                    game_write(LAMPS_RVA, 0u32);
                }
            }
        }
    }
    Ok(r)
}

/// In the game: the sound tick's and the flush's Rust versions, the others' through the
/// game (their hooks).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 => crate::sound::race_sound_tick::race_sound_tick_in(
            r,
            &mut crate::sound::turn_parts::game_part,
        ),
        6 => unsafe { crate::race::flush_draw_batch::flush_draw_batch_in_game(r) },
        _ => {
            unsafe { call_game(r, game_addr(rva(CALLEES[place as usize]))) };
            Ok(())
        }
    }
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn draw_view_scene_in_game(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { draw_view_scene(r, &mut game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_view_scene(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

const CALLS: [usize; 31] = [
    0x26, 0x2B, 0x30, 0x35, 0x3A, 0x3F, 0x44, 0x49, 0x53, 0x58, 0x67, 0x6C, 0x71, 0x76, 0x7B, 0x85,
    0x8A, 0x8F, 0x94, 0x9E, 0xA3, 0xA8, 0xAD, 0xB7, 0xBC, 0xC6, 0xCB, 0xD5, 0xDA, 0xDF, 0xE5,
];

/// `CALLEES` with their places, for a copy's stand-ins.
static PLACES: std::sync::LazyLock<Vec<(usize, u32)>> = std::sync::LazyLock::new(|| {
    CALLEES
        .iter()
        .enumerate()
        .map(|(p, &a)| (a, p as u32))
        .collect()
});

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword ptr unk_71AF80, 0
    let copy = match copy(
        0x4557AB,
        0xEB,
        &[0x83, 0x3D, 0x80, 0xAF, 0x71, 0x00, 0x00],
        &CALLS,
        PLACES.as_slice(),
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        at(COVERED_RVA),
        at(CLEARED[0]),
        at(CLEARED[1]),
        at(LAMPS_RVA),
    ];
    let saved = capture(&regions);
    for covered in [0u32, 1, 0xFFFF_FFFF] {
        run_both(
            &mut t,
            &format!("covered {covered:#x}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(COVERED_RVA, covered);
                game_write(CLEARED[0], 0x1234u32);
                game_write(CLEARED[1], 0x5678u32);
                game_write(LAMPS_RVA, 0x9ABCu32);
            },
            &|r| unsafe { draw_view_scene(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
