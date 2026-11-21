//! `race::show_frame`: the race frame's end, after its 3D scene. The frame is locked, the race's
//! sound tick runs, the credits roll while `in_credits` is set, the frame rate is drawn
//! (`race::draw_frame_rate`), and the frame is unlocked and shown. Then the sound tick runs, the
//! clock restarts (`scene::restart_clock`) and its time less `clock_base` goes into
//! `frame_time`, and the frame rate is measured (`race::measure_frame_rate`).

use crate::errors::Refused;
use crate::race::owner::Race;

/// Shows the frame; a frame that could not be locked or presented is simply not shown.
pub fn show_frame(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (system, scene) = (o.system, o.scene);
    let frame = &race.frame;
    let locked = crate::scene::lock_frame::lock_frame(o.gfx, o.system);
    if locked < 1 || lose_this_frame() {
        return Ok(());
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
    crate::race::part_notices::draw(race, o)?;
    crate::net::left::draw(o)?;
    crate::scene::unlock_frame::unlock_frame(o.gfx);
    if !crate::scene::present_frame::present_frame(o) {
        return Ok(());
    }
    let clock = crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::clock::tick_count::tick_count(&system.session.picture_clock),
    );
    let time = clock.wrapping_sub(race.shared.view.get().clock_base.get());
    race.shared.view.get().frame_time.set(time);
    crate::race::measure_frame_rate::measure_frame_rate(race, scene);
    Ok(())
}

/// Test switch (remote builds): `RC2K_LOSE_FRAMES=n` treats every n-th race frame's lock as
/// refused, so the skipped-frame path can be run; other builds never lose a frame this way.
fn lose_this_frame() -> bool {
    crate::remote::lose_this_frame()
}
