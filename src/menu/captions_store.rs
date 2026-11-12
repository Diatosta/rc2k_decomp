//! The screens' string-id tables and a few flags, out of RAL.EXE's image (IDA 0x758806 .. 0x758B72): the "screen left" flag, the stage shown, the nationalities, the duel's labels, the
//! places' points and the pointers to the results' arrays, the player views, and the lists of
//! captions' string ids. A `#[repr(C)]` struct with the image's layout, one static. Its dwords lie two bytes past a multiple of four in IDA's addresses.
//!

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::session::{PlainBytes, PlainDword, PlainFlag, with_runs};

/// The string-id tables (IDA 0x758806 .. 0x758B72).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CaptionTables {
    /// `dword_758806`: 1 when the screen was left with a key (Escape in `handle_keys`, a yes / no
    /// answer, a name entry); the loop puts its leave question on it and the screens that look at it clear it.
    pub left_by_key: PlainFlag,
    /// `dword_75880A`: Not read.
    pub dword_75880a: PlainDword,
    /// `dword_75880E`: The stage group of the stage shown (the record table's index; set by menu::option_items).
    pub shown_group: PlainDword,
    /// `dword_758812`: The index of the stage shown in its group (times 0xA4 into the group's records; set by the stage pages' steps in menu::option_items and read by screens::stage_results).
    pub shown_stage: PlainDword,
    /// `dword_758816`: The nationalities' name string ids (`screens::draw_player_details`).
    pub nationalities: [PlainDword; 47],
    /// `g_duelLabels`: The duel result's labels' string ids, by value (-1 ends).
    pub duel_labels: [PlainDword; 22],
    /// `g_placePoints`: The points of each place (0, 20000, ..., -1 ends).
    pub place_points: [PlainDword; 22],
    /// `off_758982`: The four positions in `Results::positions`, by byte offset, kept as IDA's
    /// addresses, as names (`CaptionTables::place_of` finds the cell).
    pub place_pointers: [PlainDword; 4],
    /// `off_758992`: The four sums in `Results::sums` (names, `CaptionTables::sum_of`).
    pub sum_pointers: [PlainDword; 4],
    /// Unreferenced bytes (IDA 0x7589A2 .. 0x7589A6).
    pub _gap_7589a2: PlainBytes<4>,
    /// `off_7589A6`: The four totals in `Results::totals` (names, `CaptionTables::total_of`).
    pub total_pointers: [PlainDword; 4],
    /// Unreferenced bytes (IDA 0x7589B6 .. 0x7589BA).
    pub _gap_7589b6: PlainBytes<4>,
    /// `dword_7589BA`: The views' name string ids (`screens::draw_player_details`).
    pub views: [PlainDword; 8],
    /// `dword_7589DA`: The second views' name string ids.
    pub second_views: [PlainDword; 14],
    /// `unk_758A12`: The lists of captions' string ids, each ended by -1 (`screens::draw_highlighted_caption`, `menu::option_steps`).
    pub caption_lists: [PlainDword; 88],
}

impl CaptionTables {
    /// The results' cell a name (an IDA address in `menu::results_store`'s piece) names;
    /// refused off the piece's dwords.
    fn result_at<'a>(menus: &'a Menus, name: &PlainDword) -> Result<&'a PlainDword, BadInput> {
        use crate::menu::results_store::{Results, START};
        let r = &menus.play.results;
        use std::mem::offset_of;
        let at = (name.get() as usize).wrapping_sub(START);
        let arrays: [(usize, &'a [PlainDword]); 10] = [
            (offset_of!(Results, round), std::slice::from_ref(&r.round)),
            (offset_of!(Results, group_titles), &r.group_titles),
            (offset_of!(Results, order), &r.order),
            (offset_of!(Results, time_bonus), &r.time_bonus),
            (offset_of!(Results, positions), &r.positions),
            (offset_of!(Results, points), &r.points),
            (offset_of!(Results, extra), &r.extra),
            (offset_of!(Results, sums), &r.sums),
            (offset_of!(Results, before), &r.before),
            (offset_of!(Results, totals), &r.totals),
        ];
        arrays
            .iter()
            .chain([&(offset_of!(Results, pairs), &r.pairs[..])])
            .find_map(|&(first, cells)| {
                let d = at.checked_sub(first)?;
                if d % 4 != 0 {
                    return None;
                }
                cells.get(d / 4)
            })
            .ok_or(BadInput("a results pointer off the results' dwords"))
    }

    /// Position `k` of the results (`Results::positions[k]`).
    pub fn place_of(menus: &Menus, k: usize) -> Result<&PlainDword, BadInput> {
        let name = menus
            .tables
            .captions
            .place_pointers
            .get(k)
            .ok_or(BadInput("a place pointer past the four"))?;
        Self::result_at(menus, name)
    }

    /// Sum `k` of the results (`Results::sums[k]`).
    pub fn sum_of(menus: &Menus, k: usize) -> Result<&PlainDword, BadInput> {
        let name = menus
            .tables
            .captions
            .sum_pointers
            .get(k)
            .ok_or(BadInput("a sum pointer past the four"))?;
        Self::result_at(menus, name)
    }

    /// Total `k` of the results (`Results::totals[k]`).
    pub fn total_of(menus: &Menus, k: usize) -> Result<&PlainDword, BadInput> {
        let name = menus
            .tables
            .captions
            .total_pointers
            .get(k)
            .ok_or(BadInput("a total pointer past the four"))?;
        Self::result_at(menus, name)
    }
}

/// First values the image's (`tests::captions_starts_as_the_games`).
impl CaptionTables {
    pub const fn new() -> Self {
        CaptionTables {
            left_by_key: PlainFlag::new(false),
            dword_75880a: PlainDword::new(0x1),
            shown_group: PlainDword::new(0x0),
            shown_stage: PlainDword::new(0x1),
            nationalities: [
                PlainDword::new(0x57D),
                PlainDword::new(0x57E),
                PlainDword::new(0x57F),
                PlainDword::new(0x580),
                PlainDword::new(0x581),
                PlainDword::new(0x582),
                PlainDword::new(0x583),
                PlainDword::new(0x584),
                PlainDword::new(0x585),
                PlainDword::new(0x586),
                PlainDword::new(0x587),
                PlainDword::new(0x588),
                PlainDword::new(0x589),
                PlainDword::new(0x58A),
                PlainDword::new(0x58B),
                PlainDword::new(0x58C),
                PlainDword::new(0x58D),
                PlainDword::new(0x58E),
                PlainDword::new(0x58F),
                PlainDword::new(0x590),
                PlainDword::new(0x591),
                PlainDword::new(0x592),
                PlainDword::new(0x593),
                PlainDword::new(0x594),
                PlainDword::new(0x595),
                PlainDword::new(0x596),
                PlainDword::new(0x597),
                PlainDword::new(0x598),
                PlainDword::new(0x599),
                PlainDword::new(0x59A),
                PlainDword::new(0x59B),
                PlainDword::new(0x59C),
                PlainDword::new(0x59D),
                PlainDword::new(0x59E),
                PlainDword::new(0x59F),
                PlainDword::new(0x5A0),
                PlainDword::new(0x5A1),
                PlainDword::new(0x5A2),
                PlainDword::new(0x5A3),
                PlainDword::new(0x5A4),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x3C0),
                PlainDword::new(0x3C1),
                PlainDword::new(0x3C2),
                PlainDword::new(0x3C3),
                PlainDword::new(0x3C4),
                PlainDword::new(0xFFFFFFFF),
            ],
            duel_labels: [
                PlainDword::new(0x4FE),
                PlainDword::new(0x4FF),
                PlainDword::new(0x500),
                PlainDword::new(0x501),
                PlainDword::new(0x502),
                PlainDword::new(0x503),
                PlainDword::new(0x504),
                PlainDword::new(0x505),
                PlainDword::new(0x506),
                PlainDword::new(0x507),
                PlainDword::new(0x508),
                PlainDword::new(0x509),
                PlainDword::new(0x50A),
                PlainDword::new(0x50B),
                PlainDword::new(0x50C),
                PlainDword::new(0x50D),
                PlainDword::new(0x50E),
                PlainDword::new(0x50F),
                PlainDword::new(0x510),
                PlainDword::new(0x511),
                PlainDword::new(0x512),
                PlainDword::new(0xFFFFFFFF),
            ],
            place_points: [
                PlainDword::new(0x0),
                PlainDword::new(0x4E20),
                PlainDword::new(0x4A38),
                PlainDword::new(0x4650),
                PlainDword::new(0x4268),
                PlainDword::new(0x3E80),
                PlainDword::new(0x3A98),
                PlainDword::new(0x36B0),
                PlainDword::new(0x32C8),
                PlainDword::new(0x2EE0),
                PlainDword::new(0x2AF8),
                PlainDword::new(0x2710),
                PlainDword::new(0x2328),
                PlainDword::new(0x1F40),
                PlainDword::new(0x1B58),
                PlainDword::new(0x1770),
                PlainDword::new(0x1388),
                PlainDword::new(0xFA0),
                PlainDword::new(0xBB8),
                PlainDword::new(0x7D0),
                PlainDword::new(0x3E8),
                PlainDword::new(0xFFFFFFFF),
            ],
            place_pointers: [
                PlainDword::new(0x758768),
                PlainDword::new(0x75876C),
                PlainDword::new(0x758770),
                PlainDword::new(0x758774),
            ],
            sum_pointers: [
                PlainDword::new(0x758798),
                PlainDword::new(0x75879C),
                PlainDword::new(0x7587A0),
                PlainDword::new(0x7587A4),
            ],
            _gap_7589a2: PlainBytes::new(with_runs::<4>(&[(0x0, &[0xff, 0xff, 0xff, 0xff])])),
            total_pointers: [
                PlainDword::new(0x7587B8),
                PlainDword::new(0x7587BC),
                PlainDword::new(0x7587C0),
                PlainDword::new(0x7587C4),
            ],
            _gap_7589b6: PlainBytes::new(with_runs::<4>(&[(0x0, &[0xff, 0xff, 0xff, 0xff])])),
            views: [
                PlainDword::new(0x378),
                PlainDword::new(0x377),
                PlainDword::new(0x376),
                PlainDword::new(0x37C),
                PlainDword::new(0x379),
                PlainDword::new(0x37A),
                PlainDword::new(0x37B),
                PlainDword::new(0xFFFFFFFF),
            ],
            second_views: [
                PlainDword::new(0x377),
                PlainDword::new(0x378),
                PlainDword::new(0x379),
                PlainDword::new(0x37A),
                PlainDword::new(0x37B),
                PlainDword::new(0x37C),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x37D),
                PlainDword::new(0x37E),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x186),
                PlainDword::new(0x187),
                PlainDword::new(0x188),
                PlainDword::new(0xFFFFFFFF),
            ],
            caption_lists: [
                PlainDword::new(0x130),
                PlainDword::new(0x132),
                PlainDword::new(0x133),
                PlainDword::new(0x136),
                PlainDword::new(0x536),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1D8),
                PlainDword::new(0x1D9),
                PlainDword::new(0x1DA),
                PlainDword::new(0x1DB),
                PlainDword::new(0x1DD),
                PlainDword::new(0x13A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1DA),
                PlainDword::new(0x3D6),
                PlainDword::new(0x3D7),
                PlainDword::new(0x3D8),
                PlainDword::new(0x3D9),
                PlainDword::new(0x3DA),
                PlainDword::new(0x3DB),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1D9),
                PlainDword::new(0x3DC),
                PlainDword::new(0x3DD),
                PlainDword::new(0x3DE),
                PlainDword::new(0x3DF),
                PlainDword::new(0x3E0),
                PlainDword::new(0x3E1),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1E6),
                PlainDword::new(0x1E8),
                PlainDword::new(0x1E9),
                PlainDword::new(0x1EA),
                PlainDword::new(0x1EB),
                PlainDword::new(0x13A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1DE),
                PlainDword::new(0x1DF),
                PlainDword::new(0x13A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1E6),
                PlainDword::new(0x1E7),
                PlainDword::new(0x13A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1EF),
                PlainDword::new(0x1F0),
                PlainDword::new(0x1F1),
                PlainDword::new(0x1F2),
                PlainDword::new(0x1F3),
                PlainDword::new(0x1F4),
                PlainDword::new(0x13A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x148),
                PlainDword::new(0x149),
                PlainDword::new(0x14A),
                PlainDword::new(0x13A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x22D),
                PlainDword::new(0x233),
                PlainDword::new(0x225),
                PlainDword::new(0x234),
                PlainDword::new(0x227),
                PlainDword::new(0x223),
                PlainDword::new(0x231),
                PlainDword::new(0x22E),
                PlainDword::new(0x229),
                PlainDword::new(0x222),
                PlainDword::new(0x224),
                PlainDword::new(0x228),
                PlainDword::new(0x230),
                PlainDword::new(0x22B),
                PlainDword::new(0x235),
                PlainDword::new(0x236),
                PlainDword::new(0x1038),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x22F),
                PlainDword::new(0x232),
                PlainDword::new(0x226),
                PlainDword::new(0x22C),
                PlainDword::new(0x22A),
                PlainDword::new(0x237),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x0),
                PlainDword::new(0x1),
                PlainDword::new(0x2),
                PlainDword::new(0x3),
                PlainDword::new(0x4),
                PlainDword::new(0x5),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::owner::menus;

    #[test]
    fn captions_starts_as_the_games() {
        let play = &menus().play;
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().tables.captions,
        );
        // The three pointer tables' entries name the results' arrays.
        for from in 0..3 {
            for k in 0..4 {
                let resolved = [
                    CaptionTables::place_of,
                    CaptionTables::sum_of,
                    CaptionTables::total_of,
                ][from](crate::menu::owner::menus(), k);
                let cells = [
                    &play.results.positions,
                    &play.results.sums,
                    &play.results.totals,
                ][from];
                assert!(std::ptr::eq(resolved.unwrap(), &raw const cells[k]));
            }
            assert!(CaptionTables::sum_of(crate::menu::owner::menus(), 4).is_err());
        }
    }
}
