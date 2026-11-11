//! The name plates: texts drawn into the views' plate textures (`race::update_name_plates` ->
//! `race::draw_name_plate` / `race::draw_plate_by_id` / `race::draw_time_plate`). A plate's surface is locked (the platform's `d3d::lock_texture`, carry when it
//! failed), cleared and written with
//! `text::draw_text_keep_regs`, then unlocked (`d3d::unlock_texture`). Parts: 1 lock (the
//! slot), 2 unlock (the slot), 3 a text (font, text, x, y).

use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::strings::owner::strings;

/// `race::draw_name_plate` (0x45B695): slot `slot`'s surface, locked and unlocked
/// (`d3d::lock_texture`, nothing when it fails): three lines' height cleared, text `text` drawn
/// three times in the name font (box 0x168 wide) at x 4, -0x74, -0xEC and y 0, a line, two lines (a
/// long name wrapped over the plate's three rows).
pub fn draw_name_plate(slot: u32, text: impl Into<crate::place::BytePlace>) -> Result<(), Refused> {
    let gfx = crate::gfx::owner::gfx();
    let fonts = &gfx.fonts;
    let text = text.into();
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    if lock_texture(slot)?.carry {
        return Ok(());
    }
    // Converted: `rc2k_core::race::plates::{name_rows, NAME_BOX, name_places}`.
    use rc2k_core::race::plates::{NAME_BOX, name_places, name_rows};
    let line = fonts.font_block.game_fonts[0].get().line_height as u32; // a line's height
    crate::gfx::rows::clear_surface_rows(0, name_rows(line), gfx)?;
    fonts.font_block.game_fonts[2].update(|f| f.box_width = NAME_BOX);
    for (x, y) in name_places(line) {
        draw_text_in_game(&fonts.font_block.game_fonts[2], text, x, y)?;
    }
    unlock_texture(slot)?;
    Ok(())
}

/// `race::draw_plate_by_id` (0x41B2D4): string id `id` into the text buffer (`text`, with its NUL)
/// and drawn on slot `slot`'s plate (`draw_name_plate`).
pub fn draw_plate_by_id(slot: u32, id: u32) -> Result<(), Refused> {
    let text = &strings().text;
    let mut cursor = 0usize;
    crate::strings::append_string_by_id::append_string_by_id(id, &mut |bytes| {
        for (k, &b) in bytes.iter().enumerate() {
            text.buffer.text.set(cursor + k, b);
        }
        cursor += bytes.len();
    })?;
    text.buffer.text.set(cursor, 0);
    draw_name_plate(slot, &text.buffer.text)
}

/// `race::draw_time_plate` (0x45B70A): slot `slot`'s surface, locked and unlocked as
/// `draw_name_plate`'s, below the name's three lines: two lines and three rows cleared, the texts
/// at `label_behind` and `label_ahead` in the time font (box 0x80 wide) at x 4, y three and four
/// lines.
pub fn draw_time_plate(frame: &Frame, slot: u32) -> Result<(), Refused> {
    let gfx = gfx();
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    if lock_texture(slot)?.carry {
        return Ok(());
    }
    // Converted: `rc2k_core::race::plates::{time_start, time_rows, TIME_BOX,
    // time_places}`.
    use rc2k_core::race::plates::{TIME_BOX, time_places, time_rows, time_start};
    let line = gfx.fonts.font_block.game_fonts[0].get().line_height as u32;
    let start = time_start(line, gfx.drawing.draw_surface.pitch.get());
    crate::gfx::rows::clear_surface_rows(start, time_rows(line), gfx)?;
    // A label is eight bytes (spaces after each split, no NUL): its text runs on into the
    // dword after it, which ends it.
    let overlay = &frame.panel_and_overlay.overlay;
    for ((label, after), (x, y)) in [
        (&overlay.label_behind, &overlay.unused_334),
        (&overlay.label_ahead, &overlay.unused_340),
    ]
    .into_iter()
    .zip(time_places(line))
    {
        let text: Vec<u8> = label
            .iter()
            .chain(std::iter::once(after))
            .flat_map(|d| d.get().to_le_bytes())
            .collect();
        gfx.fonts.font_block.game_fonts[3].update(|f| f.box_width = TIME_BOX);
        draw_bytes_in_game(&gfx.fonts.font_block.game_fonts[3], &text, x, y)?;
    }
    unlock_texture(slot)?;
    Ok(())
}

/// `race::update_name_plates` (0x45B3B9): when the two time texts (8 bytes each at `label_behind` /
/// `label_ahead`) differ from their last copies (`kept_behind` / `kept_ahead`): the time plate
/// drawn (`race::draw_time_plate`, slot `KindTextureTables.kind_first_textures[196]`), the copies made and both views' cars'
/// +0x720 1. Then for each view (slots `KindTextureTables.kind_first_textures[196]` / `KindTextureTables.kind_first_textures[197]`) with a name asked for at
/// its car's +0x71C: drawn by string id below 0x10000 (`race::draw_plate_by_id`) or as a text
/// pointer (`race::draw_name_plate`), into +0x724, +0x71C 0, +0x720 and +0x728 1. Refuses (before
/// anything) a null view block.
pub fn update_name_plates(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let frame = &race.frame;
    let shared = &race.shared;
    let textures = &scene.textures;
    let (Some(a), Some(b)) = (shared.view.view_car(0), shared.view.view_car(1)) else {
        return Err(crate::errors::BadInput("a view without its car block").into());
    };
    // Their notice fields: +0x71C asked, +0x720 the plate flag, +0x724 the last, +0x728 shown.
    let blocks = [a, b];
    // Converted: `rc2k_core::race::plates::{times_changed, NameAsk}`.
    use rc2k_core::race::plates::{NameAsk, times_changed};
    let now = [
        frame.panel_and_overlay.overlay.label_behind[0].get(),
        frame.panel_and_overlay.overlay.label_behind[1].get(),
        frame.panel_and_overlay.overlay.label_ahead[0].get(),
        frame.panel_and_overlay.overlay.label_ahead[1].get(),
    ];
    let kept = [
        frame.panel_and_overlay.overlay.kept_behind[0].get(),
        frame.panel_and_overlay.overlay.kept_behind[1].get(),
        frame.panel_and_overlay.overlay.kept_ahead[0].get(),
        frame.panel_and_overlay.overlay.kept_ahead[1].get(),
    ];
    if times_changed(now, kept) {
        draw_time_plate(frame, textures.kinds.kind_first_textures[196].get())?;
        for (from, to) in [
            (
                &frame.panel_and_overlay.overlay.label_behind[0],
                &frame.panel_and_overlay.overlay.kept_behind[0],
            ),
            (
                &frame.panel_and_overlay.overlay.label_behind[1],
                &frame.panel_and_overlay.overlay.kept_behind[1],
            ),
            (
                &frame.panel_and_overlay.overlay.label_ahead[0],
                &frame.panel_and_overlay.overlay.kept_ahead[0],
            ),
            (
                &frame.panel_and_overlay.overlay.label_ahead[1],
                &frame.panel_and_overlay.overlay.kept_ahead[1],
            ),
        ] {
            to.set(from.get());
        }
        for &b in &blocks {
            b.notice_counter.set(1);
        }
    }
    for (b, slot) in blocks.into_iter().zip([196, 197]) {
        let asked = b.notice_id.get();
        match NameAsk::of(asked) {
            NameAsk::Nothing => continue,
            NameAsk::Id(_) => {
                draw_plate_by_id(textures.kinds.kind_first_textures[slot].get(), asked)?;
            }
            NameAsk::Text(_) => {
                draw_name_plate(textures.kinds.kind_first_textures[slot].get(), asked)?;
            }
        }
        // The store takes the name asked for.
        b.notice_last.set(asked);
        b.notice_id.set(0);
        b.notice_counter.set(1);
        b.notice_shown.set(1);
    }
    Ok(())
}

/// A text drawn in the game: `text::draw_text_keep_regs`'s Rust version on the game's
/// surface, the font at `font`, the text at `text`, at (`x`, `y`); whether it was too wide
/// (the original's carry).
pub fn draw_text_in_game(
    font: &crate::gfx::typefaces::FontObject,
    text: impl Into<crate::place::BytePlace>,
    x: u32,
    y: u32,
) -> Result<bool, Refused> {
    draw_bytes_in_game(
        font,
        &text.into().game_ptr().bytes("a name plate's text")?,
        x,
        y,
    )
}

/// As `draw_text_in_game`, the text's bytes in hand.
pub fn draw_bytes_in_game(
    font: &crate::gfx::typefaces::FontObject,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: u32,
    y: u32,
) -> Result<bool, Refused> {
    use crate::text::draw_text::GameCanvas;
    let mut copy = font.get();
    let wide = crate::text::draw_text_keep_regs::draw_text_keep_regs(
        &mut copy,
        text,
        x as i32,
        y as i32,
        &mut GameCanvas,
    )?;
    if copy != font.get() {
        font.set(copy);
    }
    Ok(wide)
}
