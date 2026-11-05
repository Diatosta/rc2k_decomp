//! The screens' string-id tables and a few flags, out of RAL.EXE's image (IDA 0x758806 .. 0x758B72;
//! phase 7, typed): the "screen left" flag, the stage shown, the nationalities, the duel's labels, the
//! places' points and the pointers to the results' arrays, the player views, and the lists of
//! captions' string ids. A `#[repr(C)]` struct with the image's layout, one static; `moved::MOVED`
//! points the range at it. Its dwords lie two bytes past a multiple of four in IDA's addresses.
//! (Split 2026-10-25 from the front end's block `menu::store`.)

use crate::errors::BadInput;
use crate::session::{Bytes, Dword, with_runs};

/// The string-id tables (IDA 0x758806 .. 0x758B72).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CaptionTables {
    /// `dword_758806`: 1 when the screen (the car choice, a name entry) was left; the loops that look at it clear it.
    pub screen_left: Dword,
    /// `dword_75880A`: Not read.
    pub dword_75880a: Dword,
    /// `dword_75880E`: The stage group of the stage shown (the record table's index; a reading).
    pub shown_group: Dword,
    /// `dword_758812`: The index of the stage shown in its group (times 0xA4 into the group's records; a reading).
    pub shown_stage: Dword,
    /// `dword_758816`: The nationalities' name string ids (`screens::draw_player_details`).
    pub nationalities: [Dword; 47],
    /// `g_duelLabels`: The duel result's labels' string ids, by value (-1 ends).
    pub duel_labels: [Dword; 22],
    /// `g_placePoints`: The points of each place (0, 20000, ..., -1 ends).
    pub place_points: [Dword; 22],
    /// `off_758982`: The four positions in `Results::positions`, by byte offset, kept as IDA's
    /// addresses, as names (phase 13; `CaptionTables::place_of` finds the cell).
    pub place_pointers: [Dword; 4],
    /// `off_758992`: The four sums in `Results::sums` (names, `CaptionTables::sum_of`).
    pub sum_pointers: [Dword; 4],
    /// Unreferenced bytes (IDA 0x7589A2 .. 0x7589A6).
    pub _gap_7589a2: Bytes<4>,
    /// `off_7589A6`: The four totals in `Results::totals` (names, `CaptionTables::total_of`).
    pub total_pointers: [Dword; 4],
    /// Unreferenced bytes (IDA 0x7589B6 .. 0x7589BA).
    pub _gap_7589b6: Bytes<4>,
    /// `dword_7589BA`: The views' name string ids (`screens::draw_player_details`).
    pub views: [Dword; 8],
    /// `dword_7589DA`: The second views' name string ids.
    pub second_views: [Dword; 14],
    /// `unk_758A12`: The lists of captions' string ids, each ended by -1 (`screens::draw_highlighted_caption`, `menu::option_steps`).
    pub caption_lists: [Dword; 88],
}

impl CaptionTables {
    /// The results' cell a name (an IDA address in `menu::results_store`'s piece) names;
    /// refused off the piece's dwords.
    fn result_at(name: &Dword) -> Result<&'static Dword, BadInput> {
        use crate::menu::results_store::{RESULT_FIGURES as R, Results, START};
        use std::mem::offset_of;
        let at = (name.get() as usize).wrapping_sub(START);
        let arrays: [(usize, &'static [Dword]); 10] = [
            (offset_of!(Results, round), std::slice::from_ref(&R.round)),
            (offset_of!(Results, group_titles), &R.group_titles),
            (offset_of!(Results, order), &R.order),
            (offset_of!(Results, time_bonus), &R.time_bonus),
            (offset_of!(Results, positions), &R.positions),
            (offset_of!(Results, points), &R.points),
            (offset_of!(Results, extra), &R.extra),
            (offset_of!(Results, sums), &R.sums),
            (offset_of!(Results, before), &R.before),
            (offset_of!(Results, totals), &R.totals),
        ];
        arrays
            .iter()
            .chain([&(offset_of!(Results, pairs), &R.pairs[..])])
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
    pub fn place_of(k: usize) -> Result<&'static Dword, BadInput> {
        let name = CAPTIONS
            .place_pointers
            .get(k)
            .ok_or(BadInput("a place pointer past the four"))?;
        Self::result_at(name)
    }

    /// Sum `k` of the results (`Results::sums[k]`).
    pub fn sum_of(k: usize) -> Result<&'static Dword, BadInput> {
        let name = CAPTIONS
            .sum_pointers
            .get(k)
            .ok_or(BadInput("a sum pointer past the four"))?;
        Self::result_at(name)
    }

    /// Total `k` of the results (`Results::totals[k]`).
    pub fn total_of(k: usize) -> Result<&'static Dword, BadInput> {
        let name = CAPTIONS
            .total_pointers
            .get(k)
            .ok_or(BadInput("a total pointer past the four"))?;
        Self::result_at(name)
    }
}

/// First values the image's (`tests::captions_starts_as_the_games`).
pub static CAPTIONS: CaptionTables = CaptionTables {
    screen_left: Dword::new(0x0),
    dword_75880a: Dword::new(0x1),
    shown_group: Dword::new(0x0),
    shown_stage: Dword::new(0x1),
    nationalities: [
        Dword::new(0x57D),
        Dword::new(0x57E),
        Dword::new(0x57F),
        Dword::new(0x580),
        Dword::new(0x581),
        Dword::new(0x582),
        Dword::new(0x583),
        Dword::new(0x584),
        Dword::new(0x585),
        Dword::new(0x586),
        Dword::new(0x587),
        Dword::new(0x588),
        Dword::new(0x589),
        Dword::new(0x58A),
        Dword::new(0x58B),
        Dword::new(0x58C),
        Dword::new(0x58D),
        Dword::new(0x58E),
        Dword::new(0x58F),
        Dword::new(0x590),
        Dword::new(0x591),
        Dword::new(0x592),
        Dword::new(0x593),
        Dword::new(0x594),
        Dword::new(0x595),
        Dword::new(0x596),
        Dword::new(0x597),
        Dword::new(0x598),
        Dword::new(0x599),
        Dword::new(0x59A),
        Dword::new(0x59B),
        Dword::new(0x59C),
        Dword::new(0x59D),
        Dword::new(0x59E),
        Dword::new(0x59F),
        Dword::new(0x5A0),
        Dword::new(0x5A1),
        Dword::new(0x5A2),
        Dword::new(0x5A3),
        Dword::new(0x5A4),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x3C0),
        Dword::new(0x3C1),
        Dword::new(0x3C2),
        Dword::new(0x3C3),
        Dword::new(0x3C4),
        Dword::new(0xFFFFFFFF),
    ],
    duel_labels: [
        Dword::new(0x4FE),
        Dword::new(0x4FF),
        Dword::new(0x500),
        Dword::new(0x501),
        Dword::new(0x502),
        Dword::new(0x503),
        Dword::new(0x504),
        Dword::new(0x505),
        Dword::new(0x506),
        Dword::new(0x507),
        Dword::new(0x508),
        Dword::new(0x509),
        Dword::new(0x50A),
        Dword::new(0x50B),
        Dword::new(0x50C),
        Dword::new(0x50D),
        Dword::new(0x50E),
        Dword::new(0x50F),
        Dword::new(0x510),
        Dword::new(0x511),
        Dword::new(0x512),
        Dword::new(0xFFFFFFFF),
    ],
    place_points: [
        Dword::new(0x0),
        Dword::new(0x4E20),
        Dword::new(0x4A38),
        Dword::new(0x4650),
        Dword::new(0x4268),
        Dword::new(0x3E80),
        Dword::new(0x3A98),
        Dword::new(0x36B0),
        Dword::new(0x32C8),
        Dword::new(0x2EE0),
        Dword::new(0x2AF8),
        Dword::new(0x2710),
        Dword::new(0x2328),
        Dword::new(0x1F40),
        Dword::new(0x1B58),
        Dword::new(0x1770),
        Dword::new(0x1388),
        Dword::new(0xFA0),
        Dword::new(0xBB8),
        Dword::new(0x7D0),
        Dword::new(0x3E8),
        Dword::new(0xFFFFFFFF),
    ],
    place_pointers: [
        Dword::new(0x758768),
        Dword::new(0x75876C),
        Dword::new(0x758770),
        Dword::new(0x758774),
    ],
    sum_pointers: [
        Dword::new(0x758798),
        Dword::new(0x75879C),
        Dword::new(0x7587A0),
        Dword::new(0x7587A4),
    ],
    _gap_7589a2: Bytes::new(with_runs::<4>(&[(0x0, &[0xff, 0xff, 0xff, 0xff])])),
    total_pointers: [
        Dword::new(0x7587B8),
        Dword::new(0x7587BC),
        Dword::new(0x7587C0),
        Dword::new(0x7587C4),
    ],
    _gap_7589b6: Bytes::new(with_runs::<4>(&[(0x0, &[0xff, 0xff, 0xff, 0xff])])),
    views: [
        Dword::new(0x378),
        Dword::new(0x377),
        Dword::new(0x376),
        Dword::new(0x37C),
        Dword::new(0x379),
        Dword::new(0x37A),
        Dword::new(0x37B),
        Dword::new(0xFFFFFFFF),
    ],
    second_views: [
        Dword::new(0x377),
        Dword::new(0x378),
        Dword::new(0x379),
        Dword::new(0x37A),
        Dword::new(0x37B),
        Dword::new(0x37C),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x37D),
        Dword::new(0x37E),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x186),
        Dword::new(0x187),
        Dword::new(0x188),
        Dword::new(0xFFFFFFFF),
    ],
    caption_lists: [
        Dword::new(0x130),
        Dword::new(0x132),
        Dword::new(0x133),
        Dword::new(0x136),
        Dword::new(0x536),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1D8),
        Dword::new(0x1D9),
        Dword::new(0x1DA),
        Dword::new(0x1DB),
        Dword::new(0x1DD),
        Dword::new(0x13A),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1DA),
        Dword::new(0x3D6),
        Dword::new(0x3D7),
        Dword::new(0x3D8),
        Dword::new(0x3D9),
        Dword::new(0x3DA),
        Dword::new(0x3DB),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1D9),
        Dword::new(0x3DC),
        Dword::new(0x3DD),
        Dword::new(0x3DE),
        Dword::new(0x3DF),
        Dword::new(0x3E0),
        Dword::new(0x3E1),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1E6),
        Dword::new(0x1E8),
        Dword::new(0x1E9),
        Dword::new(0x1EA),
        Dword::new(0x1EB),
        Dword::new(0x13A),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1DE),
        Dword::new(0x1DF),
        Dword::new(0x13A),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1E6),
        Dword::new(0x1E7),
        Dword::new(0x13A),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1EF),
        Dword::new(0x1F0),
        Dword::new(0x1F1),
        Dword::new(0x1F2),
        Dword::new(0x1F3),
        Dword::new(0x1F4),
        Dword::new(0x13A),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x148),
        Dword::new(0x149),
        Dword::new(0x14A),
        Dword::new(0x13A),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x22D),
        Dword::new(0x233),
        Dword::new(0x225),
        Dword::new(0x234),
        Dword::new(0x227),
        Dword::new(0x223),
        Dword::new(0x231),
        Dword::new(0x22E),
        Dword::new(0x229),
        Dword::new(0x222),
        Dword::new(0x224),
        Dword::new(0x228),
        Dword::new(0x230),
        Dword::new(0x22B),
        Dword::new(0x235),
        Dword::new(0x236),
        Dword::new(0x1038),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x22F),
        Dword::new(0x232),
        Dword::new(0x226),
        Dword::new(0x22C),
        Dword::new(0x22A),
        Dword::new(0x237),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x0),
        Dword::new(0x1),
        Dword::new(0x2),
        Dword::new(0x3),
        Dword::new(0x4),
        Dword::new(0x5),
    ],
};

/// IDA 0x758806, the first byte of the piece.
pub const START: usize = 0x758806;
/// IDA 0x758B72, the end of the piece.
pub const END: usize = 0x758B72;

/// Where IDA 0x758806 is now (`moved::MOVED`'s home).
#[cfg(test)]
pub fn captions_addr() -> u32 {
    crate::pointers::game_address(&raw const CAPTIONS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn captions_starts_as_the_games() {
        assert!(
            END - START <= size_of::<CaptionTables>()
                && size_of::<CaptionTables>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(captions_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(CaptionTables, screen_left), 0x758806);
        at(offset_of!(CaptionTables, dword_75880a), 0x75880A);
        at(offset_of!(CaptionTables, shown_group), 0x75880E);
        at(offset_of!(CaptionTables, shown_stage), 0x758812);
        at(offset_of!(CaptionTables, nationalities), 0x758816);
        at(offset_of!(CaptionTables, duel_labels), 0x7588D2);
        at(offset_of!(CaptionTables, place_points), 0x75892A);
        at(offset_of!(CaptionTables, place_pointers), 0x758982);
        at(offset_of!(CaptionTables, sum_pointers), 0x758992);
        at(offset_of!(CaptionTables, total_pointers), 0x7589A6);
        at(offset_of!(CaptionTables, views), 0x7589BA);
        at(offset_of!(CaptionTables, second_views), 0x7589DA);
        at(offset_of!(CaptionTables, caption_lists), 0x758A12);
        // The three pointer tables' entries are the image's addresses, names of the results'
        // arrays: they resolve to what the image's address named.
        let image = image_bytes(0x758982, 0x7589B6 - 0x758982);
        for (table, from, positions) in [
            (0x758982, 0, 0x758768),
            (0x758992, 1, 0x758798),
            (0x7589A6, 2, 0x7587B8),
        ] {
            for k in 0..4 {
                let at = table - 0x758982 + 4 * k;
                let name = u32::from_le_bytes(image[at..at + 4].try_into().unwrap());
                assert_eq!(name as usize, positions + 4 * k);
                let resolved = [
                    CaptionTables::place_of,
                    CaptionTables::sum_of,
                    CaptionTables::total_of,
                ][from](k);
                let cells = [
                    &crate::menu::results_store::RESULT_FIGURES.positions,
                    &crate::menu::results_store::RESULT_FIGURES.sums,
                    &crate::menu::results_store::RESULT_FIGURES.totals,
                ][from];
                assert!(std::ptr::eq(resolved.unwrap(), &raw const cells[k]));
            }
            assert!(CaptionTables::sum_of(4).is_err());
        }
    }
}
