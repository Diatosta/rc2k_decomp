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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

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
pub unsafe fn draw_view_scene(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        if game_read::<u32>(COVERED_RVA) != 0 {
            crate::race::draw_view_scene::game_part(23, &mut r)?;
            return Ok(r);
        }
        for at in CLEARED {
            game_write(at, 0u32);
        }
        for (tick, places) in GROUPS {
            r.eax = tick;
            crate::race::draw_view_scene::game_part(0, &mut r)?;
            for &place in places {
                crate::race::draw_view_scene::game_part(place, &mut r)?;
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
        0 => crate::sound::race_sound_tick::race_sound_tick_in(r),
        6 => unsafe { crate::race::flush_draw_batch::flush_draw_batch_in_game(r) },
        _ => {
            crate::calls::call(rva(CALLEES[place as usize]), r);
            Ok(())
        }
    }
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn draw_view_scene_in_game(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { draw_view_scene(r) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_view_scene(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
