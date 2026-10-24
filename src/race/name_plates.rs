//! The name plates: texts drawn into the views' plate textures (`sub_45B3B9` ->
//! `sub_45B695` / `sub_41B2D4` / `sub_45B70A`, part R3, IDA 2026-10-18). A plate's surface
//! is locked (the platform's `sub_41AA4B`, carry when it failed), cleared and written with
//! `text::draw_text_keep_regs`, then unlocked (`sub_41AB02`). Parts: 1 lock (eax the slot),
//! 2 unlock (eax the slot), 3 a text (eax font, ebx text, ecx x, edx y).

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};

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

/// `race::draw_name_plate` (0x45B695): slot `slot`'s (eax) surface, locked and unlocked
/// (`d3d::lock_texture`, nothing when it fails): three lines' height cleared, text `text`
/// (ebx) drawn three times in the name font (box 0x168 wide) at x 4, -0x74, -0xEC and y 0, a
/// line, two lines (a long name wrapped over the plate's three rows). Every register kept.
pub unsafe fn draw_name_plate(slot: u32, text: u32) -> Result<(), Refused> {
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    unsafe {
        if lock_texture(slot)?.carry {
            return Ok(());
        }
        // Converted: `rc2k_core::race::plates::{name_rows, NAME_BOX, name_places}`.
        use rc2k_core::race::plates::{NAME_BOX, name_places, name_rows};
        let line = k(LINE);
        clear_rows(k(PIXELS), name_rows(line));
        w(NAME_FONT + 0x1C, NAME_BOX);
        for (x, y) in name_places(line) {
            draw_text_in_game(ga(NAME_FONT), text, x, y)?;
        }
        unlock_texture(slot)?;
        Ok(())
    }
}

/// `race::draw_plate_by_id` (0x41B2D4): string id `id` (ebx) into the text buffer (`text`,
/// with its NUL) and drawn on slot `slot`'s (eax) plate (`draw_name_plate`). Every register
/// kept.
pub unsafe fn draw_plate_by_id(slot: u32, id: u32) -> Result<(), Refused> {
    unsafe {
        let mut cursor = ga(TEXT) as *mut u8;
        crate::strings::append_string_by_id::append_string_by_id(id, &mut |p, n| {
            std::ptr::copy_nonoverlapping(p, cursor, n);
            cursor = cursor.add(n);
        });
        cursor.write(0);
        draw_name_plate(slot, ga(TEXT))
    }
}

/// `race::draw_time_plate` (0x45B70A): slot `slot`'s (eax) surface, locked and unlocked as
/// `draw_name_plate`'s, below the name's three lines: two
/// lines and three rows cleared, the texts at `dword_6D1838` and `dword_6D1844` in the time
/// font (box 0x80 wide) at x 4, y three and four lines. Every register kept.
pub unsafe fn draw_time_plate(slot: u32) -> Result<(), Refused> {
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    unsafe {
        if lock_texture(slot)?.carry {
            return Ok(());
        }
        // Converted: `rc2k_core::race::plates::{time_start, time_rows, TIME_BOX,
        // time_places}`.
        use rc2k_core::race::plates::{TIME_BOX, time_places, time_rows, time_start};
        let line = k(LINE);
        let start = time_start(line, k(PITCH));
        clear_rows(start.wrapping_add(k(PIXELS)), time_rows(line));
        for (text, (x, y)) in [0x6D1838u32, 0x6D1844].into_iter().zip(time_places(line)) {
            w(TIME_FONT + 0x1C, TIME_BOX);
            draw_text_in_game(ga(TIME_FONT), ga(text), x, y)?;
        }
        unlock_texture(slot)?;
        Ok(())
    }
}

/// `race::update_name_plates` (0x45B3B9): when the two time texts (8 bytes each at
/// `dword_6D1838` / `dword_6D1844`) differ from their last copies (`dword_6D1850` /
/// `dword_6D185C`): the time plate drawn (`race::draw_time_plate`, slot `dword_5D0DF8`),
/// the copies made and both views' cars' +0x720 1. Then for each view (slots `dword_5D0DF8` / `dword_5D0DFC`) with
/// a name asked for at its car's +0x71C: drawn by string id below 0x10000
/// (`race::draw_plate_by_id`) or as a text pointer (`race::draw_name_plate`), into +0x724,
/// +0x71C 0, +0x720 and +0x728 1. Its one caller, `race::draw_race_scene`, reads no
/// register it leaves. Refuses (before anything) a null view block.
pub unsafe fn update_name_plates() -> Result<(), Refused> {
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
            draw_time_plate(k(0x5D0DF8))?;
            for (from, to) in [
                (0x6D1838u32, 0x6D1850u32),
                (0x6D183C, 0x6D1854),
                (0x6D1844, 0x6D185C),
                (0x6D1848, 0x6D1860),
            ] {
                w(to, k(from));
            }
            for &b in &blocks {
                field(b, 0x720).write_unaligned(1);
            }
        }
        for (b, slot) in blocks.into_iter().zip([0x5D0DF8u32, 0x5D0DFC]) {
            let asked = field(b, 0x71C).read_unaligned();
            match NameAsk::of(asked) {
                NameAsk::Nothing => continue,
                NameAsk::Id(_) => draw_plate_by_id(k(slot), asked)?,
                NameAsk::Text(_) => draw_name_plate(k(slot), asked)?,
            }
            // ebx at the store: the name asked for (every drawing keeps its registers).
            field(b, 0x724).write_unaligned(asked);
            field(b, 0x71C).write_unaligned(0);
            field(b, 0x720).write_unaligned(1);
            field(b, 0x728).write_unaligned(1);
        }
        Ok(())
    }
}

/// A text drawn in the game: `text::draw_text_keep_regs`'s Rust version on the game's
/// surface, the font at `font`, the text at `text`, at (`x`, `y`); whether it was too wide
/// (the original's carry).
pub unsafe fn draw_text_in_game(font: u32, text: u32, x: u32, y: u32) -> Result<bool, Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    unsafe {
        let font = font as *mut Font;
        let mut copy = font.read_unaligned();
        let wide = crate::text::draw_text_keep_regs::draw_text_keep_regs(
            &mut copy,
            text as *const u8,
            x as i32,
            y as i32,
            &mut GameCanvas,
        )?;
        if copy != font.read_unaligned() {
            font.write_unaligned(copy);
        }
        Ok(wide)
    }
}
