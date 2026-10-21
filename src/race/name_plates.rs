//! The name plates: texts drawn into the views' plate textures (`sub_45B3B9` ->
//! `sub_45B695` / `sub_41B2D4` / `sub_45B70A`, part R3, IDA 2026-10-18). A plate's surface
//! is locked (the platform's `sub_41AA4B`, carry when it failed), cleared and written with
//! `text::draw_text_keep_regs`, then unlocked (`sub_41AB02`). Parts: 1 lock (eax the slot),
//! 2 unlock (eax the slot), 3 a text (eax font, ebx text, ecx x, edx y).

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

fn ga(a: u32) -> u32 {
    game_addr(rva(a as usize)) as u32
}

const LINE: u32 = 0x51E2CC; // dword_51E2CC: a line's height
const PIXELS: u32 = 0x4F49C0; // g_surface.pixels
const PITCH: u32 = 0x4F49BC; // g_surface.pitch
const ROW_DWORDS: u32 = 0x4F49B4; // g_surface.row_dwords
const ROW_GAP: u32 = 0x4F49C4; // g_surface.row_gap
const NAME_FONT: u32 = 0x51E304; // stru_51E304
const TIME_FONT: u32 = 0x51E324; // stru_51E324
const TEXT: u32 = 0x93E430; // text: the string buffer

/// `rows` rows of the locked surface from `edi` cleared (`rep stosd` of the row's dwords, then
/// the row's gap). Gives edi after them.
unsafe fn clear_rows(mut edi: u32, rows: u32) -> u32 {
    unsafe {
        for _ in 0..rows {
            let n = k(ROW_DWORDS);
            for d in 0..n {
                ((edi + 4 * d) as *mut u32).write_unaligned(0);
            }
            edi = edi.wrapping_add(4 * n).wrapping_add(k(ROW_GAP));
        }
        edi
    }
}

/// `race::draw_name_plate` (0x45B695): slot eax's surface: three lines' height cleared, text
/// ebx drawn three times in the name font (box 0x168 wide) at x 4, -0x74, -0xEC and y 0, a
/// line, two lines (a long name wrapped over the plate's three rows). Every register kept.
pub unsafe fn draw_name_plate(entry: &Regs, part: crate::race::frame::Part) -> Result<(), Refused> {
    unsafe {
        let mut r = *entry;
        part(1, &mut r)?;
        if r.carry() {
            return Ok(());
        }
        // Converted: `rc2k_core::race::plates::{name_rows, NAME_BOX, name_places}`.
        use rc2k_core::race::plates::{NAME_BOX, name_places, name_rows};
        let slot = r.eax;
        let line = k(LINE);
        clear_rows(k(PIXELS), name_rows(line));
        w(NAME_FONT + 0x1C, NAME_BOX);
        // The draws keep their registers: the unlock gets the last one's, eax the slot
        // again (the original's `pop eax`).
        let mut last = r;
        for (x, y) in name_places(line) {
            let mut t = Regs {
                eax: ga(NAME_FONT),
                ebx: entry.ebx,
                ecx: x,
                edx: y,
                ..last
            };
            part(3, &mut t)?;
            last = t;
        }
        let mut u = Regs { eax: slot, ..last };
        part(2, &mut u)
    }
}

/// `race::draw_plate_by_id` (0x41B2D4): string id ebx into the text buffer (`text`, with its
/// NUL) and drawn on slot eax's plate (part 1: `draw_name_plate`). Every register kept.
pub unsafe fn draw_plate_by_id(
    entry: &Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe {
        let mut cursor = ga(TEXT) as *mut u8;
        crate::strings::append_string_by_id::append_string_by_id(entry.ebx, &mut |p, n| {
            std::ptr::copy_nonoverlapping(p, cursor, n);
            cursor = cursor.add(n);
        });
        cursor.write(0);
        part(
            1,
            &mut Regs {
                ebx: ga(TEXT),
                ..*entry
            },
        )
    }
}

/// `race::draw_time_plate` (0x45B70A): slot eax's surface below the name's three lines: two
/// lines and three rows cleared, the texts at `dword_6D1838` and `dword_6D1844` in the time
/// font (box 0x80 wide) at x 4, y three and four lines. Every register kept.
pub unsafe fn draw_time_plate(entry: &Regs, part: crate::race::frame::Part) -> Result<(), Refused> {
    unsafe {
        let mut r = *entry;
        part(1, &mut r)?;
        if r.carry() {
            return Ok(());
        }
        // Converted: `rc2k_core::race::plates::{time_start, time_rows, TIME_BOX,
        // time_places}`.
        use rc2k_core::race::plates::{TIME_BOX, time_places, time_rows, time_start};
        let slot = r.eax;
        let line = k(LINE);
        let start = time_start(line, k(PITCH));
        clear_rows(start.wrapping_add(k(PIXELS)), time_rows(line));
        // As `draw_name_plate`: the unlock gets the last draw's registers, eax the slot.
        let mut last = r;
        for (text, (x, y)) in [0x6D1838u32, 0x6D1844].into_iter().zip(time_places(line)) {
            w(TIME_FONT + 0x1C, TIME_BOX);
            let mut t = Regs {
                eax: ga(TIME_FONT),
                ebx: ga(text),
                ecx: x,
                edx: y,
                ..last
            };
            part(3, &mut t)?;
            last = t;
        }
        let mut u = Regs { eax: slot, ..last };
        part(2, &mut u)
    }
}

/// `race::update_name_plates` (0x45B3B9): when the two time texts (8 bytes each at
/// `dword_6D1838` / `dword_6D1844`) differ from their last copies (`dword_6D1850` /
/// `dword_6D185C`): the time plate drawn (part 1, slot `dword_5D0DF8`), the copies made and
/// both views' cars' +0x720 1. Then for each view (slots `dword_5D0DF8` / `dword_5D0DFC`) with
/// a name asked for at its car's +0x71C: drawn by string id below 0x10000 (part 2) or as a
/// text pointer (part 3), into +0x724, +0x71C 0, +0x720 and +0x728 1. Refuses (before
/// anything) a null view block.
pub unsafe fn update_name_plates(
    r: &mut Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe {
        let blocks = [k(0x63DBB0), k(0x63DBB4)];
        if blocks.contains(&0) {
            return Err(crate::errors::BadInput("a view without its car block").into());
        }
        // Converted: `rc2k_core::race::plates::{times_changed, NameAsk}`.
        use rc2k_core::race::plates::{NameAsk, times_changed};
        let field = |b: u32, o: u32| (b + o) as *mut u32;
        let now = [0x6D1838, 0x6D183C, 0x6D1844, 0x6D1848].map(k);
        let kept = [0x6D1850, 0x6D1854, 0x6D185C, 0x6D1860].map(k);
        if times_changed(now, kept) {
            r.eax = k(0x5D0DF8);
            part(1, r)?;
            for (from, to) in [
                (0x6D1838u32, 0x6D1850u32),
                (0x6D183C, 0x6D1854),
                (0x6D1844, 0x6D185C),
                (0x6D1848, 0x6D1860),
            ] {
                r.eax = k(from);
                w(to, r.eax);
            }
            for &b in &blocks {
                r.esi = b;
                field(b, 0x720).write_unaligned(1);
            }
        }
        for (b, slot) in blocks.into_iter().zip([0x5D0DF8u32, 0x5D0DFC]) {
            r.esi = b;
            r.eax = k(slot);
            r.ebx = field(b, 0x71C).read_unaligned();
            let place = match NameAsk::of(r.ebx) {
                NameAsk::Nothing => continue,
                NameAsk::Id(_) => 2,
                NameAsk::Text(_) => 3,
            };
            part(place, r)?;
            r.esi = b;
            field(b, 0x724).write_unaligned(r.ebx);
            field(b, 0x71C).write_unaligned(0);
            field(b, 0x720).write_unaligned(1);
            field(b, 0x728).write_unaligned(1);
        }
        Ok(())
    }
}

/// A text drawn in the game: `text::draw_text_keep_regs`'s Rust version on the game's
/// surface (eax the font, ebx the text, ecx x, edx y), carry when too wide.
pub fn text_in_game(r: &mut Regs) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    unsafe {
        let font = r.eax as *mut Font;
        let mut copy = font.read_unaligned();
        let wide = crate::text::draw_text_keep_regs::draw_text_keep_regs(
            &mut copy,
            r.ebx as *const u8,
            r.ecx as i32,
            r.edx as i32,
            &mut GameCanvas,
        )?;
        if copy != font.read_unaligned() {
            font.write_unaligned(copy);
        }
        r.set_carry(wide);
        r.set_zero(!wide);
    }
    Ok(())
}

/// The plates' parts in the game: the platform's lock and unlock, the Rust text.
pub fn plate_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        3 => text_in_game(r),
        p => crate::race::frame_views::surface_part(p, r),
    }
}

/// `update_name_plates`' parts in the game.
pub fn update_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => draw_time_plate(r, &mut plate_part),
            2 => draw_plate_by_id(r, &mut by_id_part),
            _ => draw_name_plate(r, &mut plate_part),
        }
    }
}

/// `draw_plate_by_id`'s part in the game: `race::draw_name_plate`.
pub fn by_id_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { draw_name_plate(r, &mut plate_part) }
}
