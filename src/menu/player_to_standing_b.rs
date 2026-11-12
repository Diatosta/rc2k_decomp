//! `menu::player_to_standing_b` (0x46618A): a standings-B entry (edi) filled from the player
//! record esi: +4 ebx + 1, +0x14 the name's 0x20 bytes (from the record's +0x18 pointer),
//! +8 / +0xC / +0x10 its +4 / +8 / +0x90, +0x34 its +0x8C, +0x38 .. +0x50 its +0xB4 ..
//! +0xCC, +0x54 .. +0x6C its +0x98 .. +0xB0.
//!
//! Refuses a null entry, record or name, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// `e` the entry of table B (edi in the original), `p` the player's record (esi), `index`
/// the player's index (ebx).
pub fn player_to_standing_b(
    menus: &crate::menu::owner::Menus,
    e: usize,
    p: PlayerCell,
    index: u32,
) -> Result<(), Refused> {
    name_player(&p.get())?;
    // The logic is in `rc2k_core::saves::rankings::fill_b`.
    use rc2k_core::menu::standings::PlayerB;
    let source = PlayerB {
        index,
        name: name_of(menus, &p.get())?,
        nationality: p.get().nationality,
        car: p.get().car_number,
        group: p.get().class,
        total: p.get().total,
        b4: std::array::from_fn(|k| p.get().overall_points[k]),
        slots: std::array::from_fn(|k| p.get().stage_points[k]),
    };
    with_b(|t| rc2k_core::saves::rankings::fill_b(&mut t.entries[e], &source))?;
    Ok(())
}
