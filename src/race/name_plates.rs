//! The name plates: texts drawn into the views' plate textures (`sub_45B3B9` ->
//! `sub_45B695` / `sub_41B2D4` / `sub_45B70A`, part R3, IDA 2026-10-18). A plate's surface
//! is locked (the platform's `sub_41AA4B`, carry when it failed), cleared and written with
//! `text::draw_text_keep_regs`, then unlocked (`sub_41AB02`). Parts: 1 lock (eax the slot),
//! 2 unlock (eax the slot), 3 a text (eax font, ebx text, ecx x, edx y).

use crate::errors::Refused;
use crate::gfx::fonts_store::FONTS;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::pointers::GamePtr;
use crate::race::overlay_store::RACE_OVERLAY as OVERLAY;
use crate::race::view_store::RACE_VIEW;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::strings::text_buffer::TEXT_BUFFER;

/// `rows` rows of the locked surface from `edi` cleared (`rep stosd` of the row's dwords, then
/// the row's gap). Gives edi after them.
fn clear_rows(mut edi: u32, rows: u32) -> u32 {
    for _ in 0..rows {
        let n = DRAW_SURFACE.row_dwords.get();
        if let Err(e) = GamePtr::new(edi).fill(0, 4 * n as usize, "a plate's surface") {
            crate::fatal::raise::end("race::name_plates", e);
        }
        edi = edi
            .wrapping_add(4 * n)
            .wrapping_add(DRAW_SURFACE.row_gap.get());
    }
    edi
}

/// `race::draw_name_plate` (0x45B695): slot `slot`'s (eax) surface, locked and unlocked
/// (`d3d::lock_texture`, nothing when it fails): three lines' height cleared, text `text`
/// (ebx) drawn three times in the name font (box 0x168 wide) at x 4, -0x74, -0xEC and y 0, a
/// line, two lines (a long name wrapped over the plate's three rows). Every register kept.
pub fn draw_name_plate(slot: u32, text: impl Into<crate::place::Place>) -> Result<(), Refused> {
    let text = text.into();
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    if lock_texture(slot)?.carry {
        return Ok(());
    }
    // Converted: `rc2k_core::race::plates::{name_rows, NAME_BOX, name_places}`.
    use rc2k_core::race::plates::{NAME_BOX, name_places, name_rows};
    let line = FONTS.game_fonts[0].get().line_height as u32; // a line's height
    clear_rows(crate::gfx::surface::address()?, name_rows(line));
    FONTS.game_fonts[2].update(|f| f.box_width = NAME_BOX);
    for (x, y) in name_places(line) {
        draw_text_in_game(&FONTS.game_fonts[2], text, x, y)?;
    }
    unlock_texture(slot)?;
    Ok(())
}

/// `race::draw_plate_by_id` (0x41B2D4): string id `id` (ebx) into the text buffer (`text`,
/// with its NUL) and drawn on slot `slot`'s (eax) plate (`draw_name_plate`). Every register
/// kept.
pub fn draw_plate_by_id(slot: u32, id: u32) -> Result<(), Refused> {
    let mut cursor = 0usize;
    crate::strings::append_string_by_id::append_string_by_id(id, &mut |bytes| {
        for (k, &b) in bytes.iter().enumerate() {
            TEXT_BUFFER.text.set(cursor + k, b);
        }
        cursor += bytes.len();
    })?;
    TEXT_BUFFER.text.set(cursor, 0);
    draw_name_plate(slot, &TEXT_BUFFER.text)
}

/// `race::draw_time_plate` (0x45B70A): slot `slot`'s (eax) surface, locked and unlocked as
/// `draw_name_plate`'s, below the name's three lines: two
/// lines and three rows cleared, the texts at `dword_6D1838` and `dword_6D1844` in the time
/// font (box 0x80 wide) at x 4, y three and four lines. Every register kept.
pub fn draw_time_plate(slot: u32) -> Result<(), Refused> {
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    if lock_texture(slot)?.carry {
        return Ok(());
    }
    // Converted: `rc2k_core::race::plates::{time_start, time_rows, TIME_BOX,
    // time_places}`.
    use rc2k_core::race::plates::{TIME_BOX, time_places, time_rows, time_start};
    let line = FONTS.game_fonts[0].get().line_height as u32;
    let start = time_start(line, DRAW_SURFACE.pitch.get());
    clear_rows(
        start.wrapping_add(crate::gfx::surface::address()?),
        time_rows(line),
    );
    for (text, (x, y)) in [&OVERLAY.label_behind[0], &OVERLAY.label_ahead[0]]
        .into_iter()
        .zip(time_places(line))
    {
        FONTS.game_fonts[3].update(|f| f.box_width = TIME_BOX);
        draw_text_in_game(&FONTS.game_fonts[3], text.addr(), x, y)?;
    }
    unlock_texture(slot)?;
    Ok(())
}

/// `race::update_name_plates` (0x45B3B9): when the two time texts (8 bytes each at
/// `dword_6D1838` / `dword_6D1844`) differ from their last copies (`dword_6D1850` /
/// `dword_6D185C`): the time plate drawn (`race::draw_time_plate`, slot `dword_5D0DF8`),
/// the copies made and both views' cars' +0x720 1. Then for each view (slots `dword_5D0DF8` / `dword_5D0DFC`) with
/// a name asked for at its car's +0x71C: drawn by string id below 0x10000
/// (`race::draw_plate_by_id`) or as a text pointer (`race::draw_name_plate`), into +0x724,
/// +0x71C 0, +0x720 and +0x728 1. Its one caller, `race::draw_race_scene`, reads no
/// register it leaves. Refuses (before anything) a null view block.
pub fn update_name_plates() -> Result<(), Refused> {
    let blocks = [RACE_VIEW.view_block(0), RACE_VIEW.view_block(1)];
    if blocks.contains(&0) {
        return Err(crate::errors::BadInput("a view without its car block").into());
    }
    // Converted: `rc2k_core::race::plates::{times_changed, NameAsk}`.
    use rc2k_core::race::plates::{NameAsk, times_changed};
    let field = |b: u32, o: u32| GamePtr::new(b + o);
    let now = [
        OVERLAY.label_behind[0].get(),
        OVERLAY.label_behind[1].get(),
        OVERLAY.label_ahead[0].get(),
        OVERLAY.label_ahead[1].get(),
    ];
    let kept = [
        OVERLAY.kept_behind[0].get(),
        OVERLAY.kept_behind[1].get(),
        OVERLAY.kept_ahead[0].get(),
        OVERLAY.kept_ahead[1].get(),
    ];
    if times_changed(now, kept) {
        draw_time_plate(KIND_TEXTURES.kind_first_textures[196].get())?;
        for (from, to) in [
            (&OVERLAY.label_behind[0], &OVERLAY.kept_behind[0]),
            (&OVERLAY.label_behind[1], &OVERLAY.kept_behind[1]),
            (&OVERLAY.label_ahead[0], &OVERLAY.kept_ahead[0]),
            (&OVERLAY.label_ahead[1], &OVERLAY.kept_ahead[1]),
        ] {
            to.set(from.get());
        }
        for &b in &blocks {
            field(b, 0x720).write(1u32, "a view block's plate flag")?;
        }
    }
    for (b, slot) in blocks.into_iter().zip([196, 197]) {
        let asked: u32 = field(b, 0x71C).read("a view block's plate ask")?;
        match NameAsk::of(asked) {
            NameAsk::Nothing => continue,
            NameAsk::Id(_) => {
                draw_plate_by_id(KIND_TEXTURES.kind_first_textures[slot].get(), asked)?;
            }
            NameAsk::Text(_) => {
                draw_name_plate(KIND_TEXTURES.kind_first_textures[slot].get(), asked)?;
            }
        }
        // ebx at the store: the name asked for (every drawing keeps its registers).
        field(b, 0x724).write(asked, "a view block's plate ask")?;
        field(b, 0x71C).write(0u32, "a view block's plate ask")?;
        field(b, 0x720).write(1u32, "a view block's plate flag")?;
        field(b, 0x728).write(1u32, "a view block's plate flag")?;
    }
    Ok(())
}

/// A text drawn in the game: `text::draw_text_keep_regs`'s Rust version on the game's
/// surface, the font at `font`, the text at `text`, at (`x`, `y`); whether it was too wide
/// (the original's carry).
pub fn draw_text_in_game(
    font: &crate::session::Shared<crate::text::font::Font>,
    text: impl Into<crate::place::Place>,
    x: u32,
    y: u32,
) -> Result<bool, Refused> {
    use crate::text::draw_text::GameCanvas;
    let mut copy = font.get();
    let wide = crate::text::draw_text_keep_regs::draw_text_keep_regs(
        &mut copy,
        text.into().game_ptr(),
        x as i32,
        y as i32,
        &mut GameCanvas,
    )?;
    if copy != font.get() {
        font.set(copy);
    }
    Ok(wide)
}
