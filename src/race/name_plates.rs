//! The name plates: texts drawn into the views' plate textures (`race::update_name_plates` ->
//! `race::draw_name_plate` / `race::draw_plate_by_id` / `race::draw_time_plate`). A plate's surface
//! is locked (`d3d::lock_texture`), cleared and written with `text::draw_text_keep_regs`, then
//! unlocked (`d3d::unlock_texture`).

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::strings::owner::Strings;

/// Draws a name on slot `slot`'s surface, locked and unlocked (`d3d::lock_texture`; nothing is
/// drawn when it fails): three lines' height is cleared and text `text` is drawn three times in
/// the name font (box 0x168 wide) at x 4, -0x74, -0xEC and y 0, a line, two lines (a long name
/// wrapped over the plate's three rows).
fn draw_name_plate(
    slot: u32,
    text: impl Into<crate::memory::place::BytePlace>,
    gfx: &'static Gfx,
) -> Result<(), Refused> {
    let fonts = &gfx.fonts;
    let text = text.into();
    use crate::d3d::{lock_texture::lock_texture_in, unlock_texture::unlock_texture};
    if !lock_texture_in(gfx, slot)? {
        return Ok(());
    }
    // Logic in `rc2k_core::race::plates::{name_rows, NAME_BOX, name_places}`.
    use rc2k_core::race::plates::{NAME_BOX, name_places, name_rows};
    let line = fonts.font_block.game_fonts[0].get().line_height as u32;
    crate::gfx::rows::clear_surface_rows(0, name_rows(line), gfx)?;
    fonts.font_block.game_fonts[2].update(|f| f.box_width = NAME_BOX);
    for (x, y) in name_places(line) {
        draw_text_in_game(&fonts.font_block.game_fonts[2], text, x, y, gfx)?;
    }
    unlock_texture(gfx, slot)?;
    Ok(())
}

/// Writes string id `id` into the text buffer (`text`, with its NUL) and draws it on slot
/// `slot`'s plate (`draw_name_plate`).
fn draw_plate_by_id(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    slot: u32,
    id: u32,
    gfx: &'static Gfx,
    strings: &'static Strings,
) -> Result<(), Refused> {
    let text = &strings.text;
    let mut cursor = 0usize;
    crate::strings::append_string_by_id::append_string_by_id(so, id, &mut |bytes| {
        for (k, &b) in bytes.iter().enumerate() {
            text.buffer.text.set(cursor + k, b);
        }
        cursor += bytes.len();
    })?;
    text.buffer.text.set(cursor, 0);
    draw_name_plate(slot, &text.buffer.text, gfx)
}

/// Draws the times on slot `slot`'s surface, locked and unlocked as in `draw_name_plate`, below
/// the name's three lines: two lines and three rows are cleared, and the texts at `label_behind`
/// and `label_ahead` are drawn in the time font (box 0x80 wide) at x 4, y three and four lines.
fn draw_time_plate(frame: &Frame, slot: u32, gfx: &'static Gfx) -> Result<(), Refused> {
    use crate::d3d::{lock_texture::lock_texture_in, unlock_texture::unlock_texture};
    if !lock_texture_in(gfx, slot)? {
        return Ok(());
    }
    // Logic in `rc2k_core::race::plates::{time_start, time_rows, TIME_BOX,
    // time_places}`.
    use rc2k_core::race::plates::{TIME_BOX, time_places, time_rows, time_start};
    let line = gfx.fonts.font_block.game_fonts[0].get().line_height as u32;
    let start = time_start(line, gfx.drawing.draw_surface.pitch.get());
    crate::gfx::rows::clear_surface_rows(start, time_rows(line), gfx)?;
    // A label is eight bytes (spaces after each split, no NUL): its text runs on into the
    // dword after it, which ends it.
    let overlay = &frame.panel_and_overlay.get().overlay;
    for ((label, after), (x, y)) in [
        (&overlay.label_behind, &overlay.reserved_334),
        (&overlay.label_ahead, &overlay.reserved_340),
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
        draw_bytes_in_game(&gfx.fonts.font_block.game_fonts[3], &text, x, y, gfx)?;
    }
    unlock_texture(gfx, slot)?;
    Ok(())
}

/// When the two time texts (8 bytes each at `label_behind` / `label_ahead`) differ from their last
/// copies (`kept_behind` / `kept_ahead`), draws the time plate (`race::draw_time_plate`, slot
/// `KindTextureTables.kind_first_textures[196]`), makes the copies and sets both views' cars'
/// `notice_counter` to 1. Then for each view (slots `kind_first_textures[196]` / `[197]`) with a
/// name asked for in its car's `notice_id`, draws it by string id below 0x10000
/// (`race::draw_plate_by_id`) and sets `notice_last` to it, `notice_id` 0, `notice_counter` and
/// `notice_shown` 1. Refuses (before anything) a null view block; and a word from 0x10000 up,
/// which nothing writes to `notice_id`.
pub(super) fn update_name_plates(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &'static Gfx,
    strings: &'static Strings,
) -> Result<(), Refused> {
    let frame = &race.frame;
    let shared = &race.shared;
    let textures = &scene.textures;
    let (Some(a), Some(b)) = (shared.view_car(0), shared.view_car(1)) else {
        return Err(crate::errors::BadInput("a view without its car block").into());
    };
    // Their notice fields: `notice_id` asked, `notice_counter` the plate flag, `notice_last` the
    // last, `notice_shown` shown.
    let blocks = [a, b];
    // Logic in `rc2k_core::race::plates::{times_changed, NameAsk}`.
    use rc2k_core::race::plates::{NameAsk, times_changed};
    let now = [
        frame.panel_and_overlay.get().overlay.label_behind[0].get(),
        frame.panel_and_overlay.get().overlay.label_behind[1].get(),
        frame.panel_and_overlay.get().overlay.label_ahead[0].get(),
        frame.panel_and_overlay.get().overlay.label_ahead[1].get(),
    ];
    let kept = [
        frame.panel_and_overlay.get().overlay.kept_behind[0].get(),
        frame.panel_and_overlay.get().overlay.kept_behind[1].get(),
        frame.panel_and_overlay.get().overlay.kept_ahead[0].get(),
        frame.panel_and_overlay.get().overlay.kept_ahead[1].get(),
    ];
    if times_changed(now, kept) {
        draw_time_plate(
            frame,
            textures.kinds.get().kind_first_textures[196].get(),
            gfx,
        )?;
        for (from, to) in [
            (
                &frame.panel_and_overlay.get().overlay.label_behind[0],
                &frame.panel_and_overlay.get().overlay.kept_behind[0],
            ),
            (
                &frame.panel_and_overlay.get().overlay.label_behind[1],
                &frame.panel_and_overlay.get().overlay.kept_behind[1],
            ),
            (
                &frame.panel_and_overlay.get().overlay.label_ahead[0],
                &frame.panel_and_overlay.get().overlay.kept_ahead[0],
            ),
            (
                &frame.panel_and_overlay.get().overlay.label_ahead[1],
                &frame.panel_and_overlay.get().overlay.kept_ahead[1],
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
                draw_plate_by_id(
                    so,
                    textures.kinds.get().kind_first_textures[slot].get(),
                    asked,
                    gfx,
                    strings,
                )?;
            }
            NameAsk::Text(_) => {
                // Every writer of `notice_id` gives a string id: the notices' constants, the view
                // keys' counters, the race state's 0xCA.
                return Err(crate::errors::BadInput(
                    "a name plate asked for by a text's address (no writer gives one)",
                )
                .into());
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

/// Draws `text` in `font` at (`x`, `y`) on the plate's surface (as `text::draw_text_keep_regs`);
/// returns whether it was too wide.
pub(super) fn draw_text_in_game(
    font: &crate::gfx::typefaces::FontObject,
    text: impl Into<crate::memory::place::BytePlace>,
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    draw_bytes_in_game(font, &text.into().bytes("a name plate's text")?, x, y, gfx)
}

/// As `draw_text_in_game`, the text's bytes in hand.
fn draw_bytes_in_game(
    font: &crate::gfx::typefaces::FontObject,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    use crate::text::draw_text::GameCanvas;
    let mut copy = font.get();
    let wide = crate::text::draw_text_keep_regs::draw_text_keep_regs(
        &mut copy,
        text,
        x as i32,
        y as i32,
        &mut GameCanvas::new(gfx),
    )?;
    if copy != font.get() {
        font.set(copy);
    }
    Ok(wide)
}
