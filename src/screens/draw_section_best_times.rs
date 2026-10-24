//! `screens::draw_section_best_times` (0x46762B; IDA had it as code outside any function,
//! made one 2026-10-19): ten rows of the section's best times (`off_742C98`[section], the
//! game's own tables of 0x9C-byte entries, six sections), from y 0xF0, 0x19 apart, row n
//! (1 .. 10, kept in `seconds`): a player's (+0x40 1) in 0x51E524 / 0x51E544, else 0x51E584 /
//! 0x51E5A4: the name (the text at +0x20) at 0xF0, the time (+0x18, `strings::append_time`,
//! drawn as text) at 0x1E0, n at 0xC8; then the nationality's sprite (0x1C + +8) at 0xD2,
//! the flag's (+0x10, 0x18 and 0x19 swapped, + 0x44) at 0x276, and again at 0x276 the glyph
//! that number gives once more (+0x44 on it, sic: `notes/bugs.md`), each with the palette
//! the last text left (ebx). Each drawer's y is edx as the text drawers leave it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Refuses (before anything) a section past the six or with no table, and a line past the line
//! buffer. In the draw list 0x758E4E of `menu::unused_section_times_loop_1` / `_2` (0x466E50,
//! 0x467392), which nothing reaches (no code or data refers to them, 2026-10-20). The menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::strings::append_time::show_hours;
use crate::text::draw_text::Canvas;

const TABLES_RVA: usize = rva(0x742C98); // off_742C98: six tables
const SECTIONS: u32 = 6;
const SECTION_RVA: usize = rva(0x75595C); // section
// seconds
const ENTRY_LEN: u32 = 0x9C;
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version.
pub unsafe fn draw_section_best_times(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let section: u32 = game_read(SECTION_RVA);
        if section >= SECTIONS {
            return Err(BadInput("a section past the six best-times tables").into());
        }
        let e: u32 = game_read(TABLES_RVA + 4 * section as usize);
        if e == 0 {
            return Err(BadInput("a section with no best-times table").into());
        }
        // Converted: `rc2k_core::screens::best_scores::section_best_times`.
        use rc2k_core::screens::best_scores::SectionRow;
        let rows: [SectionRow<u32>; 10] = std::array::from_fn(|n| {
            let e = e + n as u32 * ENTRY_LEN;
            SectionRow {
                name: e + 0x20,
                time: get(e + 0x18),
                nation: get(e + 8),
                car: get(e + 0x10),
                player: get(e + 0x40) == 1,
            }
        });
        rc2k_core::screens::best_scores::section_best_times(
            &mut crate::screens::best_scores::ScoresScreen::new(canvas),
            &rows,
            show_hours(),
        )?;
    }
    Ok(())
}
