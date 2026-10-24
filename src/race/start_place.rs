//! Race init's first step (`sub_446463` -> `sub_4464AD`, IDA 2026-10-17): the current car
//! (the work copy `g_car`, 0x719FD8) set at its start on the best line. Here its steps:
//! the place on the line between two checkpoints (`race::place_car_on_line`), the nearest
//! checkpoint to the car (`race::nearest_checkpoint` and its two helpers), and the car's
//! matrix undone (`race::invert_car_matrix`).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::best_line::{
    CHECK_COUNT_RVA, CHECK_FACES_RVA, FACES_RVA, MOST_CHECKS, POINTS_RVA, SCRATCH_RVA,
};

pub const FROM_X_RVA: usize = rva(0x639824); // dword_639824: the place measured from
pub const FROM_Z_RVA: usize = rva(0x63982C); // dword_63982C
pub const NEAREST_DISTANCE_RVA: usize = rva(0x614C68); // g_nearestCheckpointDistance
const SCALE_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0
/// The work car's place (+0x134 .. +0x13C) and heading (+0x9C0).
pub const CAR_X_RVA: usize = rva(0x71A10C); // dword_71A10C
pub const CAR_Y_RVA: usize = rva(0x71A110); // dword_71A110
pub const CAR_Z_RVA: usize = rva(0x71A114); // dword_71A114
pub const CAR_HEADING_RVA: usize = rva(0x71A998); // dword_71A998
/// The work car's matrix (+0xB4) and its inverse (+0x9CC).
pub const CAR_MATRIX_RVA: usize = crate::race::stage_prefs::CRASH_MATRIX_RVA; // g_crashMatrix
pub const CAR_INVERSE_RVA: usize = rva(0x71A9A4); // g_carInverseMatrix
pub const LINE_PLACE_RVA: usize = rva(0x6365E4); // dword_6365E4: the checkpoint
pub const LINE_FRACTION_RVA: usize = rva(0x6365EC); // dword_6365EC: on to the next, / 65536
pub const DROP_RVA: usize = rva(0x613FC4); // dword_613FC4
pub const LINE_HEADINGS_RVA: usize = rva(0x9ED808); // dword_9ED808 (g_checkpointAngles + 8)
pub const PLACE_FACE_RVA: usize = rva(0x5CCD28); // dword_5CCD28
pub const PLACE_X_RVA: usize = rva(0x5CCDA0); // dword_5CCDA0
pub const PLACE_Y_RVA: usize = rva(0x5CCDC0); // dword_5CCDC0
pub const PLACE_Z_RVA: usize = rva(0x5CCDE0); // dword_5CCDE0
pub const PLACE_HEADING_RVA: usize = rva(0x5CCD60); // dword_5CCD60

fn count() -> Result<u32, Refused> {
    let n: u32 = unsafe { game_read(CHECK_COUNT_RVA) };
    if n > MOST_CHECKS {
        return Err(BadInput("more checkpoints than the lists hold").into());
    }
    Ok(n)
}

/// Checkpoint n's face (number, address) and its first point's (+4) number and address.
unsafe fn check_point(n: u32) -> (u32, u32, u32, u32) {
    unsafe {
        let face: u32 = game_read(CHECK_FACES_RVA + 4 * n as usize);
        let address = (face << 6).wrapping_add(game_read(FACES_RVA));
        let point = *((address + 4) as *const u32);
        let place = (point.wrapping_mul(3) << 2).wrapping_add(game_read(POINTS_RVA));
        (face, address, point, place)
    }
}

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// `race::place_car_on_line` (0x44666F): checkpoint `dword_6365E4` (held below
/// `g_checkpointCount`) and the next (held so too); their faces' first points and their
/// headings (`dword_9ED808`) blended by `dword_6365EC` / 65536 into `dword_5CCDA0` /
/// `C0` (+ `dword_613FC4`) / `E0` and `dword_5CCD60`, then into the work car's place and
/// heading; the first face's number in `dword_5CCD28`. The registers it leaves are read
/// by none of its callers (conversion phase 3, 2026-10-24). Refuses no checkpoints (the
/// original reads entry -1) or more than the lists.
pub unsafe fn place_car_on_line() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::{line_neighbours, lerp}`.
    use rc2k_core::race::checkpoints::{lerp, line_neighbours};
    let count = count()?;
    unsafe {
        let (c, n) = line_neighbours(game_read(LINE_PLACE_RVA), count)?;
        let t: u32 = game_read(LINE_FRACTION_RVA);
        let (face, _, _, a) = check_point(c);
        game_write(PLACE_FACE_RVA, face);
        let (_, _, _, b) = check_point(n);
        let (x, _) = lerp(get(a), get(b), t);
        let (y, _) = lerp(get(a + 4), get(b + 4), t);
        let y = y.wrapping_add(game_read(DROP_RVA));
        let (z, _) = lerp(get(a + 8), get(b + 8), t);
        let heading_of = |k: u32| -> u32 { game_read(LINE_HEADINGS_RVA + 4 * k as usize) };
        let (heading, _) = lerp(heading_of(c), heading_of(n), t);
        game_write(PLACE_X_RVA, x);
        game_write(PLACE_Y_RVA, y);
        game_write(PLACE_Z_RVA, z);
        game_write(PLACE_HEADING_RVA, heading);
        game_write(CAR_X_RVA, x);
        game_write(CAR_Z_RVA, z);
        game_write(CAR_HEADING_RVA, heading);
        game_write(CAR_Y_RVA, y);
        Ok(())
    }
}

/// `race::distance_to_checkpoint` (0x44141D): with `c` a checkpoint (0 .. count - 1, by
/// sign tests), `dword_62400C` the distance in x and z (`x87::int_hypot`) from
/// (`dword_639824`, `dword_63982C`) to its face's first point; else -1. Refuses a
/// checkpoint past the list.
pub unsafe fn distance_to_checkpoint(c: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::checkpoints::on_line`; the distance the x87's.
    let count: u32 = unsafe { game_read(CHECK_COUNT_RVA) };
    if !rc2k_core::race::checkpoints::on_line(c, count) {
        unsafe { game_write(SCRATCH_RVA, u32::MAX) };
        return Ok(());
    }
    if c >= MOST_CHECKS {
        return Err(BadInput("a checkpoint past the list").into());
    }
    unsafe {
        let (_, _, _, place) = check_point(c);
        let dx = get(place).wrapping_sub(game_read(FROM_X_RVA)) as i32;
        let dz = get(place + 8).wrapping_sub(game_read(FROM_Z_RVA)) as i32;
        game_write(SCRATCH_RVA, x87::int_hypot(dx, dz));
    }
    Ok(())
}

/// The nearest of a stretch of checkpoints and its distance (ebp and edi in the original).
pub struct Nearest {
    pub checkpoint: u32,
    pub distance: u32,
}

/// `race::nearest_checkpoint_in` (0x4413F8): checkpoints `from` up to `to` (once at
/// least, `jb`) measured by `distance_to_checkpoint`; the nearest (unsigned, the later of
/// equals) and its distance (-1 and 0 for none). Refuses more than 0x1418 measures or
/// checkpoints, before anything.
pub unsafe fn nearest_checkpoint_in(from: u32, to: u32) -> Result<Nearest, Refused> {
    count()?;
    let steps = if from < to { to - from } else { 1 };
    if steps > MOST_CHECKS {
        return Err(BadInput("more measures than checkpoints").into());
    }
    // Converted: `rc2k_core::race::checkpoints::Nearest`.
    let mut nearest = rc2k_core::race::checkpoints::Nearest::default();
    let mut c = from;
    loop {
        unsafe { distance_to_checkpoint(c) }?;
        nearest.consider(c, unsafe { game_read(SCRATCH_RVA) });
        c = c.wrapping_add(1);
        if c >= to {
            return Ok(Nearest {
                checkpoint: nearest.checkpoint,
                distance: nearest.distance,
            });
        }
    }
}

/// `race::nearest_checkpoint` (0x44134F): the work car's x and z into `dword_639824` /
/// `dword_63982C`; with checkpoints, the nearest of them all (`nearest_checkpoint_in` 0 ..
/// count), its distance in `dword_62400C` and / 65536 in `flt_614C68`; else 0. The other
/// registers it leaves are read by none of its callers. Refuses more checkpoints than the
/// lists, before anything.
pub unsafe fn nearest_checkpoint() -> Result<u32, Refused> {
    let count = count()?;
    unsafe {
        game_write(FROM_X_RVA, game_read::<u32>(CAR_X_RVA));
        game_write(FROM_Z_RVA, game_read::<u32>(CAR_Z_RVA));
        if count == 0 {
            return Ok(0);
        }
        let near = nearest_checkpoint_in(0, count)?;
        game_write(SCRATCH_RVA, near.distance);
        // Converted: `rc2k_core::race::checkpoints::distance_units`.
        let units = rc2k_core::race::checkpoints::distance_units(
            near.distance,
            f32::from_bits(game_read(SCALE_RVA)),
        );
        game_write(NEAREST_DISTANCE_RVA, units.to_bits());
        Ok(near.checkpoint)
    }
}

/// `race::invert_car_matrix` (0x43F16C): `scene::invert_turn` of the work car's matrix
/// (`g_crashMatrix`, +0xB4) into its inverse (+0x9CC, 0x71A9A4), every register kept.
pub unsafe fn invert_car_matrix() {
    unsafe {
        crate::scene::view_matrix::invert_turn(
            game_addr(CAR_MATRIX_RVA),
            game_addr(CAR_INVERSE_RVA),
        )
    };
}
