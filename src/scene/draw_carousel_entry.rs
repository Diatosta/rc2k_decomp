//! `scene::draw_carousel_entry` (0x4906D5): one entry of the main menu's carousel at
//! its place in the row.
//!
//! In:  ebx = the entry, ecx = its place in the row (-1 before the front one, 0 the
//!      front one, 1 and 2 after it)
//! Out: every register preserved
//!
//! The object's variables for `scene::draw_menu_object`:
//! - x (`dword_63501C`) = the spacing `dword_75A664` times (place - 1), plus the high
//!   half of the spacing times the turn `g_carouselAngle` inverted (the turn as a
//!   fraction of 2^32);
//! - the angle about y (`dword_635040`) from the clock: its low sixteen bits times 8,
//!   as sixteen bits; the angle about x (`dword_63503C`) = 0;
//! - y and z (`dword_635020`, `dword_635024`) = `dword_75A668`, `dword_75A66C`; the
//!   front entry (place 0) is brought forward by the high half of `dword_75A670` times
//!   the turn inverted, the next one (place 1) by that of the turn itself: the one
//!   leaving the front goes back as the one arriving comes forward;
//! - `dword_623FB0` and `dword_623FB4` = 0;
//! - the entry's kind `dword_75A674[entry]`, kept in `dword_75A658`; a kind above
//!   0x258 has a size, a y and an angle about x of its own in the records of twelve
//!   bytes at `unk_75950E` (by the kind less 0x259);
//! - `g_objectNumber` = the first object of that kind (`scene::find_object_of_kind`
//!   0x422298: objects 1 .. 0xFF, else 0).
//!
//! Then `scene::draw_menu_object`.
//! Five call sites in `scene::draw_carousel`.
//! Refuses an entry of 0x100 or more and a kind past the records.
//! Shadow mode runs the original alone and compares the variables it set with the
//! Rust arithmetic.
//! Self-test: a redirected copy whose drawing notes everything set, 240 cases by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};

// ---- game data ----

pub(crate) const PLACE_RVA: usize = rva(0x63501C); // dword_63501C, dword_635020, dword_635024
pub(crate) const SIZE_RVA: usize = rva(0x635038); // dword_635038
pub(crate) const ANGLE_X_RVA: usize = rva(0x63503C); // dword_63503C
pub(crate) const ANGLE_Y_RVA: usize = rva(0x635040); // dword_635040
const CLEARED_RVA: usize = rva(0x623FB0); // dword_623FB0, dword_623FB4
const SPACING_RVA: usize = rva(0x75A664); // dword_75A664, then y, z and the way forward
pub(crate) const KINDS_RVA: usize = rva(0x75A674); // dword_75A674
pub(crate) const ENTRIES: u32 = 0x100;
pub(crate) const KIND_RVA: usize = rva(0x75A658); // dword_75A658
pub(crate) const OWN_RVA: usize = rva(0x75950E); // unk_75950E
pub(crate) const OWN: u32 = 0x88;
const ANGLE_RVA: usize = rva(0x75AA80); // g_carouselAngle
pub(crate) const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
pub(crate) const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
// g_objectKinds

// ---- Rust version ----

/// What the function sets before it draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    place: [u32; 3],
    /// None: left as it is.
    size: Option<u32>,
    angle_x: u32,
    angle_y: u32,
    kind: u32,
    object: u32,
}

pub(crate) use crate::scene::find_object_of_kind::find_object_of_kind;

/// Reads only.
unsafe fn settings(entry: u32, row: u32) -> Result<Settings, BadInput> {
    unsafe {
        if entry >= ENTRIES {
            return Err(BadInput("a carousel entry past the table"));
        }
        let [spacing, y, z, forward]: [u32; 4] = game_read(SPACING_RVA);
        let turn: u32 = game_read(ANGLE_RVA);
        let high = |a: u32, b: u32| ((a as u64 * b as u64) >> 32) as u32;
        let x = high(spacing, !turn)
            .wrapping_add((spacing as i32).wrapping_mul(row.wrapping_sub(1) as i32) as u32);
        let z = match row {
            0 => z.wrapping_sub(high(!turn, forward)),
            1 => z.wrapping_sub(high(turn, forward)),
            _ => z,
        };
        let clock: u32 = game_read(CLOCK_RVA);
        let mut set = Settings {
            place: [x, y, z],
            size: None,
            angle_x: 0,
            angle_y: ((clock as u16) << 3) as u32,
            kind: game_read(KINDS_RVA + entry as usize * 4),
            object: 0,
        };
        if set.kind as i32 > 0x258 {
            let own = set.kind - 0x259;
            if own >= OWN {
                return Err(BadInput("a kind past the records of the kinds' own places"));
            }
            let [size, y, angle]: [u32; 3] = game_read(OWN_RVA + own as usize * 12);
            set.size = Some(size);
            set.place[1] = y;
            set.angle_x = angle;
        }
        set.object = find_object_of_kind(set.kind);
        Ok(set)
    }
}

/// Carousel entry `entry` in row place `row` (ebx and ecx at the original's entry) set
/// up and drawn by `draw`.
pub unsafe fn draw_carousel_entry(
    entry: u32,
    row: u32,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let set = settings(entry, row)?;
        game_write(PLACE_RVA, set.place);
        if let Some(size) = set.size {
            game_write(SIZE_RVA, size);
        }
        game_write(ANGLE_X_RVA, set.angle_x);
        game_write(ANGLE_Y_RVA, set.angle_y);
        game_write(CLEARED_RVA, [0u32; 2]);
        game_write(KIND_RVA, set.kind);
        game_write(OBJECT_RVA, set.object);
        draw()
    }
}
