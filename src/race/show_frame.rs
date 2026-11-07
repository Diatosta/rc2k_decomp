//! `race::show_frame` (0x4547AE): the race frame's end, after its 3D scene: the frame
//! locked (`scene::lock_frame`; carry: lost), the race's sound tick (eax 0x13), the credits
//! rolled while `dword_71C3E0` is set (`race::roll_credits`, eax 1), the frame rate drawn
//! (`race::draw_frame_rate`), the frame unlocked and shown (`scene::unlock_frame`,
//! `scene::present_frame`; carry: lost); the sound tick (eax 0x14), the clock restarted
//! (`scene::restart_clock`) and its time less `dword_63DC38` into `dword_63DC3C`, the frame
//! rate measured (`race::measure_frame_rate`); carry clear.
//!
//! Caller: `drive::race_frame`. Its calls are the Rust versions: `scene::lock_frame`
//! (0x44E837), `sound::race_sound_tick` (0x421BA9), `race::roll_credits` (0x415FC2),
//! `race::draw_frame_rate` (0x4562B2), `scene::unlock_frame` (0x44E851),
//! `scene::present_frame` (0x44E895), `scene::restart_clock` (0x450381),
//! `race::measure_frame_rate` (0x42009E).

use crate::errors::Refused;
use crate::race::owner::Race;

/// What `show_frame` leaves: whether the frame was lost (the carry) and eax.
pub struct Shown {
    pub lost: bool,
    pub eax: u32,
}

/// The Rust version (see the header).
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
    // The sound tick with eax 0x13 (it reads no register).
    crate::sound::race_sound_tick::race_sound_tick(&race.frame)?;
    if race.shared.work.in_credits.get() != 0 {
        crate::race::frame_text::roll_credits(
            frame,
            1,
            &mut crate::text::draw_text::GameCanvas,
            &mut crate::race::frame_text::credits_tune,
            system,
        )?;
    }
    {
        crate::race::frame_text::draw_frame_rate(
            race,
            &mut crate::text::draw_text::GameCanvas,
            scene,
        )
    }?;
    // `scene::unlock_frame` leaves eax 0 (its carry is `present_frame`'s next).
    crate::scene::unlock_frame::unlock_frame(&race.frame);
    if !crate::scene::present_frame::present_frame() {
        return Ok(Shown { lost: true, eax: 0 });
    }
    // The sound tick with eax 0x14.
    crate::sound::race_sound_tick::race_sound_tick(&race.frame)?;
    let clock = crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::system::tick_count::tick_count(),
    );
    let time = clock.wrapping_sub(race.shared.view.clock_base.bits());
    race.shared.view.frame_time.set_bits(time);
    // The measuring leaves eax the counter's low half on the 16th call, else as it was.
    let eax = { crate::race::measure_frame_rate::measure_frame_rate(race, scene, system) }
        .unwrap_or(time);
    Ok(Shown { lost: false, eax })
}
