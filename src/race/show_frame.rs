//! `race::show_frame`: the race frame's end, after its 3D scene. The frame is locked, the race's
//! sound tick runs, the credits roll while `in_credits` is set, the frame rate is drawn
//! (`race::draw_frame_rate`), and the frame is unlocked and shown. Then the sound tick runs, the
//! clock restarts (`scene::restart_clock`) and its time less `clock_base` goes into
//! `frame_time`, and the frame rate is measured (`race::measure_frame_rate`).

use crate::errors::Refused;
use crate::race::owner::Race;

/// What `show_frame` leaves: a leftover value the caller passes on. A frame that could not be
/// locked or presented is simply not shown.
pub struct Shown {
    pub eax: u32,
}

pub fn show_frame(race: &Race, o: crate::owners::Owners<'static>) -> Result<Shown, Refused> {
    let (system, scene) = (o.system, o.scene);
    let frame = &race.frame;
    let (locked, kept) = crate::scene::lock_frame::lock_frame(race, o.gfx, o.system);
    if locked < 1 || lose_this_frame() {
        return Ok(Shown { eax: kept });
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
    crate::net::left::draw(o)?;
    // The leftover value after the unlock is 0.
    crate::scene::unlock_frame::unlock_frame(&race.frame, o.gfx);
    if !crate::scene::present_frame::present_frame(o) {
        return Ok(Shown { eax: 0 });
    }
    let clock = crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::clock::tick_count::tick_count(&system.session.picture_clock),
    );
    let time = clock.wrapping_sub(race.shared.view.get().clock_base.get());
    race.shared.view.get().frame_time.set(time);
    // The measuring leaves the counter's low half as the leftover value on the 16th call, else as
    // it was.
    let eax = { crate::race::measure_frame_rate::measure_frame_rate(race, scene) }.unwrap_or(time);
    Ok(Shown { eax })
}

/// Test switch (remote builds): `RC2K_LOSE_FRAMES=n` treats every n-th race frame's lock as
/// refused, so the skipped-frame path can be run; other builds never lose a frame this way.
fn lose_this_frame() -> bool {
    #[cfg(feature = "remote")]
    {
        use std::sync::OnceLock;
        use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
        static EVERY: OnceLock<u32> = OnceLock::new();
        static COUNT: AtomicU32 = AtomicU32::new(0);
        let every = *EVERY.get_or_init(|| {
            std::env::var("RC2K_LOSE_FRAMES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        });
        every != 0 && COUNT.fetch_add(1, Relaxed) % every == every - 1
    }
    #[cfg(not(feature = "remote"))]
    false
}
