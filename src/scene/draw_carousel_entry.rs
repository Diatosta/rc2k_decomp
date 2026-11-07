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
//! Then `scene::draw_menu_object`. Five call sites in `scene::draw_carousel`. Refuses an entry
//! of 0x100 or more and a kind past the records.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::{Menus, Tables};
use crate::race::owner_shared::Shared;

// ---- game data ----

pub(crate) const ENTRIES: u32 = 0x100;
/// The kinds' own places the original allows (`MENU_LISTS.own_places` holds 73: past them it
/// reads the lists that follow in the image, as the original does).
pub(crate) const OWN: u32 = 0x88;
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

/// A kind's own place: its size, y and angle (`unk_75950E`, 12 bytes each). The original
/// reads on past the table's 73 for a kind below `OWN` (fixed: refused).
pub(crate) fn own_place(tables: &Tables, own: u32) -> Result<[u32; 3], BadInput> {
    let place = tables
        .lists
        .own_places
        .get(own as usize)
        .ok_or(BadInput("a kind past the 73 own places"))?;
    Ok([place.size.get(), place.y.get(), place.angle.get()])
}

/// Reads only.
fn settings(shared: &Shared, menus: &Menus, entry: u32, row: u32) -> Result<Settings, BadInput> {
    let scene = crate::scene::owner::scene();
    let screens = &menus.screens;
    if entry >= ENTRIES {
        return Err(BadInput("a carousel entry past the table"));
    }
    let [spacing, y, z, forward] = [
        screens.state.carousel.spacing.get(),
        screens.state.carousel.y.get(),
        screens.state.carousel.z.get(),
        screens.state.carousel.forward.get(),
    ];
    let turn: u32 = screens.state.carousel.angle.get();
    let high = |a: u32, b: u32| ((a as u64 * b as u64) >> 32) as u32;
    let x = high(spacing, !turn)
        .wrapping_add((spacing as i32).wrapping_mul(row.wrapping_sub(1) as i32) as u32);
    let z = match row {
        0 => z.wrapping_sub(high(!turn, forward)),
        1 => z.wrapping_sub(high(turn, forward)),
        _ => z,
    };
    let clock: u32 = shared.physics_tables.scene_clock.bits();
    let mut set = Settings {
        place: [x, y, z],
        size: None,
        angle_x: 0,
        angle_y: ((clock as u16) << 3) as u32,
        kind: screens.state.carousel.kinds[entry as usize].get(),
        object: 0,
    };
    if set.kind as i32 > 0x258 {
        let own = set.kind - 0x259;
        if own >= OWN {
            return Err(BadInput("a kind past the records of the kinds' own places"));
        }
        let [size, y, angle] = own_place(&menus.tables, own)?;
        set.size = Some(size);
        set.place[1] = y;
        set.angle_x = angle;
    }
    set.object = find_object_of_kind(set.kind, scene);
    Ok(set)
}

/// Carousel entry `entry` in row place `row` (ebx and ecx at the original's entry) set
/// up and drawn by `draw`.
pub fn draw_carousel_entry(
    shared: &Shared,
    menus: &Menus,
    entry: u32,
    row: u32,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let set = settings(shared, menus, entry, row)?;
    crate::session::write_bits(&drawing.object_work.place, set.place);
    if let Some(size) = set.size {
        drawing.object_work.size.set_bits(size);
    }
    drawing.object_work.angles[0].set_bits(set.angle_x);
    drawing.object_work.angles[1].set_bits(set.angle_y);
    crate::session::write_bits(&drawing.scene.view_offset, [0u32; 2]);
    menus.screens.state.carousel.kind.set(set.kind);
    shared.switches.object_number.set_bits(set.object);
    draw()
}
