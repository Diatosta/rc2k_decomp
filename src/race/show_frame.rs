//! `race::show_frame` (0x4547AE): the race frame's end, after its 3D scene: the frame locked, the
//! race's sound tick, the credits rolled while `in_credits` is set, the frame rate drawn
//! (`race::draw_frame_rate`), the frame unlocked and shown; the sound tick, the clock restarted
//! (`scene::restart_clock`) and its time less `clock_base` into `frame_time`, the frame rate
//! measured (`race::measure_frame_rate`); carry clear.
//!
//! Caller: `drive::race_frame`. Its calls are the Rust versions: `scene::lock_frame`,
//! `sound::race_sound_tick`, `race::roll_credits`, `race::draw_frame_rate`, `scene::unlock_frame`,
//! `scene::present_frame`, `scene::restart_clock`, `race::measure_frame_rate`.

use crate::errors::Refused;
use crate::race::owner::Race;

/// What `show_frame` leaves: whether the frame was lost (the carry) and the original's leftover value.
pub struct Shown {
    pub lost: bool,
    pub eax: u32,
}

pub fn show_frame(race: &Race, o: crate::owners::Owners<'static>) -> Result<Shown, Refused> {
    let (system, scene) = (o.system, o.scene);
    let frame = &race.frame;
    let (locked, kept) = crate::scene::lock_frame::lock_frame(race, o.gfx, o.system);
    if locked < 1 {
        return Ok(Shown {
            lost: true,
            eax: kept,
        });
    }
    if race.shared.work.get().in_credits.is_set() {
        crate::race::frame_text::roll_credits(
            crate::strings::append_string_by_id::StringOwners::of(o),
            frame,
            1,
            &mut crate::text::draw_text::GameCanvas::new(o.gfx),
            &mut || crate::race::frame_text::credits_tune(o),
            system,
            o.audio,
        )?;
    }
    {
        crate::race::frame_text::draw_frame_rate(
            race,
            o.strings,
            &mut crate::text::draw_text::GameCanvas::new(o.gfx),
            o.gfx,
        )
    }?;
    // `scene::unlock_frame` leaves 0 as the leftover value (its carry is `present_frame`'s next).
    crate::scene::unlock_frame::unlock_frame(&race.frame, o.gfx);
    if !crate::scene::present_frame::present_frame() {
        return Ok(Shown { lost: true, eax: 0 });
    }
    let clock = crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::system::tick_count::tick_count(),
    );
    let time = clock.wrapping_sub(race.shared.view.get().clock_base.get());
    race.shared.view.get().frame_time.set(time);
    // The measuring leaves the counter's low half as the leftover value on the 16th call, else as it was.
    let eax = { crate::race::measure_frame_rate::measure_frame_rate(race, scene, system) }
        .unwrap_or(time);
    Ok(Shown { lost: false, eax })
}
