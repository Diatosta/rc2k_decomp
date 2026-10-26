//! The race loop's shared calls (stream B, 2026-10-18): the Rust versions of stream A's
//! and the input module's hooks the race loop (`drive::race_loop`, `drive::quit_question`,
//! `drive::pause_until_key`) calls, with the registers their hooks would leave, and the
//! race frame `sub_4541BF` (stream A's, held: a call into the game until ported).

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};

/// `sub_4541BF`: the race's frame (drawing and the sound tick), stream A's, through the
/// game.
pub fn race_frame(r: &mut Regs) {
    crate::calls::call(rva(0x4541BF), r);
}

/// `scene::restart_clock`: eax the clock.
pub fn restart_clock(r: &mut Regs) {
    r.eax = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
}

/// `input::update`.
pub fn input_update(r: &mut Regs) {
    *r = unsafe { crate::input::keys::update(r, &mut crate::input::keys::game_screenshot) };
}

/// `input::take_key`: al the key, ZF when none.
pub fn take_key(r: &mut Regs) {
    let key = crate::input::take_key::take_key();
    r.eax = (r.eax & !0xFF) | key as u32;
    r.set_zero(key == 0);
    r.set_carry(false);
}

/// `race::race_keys`.
pub fn race_keys(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { crate::race::frame::race_keys(r, &mut crate::race::frame::key_sound_part) }?;
    Ok(())
}

/// `race::tyre_change`: eax, esi.
pub fn tyre_change(r: &mut Regs) -> Result<(), Refused> {
    let after = unsafe {
        crate::race::frame::tyre_change_at(
            r,
            &crate::race::frame::tyre_variables(),
            &mut crate::race::frame::tyre_part,
        )
    }?;
    r.eax = after.eax;
    r.esi = after.esi;
    Ok(())
}

/// `race::players_all_out`: CF.
pub fn players_all_out(r: &mut Regs) -> Result<bool, Refused> {
    let out = unsafe { crate::race::frame::players_all_out() }?;
    r.set_carry(out);
    Ok(out)
}

/// `race::stop_race_sounds`.
pub fn stop_race_sounds(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe {
        crate::race::stop_race_sounds::stop_race_sounds(r, &mut |_, r| {
            *r = crate::race::race_end::reset_race_sounds(
                r,
                &crate::race::race_end::SoundVars::game(),
                &mut crate::race::race_end::sound_part,
            )?;
            Ok(())
        })
    }?;
    Ok(())
}

/// `race::wake_car_sounds` (its callers keep the registers round it).
pub fn wake_car_sounds(r: &Regs) -> Result<(), Refused> {
    unsafe { crate::race::sounds::wake_car_sounds(r, &mut crate::race::sounds::game_wake_part) }?;
    Ok(())
}

/// `logfile::log_newline_l1` and the `pusha`'d `log_write_l1_nl` of a text.
pub fn log_line(text: usize) {
    unsafe { crate::logfile::log_write::log_write(game_addr(rva(text)) as *const u8, true, 1) };
}
