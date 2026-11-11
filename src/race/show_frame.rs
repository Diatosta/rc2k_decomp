//! `race::show_frame` (0x4547AE): the race frame's end, after its 3D scene: the frame locked, the
//! race's sound tick, the credits rolled while `dword_71C3E0` is set, the frame rate drawn
//! (`race::draw_frame_rate`), the frame unlocked and shown; the sound tick, the clock restarted
//! (`scene::restart_clock`) and its time less `dword_63DC38` into `dword_63DC3C`, the frame rate
//! measured (`race::measure_frame_rate`); carry clear.
//!
//! Caller: `drive::race_frame`. Its calls are the Rust versions: `scene::lock_frame`,
//! `sound::race_sound_tick`, `race::roll_credits`, `race::draw_frame_rate`, `scene::unlock_frame`,
//! `scene::present_frame`, `scene::restart_clock`, `race::measure_frame_rate`.

use crate::errors::Refused;
use crate::race::owner::Race;

/// What `show_frame` leaves: whether the frame was lost (the carry) and eax.
pub struct Shown {
    pub lost: bool,
    pub eax: u32,
}

pub fn show_frame(race: &Race) -> Result<Shown, Refused> {
    let system = crate::system::owner::system();
    let scene = crate::scene::owner::scene();
    let frame = &race.frame;
    let (locked, kept) = crate::scene::lock_frame::lock_frame(race);
    if locked < 1 {
        return Ok(Shown {
            lost: true,
            eax: kept,
        });
    }
    if race.shared.work.in_credits.get() != 0 {
        crate::race::frame_text::roll_credits(
            frame,
            1,
            &mut crate::text::draw_text::GameCanvas,
            &mut crate::race::frame_text::credits_tune,
            system,
        )?;
    }
    { crate::race::frame_text::draw_frame_rate(race, &mut crate::text::draw_text::GameCanvas) }?;
    // `scene::unlock_frame` leaves eax 0 (its carry is `present_frame`'s next).
    crate::scene::unlock_frame::unlock_frame(&race.frame);
    if !crate::scene::present_frame::present_frame() {
        return Ok(Shown { lost: true, eax: 0 });
    }
    let clock = crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::system::tick_count::tick_count(),
    );
    let time = clock.wrapping_sub(race.shared.view.clock_base.get());
    race.shared.view.frame_time.set(time);
    // The measuring leaves eax the counter's low half on the 16th call, else as it was.
    let eax = { crate::race::measure_frame_rate::measure_frame_rate(race, scene, system) }
        .unwrap_or(time);
    Ok(Shown { lost: false, eax })
}
