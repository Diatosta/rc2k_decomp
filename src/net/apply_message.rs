//! `net::apply_message` (0x4644EC): one message received from player ebp put into that
//! player's car block (`g_carBlocks[ebp]`), after logging the own player, the step, the
//! message's first byte, "rcvd fm plyr" and ebp. Players 8 and up: nothing more.
//! - 'JGSM': the four input dwords into +0xFD0 .. +0xFDC;
//! - 'XGSM': nine dwords into +0x64 .. +0x84;
//! - 'PGSM': the place into +0xFC0 .. +0xFC8 (where the car is to be), the other 37
//!   dwords where `net::send_position` took them from. Then, after the car's first 1000
//!   steps (+8; `js` on +8 - 1000) and when the place received is 10 or more away from the
//!   car's (+0x134 .. +0x13C, whole numbers: `fild` / `fisub`, `fsqrt`, `flt_5C5D54`) the
//!   car is put there; else +0xFC0 .. +0xFC8 become a 32nd of the way (`sar 5`) and
//!   +0xFCC 32, for `net::follow_remote` to glide it;
//! - others: nothing.
//!
//! In:  `player` (ebp), `message` (ebx: its first dword the tag)
//! Out: nothing (the original leaves eax, ecx, edi as it goes; its one caller has it inside
//!      `pusha` .. `popa`)
//!
//! See `net/mod.rs`. One caller, `net::apply_received`.

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::net::store::NET;
use crate::pointers::GamePtr;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use rc2k_core::net::race::{self, Arrival, Kind};

/// g_carBlocks: the eight cars' blocks.
const TEXT: &std::ffi::CStr = c"rcvd fm plyr"; // "rcvd fm plyr"

/// A message no car could send, told once in the log.
fn note_unbelievable(player: u32) {
    static TOLD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !TOLD.swap(true, std::sync::atomic::Ordering::Relaxed) {
        crate::framework::log(&format!(
            "net: player {player}'s message has values no car could hold: not applied"
        ));
    }
}

/// The Rust version.
pub fn apply_message(player: u32, message: GamePtr) -> Result<(), BadInput> {
    log_int(NET.player.get() as i32, false, 1);
    log_int(RACE_VIEW.local_car.get() as i32, false, 1);
    log_int(message.read::<u8>("a received message")? as i32, false, 1);
    crate::logfile::log_write::log_nul(&TEXT, false, 1);
    log_int(player as i32, true, 1);
    if player >= 8 {
        return Ok(());
    }
    let block = RACE_WORK.car_block(player as usize);
    // The block is looked up at each access: a null one refuses at the first, as the original's.
    let at = |offset: u32| {
        Ok::<_, BadInput>(GamePtr::to(crate::race::trap::car_at(block)?).offset(offset))
    };
    let rd = |offset: u32| at(offset)?.read::<u32>("a received message or car block");
    let wr = |offset: u32, v: u32| at(offset)?.write(v, "a car block's field");
    // Converted: `rc2k_core::net::race` (`Kind`, `arrival`; the distance's test in `f64`,
    // exact for the decision).
    let Some(kind) = Kind::of(message.read("a received message")?) else {
        return Ok(());
    };
    let mut values = Vec::with_capacity(kind.fields().len());
    for n in 0..kind.fields().len() as u32 {
        values.push(message.offset(4 + 4 * n).read("a received message")?);
    }
    if !kind.believable(&values) {
        note_unbelievable(player);
        return Ok(());
    }
    for (n, &value) in values.iter().enumerate() {
        wr(kind.destination(n), value)?;
    }
    if kind != Kind::Position {
        return Ok(());
    }
    let three = |at: u32| Ok::<_, BadInput>([rd(at)?, rd(at + 4)?, rd(at + 8)?]);
    let (wanted, now) = (three(race::WANTED)?, three(race::PLACE)?);
    let steps = rd(race::STEPS)?;
    match race::arrival(wanted, now, steps, race::SNAP_FROM) {
        Arrival::Jump => {
            for (k, v) in (0..).zip(wanted) {
                wr(race::PLACE + 4 * k, v)?;
            }
            // +0xFC0 now holds a place, not a step: a glide under way ends.
            wr(race::GLIDE_STEPS, 0)?;
        }
        Arrival::Glide(step) => {
            for (k, v) in (0..).zip(step) {
                wr(race::WANTED + 4 * k, v)?;
            }
            wr(race::GLIDE_STEPS, race::GLIDE)?;
        }
    }
    Ok(())
}
