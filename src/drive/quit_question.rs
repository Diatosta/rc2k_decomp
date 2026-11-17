//! The "quit?" question in a race. The race's sounds stopped,
//! `scene_switch` 1; the best whole frame less 7 (the first block's +8 / 100) kept in
//! `last_kept` when not below it (by the sign); the eight blocks' notice 0xC2 (with
//! `no_replay`) or 0xC1, +0x730 0, +0x72C -1. Then until answered (`input::yes_no_key`, the
//! race's frame drawn each turn, `drive::race_frame` through the game): yes -1, no 0, and outside
//! mode 2 with `no_replay` the third answer -2. The notices cleared, `scene_switch` 0 and the
//! sounds woken, the eight blocks' +0x468 0.
//!
//! Gives the answer.
//!
//! Refuses what its calls refuse.

use crate::drive::race_parts::{stop_race_sounds, wake_car_sounds};
use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::race_ticks;

/// The answer: 0 to race on.
pub fn quit_question(race: &Race, o: crate::owners::Owners<'static>) -> Result<u32, Refused> {
    let input = o.input;
    let shared = &race.shared;
    let replay = &race.replay;
    stop_race_sounds(o)?;
    // A button from before the question answers nothing.
    crate::input::menu_pad::take_answer(input);
    shared.vars.get().scene_switch.set(1);
    // The carried value: the best frame, then each key's answer, then what the frame leaves; it goes
    // into `input::yes_no_key` (the key given, when one is).
    let (mut eax, _) = race_ticks::best_frame(race.car_blocks().cars[0].step_clock.get());
    if race_ticks::keeps_frame(eax, replay.replay_frames().last_kept.get()) {
        replay.replay_frames().last_kept.set(eax);
    }
    let kind = race_ticks::quit_notice(shared.vars.get().no_replay.get() != 0);
    // With pictures for the answers the question is drawn here, over the frame, with them in the
    // text; the race's own notice then stays off.
    let inline = Inline::of(o, kind);
    for c in 0..8 {
        race.car_blocks().cars[c as usize]
            .notice_id
            .set(if inline.is_some() { 0 } else { kind });
    }
    for c in 0..8 {
        race.car_blocks().cars[c as usize].notice_time_b.set(0);
    }
    for c in 0..8 {
        race.car_blocks().cars[c as usize].notice_time.set(u32::MAX);
    }
    let answer = loop {
        // A gamepad's button answers directly: this question has no items on screen to choose from.
        eax = match crate::input::menu_pad::take_answer(input) {
            Some(answer) => answer,
            None => {
                crate::input::yes_no_key::yes_no_key(
                    crate::strings::append_string_by_id::StringOwners::of(o),
                    eax,
                    input,
                )
            }?,
        };
        match race_ticks::quit_answer(
            eax,
            shared.vars.get().run_kind.get(),
            shared.vars.get().no_replay.get() != 0,
        ) {
            Some(answer) => break answer,
            None => {
                eax = crate::drive::race_frame::race_frame_over(race, o, eax, &|| match &inline {
                    Some(inline) => inline.draw(o),
                    None => answer_hint(o),
                })?;
            }
        }
    };
    for c in 0..8 {
        let car = &race.car_blocks().cars[c];
        for cell in [&car.notice_shown, &car.notice_id, &car.notice_last] {
            cell.set(0);
        }
    }
    shared.vars.get().scene_switch.set(0);
    wake_car_sounds(race, o.sound, o.audio, o.system)?;
    for c in 0..8 {
        race.car_blocks().cars[c as usize].input_keys.set(0);
    }
    Ok(answer)
}

/// The plate under the hint line: its colour and the border round the line.
const PLATE: u32 = 0x0C1238;
const PLATE_MARGIN: i32 = 8;

/// The hint under the question: the pictures of the buttons that answer yes and no for the
/// controller in use, drawn over the frame. Nothing when the controller's buttons are shown as
/// numbers or the keyboard is in use.
fn answer_hint(o: crate::owners::Owners<'static>) {
    use rc2k_core::input::profile::MenuControl as C;
    use rc2k_core::strings::own::id::HINTS;
    let wanted = [(C::Enter, HINTS), (C::Back, HINTS + 1)];
    let Some((family, hints)) = crate::gfx::button_hints::menu_hints(o, &wanted, None) else {
        return;
    };
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return;
    }
    let mut canvas = crate::text::draw_text::GameCanvas::new(o.gfx);
    let width = crate::gfx::button_hints::line_width(&hints, &mut canvas);
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
    // On a dark plate: a pixel of the race's frame is not the canvas's to blend with, so the
    // pictures go over pixels the plate has written.
    let (plate_w, plate_h) = (
        width + 2 * PLATE_MARGIN,
        crate::gfx::button_hints::SIZE + 2 * PLATE_MARGIN,
    );
    let (left, top) = ((w - width) / 2, h * 27 / 100);
    crate::input::osk_draw::fill(
        o,
        [left - PLATE_MARGIN, top - PLATE_MARGIN, plate_w, plate_h],
        PLATE,
    );
    crate::gfx::button_hints::draw_line(o, family, &hints, left, top);
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
}

/// The question's own text with a picture after each key letter ("RETIRAR? (s [X]/n [O])"): the
/// text, where its two key letters are, and the pictures.
struct Inline {
    family: rc2k_core::input::icons::Family,
    icons: [rc2k_core::input::icons::Icon; 2],
    text: Vec<u8>,
    letters: (usize, usize),
}

/// The race's notice font: the one the question is drawn in.
fn question_font(o: crate::owners::Owners<'static>) -> &'static crate::gfx::typefaces::FontObject {
    &o.gfx.fonts.font_block.game_fonts[2]
}

impl Inline {
    /// The inline question when the controller in use has pictures for yes and no and the
    /// text of notice `kind` has a "(a/b" group.
    fn of(o: crate::owners::Owners<'static>, kind: u32) -> Option<Inline> {
        use rc2k_core::input::profile::MenuControl as C;
        let (pictures, bindings) = crate::gfx::button_hints::pictures_in_use(o)?;
        let icon = |c| {
            pictures
                .menu(c, bindings.menu(c))
                .filter(|&i| crate::gfx::button_pictures::has(pictures.family, i))
        };
        let icons = [icon(C::Enter)?, icon(C::Back)?];
        let mut text = Vec::new();
        crate::strings::append_string_by_id::push_string(
            crate::strings::append_string_by_id::StringOwners::of(o),
            &mut text,
            kind,
        )
        .ok()?;
        let letters = race_ticks::answer_letters(&text)?;
        Some(Inline {
            family: pictures.family,
            icons,
            text,
            letters,
        })
    }

    /// The line drawn over the frame, centred where the race's notice puts the question.
    fn draw(&self, o: crate::owners::Owners<'static>) {
        use crate::text::draw_text::{GameCanvas, draw_text};
        let locked_here = o.gfx.locked.surface.get() != 1;
        if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
            return;
        }
        let mut canvas = GameCanvas::new(o.gfx);
        let mut font = question_font(o).get();
        font.mode = rc2k_core::strings::text::MODE_AT_X;
        let size = font.line_height * 5 / 4;
        let width_of = |bytes: &[u8], canvas: &mut GameCanvas| {
            let mut copy = font;
            copy.mode = rc2k_core::strings::text::MODE_MEASURE;
            let mut nul = bytes.to_vec();
            nul.push(0);
            crate::text::measure_text::measure_text(&mut copy, &nul[..], 0, canvas)
                .map_or(0, |m| m.width as i32)
        };
        let (a, b) = self.letters;
        let parts = [&self.text[..=a], &self.text[a + 1..=b], &self.text[b + 1..]];
        let space = (width_of(b" ", &mut canvas) / 2).max(2);
        let widths = parts.map(|p| width_of(p, &mut canvas));
        let total = widths.iter().sum::<i32>() + 2 * (space + size);
        let screen = o.system.session.screen.get();
        let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
        let (mut x, y) = ((w - total) / 2, h * 22 / 100);
        for (n, part) in parts.iter().enumerate() {
            let mut text = part.to_vec();
            text.push(0);
            let _ = draw_text(&mut font, &text[..], x, y, &mut canvas);
            x += widths[n];
            if let Some(&icon) = self.icons.get(n) {
                x += space;
                crate::gfx::button_pictures::draw_glyph(
                    o,
                    self.family,
                    icon,
                    x,
                    y + font.line_height / 2 - size / 2 + 2,
                    size,
                );
                x += size;
            }
        }
        if locked_here {
            crate::platform::unlock_surface(o.gfx);
        }
    }
}
